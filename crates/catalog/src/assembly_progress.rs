use crate::inputs::Progress;
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    ClipTime, GlyphProfile, Motion, Period, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};
use std::time::Duration;

/// R02 / visual revision 1: a gantry, assembled modules, and a rolling conveyor.
/// Known targets control the module count without time interpolation or overshoot.
/// Unknown progress is an unmistakable question-mark workpiece, never a percentage.
///
/// The full machine fits 12–80 columns × 3 rows. Smaller requests receive a
/// static, truthful meter or point; larger requests center an 80×3 composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssemblyProgress {
    seed: u64,
    period: Period,
}
impl Default for AssemblyProgress {
    fn default() -> Self {
        Self {
            seed: 13,
            period: Period::new(Duration::from_millis(1800)).unwrap(),
        }
    }
}
impl AssemblyProgress {
    pub const MIN_WIDTH: u16 = 12;
    pub const MAX_WIDTH: u16 = 80;
    pub const HEIGHT: u16 = 3;

    pub fn new() -> Self {
        Self::default()
    }
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
    pub const fn with_period(mut self, period: Period) -> Self {
        self.period = period;
        self
    }
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    pub const fn period(&self) -> Period {
        self.period
    }
    pub fn sample<'a>(
        &'a self,
        context: &SampleContext<'a>,
        progress: Progress,
    ) -> AssemblySample<'a> {
        AssemblySample {
            design: self,
            context: *context,
            progress,
            protected: &[],
        }
    }
}

/// Borrowed, stateless render view. Labels and business completion remain host-owned.
#[derive(Debug, Clone, Copy)]
pub struct AssemblySample<'a> {
    design: &'a AssemblyProgress,
    context: SampleContext<'a>,
    progress: Progress,
    protected: &'a [Rect],
}
impl<'a> AssemblySample<'a> {
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    /// Advice only. Complete, static, tiny, and empty samples do not move with time.
    pub fn change_hint(&self, requested: Size) -> spark_runtime::ChangeHint {
        spark_runtime::ChangeHint::from_interval(self.frame_interval(requested))
    }
    pub fn frame_interval(&self, requested: Size) -> Option<Duration> {
        if requested.width < AssemblyProgress::MIN_WIDTH
            || requested.height < AssemblyProgress::HEIGHT
            || self.progress.value() == Some(1.0)
            || self.context.profile.motion == Motion::Static
        {
            None
        } else {
            Some(Duration::from_millis(
                if self.context.profile.motion == Motion::Reduced {
                    200
                } else {
                    80
                },
            ))
        }
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let width = area.width.min(AssemblyProgress::MAX_WIDTH);
        let height = area.height.min(AssemblyProgress::HEIGHT);
        let ox = (area.width - width) / 2;
        let oy = (area.height - height) / 2;
        let profile = self.context.profile;
        let palette = *self.context.palette;
        let style = |style| palette.resolve(style, profile.colors);
        let blank = style(Style::new());
        let muted = style(palette.muted);
        let accent = style(palette.accent);
        let bright = style(palette.highlight.add_modifier(Modifier::BOLD));
        let glyph = |ascii, unicode| {
            Glyph::new(if profile.glyphs == GlyphProfile::Unicode {
                unicode
            } else {
                ascii
            })
            .unwrap()
        };
        let mut surface = OpaqueSurface::new(buffer, area, self.protected);
        if width < AssemblyProgress::MIN_WIDTH || height < AssemblyProgress::HEIGHT {
            let slots = width.saturating_sub(2);
            let filled = self.progress.filled(slots);
            for y in 0..height {
                for x in 0..width {
                    let op = if y != height / 2 {
                        CellOp::Erase(blank)
                    } else if width < 4 {
                        if x == width / 2 {
                            let (ascii, unicode) = match self.progress.value() {
                                None => ('?', '?'),
                                Some(1.0) => ('#', '■'),
                                Some(_) => ('.', '·'),
                            };
                            CellOp::Replace(glyph(ascii, unicode), accent)
                        } else {
                            CellOp::Erase(blank)
                        }
                    } else if x == 0 {
                        CellOp::Replace(glyph('[', '['), muted)
                    } else if x == width - 1 {
                        CellOp::Replace(glyph(']', ']'), muted)
                    } else if filled.is_some_and(|count| x - 1 < count) {
                        CellOp::Replace(glyph('#', '■'), accent)
                    } else if filled.is_none() && x == width / 2 {
                        CellOp::Replace(glyph('?', '?'), bright)
                    } else {
                        CellOp::Replace(glyph('.', '·'), muted)
                    };
                    surface.write(ox + x, oy + y, op);
                }
            }
            return;
        }
        let slots = width - 4;
        let filled = self.progress.filled(slots);
        let still = profile.motion == Motion::Static || self.progress.value() == Some(1.0);
        let time = match profile.motion {
            Motion::Reduced => ClipTime::from_nanos(self.context.time.as_nanos() / 4),
            Motion::Full => self.context.time,
            Motion::Static => ClipTime::ZERO,
        };
        // The carrier can travel above the blueprint, but only the middle-row
        // solid modules represent progress. None of them is driven by clip time.
        let frontier = filled.unwrap_or(slots / 2).min(slots - 1);
        let travel = u32::from(frontier.max(1));
        let phase = if still {
            travel
        } else {
            (self.design.period.step(time, travel * 2)
                + (self.design.seed % u64::from(travel * 2)) as u32)
                % (travel * 2)
        };
        let carrier = 2 + phase.min(travel * 2 - phase) as u16;
        let roller_phase = if still {
            0
        } else {
            (self.design.period.step(time, 4) + (self.design.seed % 4) as u32) % 4
        } as u16;
        for y in 0..height {
            for x in 0..width {
                let op = match y {
                    0 if x == carrier => CellOp::Replace(glyph('T', '┳'), bright),
                    0 if x >= 2 && x < width - 2 => CellOp::Replace(glyph('-', '─'), muted),
                    1 if x == 0 => CellOp::Replace(glyph('>', '╞'), accent),
                    1 if x == width - 1 => CellOp::Replace(glyph('<', '╡'), accent),
                    1 if x == 1 || x == width - 2 => CellOp::Replace(glyph('|', '│'), muted),
                    1 if filled.is_some_and(|count| x - 2 < count) => {
                        CellOp::Replace(glyph('#', '■'), accent)
                    }
                    1 if filled.is_none() && x == width / 2 => {
                        CellOp::Replace(glyph('?', '?'), bright)
                    }
                    1 => CellOp::Replace(glyph('.', '·'), muted),
                    2 if x == 0 => CellOp::Replace(glyph('\\', '╘'), muted),
                    2 if x == width - 1 => CellOp::Replace(glyph('/', '╛'), muted),
                    2 if (x + roller_phase).is_multiple_of(4) => {
                        CellOp::Replace(glyph('o', '○'), accent)
                    }
                    2 => CellOp::Replace(glyph('=', '═'), muted),
                    _ => CellOp::Erase(blank),
                };
                surface.write(ox + x, oy + y, op);
            }
        }
    }
}
impl Widget for AssemblySample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &AssemblySample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
