//! Optional host-owned elapsed-time mapping. No clock reads or scheduling.
use crate::ClipTime;
use std::{fmt, num::NonZeroU32, time::Duration};

/// A monotonic elapsed timestamp supplied by the host, separate from clip time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostTime(u64);
impl HostTime {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);
    pub const fn from_nanos(value: u64) -> Self {
        Self(value)
    }
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
    pub fn saturating_from_duration(value: Duration) -> Self {
        Self(value.as_nanos().min(u128::from(u64::MAX)) as u64)
    }
}

/// Positive rational speed. Pause is a separate command, never a zero rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackRate {
    numerator: NonZeroU32,
    denominator: NonZeroU32,
}
impl PlaybackRate {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, PlaybackError> {
        if numerator == 0 || denominator == 0 {
            return Err(PlaybackError::ZeroRate);
        }
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Ok(Self {
            numerator: NonZeroU32::new(numerator / a).unwrap(),
            denominator: NonZeroU32::new(denominator / a).unwrap(),
        })
    }
    pub const fn numerator(self) -> u32 {
        self.numerator.get()
    }
    pub const fn denominator(self) -> u32 {
        self.denominator.get()
    }
}
impl Default for PlaybackRate {
    fn default() -> Self {
        Self::new(1, 1).unwrap()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HiddenPolicy {
    #[default]
    Freeze,
    CatchUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Playback {
    anchor_host: HostTime,
    anchor_clip: ClipTime,
    last_command: HostTime,
    rate: PlaybackRate,
    playing: bool,
    hidden: bool,
    hidden_policy: HiddenPolicy,
}
impl Playback {
    /// Starts playing at clip zero. Hidden behavior defaults to Freeze.
    pub fn new(host: HostTime) -> Self {
        Self {
            anchor_host: host,
            anchor_clip: ClipTime::ZERO,
            last_command: host,
            rate: PlaybackRate::default(),
            playing: true,
            hidden: false,
            hidden_policy: HiddenPolicy::Freeze,
        }
    }
    pub const fn rate(&self) -> PlaybackRate {
        self.rate
    }
    pub const fn is_playing(&self) -> bool {
        self.playing
    }
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub const fn hidden_policy(&self) -> HiddenPolicy {
        self.hidden_policy
    }
    fn advancing(&self) -> bool {
        self.playing && (!self.hidden || self.hidden_policy == HiddenPolicy::CatchUp)
    }
    pub fn requests_animation(&self) -> bool {
        self.playing && !self.hidden
    }
    /// Sampling does not mutate anchors. Before-anchor reads return the anchor
    /// pose; arbitrary backwards seeking uses seek with a current host timestamp.
    pub fn sample(&self, host: HostTime) -> ClipTime {
        if !self.advancing() {
            return self.anchor_clip;
        }
        let elapsed = host.0.saturating_sub(self.anchor_host.0);
        let advanced = u128::from(elapsed) * u128::from(self.rate.numerator())
            / u128::from(self.rate.denominator());
        ClipTime::from_nanos(
            (u128::from(self.anchor_clip.as_nanos()) + advanced).min(u128::from(u64::MAX)) as u64,
        )
    }
    fn validate_command(&mut self, host: HostTime) -> Result<(), PlaybackError> {
        if host < self.last_command {
            return Err(PlaybackError::BackwardsHostTime);
        }
        self.last_command = host;
        Ok(())
    }
    fn reanchor(&mut self, host: HostTime) -> Result<(), PlaybackError> {
        self.validate_command(host)?;
        self.anchor_clip = self.sample(host);
        self.anchor_host = host;
        self.last_command = host;
        Ok(())
    }
    pub fn pause(&mut self, host: HostTime) -> Result<(), PlaybackError> {
        if !self.playing {
            return self.validate_command(host);
        }
        self.reanchor(host)?;
        self.playing = false;
        Ok(())
    }
    pub fn resume(&mut self, host: HostTime) -> Result<(), PlaybackError> {
        if self.playing {
            return self.validate_command(host);
        }
        self.reanchor(host)?;
        self.playing = true;
        Ok(())
    }
    pub fn seek(&mut self, host: HostTime, clip: ClipTime) -> Result<(), PlaybackError> {
        self.reanchor(host)?;
        self.anchor_clip = clip;
        Ok(())
    }
    pub fn restart(&mut self, host: HostTime) -> Result<(), PlaybackError> {
        self.seek(host, ClipTime::ZERO)?;
        self.playing = true;
        Ok(())
    }
    pub fn set_rate(&mut self, host: HostTime, rate: PlaybackRate) -> Result<(), PlaybackError> {
        if self.rate == rate {
            return self.validate_command(host);
        }
        self.reanchor(host)?;
        self.rate = rate;
        Ok(())
    }
    pub fn set_hidden(&mut self, host: HostTime, hidden: bool) -> Result<(), PlaybackError> {
        if self.hidden == hidden {
            return self.validate_command(host);
        }
        self.reanchor(host)?;
        self.hidden = hidden;
        Ok(())
    }
    pub fn set_hidden_policy(
        &mut self,
        host: HostTime,
        policy: HiddenPolicy,
    ) -> Result<(), PlaybackError> {
        if self.hidden_policy == policy {
            return self.validate_command(host);
        }
        self.reanchor(host)?;
        self.hidden_policy = policy;
        Ok(())
    }
    /// Convert a future absolute clip boundary using ceiling division, preventing
    /// early repeated wakes at fractional-rate boundaries. None if unreachable.
    pub fn host_for_clip(&self, now: HostTime, clip: ClipTime) -> Option<HostTime> {
        if !self.requests_animation() {
            return None;
        }
        if clip <= self.sample(now) {
            return Some(now);
        }
        // Use the anchor, rather than rounded current clip time, to retain phase.
        let delta = clip.as_nanos().checked_sub(self.anchor_clip.as_nanos())?;
        let scaled = u128::from(delta) * u128::from(self.rate.denominator());
        let host_delta = scaled.div_ceil(u128::from(self.rate.numerator()));
        let value = u128::from(self.anchor_host.0) + host_delta;
        u64::try_from(value)
            .ok()
            .map(HostTime)
            .map(|time| time.max(now))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackError {
    ZeroRate,
    BackwardsHostTime,
}
impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroRate => "playback rate numerator and denominator must be positive",
            Self::BackwardsHostTime => "playback commands require nondecreasing host time",
        })
    }
}
impl std::error::Error for PlaybackError {}
