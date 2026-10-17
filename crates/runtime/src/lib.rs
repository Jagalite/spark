#![forbid(unsafe_code)]
//! Small explicit-time painting building blocks. No clocks, IO, threads, or scheduler.
//! Hosts own layout, base content, invalidation, and presentation.

pub mod composition;
pub mod palette;
pub mod playback;
pub mod profile;
pub mod repaint;
pub mod sprite;
pub mod surface;
pub mod time;

pub use palette::Palette;
pub use playback::{HiddenPolicy, HostTime, Playback, PlaybackError, PlaybackRate};
pub use profile::{ColorProfile, GlyphProfile, Motion, Profile};
pub use repaint::{ChangeHint, RepaintPlanner};
pub use sprite::{FrameSequence, SpriteError};
pub use time::{ClipTime, FiniteClip, FiniteSample, Period, TimeError};

/// All semantic inputs for one sample. Creating this does not advance anything.
#[derive(Debug, Clone, Copy)]
pub struct SampleContext<'a> {
    pub time: ClipTime,
    pub palette: &'a Palette,
    pub profile: &'a Profile,
}
impl<'a> SampleContext<'a> {
    pub const fn new(time: ClipTime, palette: &'a Palette, profile: &'a Profile) -> Self {
        Self {
            time,
            palette,
            profile,
        }
    }
}
