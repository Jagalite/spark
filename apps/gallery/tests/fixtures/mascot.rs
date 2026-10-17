// Spark playground: pocket-mascot
// Dependencies: spark-catalog, spark-runtime, ratatui 0.30.2.
// Host requested 12x6; terminal 128x42; embedded=true; inspector=true.
// Snapshot uses the actual fitted rectangle. Supply your own area/base in your app.
// Gallery chrome is omitted; the border's host base and title exclusion are retained.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::{PocketMascot};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, HostTime, Motion, Palette,
    Playback, PlaybackRate, Profile, SampleContext,
};

// Store this host-owned mapping. Supply elapsed monotonic host time yourself.
pub fn configured_playback(host: HostTime) -> Playback {
    let mut playback = Playback::new(host);
    playback.set_rate(host, PlaybackRate::new(500, 1000).unwrap()).unwrap();
    playback.seek(host, ClipTime::from_nanos(750000000)).unwrap();
    playback.pause(host).unwrap();
    playback
}

// One explicit-time frame. In a live app, store design/palette/profile once,
// sample with playback.sample(now), and render on the host's ordinary redraws.
pub fn snapshot() -> Buffer {
    let design = PocketMascot::new();
    let palette = Palette::default();
    let profile = Profile {
        glyphs: GlyphProfile::Unicode,
        colors: ColorProfile::Ansi16,
        motion: Motion::Reduced,
    };
    let host = HostTime::ZERO;
    let playback = configured_playback(host);
    let context = SampleContext::new(playback.sample(host), &palette, &profile);
    let area = Rect::new(15, 18, 12, 6);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 128, 42));
    let protected: [Rect; 0] = [];
    design.sample(&context).with_protected(&protected).render(area, &mut buffer);
    buffer
}

fn main() {
    // No terminal or output side effects. Embed the sample in your own host.
    let _buffer = snapshot();
}
