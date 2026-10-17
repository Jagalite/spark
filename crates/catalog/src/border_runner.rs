//! A restrained traveling highlight over host-owned border glyphs.
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    ClipTime, ColorProfile, Motion, Period, SampleContext,
    surface::{PaintMask, StylePermissions, StyleSurface},
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderRunner {
    period: Period,
    seed: u64,
}
impl Default for BorderRunner {
    fn default() -> Self {
        Self {
            period: Period::new(Duration::from_millis(2800)).unwrap(),
            seed: 0,
        }
    }
}
impl BorderRunner {
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
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> BorderSample<'a> {
        BorderSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct BorderSample<'a> {
    design: &'a BorderRunner,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}
impl<'a> BorderSample<'a> {
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    pub fn change_hint(&self, requested: Size) -> spark_runtime::ChangeHint {
        spark_runtime::ChangeHint::from_interval(self.frame_interval(requested))
    }
    pub fn frame_interval(&self, size: Size) -> Option<Duration> {
        (size.width >= 3
            && size.height >= 3
            && size.width <= 256
            && size.height <= 128
            && self.context.profile.motion != Motion::Static)
            .then_some(Duration::from_millis(
                if self.context.profile.motion == Motion::Reduced {
                    100
                } else {
                    40
                },
            ))
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        // A maximum perimeter bounds all paint work, including pathological requests.
        if area.width < 3 || area.height < 3 || area.width > 256 || area.height > 128 {
            return;
        }
        let w = u32::from(area.width);
        let h = u32::from(area.height);
        let perimeter = 2 * w + 2 * h - 4;
        let profile = self.context.profile;
        let t = match profile.motion {
            Motion::Full => self.context.time,
            Motion::Reduced => ClipTime::from_nanos(self.context.time.as_nanos() / 4),
            Motion::Static => ClipTime::ZERO,
        };
        let head = if profile.motion == Motion::Static {
            w / 2
        } else {
            ((u64::from(self.design.period.step(t, perimeter))
                + self.design.seed % u64::from(perimeter))
                % u64::from(perimeter)) as u32
        };
        let mut surface = StyleSurface::new(
            buffer,
            area,
            self.protected,
            PaintMask::Border,
            StylePermissions::foreground_and_modifiers(),
        );
        let tail = (perimeter / 6).clamp(3, 12);
        // Row-major traversal preserves the writer's linear row-prefix scan.
        for y in 0..area.height {
            for x in 0..area.width {
                let index = if y == 0 {
                    u32::from(x)
                } else if x == area.width - 1 {
                    w - 1 + u32::from(y)
                } else if y == area.height - 1 {
                    w + h - 2 + (w - 1 - u32::from(x))
                } else if x == 0 {
                    perimeter - u32::from(y)
                } else {
                    continue;
                };
                let behind = (head + perimeter - index) % perimeter;
                if behind >= tail {
                    continue;
                }
                let raw = if profile.colors == ColorProfile::Monochrome {
                    if behind < 2 {
                        Style::new().add_modifier(Modifier::BOLD)
                    } else {
                        Style::new()
                    }
                } else {
                    let role = if behind < 2 {
                        self.context.palette.highlight
                    } else if behind < tail / 2 {
                        self.context.palette.accent
                    } else {
                        self.context.palette.foreground
                    };
                    self.context.palette.resolve(role, profile.colors)
                };
                surface.patch(x, y, raw);
            }
        }
    }
}
impl Widget for BorderSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &BorderSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
