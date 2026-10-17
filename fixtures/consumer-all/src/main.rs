//! A separate downstream application using all six typed designs, no gallery.
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    widgets::{Block, Paragraph},
};
use spark_catalog::{
    AssemblyProgress, BorderRunner, Level, PocketMascot, PrismReveal, Progress, SegmentedComet,
    TelemetryEmbers,
};
use spark_runtime::{ClipTime, Palette, Profile, SampleContext};

fn main() {
    let comet = SegmentedComet::new();
    let assembly = AssemblyProgress::new();
    let border = BorderRunner::new();
    let embers = TelemetryEmbers::new();
    let mascot = PocketMascot::new();
    let reveal = PrismReveal::new("Ready to inspect").unwrap();
    let palette = Palette::default();
    let profile = Profile::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    for nanos in [0, 300_000_000, 1_200_000_000, u64::MAX] {
        let context = SampleContext::new(ClipTime::from_nanos(nanos), &palette, &profile);
        terminal
            .draw(|frame| {
                frame.render_widget(
                    Block::bordered().title(" All-six downstream host "),
                    frame.area(),
                );
                frame.render_widget(
                    border
                        .sample(&context)
                        .with_protected(&[Rect::new(1, 0, 28, 1)]),
                    frame.area(),
                );
                frame.render_widget(comet.sample(&context), Rect::new(2, 2, 28, 3));
                frame.render_widget(
                    assembly.sample(&context, Progress::known(0.6).unwrap()),
                    Rect::new(2, 7, 28, 3),
                );
                frame.render_widget(
                    Paragraph::new("Host percentage: 60%"),
                    Rect::new(2, 11, 28, 1),
                );
                frame.render_widget(
                    embers.sample(&context, Level::new(0.7).unwrap()),
                    Rect::new(34, 2, 42, 16),
                );
                frame.render_widget(mascot.sample(&context), Rect::new(2, 13, 16, 6));
                frame.render_widget(reveal.sample(&context), Rect::new(22, 20, 25, 1));
            })
            .unwrap();
    }
    assert!(
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() != " ")
    );
    println!("all-six downstream consumer: four real TestBackend frames passed");
}
