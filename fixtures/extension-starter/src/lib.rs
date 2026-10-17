#![forbid(unsafe_code)]
//! A copyable downstream design using Spark's public runtime only.
//!
//! Hanging Lantern is decorative: its light does not report progress, health, or
//! completion. The host supplies time, draws fresh base content, and schedules
//! presentation. This library reads no clock and performs no IO.

use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    widgets::Widget,
};
use spark_runtime::{
    ChangeHint, ClipTime, FrameSequence, Motion, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};

/// Native cell geometry. Larger requests center it without stretching.
pub const NATIVE_SIZE: Size = Size::new(13, 7);

// Complete original ASCII poses. A space is an intentional styled erase, not
// transparency. The light grows inside a fixed cage; the silhouette stays quiet.
const EMBER: [&str; 7] = [
    "      |      ",
    "    .-+-.    ",
    "   /     \\   ",
    "   |  .  |   ",
    "   |     |   ",
    "   '-----'   ",
    "     / \\     ",
];
const CORE: [&str; 7] = [
    "      |      ",
    "    .-+-.    ",
    "   /     \\   ",
    "   |  *  |   ",
    "   |     |   ",
    "   '-----'   ",
    "     / \\     ",
];
const RAYS: [&str; 7] = [
    "      |      ",
    "    .-+-.    ",
    "   /     \\   ",
    "   | -*- |   ",
    "   |     |   ",
    "   '-----'   ",
    "     / \\     ",
];
const OPEN: [&str; 7] = [
    "      |      ",
    "    .-+-.    ",
    "   /  '  \\   ",
    "   | -*- |   ",
    "   |  .  |   ",
    "   '-----'   ",
    "     / \\     ",
];
const POSES: [[&str; 7]; 6] = [EMBER, CORE, RAYS, OPEN, RAYS, CORE];
const FRAME_ENDS_NS: [u64; 6] = [
    240_000_000,
    360_000_000,
    520_000_000,
    840_000_000,
    1_000_000_000,
    1_120_000_000,
];
const POSTER: usize = 3;

/// Immutable authored configuration, cheap to store and reuse.
///
/// The 1.12-second loop has six unequal holds. Reduced motion runs at one
/// quarter speed. Static motion uses the open-light poster. A request smaller
/// than 13x7 paints one centered, stationary `*` instead, with no timer hint.
#[derive(Debug, Clone, Copy)]
pub struct HangingLantern {
    sequence: FrameSequence<'static>,
}

impl Default for HangingLantern {
    fn default() -> Self {
        Self {
            sequence: FrameSequence::new(&FRAME_ENDS_NS)
                .expect("authored frame ends are positive and increasing"),
        }
    }
}

impl HangingLantern {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sampling borrows configuration and palette/profile; it advances nothing.
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> LanternSample<'a> {
        LanternSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}

/// A temporary widget and its repaint advice. Paint by value or shared reference.
#[derive(Debug, Clone, Copy)]
pub struct LanternSample<'a> {
    design: &'a HangingLantern,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}

impl<'a> LanternSample<'a> {
    /// Absolute host-buffer rectangles, not coordinates relative to the art.
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }

    fn authored_time(&self) -> ClipTime {
        match self.context.profile.motion {
            Motion::Reduced => ClipTime::from_nanos(self.context.time.as_nanos() / 4),
            Motion::Full | Motion::Static => self.context.time,
        }
    }

    /// An absolute future boundary in the supplied clip's time domain.
    ///
    /// The host may pass this to `RepaintPlanner`, which also handles playback
    /// pause/visibility and presentation caps. It is advice, never a timer.
    /// Callers suppress advice for fully clipped/occluded art themselves: this
    /// size-only query cannot know the buffer or visibility.
    pub fn change_hint(&self, size: Size) -> ChangeHint {
        if !fits_native(size) || self.context.profile.motion == Motion::Static {
            return ChangeHint::None;
        }
        let boundary = self.design.sequence.next_boundary(self.authored_time());
        let boundary = if self.context.profile.motion == Motion::Reduced {
            boundary.and_then(|time| time.as_nanos().checked_mul(4).map(ClipTime::from_nanos))
        } else {
            boundary
        };
        boundary.map_or(ChangeHint::None, ChangeHint::At)
    }

    fn paint(self, requested: Rect, buffer: &mut Buffer) {
        if requested.is_empty() {
            return;
        }
        let palette = *self.context.palette;
        let colors = self.context.profile.colors;
        let mut surface = OpaqueSurface::new(buffer, requested, self.protected);

        if !fits_native(requested.as_size()) {
            surface.write(
                requested.width / 2,
                requested.height / 2,
                CellOp::Replace(
                    Glyph::new('*').expect("one-column ASCII"),
                    palette.resolve(palette.highlight, colors),
                ),
            );
            return;
        }

        let pose = if self.context.profile.motion == Motion::Static {
            POSTER
        } else {
            self.design.sequence.frame_index(self.authored_time())
        };
        let ox = (requested.width - NATIVE_SIZE.width) / 2;
        let oy = (requested.height - NATIVE_SIZE.height) / 2;
        // Row-major order lets the scoped writer reuse its wide-cell scan.
        // Only these 91 native cells are visited, regardless of requested size.
        for (y, row) in POSES[pose].iter().enumerate() {
            for (x, symbol) in row.chars().enumerate() {
                let role = if x == 6 && y == 3 {
                    palette.highlight
                } else if (4..=8).contains(&x) && (2..=4).contains(&y) {
                    palette.accent
                } else {
                    palette.foreground
                };
                let operation = if symbol == ' ' {
                    CellOp::Erase(palette.resolve(Style::new(), colors))
                } else {
                    CellOp::Replace(
                        Glyph::new(symbol).expect("authored printable one-column ASCII"),
                        palette.resolve(role, colors),
                    )
                };
                surface.write(ox + x as u16, oy + y as u16, operation);
            }
        }
    }
}

fn fits_native(size: Size) -> bool {
    size.width >= NATIVE_SIZE.width && size.height >= NATIVE_SIZE.height
}

impl Widget for LanternSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}

impl Widget for &LanternSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_poses_have_complete_safe_native_coverage() {
        for pose in POSES {
            assert_eq!(pose.len(), usize::from(NATIVE_SIZE.height));
            for row in pose {
                assert_eq!(row.len(), usize::from(NATIVE_SIZE.width));
                assert!(row.is_ascii());
                assert!(row.chars().all(|symbol| Glyph::new(symbol).is_some()));
            }
        }
    }
}
