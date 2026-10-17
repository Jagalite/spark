// Spark playground: segmented-comet
// Dependencies: spark-catalog, spark-runtime, ratatui 0.30.2.
// Host requested 40x3; terminal 100x32; embedded=false; inspector=false.
// Snapshot uses the actual fitted rectangle. Supply your own area/base in your app.
// Gallery chrome is omitted; the border's host base and title exclusion are retained.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::{SegmentedComet};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, HostTime, Motion, Palette,
    Playback, PlaybackRate, Profile, SampleContext,
};

// Store this host-owned mapping. Supply elapsed monotonic host time yourself.
pub fn configured_playback(host: HostTime) -> Playback {
    let mut playback = Playback::new(host);
    playback.set_rate(host, PlaybackRate::new(1000, 1000).unwrap()).unwrap();
    playback.seek(host, ClipTime::from_nanos(750000000)).unwrap();
    playback
}

// One explicit-time frame. In a live app, store design/palette/profile once,
// sample with playback.sample(now), and render on the host's ordinary redraws.
pub fn snapshot() -> Buffer {
    let design = SegmentedComet::new().with_seed(7);
    let palette = Palette::default();
    let profile = Profile {
        glyphs: GlyphProfile::Unicode,
        colors: ColorProfile::TrueColor,
        motion: Motion::Full,
    };
    let host = HostTime::ZERO;
    let playback = configured_playback(host);
    let context = SampleContext::new(playback.sample(host), &palette, &profile);
    let area = Rect::new(43, 15, 40, 3);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 32));
    let protected: [Rect; 0] = [];
    design.sample(&context).with_protected(&protected).render(area, &mut buffer);
    buffer
}

fn main() {
    // No terminal or output side effects. Embed the sample in your own host.
    let _buffer = snapshot();
}
