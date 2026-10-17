//! Pure repaint advice. The host still owns waiting, input, and presentation.
use crate::{ClipTime, HostTime, Playback};
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChangeHint {
    #[default]
    None,
    /// An absolute clip timestamp, never a modulo phase.
    At(ClipTime),
    /// Preferred host presentation cadence, independent of producer cadence.
    Continuous(Duration),
}
impl ChangeHint {
    pub fn from_interval(interval: Option<Duration>) -> Self {
        interval.map_or(Self::None, Self::Continuous)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepaintPlanner {
    minimum_interval: u64,
}
impl RepaintPlanner {
    /// Zero is safely rounded up to one nanosecond. Applications normally cap at
    /// a practical display interval (for example 16 or 33 milliseconds).
    pub fn new(minimum_interval: Duration) -> Self {
        Self {
            minimum_interval: (minimum_interval.as_nanos().min(u128::from(u64::MAX)) as u64).max(1),
        }
    }
    /// Every returned deadline is strictly after now and respects the last-frame
    /// cap. Paused/hidden samples return None but must paint on unrelated redraws.
    pub fn next_deadline(
        self,
        now: HostTime,
        last_presented: HostTime,
        playback: &Playback,
        hint: ChangeHint,
    ) -> Option<HostTime> {
        if !playback.requests_animation() {
            return None;
        }
        let desired = match hint {
            ChangeHint::None => return None,
            ChangeHint::At(clip) => playback.host_for_clip(now, clip)?,
            ChangeHint::Continuous(interval) => {
                let nanos = (interval.as_nanos().min(u128::from(u64::MAX)) as u64).max(1);
                HostTime::from_nanos(last_presented.as_nanos().checked_add(nanos)?)
            }
        };
        let future = now.as_nanos().checked_add(1)?;
        let capped = last_presented
            .as_nanos()
            .checked_add(self.minimum_interval)?;
        Some(HostTime::from_nanos(
            desired.as_nanos().max(future).max(capped),
        ))
    }
    pub fn earliest(deadlines: impl IntoIterator<Item = Option<HostTime>>) -> Option<HostTime> {
        deadlines.into_iter().flatten().min()
    }
}
