//! Finite authored-text reveal, with conservative style-only Unicode fallback.
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    FiniteClip, Motion, Period, SampleContext,
    surface::{CellOp, Glyph, MaskedSurface, PaintMask, StylePermissions, StyleSurface},
};
use std::{fmt, time::Duration};

pub const MAX_TEXT_SCALARS: usize = 256;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrismReveal {
    text: String,
    display_width: u16,
    glyphs: Option<Vec<Glyph>>,
    clip: FiniteClip,
}
impl PrismReveal {
    /// At most 256 non-control scalars. Unsupported widths preserve host glyphs.
    /// For the style fallback, draw the original text before applying this sample.
    pub fn new(text: &str) -> Result<Self, RevealError> {
        if text.len() > MAX_TEXT_SCALARS * 4 || text.chars().count() > MAX_TEXT_SCALARS {
            return Err(RevealError::TooLong);
        }
        if text.chars().any(char::is_control) {
            return Err(RevealError::Control);
        }
        Ok(Self {
            text: text.to_owned(),
            display_width: ratatui_core::text::Line::raw(text)
                .width()
                .min(MAX_TEXT_SCALARS) as u16,
            glyphs: text.chars().map(Glyph::new).collect(),
            clip: FiniteClip::new(Period::new(Duration::from_millis(1200)).unwrap()),
        })
    }
    pub fn with_duration(mut self, duration: Period) -> Self {
        self.clip = FiniteClip::new(duration);
        self
    }
    pub const fn duration(&self) -> Period {
        self.clip.duration()
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub const fn uses_style_fallback(&self) -> bool {
        self.glyphs.is_none()
    }
    pub fn sample<'a>(&'a self, context: &SampleContext<'a>) -> RevealSample<'a> {
        RevealSample {
            design: self,
            context: *context,
            protected: &[],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealError {
    TooLong,
    Control,
}
impl fmt::Display for RevealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLong => "reveal text is limited to 256 scalars",
            Self::Control => "reveal text cannot contain control characters",
        })
    }
}
impl std::error::Error for RevealError {}

#[derive(Debug, Clone, Copy)]
pub struct RevealSample<'a> {
    design: &'a PrismReveal,
    context: SampleContext<'a>,
    protected: &'a [Rect],
}
impl<'a> RevealSample<'a> {
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    pub fn complete(&self) -> bool {
        self.context.profile.motion == Motion::Static
            || self.design.clip.sample(self.context.time).complete
    }
    pub fn change_hint(&self, requested: Size) -> spark_runtime::ChangeHint {
        let Some(interval) = self.frame_interval(requested) else {
            return spark_runtime::ChangeHint::None;
        };
        let remaining = self
            .design
            .duration()
            .as_nanos()
            .saturating_sub(self.context.time.as_nanos());
        if remaining <= interval.as_nanos() as u64 {
            spark_runtime::ChangeHint::At(spark_runtime::ClipTime::from_nanos(
                self.design.duration().as_nanos(),
            ))
        } else {
            spark_runtime::ChangeHint::Continuous(interval)
        }
    }
    pub fn frame_interval(&self, size: Size) -> Option<Duration> {
        (size.width > 0 && size.height > 0 && self.design.display_width > 0 && !self.complete())
            .then_some(Duration::from_millis(
                if self.context.profile.motion == Motion::Reduced {
                    80
                } else {
                    33
                },
            ))
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let profile = self.context.profile;
        let palette = *self.context.palette;
        let final_style = palette.resolve(palette.foreground, profile.colors);
        let highlight = palette.resolve(
            palette.highlight.add_modifier(Modifier::BOLD),
            profile.colors,
        );
        let accent = palette.resolve(palette.accent, profile.colors);
        let complete = self.complete();
        if let Some(glyphs) = &self.design.glyphs {
            let count = glyphs.len() as u32;
            // Two-column anticipation, a narrow sweep, then a stable final pose.
            let position = self.design.clip.step(self.context.time, count + 4);
            let mut surface = MaskedSurface::new(buffer, area, self.protected, PaintMask::Region);
            for (x, glyph) in glyphs.iter().enumerate().take(usize::from(area.width)) {
                let x = x as u16;
                let distance = position as i64 - i64::from(x) - 2;
                let op = if complete {
                    CellOp::Replace(*glyph, final_style)
                } else if distance < 0 {
                    CellOp::Erase(palette.resolve(Style::new(), profile.colors))
                } else {
                    CellOp::Replace(
                        *glyph,
                        if distance == 0 && profile.motion == Motion::Full {
                            highlight
                        } else if distance < 3 {
                            accent
                        } else {
                            final_style
                        },
                    )
                };
                surface.write(x, 0, op);
            }
        } else {
            // Unsupported multi-column/combining text stays exactly as the host drew it.
            // Only safe single-column cells may receive foreground/modifier patches.
            let width = area.width.min(self.design.display_width);
            let position = self
                .design
                .clip
                .step(self.context.time, u32::from(width) + 4);
            let mut surface = StyleSurface::new(
                buffer,
                area,
                self.protected,
                PaintMask::Region,
                StylePermissions::foreground_and_modifiers(),
            );
            for x in 0..width {
                let distance = position as i64 - i64::from(x) - 2;
                let style = if complete || distance > 2 {
                    final_style
                } else if distance >= 0 {
                    highlight
                } else {
                    palette.resolve(palette.muted, profile.colors)
                };
                surface.patch(x, 0, style);
            }
        }
    }
}
impl Widget for RevealSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &RevealSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}
