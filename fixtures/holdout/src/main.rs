use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_public_holdout::{CrossWeave, WeaveSample};
use spark_runtime::{ClipTime, GlyphProfile, Palette, Profile, SampleContext};

fn main() {
    let design = CrossWeave::new().with_seed(3);
    let palette = Palette::default();
    let profile = Profile {
        glyphs: GlyphProfile::Ascii,
        ..Profile::default()
    };
    let context = SampleContext::new(ClipTime::from_nanos(360_000_000), &palette, &profile);
    let sample = design.sample(&context);
    let area = Rect::new(0, 0, 17, 7);
    let mut buffer = Buffer::empty(area);
    let borrowed = &sample;
    <&WeaveSample<'_> as Widget>::render(borrowed, area, &mut buffer);
    for row in buffer.content.chunks(usize::from(area.width)) {
        println!(
            "{}",
            row.iter().map(|cell| cell.symbol()).collect::<String>()
        );
    }
    println!("Next change: {:?}", sample.change_hint(area.as_size()));
}
