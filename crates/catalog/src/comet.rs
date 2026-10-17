use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use spark_runtime::{
    ClipTime, GlyphProfile, Motion, Period, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};
use std::time::Duration;

/// R01 / visual revision 1: a segmented spine, luminous head, and detached sparks.
/// Adaptive at 8–256 columns; narrower requests use a quiet point. At larger
/// sizes a 256×3 strip is centered. Sampling never changes this configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentedComet {
    period: Period,
    seed: u64,
}
impl Default for SegmentedComet {
    fn default() -> Self {
        Self {
            period: Period::new(Duration::from_millis(1600)).unwrap(),
            seed: 7,
        }
    }
}
impl SegmentedComet {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_period(mut self, period: Period) -> Self {
        self.period = period;
        self
    }
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
    pub const fn period(&self) -> Period {
        self.period
    }
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> CometSample<'a> {
        CometSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}

/// A cheap borrowed render view; no intermediate screen or hidden playback state.
#[derive(Debug, Clone, Copy)]
pub struct CometSample<'a> {
    design: &'a SegmentedComet,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}
impl<'a> CometSample<'a> {
    /// Owned, exactly `width` columns (including spaces), capped at 256. This
    /// convenience adapter allocates a one-row buffer and owned spans; it is not
    /// the zero-allocation widget path. Zero width returns an empty line.
    pub fn inline(&self, width: u16) -> Option<Line<'static>> {
        if width > 256 {
            return None;
        }
        let area = Rect::new(0, 0, width, 1);
        let mut buffer = Buffer::empty(area);
        // Inline output has its own owned row; absolute host exclusions do not apply.
        let mut sample = *self;
        sample.protected = &[];
        sample.paint(area, &mut buffer);
        Some(Line::from(
            buffer
                .content
                .iter()
                .map(|cell| Span::styled(cell.symbol().to_owned(), cell.style()))
                .collect::<Vec<_>>(),
        ))
    }
    /// Host-supplied absolute no-write regions (text, caret, selections, media).
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    /// Minimal advisory cadence for this slice; not a timer or deadline planner.
    /// Static/tiny/empty presentations never need time-only redraws.
    pub fn change_hint(&self, requested: Size) -> spark_runtime::ChangeHint {
        spark_runtime::ChangeHint::from_interval(self.frame_interval(requested))
    }
    pub fn frame_interval(&self, requested: Size) -> Option<Duration> {
        if requested.width < 8
            || requested.height == 0
            || self.context.profile.motion == Motion::Static
        {
            None
        } else {
            Some(Duration::from_millis(
                if self.context.profile.motion == Motion::Reduced {
                    100
                } else {
                    33
                },
            ))
        }
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let width = area.width.min(256);
        let height = area.height.min(3);
        let ox = (area.width - width) / 2;
        let oy = (area.height - height) / 2;
        let cy = height / 2;
        let profile = self.context.profile;
        let time = match profile.motion {
            Motion::Full => self.context.time,
            Motion::Reduced => ClipTime::from_nanos(self.context.time.as_nanos() / 4),
            Motion::Static => ClipTime::ZERO,
        };
        let head = if width < 8 {
            width / 2
        } else if profile.motion == Motion::Static {
            width * 3 / 4
        } else {
            ((u64::from(self.design.period.step(time, u32::from(width)))
                + self.design.seed % u64::from(width))
                % u64::from(width)) as u16
        };
        let palette = *self.context.palette;
        let style = |s: Style| palette.resolve(s, profile.colors);
        let blank = style(Style::new());
        let muted = style(palette.muted);
        let accent = style(palette.accent);
        let bright = style(palette.highlight.add_modifier(Modifier::BOLD));
        let faint = style(palette.foreground);
        let unicode = profile.glyphs == GlyphProfile::Unicode;
        let glyph = |ascii: char, uni: char| Glyph::new(if unicode { uni } else { ascii }).unwrap();
        let mut surface = OpaqueSurface::new(buffer, area, self.protected);
        for y in 0..height {
            for x in 0..width {
                let behind = (head + width - x) % width;
                let tail = (width / 2).clamp(4, 14);
                let op = if width < 8 {
                    if y == cy && x == head {
                        CellOp::Replace(glyph('*', '◆'), bright)
                    } else {
                        CellOp::Erase(blank)
                    }
                } else if y == cy {
                    match behind {
                        0 => CellOp::Replace(glyph('>', '◆'), bright),
                        1..=2 => CellOp::Replace(glyph('=', '━'), bright),
                        d if d < tail && d % 3 != 0 => CellOp::Replace(
                            glyph('-', '━'),
                            if d < tail / 2 { accent } else { faint },
                        ),
                        d if d < tail => CellOp::Replace(glyph('.', '·'), muted),
                        _ if x % 4 == 0 => CellOp::Replace(glyph('.', '·'), muted),
                        _ => CellOp::Erase(blank),
                    }
                } else if profile.motion == Motion::Full
                    && height == 3
                    && behind > 2
                    && behind < tail
                    && (mix(self.design.seed.wrapping_add(u64::from(behind))) % 5
                        == u64::from(y % 2))
                {
                    CellOp::Replace(
                        glyph('.', '·'),
                        if behind < tail / 2 { accent } else { muted },
                    )
                } else {
                    CellOp::Erase(blank)
                };
                surface.write(ox + x, oy + y, op);
            }
        }
    }
}
impl Widget for CometSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &CometSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}
