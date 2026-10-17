//! Validated, allocation-free direct selection of complete authored frames.
use crate::{ClipTime, Period};
use std::{fmt, time::Duration};

pub const MAX_FRAMES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSequence<'a> {
    ends: &'a [u64],
    period: Period,
}
impl<'a> FrameSequence<'a> {
    /// Cumulative frame-end nanoseconds must be positive and strictly increasing.
    pub fn new(ends: &'a [u64]) -> Result<Self, SpriteError> {
        if ends.is_empty() || ends.len() > MAX_FRAMES {
            return Err(SpriteError::FrameCount);
        }
        let mut previous = 0;
        for &end in ends {
            if end <= previous {
                return Err(SpriteError::InvalidDuration);
            }
            previous = end;
        }
        Ok(Self {
            ends,
            period: Period::new(Duration::from_nanos(previous))
                .map_err(|_| SpriteError::InvalidDuration)?,
        })
    }
    pub fn frame_index(self, time: ClipTime) -> usize {
        let phase = self.period.phase_nanos(time);
        self.ends.partition_point(|end| *end <= phase)
    }
    pub const fn period(self) -> Period {
        self.period
    }
    /// Absolute, strictly future clip boundary. None means unrepresentable.
    pub fn next_boundary(self, time: ClipTime) -> Option<ClipTime> {
        let phase = self.period.phase_nanos(time);
        time.as_nanos()
            .checked_add(self.ends[self.frame_index(time)] - phase)
            .map(ClipTime::from_nanos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteError {
    FrameCount,
    InvalidDuration,
}
impl fmt::Display for SpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FrameCount => "a sprite requires 1..=256 frames",
            Self::InvalidDuration => {
                "cumulative frame ends must be positive and strictly increasing"
            }
        })
    }
}
impl std::error::Error for SpriteError {}
