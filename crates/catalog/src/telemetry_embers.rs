use crate::inputs::Level;
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    GlyphProfile, Motion, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};
use std::time::Duration;

/// R04 / visual revision 1: sparse rising embers with a quiet, low hearth.
/// Level controls density and brightness; the host owns its meaning and cadence.
/// Every indexed particle is sampled analytically, with no simulation backlog.
///
/// Adapts from 8×4 to a bounded 120×40 scene. Smaller requests use a static
/// level-dependent point. Larger requests center the capped scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryEmbers {
    seed: u64,
}
impl Default for TelemetryEmbers {
    fn default() -> Self {
        Self { seed: 29 }
    }
}
impl TelemetryEmbers {
    pub const MIN_WIDTH: u16 = 8;
    pub const MIN_HEIGHT: u16 = 4;
    pub const MAX_WIDTH: u16 = 120;
    pub const MAX_HEIGHT: u16 = 40;
    pub const MAX_PARTICLES: usize = 64;

    pub fn new() -> Self {
        Self::default()
    }
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>, level: Level) -> EmbersSample<'a> {
        EmbersSample {
            design: self,
            context: *context,
            level,
            protected: &[],
        }
    }
}

/// A cheap borrowed sample. Calling it again cannot consume or accumulate telemetry.
#[derive(Debug, Clone, Copy)]
pub struct EmbersSample<'a> {
    design: &'a TelemetryEmbers,
    context: SampleContext<'a>,
    level: Level,
    protected: &'a [Rect],
}
impl<'a> EmbersSample<'a> {
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    /// Advice only; zero-level, tiny, empty, and static presentations do not move.
    pub fn change_hint(&self, requested: Size) -> spark_runtime::ChangeHint {
        spark_runtime::ChangeHint::from_interval(self.frame_interval(requested))
    }
    pub fn frame_interval(&self, requested: Size) -> Option<Duration> {
        if requested.width < TelemetryEmbers::MIN_WIDTH
            || requested.height < TelemetryEmbers::MIN_HEIGHT
            || self.level == Level::ZERO
            || self.context.profile.motion == Motion::Static
        {
            None
        } else {
            Some(Duration::from_millis(
                if self.context.profile.motion == Motion::Reduced {
                    150
                } else {
                    50
                },
            ))
        }
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let width = area.width.min(TelemetryEmbers::MAX_WIDTH);
        let height = area.height.min(TelemetryEmbers::MAX_HEIGHT);
        let ox = (area.width - width) / 2;
        let oy = (area.height - height) / 2;
        let profile = self.context.profile;
        let palette = *self.context.palette;
        let style = |style| palette.resolve(style, profile.colors);
        let styles = [
            style(Style::new()),
            style(palette.muted),
            style(palette.foreground),
            style(palette.accent),
            style(palette.highlight.add_modifier(Modifier::BOLD)),
        ];
        let glyph = |ascii, unicode| {
            Glyph::new(if profile.glyphs == GlyphProfile::Unicode {
                unicode
            } else {
                ascii
            })
            .unwrap()
        };
        let mut surface = OpaqueSurface::new(buffer, area, self.protected);
        if width < TelemetryEmbers::MIN_WIDTH || height < TelemetryEmbers::MIN_HEIGHT {
            let (ascii, unicode, intensity) = match self.level.value() {
                0.0 => ('.', '·', 1),
                value if value < 0.34 => ('.', '·', 2),
                value if value < 0.67 => ('*', '✦', 3),
                _ => ('*', '◆', 4),
            };
            for y in 0..height {
                for x in 0..width {
                    let op = if x == width / 2 && y == height / 2 {
                        CellOp::Replace(glyph(ascii, unicode), styles[intensity])
                    } else {
                        CellOp::Erase(styles[0])
                    };
                    surface.write(ox + x, oy + y, op);
                }
            }
            return;
        }
        // Fixed, bounded stack scratch stores only intensity, not Ratatui cells.
        // Row-major final writes preserve the scoped writer's forward-scan path.
        let mut field =
            [0_u8; TelemetryEmbers::MAX_WIDTH as usize * TelemetryEmbers::MAX_HEIGHT as usize];
        let capacity = (usize::from(width) * usize::from(height) / 12)
            .clamp(1, TelemetryEmbers::MAX_PARTICLES);
        let count = if self.level == Level::ZERO {
            0
        } else {
            1 + (self.level.value() * (capacity - 1) as f64).floor() as usize
        };
        let time = match profile.motion {
            Motion::Full => self.context.time.as_nanos(),
            Motion::Reduced => self.context.time.as_nanos() / 4,
            Motion::Static => 0,
        };
        for index in 0..count {
            let seed = mix(self.design.seed.wrapping_add(index as u64));
            let life = 3_500_000_000 + seed % 4_500_000_000;
            let offset = mix(seed) % life;
            let age = ((u128::from(time % life) + u128::from(offset)) % u128::from(life)) as u64;
            // Three seeded positions form a soft central plume. A bounded
            // triangular drift gives each particle a distinct direct-time path.
            let span = u64::from(width - 2);
            let base = 1 + (seed % span + (seed >> 16) % span + (seed >> 32) % span) / 3;
            let quarter = ((u128::from(age) * 256) / u128::from(life)) as i32;
            let wave = if quarter < 128 {
                quarter
            } else {
                256 - quarter
            } - 64;
            let amplitude = i32::from((width / 12).clamp(1, 5));
            let direction = if seed & 1 == 0 { 1 } else { -1 };
            let x = (base as i32 + wave * amplitude * direction / 64).clamp(1, i32::from(width - 2))
                as u16;
            let rise = ((u128::from(age) * u128::from(height - 1)) / u128::from(life)) as u16;
            let y = height - 2 - rise.min(height - 2);
            let intensity = if age > life * 3 / 4 {
                2
            } else if self.level.value() >= 0.67 && index % 3 == 0 {
                4
            } else {
                3
            };
            let cell = usize::from(y) * usize::from(width) + usize::from(x);
            field[cell] = field[cell].max(intensity);
            if profile.motion == Motion::Full && intensity >= 3 && y + 1 < height - 1 {
                let tail = cell + usize::from(width);
                field[tail] = field[tail].max(1);
            }
        }
        // The low hearth is decorative, never a scale or fabricated measurement.
        for x in width / 3..width - width / 3 {
            if x % 2 == 0 {
                field[usize::from(height - 1) * usize::from(width) + usize::from(x)] = 1;
            }
        }
        for y in 0..height {
            for x in 0..width {
                let intensity = field[usize::from(y) * usize::from(width) + usize::from(x)];
                let op = match intensity {
                    0 => CellOp::Erase(styles[0]),
                    1 => CellOp::Replace(glyph('.', '·'), styles[1]),
                    2 => CellOp::Replace(glyph('.', '∙'), styles[2]),
                    3 => CellOp::Replace(glyph('*', '✦'), styles[3]),
                    _ => CellOp::Replace(glyph('+', '◆'), styles[4]),
                };
                surface.write(ox + x, oy + y, op);
            }
        }
    }
}
impl Widget for EmbersSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &EmbersSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}
