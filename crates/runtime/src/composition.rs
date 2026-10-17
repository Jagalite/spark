//! Finite, borrowed timelines and integer transitions. These helpers select time;
//! the host selects domain state, renders ordinary widgets, and owns all waiting.
use crate::{ClipTime, FiniteClip, Period};
use std::{fmt, time::Duration};

/// Maximum borrowed segments/tracks. Construction and sampling are bounded O(n).
pub const MAX_PARTS: usize = 256;
/// Integer transition weight denominator. Intermediate weights are rounded down.
pub const WEIGHT_SCALE: u32 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionError {
    Empty,
    TooManyParts,
    TimeOverflow,
}
impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "a composition needs at least one part",
            Self::TooManyParts => "a composition supports at most 256 parts",
            Self::TimeOverflow => "composition end exceeds the clip-time range",
        })
    }
}
impl std::error::Error for CompositionError {}

fn validate_count(count: usize) -> Result<(), CompositionError> {
    match count {
        0 => Err(CompositionError::Empty),
        1..=MAX_PARTS => Ok(()),
        _ => Err(CompositionError::TooManyParts),
    }
}

/// A finite concatenation of positive durations, without storage or allocation.
/// At an internal seam the next segment starts at zero. At/beyond the total end,
/// the last segment is held at its exact duration. This never completes a task.
#[derive(Debug, Clone, Copy)]
pub struct Sequence<'a> {
    durations: &'a [Period],
    total: u64,
}
impl<'a> Sequence<'a> {
    pub fn new(durations: &'a [Period]) -> Result<Self, CompositionError> {
        validate_count(durations.len())?;
        let total = durations.iter().try_fold(0_u64, |sum, duration| {
            sum.checked_add(duration.as_nanos())
                .ok_or(CompositionError::TimeOverflow)
        })?;
        Ok(Self { durations, total })
    }
    pub const fn duration(self) -> Duration {
        Duration::from_nanos(self.total)
    }
    pub fn sample(self, time: ClipTime) -> SequenceSample {
        let mut start = 0;
        for (index, duration) in self.durations.iter().enumerate() {
            // The constructor validated the complete positive sum.
            let end = start + duration.as_nanos();
            if time.as_nanos() < end || index == self.durations.len() - 1 {
                return SequenceSample {
                    index,
                    start: ClipTime::from_nanos(start),
                    end: ClipTime::from_nanos(end),
                    local_time: ClipTime::from_nanos(
                        time.as_nanos()
                            .saturating_sub(start)
                            .min(duration.as_nanos()),
                    ),
                    complete: time.as_nanos() >= self.total,
                };
            }
            start = end;
        }
        unreachable!("Sequence construction rejects empty input")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceSample {
    index: usize,
    /// Absolute outer timeline coordinates, not modulo phases.
    start: ClipTime,
    end: ClipTime,
    local_time: ClipTime,
    complete: bool,
}
impl SequenceSample {
    pub const fn index(self) -> usize {
        self.index
    }
    pub const fn start(self) -> ClipTime {
        self.start
    }
    pub const fn end(self) -> ClipTime {
        self.end
    }
    pub const fn local_time(self) -> ClipTime {
        self.local_time
    }
    pub const fn complete(self) -> bool {
        self.complete
    }

    /// Keep this boundary separate from a child's Continuous host-cadence hint.
    pub fn next_boundary(self) -> Option<ClipTime> {
        (!self.complete).then_some(self.end)
    }
    /// Translate a strictly future child-local deadline, clamping it to this
    /// segment's end. Stale deadlines and completed sequences return None.
    pub fn map_deadline(self, local: ClipTime) -> Option<ClipTime> {
        if self.complete || local <= self.local_time {
            return None;
        }
        Some(ClipTime::from_nanos(
            self.start.as_nanos()
                + local
                    .as_nanos()
                    .min(self.end.as_nanos() - self.start.as_nanos()),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Before,
    Active,
    Finished,
}

/// A finite visual track with an explicit delay. A Finished sample retains its
/// terminal pose; the host chooses whether to hold it or omit that layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipTrack {
    start: ClipTime,
    clip: FiniteClip,
    end: ClipTime,
}
impl ClipTrack {
    pub fn new(start: ClipTime, duration: Period) -> Result<Self, CompositionError> {
        let end = start
            .as_nanos()
            .checked_add(duration.as_nanos())
            .ok_or(CompositionError::TimeOverflow)?;
        Ok(Self {
            start,
            clip: FiniteClip::new(duration),
            end: ClipTime::from_nanos(end),
        })
    }
    pub const fn start(self) -> ClipTime {
        self.start
    }
    pub const fn end(self) -> ClipTime {
        self.end
    }
    pub const fn duration(self) -> Period {
        self.clip.duration()
    }
    pub fn sample(self, time: ClipTime) -> TrackSample {
        let phase = if time < self.start {
            Phase::Before
        } else if time < self.end {
            Phase::Active
        } else {
            Phase::Finished
        };
        TrackSample {
            phase,
            start: self.start,
            end: self.end,
            local_time: self
                .clip
                .sample(ClipTime::from_nanos(
                    time.as_nanos().saturating_sub(self.start.as_nanos()),
                ))
                .time,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackSample {
    phase: Phase,
    start: ClipTime,
    end: ClipTime,
    local_time: ClipTime,
}
impl TrackSample {
    pub const fn phase(self) -> Phase {
        self.phase
    }
    pub const fn start(self) -> ClipTime {
        self.start
    }
    pub const fn end(self) -> ClipTime {
        self.end
    }
    pub const fn local_time(self) -> ClipTime {
        self.local_time
    }

    pub fn next_boundary(self) -> Option<ClipTime> {
        match self.phase {
            Phase::Before => Some(self.start),
            Phase::Active => Some(self.end),
            Phase::Finished => None,
        }
    }
    pub fn map_deadline(self, local: ClipTime) -> Option<ClipTime> {
        if self.phase != Phase::Active || local <= self.local_time {
            return None;
        }
        Some(ClipTime::from_nanos(
            self.start.as_nanos()
                + local
                    .as_nanos()
                    .min(self.end.as_nanos() - self.start.as_nanos()),
        ))
    }
}

/// Borrowed independent tracks, preserving the caller's explicit paint order.
/// Sampling does not paint or imply opacity/blending between ordinary Widgets.
#[derive(Debug, Clone, Copy)]
pub struct Parallel<'a> {
    tracks: &'a [ClipTrack],
}
impl<'a> Parallel<'a> {
    pub fn new(tracks: &'a [ClipTrack]) -> Result<Self, CompositionError> {
        validate_count(tracks.len())?;
        Ok(Self { tracks })
    }
    pub fn sample(self, time: ClipTime) -> impl ExactSizeIterator<Item = TrackSample> + 'a {
        self.tracks.iter().map(move |track| track.sample(time))
    }
    pub fn next_boundary(self, time: ClipTime) -> Option<ClipTime> {
        self.sample(time)
            .filter_map(TrackSample::next_boundary)
            .min()
    }
    pub fn duration(self) -> Duration {
        Duration::from_nanos(
            self.tracks
                .iter()
                .map(|track| track.end.as_nanos())
                .max()
                .unwrap(),
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    SmoothStep,
}

/// A finite integer transition, useful for bounded coordinates or role mixing.
/// No background-color assumption or pixel alpha blending is imposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    clip: FiniteClip,
    easing: Easing,
}
impl Transition {
    pub const fn new(duration: Period) -> Self {
        Self {
            clip: FiniteClip::new(duration),
            easing: Easing::Linear,
        }
    }
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
    pub const fn duration(self) -> Period {
        self.clip.duration()
    }
    pub fn sample(self, time: ClipTime) -> TransitionSample {
        let sample = self.clip.sample(time);
        let scale = u128::from(WEIGHT_SCALE);
        let linear = u128::from(sample.time.as_nanos()) * scale
            / u128::from(self.clip.duration().as_nanos());
        let weight = match self.easing {
            Easing::Linear => linear,
            Easing::SmoothStep => linear * linear * (3 * scale - 2 * linear) / (scale * scale),
        };
        TransitionSample {
            weight: weight as u32,
            complete: sample.complete,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionSample {
    weight: u32,
    complete: bool,
}
impl TransitionSample {
    pub const fn weight(self) -> u32 {
        self.weight
    }
    /// Exact time-based completion, separate from quantized geometry and domain state.
    pub const fn complete(self) -> bool {
        self.complete
    }
    /// Signed interpolation avoids underflow for descending values. The signed
    /// displacement is truncated toward zero; endpoints are exact and monotone.
    pub fn interpolate(self, from: u16, to: u16) -> u16 {
        (i64::from(from)
            + (i64::from(to) - i64::from(from)) * i64::from(self.weight) / i64::from(WEIGHT_SCALE))
            as u16
    }
}
