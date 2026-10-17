#![forbid(unsafe_code)]
//! Independent F21 extension exercise. A two-axis woven lattice, using only
//! spark-runtime's public contract and Ratatui core. Not a seventh catalog entry.
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    widgets::Widget,
};
use spark_runtime::{
    ChangeHint, ClipTime, FrameSequence, GlyphProfile, Motion, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};

const ENDS_NS: [u64; 8] = [
    80_000_000,
    160_000_000,
    240_000_000,
    360_000_000,
    440_000_000,
    520_000_000,
    600_000_000,
    800_000_000,
];

/// A small interlaced textile motif. Fixed at most 17×7, centered without
/// stretching; requested dimensions below that crop the authored lattice.
/// Fewer than three columns or rows use a static, centered stitch.
#[derive(Debug, Clone, Copy)]
pub struct CrossWeave {
    sequence: FrameSequence<'static>,
    seed: u64,
}
impl Default for CrossWeave {
    fn default() -> Self {
        Self {
            sequence: FrameSequence::new(&ENDS_NS).expect("positive authored holds"),
            seed: 0,
        }
    }
}
impl CrossWeave {
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> WeaveSample<'a> {
        WeaveSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WeaveSample<'a> {
    design: &'a CrossWeave,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}
impl<'a> WeaveSample<'a> {
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    fn clip_time(&self) -> ClipTime {
        match self.context.profile.motion {
            Motion::Full => self.context.time,
            Motion::Reduced => ClipTime::from_nanos(self.context.time.as_nanos() / 4),
            Motion::Static => ClipTime::ZERO,
        }
    }
    pub fn change_hint(&self, size: Size) -> ChangeHint {
        if size.width < 3 || size.height < 3 || self.context.profile.motion == Motion::Static {
            return ChangeHint::None;
        }
        let boundary = self.design.sequence.next_boundary(self.clip_time());
        let absolute = match self.context.profile.motion {
            Motion::Full => boundary,
            Motion::Reduced => {
                boundary.and_then(|time| time.as_nanos().checked_mul(4).map(ClipTime::from_nanos))
            }
            Motion::Static => None,
        };
        absolute.map_or(ChangeHint::None, ChangeHint::At)
    }
    fn paint(self, requested: Rect, buffer: &mut Buffer) {
        if requested.is_empty() {
            return;
        }
        let width = requested.width.min(17);
        let height = requested.height.min(7);
        let ox = (requested.width - width) / 2;
        let oy = (requested.height - height) / 2;
        let palette = *self.context.palette;
        let profile = self.context.profile;
        let resolve = |style| palette.resolve(style, profile.colors);
        let phase = (self.design.sequence.frame_index(self.clip_time()) as u16
            + (self.design.seed % 8) as u16)
            % 8;
        let mut surface = OpaqueSurface::new(buffer, requested, self.protected);
        for y in 0..height {
            for x in 0..width {
                let symbol = if width < 3 || height < 3 {
                    (x == width / 2 && y == height / 2).then_some(('*', palette.highlight))
                } else {
                    let warp = (x + phase).is_multiple_of(8);
                    let weft = (x + 2 * y + 8 - phase).is_multiple_of(8);
                    match (warp, weft, profile.glyphs) {
                        (true, true, GlyphProfile::Unicode) => Some(('◆', palette.highlight)),
                        (true, true, GlyphProfile::Ascii) => Some(('+', palette.highlight)),
                        (true, false, _) => Some(('|', palette.accent)),
                        (false, true, _) => Some(('/', palette.foreground)),
                        _ => None,
                    }
                };
                let operation = symbol.map_or_else(
                    || CellOp::Erase(resolve(Style::new())),
                    |(glyph, style)| CellOp::Replace(Glyph::new(glyph).unwrap(), resolve(style)),
                );
                surface.write(ox + x, oy + y, operation);
            }
        }
    }
}
impl Widget for WeaveSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &WeaveSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
