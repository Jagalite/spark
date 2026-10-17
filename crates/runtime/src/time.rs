//! Integer clip time. Reduction happens before any conversion or multiplication.
use std::{fmt, num::NonZeroU64, time::Duration};

/// A timestamp in a clip, in nanoseconds. Samples may be requested in any order.
/// Range: zero through `u64::MAX` nanoseconds (about 584 years).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClipTime(u64);
impl ClipTime {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
    pub const fn as_duration(self) -> Duration {
        Duration::from_nanos(self.0)
    }
    /// Explicit saturation, useful when converting a long-running host clock.
    pub fn saturating_from_duration(duration: Duration) -> Self {
        Self(duration.as_nanos().min(u64::MAX as u128) as u64)
    }
}
impl TryFrom<Duration> for ClipTime {
    type Error = TimeError;
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        u64::try_from(value.as_nanos())
            .map(Self)
            .map_err(|_| TimeError::TooLarge)
    }
}

/// A strictly positive loop duration within the clip-time range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period(NonZeroU64);
impl Period {
    pub fn new(duration: Duration) -> Result<Self, TimeError> {
        let time = ClipTime::try_from(duration)?;
        NonZeroU64::new(time.0)
            .map(Self)
            .ok_or(TimeError::ZeroPeriod)
    }
    pub const fn as_nanos(self) -> u64 {
        self.0.get()
    }
    pub const fn as_duration(self) -> Duration {
        Duration::from_nanos(self.0.get())
    }
    pub fn phase_nanos(self, time: ClipTime) -> u64 {
        time.0 % self.0.get()
    }
    /// Exact index into `steps`, even at the maximum timestamp or period.
    pub fn step(self, time: ClipTime, steps: u32) -> u32 {
        ((self.phase_nanos(time) as u128 * steps as u128) / self.0.get() as u128) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    ZeroPeriod,
    TooLarge,
}
impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroPeriod => "a period must be positive",
            Self::TooLarge => "duration exceeds u64::MAX nanoseconds",
        })
    }
}
impl std::error::Error for TimeError {}

/// A finite visual clamps to its final pose; completion is queryable, never an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiniteClip(Period);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiniteSample {
    pub time: ClipTime,
    pub complete: bool,
}
impl FiniteClip {
    pub const fn new(duration: Period) -> Self {
        Self(duration)
    }
    pub const fn duration(self) -> Period {
        self.0
    }
    pub fn sample(self, time: ClipTime) -> FiniteSample {
        FiniteSample {
            time: ClipTime::from_nanos(time.as_nanos().min(self.0.as_nanos())),
            complete: time.as_nanos() >= self.0.as_nanos(),
        }
    }
    /// Exact finite position in 0..=steps, including the settled terminal frame.
    pub fn step(self, time: ClipTime, steps: u32) -> u32 {
        ((self.sample(time).time.as_nanos() as u128 * u128::from(steps))
            / u128::from(self.0.as_nanos())) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_values() {
        assert_eq!(Period::new(Duration::ZERO), Err(TimeError::ZeroPeriod));
        assert_eq!(ClipTime::try_from(Duration::MAX), Err(TimeError::TooLarge));
        assert_eq!(Period::new(Duration::MAX), Err(TimeError::TooLarge));
        assert_eq!(
            ClipTime::saturating_from_duration(Duration::MAX),
            ClipTime::MAX
        );
    }
    #[test]
    fn exact_boundaries_and_extreme_arithmetic() {
        let p = Period::new(Duration::from_nanos(10)).unwrap();
        for (t, step) in [(0, 0), (9, 8), (10, 0), (11, 0), (19, 8)] {
            assert_eq!(p.step(ClipTime::from_nanos(t), 9), step);
        }
        let p = Period::new(Duration::from_nanos(u64::MAX)).unwrap();
        assert_eq!(p.step(ClipTime::MAX, u32::MAX), 0);
        assert_eq!(
            p.step(ClipTime::from_nanos(u64::MAX - 1), u32::MAX),
            u32::MAX - 1
        );
        assert_eq!(p.step(ClipTime::MAX, 0), 0);
    }
}
