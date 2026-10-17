//! Independent application crate. Run with:
//! cargo run --manifest-path fixtures/consumer/Cargo.toml --locked
use std::time::Duration;

use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use spark_catalog::{CometSample, SegmentedComet};
use spark_runtime::{
    ClipTime, Palette, Period, Profile, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};

/// A downstream design, expressed solely through the public runtime surface.
/// It does not depend on catalog internals or a gallery registry/trait.
struct TravelingMarker<'a> {
    context: SampleContext<'a>,
    period: Period,
}

impl Widget for &TravelingMarker<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let x = self.period.step(self.context.time, u32::from(area.width)) as u16;
        let style = self
            .context
            .palette
            .resolve(Style::new().fg(Color::Green), self.context.profile.colors);
        let mut surface = OpaqueSurface::new(buffer, area, &[]);
        surface.write(x, 0, CellOp::Replace(Glyph::new('+').unwrap(), style));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let design = SegmentedComet::new()
        .with_seed(17)
        .with_period(Period::new(Duration::from_millis(1200))?);
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(
        ClipTime::try_from(Duration::from_millis(600))?,
        &palette,
        &profile,
    );
    let view = design.sample(&context);
    let custom = TravelingMarker {
        context,
        period: Period::new(Duration::from_millis(1200))?,
    };
    let mut terminal = Terminal::new(TestBackend::new(32, 6))?;
    terminal.draw(|frame| {
        frame.render_widget::<&CometSample<'_>>(&view, Rect::new(4, 1, 24, 3));
        frame.render_widget(&custom, Rect::new(4, 5, 24, 1));
    })?;
    let first = terminal.backend().buffer().clone();
    terminal.draw(|frame| {
        frame.render_widget::<&CometSample<'_>>(&view, Rect::new(4, 1, 24, 3));
        frame.render_widget(&custom, Rect::new(4, 5, 24, 1));
    })?;
    assert_eq!(*terminal.backend().buffer(), first);
    assert_eq!(first[(16, 5)].symbol(), "+");
    assert_eq!(first[(0, 0)].symbol(), " ");
    assert_eq!(first[(31, 5)].symbol(), " ");
    println!(
        "verified: borrowed catalog Widget and downstream runtime-only design on ratatui 0.30.2"
    );
    Ok(())
}
