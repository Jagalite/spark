// Spark playground: border-runner
// Dependencies: spark-catalog, spark-runtime, ratatui 0.30.2.
// Host requested 40x9; terminal 1x1; embedded=false; inspector=false.
// Snapshot uses the actual fitted rectangle. Supply your own area/base in your app.
// Gallery chrome is omitted; the border's host base and title exclusion are retained.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use ratatui::{style::Style, widgets::{Block, Paragraph}};
use spark_catalog::{BorderRunner};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, HostTime, Motion, Palette,
    Playback, PlaybackRate, Profile, SampleContext,
};

// Store this host-owned mapping. Supply elapsed monotonic host time yourself.
pub fn configured_playback(host: HostTime) -> Playback {
    let mut playback = Playback::new(host);
    playback.set_rate(host, PlaybackRate::new(2000, 1000).unwrap()).unwrap();
    playback.seek(host, ClipTime::from_nanos(750000000)).unwrap();
    playback
}

// One explicit-time frame. In a live app, store design/palette/profile once,
// sample with playback.sample(now), and render on the host's ordinary redraws.
pub fn snapshot() -> Buffer {
    let design = BorderRunner::new().with_seed(7);
    let palette = Palette::default();
    let profile = Profile {
        glyphs: GlyphProfile::Unicode,
        colors: ColorProfile::TrueColor,
        motion: Motion::Full,
    };
    let host = HostTime::ZERO;
    let playback = configured_playback(host);
    let context = SampleContext::new(playback.sample(host), &palette, &profile);
    let area = Rect::new(0, 0, 1, 0);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
    let mut protected = Vec::new();
    if !area.is_empty() {
        let style = if profile.colors == ColorProfile::Monochrome {
            Style::default()
        } else {
            palette.resolve(palette.foreground, profile.colors)
        };
        let mut block = Block::bordered().title(" HOST TITLE ").style(style);
        if profile.glyphs == GlyphProfile::Ascii {
            block = block.border_set(ratatui::symbols::border::Set {
                top_left: "+", top_right: "+", bottom_left: "+", bottom_right: "+",
                vertical_left: "|", vertical_right: "|", horizontal_top: "-", horizontal_bottom: "-",
            });
        }
        block.render(area, &mut buffer);
        if area.width > 4 && area.height > 3 {
            Paragraph::new("Host text stays readable\nSelection / caret / media")
                .style(style)
                .render(Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 3), &mut buffer);
        }
        protected.push(Rect::new(area.x.saturating_add(1), area.y, area.width.saturating_sub(2).min(12), 1));
    }
    design.sample(&context).with_protected(&protected).render(area, &mut buffer);
    buffer
}

fn main() {
    // No terminal or output side effects. Embed the sample in your own host.
    let _buffer = snapshot();
}
