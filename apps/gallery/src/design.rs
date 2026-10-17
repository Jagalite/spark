//! Private inspection-tool adapters. Consumers use the catalogue's typed designs.
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};
use spark_catalog::{
    AssemblyProgress, BorderRunner, Level, PocketMascot, PrismReveal, Progress, SegmentedComet,
    TelemetryEmbers,
};
use spark_runtime::{ColorProfile, GlyphProfile, SampleContext};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Selection {
    #[default]
    Comet,
    Assembly,
    Border,
    Embers,
    Mascot,
    Prism,
}
impl Selection {
    pub const ALL: [Self; 6] = [
        Self::Comet,
        Self::Assembly,
        Self::Border,
        Self::Embers,
        Self::Mascot,
        Self::Prism,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Comet => "segmented-comet",
            Self::Assembly => "assembly-progress",
            Self::Border => "border-runner",
            Self::Embers => "telemetry-embers",
            Self::Mascot => "pocket-mascot",
            Self::Prism => "prism-reveal",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Comet => "Segmented Comet",
            Self::Assembly => "Assembly Progress",
            Self::Border => "Border Runner",
            Self::Embers => "Telemetry Embers",
            Self::Mascot => "Pocket Mascot",
            Self::Prism => "Prism Reveal",
        }
    }
    pub fn number(self) -> usize {
        Self::ALL.iter().position(|v| *v == self).unwrap() + 1
    }
    #[allow(dead_code)]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
    pub fn next(self) -> Self {
        Self::ALL[self.number() % Self::ALL.len()]
    }
    pub fn preferred(self) -> Size {
        match self {
            Self::Comet | Self::Assembly => Size::new(40, 3),
            Self::Border => Size::new(40, 9),
            Self::Embers => Size::new(48, 12),
            Self::Mascot => Size::new(12, 6),
            Self::Prism => Size::new(24, 3),
        }
    }
    pub fn capability(self) -> &'static str {
        match self {
            Self::Comet => "Widget / inline; seeded loop",
            Self::Assembly => "Widget; known / unknown progress",
            Self::Border => "Decorator; border style only",
            Self::Embers => "Widget / scene; caller level",
            Self::Mascot => "Widget; authored cell frames",
            Self::Prism => "Masked text; finite reveal",
        }
    }
    pub fn geometry_label(self) -> &'static str {
        match self {
            Self::Comet => "min 8x1 / cap 256x3",
            Self::Assembly => "min 12x3 / cap 80x3",
            Self::Border => "min 3x3 / cap 256x128",
            Self::Embers => "min 8x4 / cap 120x40",
            Self::Mascot => "native 12x6 / no stretch",
            Self::Prism => "1 row / max 256 scalars",
        }
    }
    pub fn fit_label(self) -> &'static str {
        match self {
            Self::Comet | Self::Assembly | Self::Embers => "Adaptive / quiet fallback",
            Self::Border => "Host border + exclusions",
            Self::Mascot => "Centered / compact face",
            Self::Prism => "Finite / style fallback",
        }
    }
    pub fn input_label(self, value: u8, unknown: bool) -> String {
        match self {
            Self::Assembly if unknown => "Host progress: unknown".into(),
            Self::Assembly => format!("Host progress: {value}%"),
            Self::Embers => format!("Host level: {:.2}", f64::from(value) / 100.0),
            _ => "No data input for this design".into(),
        }
    }
}

pub(crate) struct Designs {
    comet: SegmentedComet,
    assembly: AssemblyProgress,
    border: BorderRunner,
    embers: TelemetryEmbers,
    mascot: PocketMascot,
    prism: PrismReveal,
}
impl Designs {
    pub fn new(seed: u64) -> Self {
        Self {
            comet: SegmentedComet::new().with_seed(seed),
            assembly: AssemblyProgress::new().with_seed(seed),
            border: BorderRunner::new().with_seed(seed),
            embers: TelemetryEmbers::new().with_seed(seed),
            mascot: PocketMascot::new(),
            prism: PrismReveal::new("SIGNAL RESTORED").expect("literal single-column text"),
        }
    }
    #[allow(dead_code)]
    pub fn duration_ms(&self, selection: Selection) -> Option<u64> {
        let duration = match selection {
            Selection::Comet => self.comet.period().as_duration(),
            Selection::Assembly => self.assembly.period().as_duration(),
            Selection::Border => self.border.period().as_duration(),
            Selection::Prism => self.prism.duration().as_duration(),
            Selection::Mascot => self.mascot.period().as_duration(),
            Selection::Embers => return None,
        };
        Some(duration.as_millis() as u64)
    }
    pub fn complete(&self, selection: Selection, context: &SampleContext<'_>) -> bool {
        selection == Selection::Prism && self.prism.sample(context).complete()
    }
    pub fn interval(
        &self,
        selection: Selection,
        context: &SampleContext<'_>,
        input: u8,
        unknown: bool,
        size: Size,
    ) -> Option<Duration> {
        match selection {
            Selection::Comet => self.comet.sample(context).frame_interval(size),
            Selection::Assembly => self
                .assembly
                .sample(context, progress(input, unknown))
                .frame_interval(size),
            Selection::Border => self.border.sample(context).frame_interval(size),
            Selection::Embers => self
                .embers
                .sample(context, level(input))
                .frame_interval(size),
            Selection::Mascot => match self.mascot.sample(context).change_hint(size) {
                spark_runtime::ChangeHint::At(next) => Some(Duration::from_nanos(
                    next.as_nanos()
                        .saturating_sub(context.time.as_nanos())
                        .max(1),
                )),
                spark_runtime::ChangeHint::Continuous(interval) => Some(interval),
                spark_runtime::ChangeHint::None => None,
            },
            Selection::Prism => self.prism.sample(context).frame_interval(size),
        }
    }
    /// Draw host base first. Decorators never acquire an opaque base through this adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        selection: Selection,
        context: &SampleContext<'_>,
        input: u8,
        unknown: bool,
        area: Rect,
        protected: &[Rect],
        buffer: &mut Buffer,
    ) {
        match selection {
            Selection::Comet => self
                .comet
                .sample(context)
                .with_protected(protected)
                .render(area, buffer),
            Selection::Assembly => self
                .assembly
                .sample(context, progress(input, unknown))
                .with_protected(protected)
                .render(area, buffer),
            Selection::Border => self
                .border
                .sample(context)
                .with_protected(protected)
                .render(area, buffer),
            Selection::Embers => self
                .embers
                .sample(context, level(input))
                .with_protected(protected)
                .render(area, buffer),
            Selection::Mascot => self
                .mascot
                .sample(context)
                .with_protected(protected)
                .render(area, buffer),
            Selection::Prism => self
                .prism
                .sample(context)
                .with_protected(protected)
                .render(area, buffer),
        }
    }
}
fn progress(value: u8, unknown: bool) -> Progress {
    if unknown {
        Progress::unknown()
    } else {
        Progress::known(f64::from(value.min(100)) / 100.0).expect("bounded tool input")
    }
}
fn level(value: u8) -> Level {
    Level::new(f64::from(value.min(100)) / 100.0).expect("bounded tool input")
}

/// Synthetic base content for isolated and embedded decorator inspection.
pub(crate) fn base(
    selection: Selection,
    area: Rect,
    context: &SampleContext<'_>,
    buffer: &mut Buffer,
) -> Vec<Rect> {
    if area.is_empty() {
        return Vec::new();
    }
    let style = if context.profile.colors == ColorProfile::Monochrome {
        Style::default()
    } else {
        context
            .palette
            .resolve(context.palette.foreground, context.profile.colors)
    };
    if selection == Selection::Border {
        let mut block = Block::bordered().title(" HOST TITLE ").style(style);
        if context.profile.glyphs == GlyphProfile::Ascii {
            block = block.border_set(ratatui::symbols::border::Set {
                top_left: "+",
                top_right: "+",
                bottom_left: "+",
                bottom_right: "+",
                vertical_left: "|",
                vertical_right: "|",
                horizontal_top: "-",
                horizontal_bottom: "-",
            });
        }
        block.render(area, buffer);
        if area.width > 4 && area.height > 3 {
            Paragraph::new("Host text stays readable\nSelection / caret / media")
                .style(style)
                .render(
                    Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 3),
                    buffer,
                );
        }
        vec![Rect::new(
            area.x.saturating_add(1),
            area.y,
            area.width.saturating_sub(2).min(12),
            1,
        )]
    } else if selection == Selection::Prism {
        // ASCII authored text is supplied by PrismReveal itself. A separate wide-text
        // fallback is exercised by contract tests, not silently substituted here.
        Vec::new()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_are_unique_and_selector_cycles() {
        for (i, value) in Selection::ALL.into_iter().enumerate() {
            assert_eq!(Selection::from_id(value.id()), Some(value));
            assert_eq!(value.number(), i + 1);
            assert_ne!(value, value.next());
        }
        assert_eq!(Selection::ALL[5].next(), Selection::Comet);
    }
}
