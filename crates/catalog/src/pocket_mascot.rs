//! R05: an authored, asymmetric seed-pod creature with deliberate frame holds.
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    widgets::Widget,
};
use spark_runtime::{
    ChangeHint, ClipTime, FrameSequence, Motion, Period, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};
use std::time::Duration;

#[path = "generated/mascot.rs"]
mod asset;

#[derive(Debug, Clone, Copy)]
enum Role {
    Foreground,
    Muted,
    Accent,
    Highlight,
}
#[derive(Debug, Clone, Copy)]
enum AssetCell {
    Keep,
    Erase(Role),
    Replace(char, Role),
}

/// R05 / visual revision 1. A fixed 12×6 creature, centered without stretching.
///
/// Full motion uses eight complete, authored ASCII frames with unequal holds.
/// Reduced motion slows the same gestures fourfold. Static motion uses an
/// attentive poster. Below native size the mascot becomes a centered, static
/// `o-o` face (or a single `o` below three columns). These are visual decorations,
/// never authoritative host status indicators.
#[derive(Debug, Clone, Copy)]
pub struct PocketMascot {
    frames: FrameSequence<'static>,
}
impl Default for PocketMascot {
    fn default() -> Self {
        Self {
            frames: FrameSequence::new(&asset::ENDS_NS).expect("validated generated frame timing"),
        }
    }
}
impl PocketMascot {
    pub fn new() -> Self {
        Self::default()
    }
    /// Full-motion loop duration. Reduced motion uses four times this duration.
    pub fn period(&self) -> Period {
        self.frames.period()
    }
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> MascotSample<'a> {
        MascotSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}

/// A borrowed immutable authored-frame sample, with no screen-sized allocation.
#[derive(Debug, Clone, Copy)]
pub struct MascotSample<'a> {
    design: &'a PocketMascot,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}
impl<'a> MascotSample<'a> {
    /// Host-owned absolute no-write regions, including text, selections, and media.
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    /// Conservative cadence equal to the shortest authored hold. Hosts still own
    /// scheduling; static, compact, and empty views need no time-only redraws.
    pub fn frame_interval(&self, requested: Size) -> Option<Duration> {
        if requested.width < asset::WIDTH
            || requested.height < asset::HEIGHT
            || self.context.profile.motion == Motion::Static
        {
            return None;
        }
        let mut previous = 0;
        let shortest = asset::ENDS_NS
            .iter()
            .map(|&end| {
                let hold = end - previous;
                previous = end;
                hold
            })
            .min()
            .expect("generated frames are nonempty");
        Some(Duration::from_nanos(
            shortest
                * if self.context.profile.motion == Motion::Reduced {
                    4
                } else {
                    1
                },
        ))
    }
    /// Next absolute clip-time frame boundary, including the fourfold reduced
    /// schedule. No deadline is invented after the clip-time range is exhausted.
    pub fn change_hint(&self, requested: Size) -> ChangeHint {
        if requested.width < asset::WIDTH
            || requested.height < asset::HEIGHT
            || self.context.profile.motion == Motion::Static
        {
            return ChangeHint::None;
        }
        let scale = if self.context.profile.motion == Motion::Reduced {
            4
        } else {
            1
        };
        let authored_time = ClipTime::from_nanos(self.context.time.as_nanos() / scale);
        self.design
            .frames
            .next_boundary(authored_time)
            .and_then(|time| time.as_nanos().checked_mul(scale))
            .map(ClipTime::from_nanos)
            .map_or(ChangeHint::None, ChangeHint::At)
    }
    fn style(&self, role: Role) -> Style {
        let palette = self.context.palette;
        let style = match role {
            Role::Foreground => palette.foreground,
            Role::Muted => palette.muted,
            Role::Accent => palette.accent,
            Role::Highlight => palette.highlight,
        };
        palette.resolve(style, self.context.profile.colors)
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let mut surface = OpaqueSurface::new(buffer, area, self.protected);
        if area.width < asset::WIDTH || area.height < asset::HEIGHT {
            let compact: &[char] = if area.width >= 3 {
                &['o', '-', 'o']
            } else {
                &['o']
            };
            let x = (area.width - compact.len() as u16) / 2;
            let y = (area.height - 1) / 2;
            for (i, &glyph) in compact.iter().enumerate() {
                surface.write(
                    x + i as u16,
                    y,
                    CellOp::Replace(Glyph::new(glyph).unwrap(), self.style(Role::Highlight)),
                );
            }
            return;
        }
        let index = match self.context.profile.motion {
            Motion::Static => asset::POSTER,
            Motion::Full => self.design.frames.frame_index(self.context.time),
            Motion::Reduced => self
                .design
                .frames
                .frame_index(ClipTime::from_nanos(self.context.time.as_nanos() / 4)),
        };
        let ox = (area.width - asset::WIDTH) / 2;
        let oy = (area.height - asset::HEIGHT) / 2;
        for (i, cell) in asset::FRAMES[index].iter().enumerate() {
            let operation = match *cell {
                AssetCell::Keep => CellOp::Keep,
                AssetCell::Erase(role) => CellOp::Erase(self.style(role)),
                AssetCell::Replace(glyph, role) => CellOp::Replace(
                    Glyph::new(glyph).expect("generated ASCII glyph"),
                    self.style(role),
                ),
            };
            surface.write(
                ox + i as u16 % asset::WIDTH,
                oy + i as u16 / asset::WIDTH,
                operation,
            );
        }
    }
}
impl Widget for MascotSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &MascotSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
