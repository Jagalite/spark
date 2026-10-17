//! Rust source export belongs to the gallery host, never to a renderer.
use crate::{app::App, design::Selection, view};
use ratatui::layout::Size;
use spark_runtime::ClipTime;

/// A complete, dependency-only Rust program. No clocks, terminal setup, IO, or
/// environment-dependent source is included. Export the displayed pose exactly.
pub fn rust(app: &App, clip: ClipTime, terminal: Size) -> String {
    let area = view::sample_area(terminal, app);
    let (catalog_import, constructor, input) = match app.selection {
        Selection::Comet => (
            "SegmentedComet",
            format!("SegmentedComet::new().with_seed({})", app.seed),
            String::new(),
        ),
        Selection::Assembly => (
            "AssemblyProgress, Progress",
            format!("AssemblyProgress::new().with_seed({})", app.seed),
            if app.unknown {
                ", Progress::unknown()".into()
            } else {
                format!(", Progress::known({}.0 / 100.0).unwrap()", app.input)
            },
        ),
        Selection::Border => (
            "BorderRunner",
            format!("BorderRunner::new().with_seed({})", app.seed),
            String::new(),
        ),
        Selection::Embers => (
            "Level, TelemetryEmbers",
            format!("TelemetryEmbers::new().with_seed({})", app.seed),
            format!(", Level::new({}.0 / 100.0).unwrap()", app.input),
        ),
        Selection::Mascot => ("PocketMascot", "PocketMascot::new()".into(), String::new()),
        Selection::Prism => (
            "PrismReveal",
            "PrismReveal::new(\"SIGNAL RESTORED\").unwrap()".into(),
            String::new(),
        ),
    };
    let host_imports = if app.selection == Selection::Border {
        "use ratatui::{style::Style, widgets::{Block, Paragraph}};\n"
    } else {
        ""
    };
    let base = if app.selection == Selection::Border {
        BORDER_BASE
    } else {
        "    let protected: [Rect; 0] = [];\n"
    };
    let pause = if app.paused {
        "    playback.pause(host).unwrap();\n"
    } else {
        ""
    };
    format!(
        r#"// Spark playground: {id}
// Dependencies: spark-catalog, spark-runtime, ratatui 0.30.2.
// Host requested {width}x{height}; terminal {tw}x{th}; embedded={embedded}; inspector={inspector}.
// Snapshot uses the actual fitted rectangle. Supply your own area/base in your app.
// Gallery chrome is omitted; the border's host base and title exclusion are retained.
use ratatui::{{buffer::Buffer, layout::Rect, widgets::Widget}};
{host_imports}use spark_catalog::{{{catalog_import}}};
use spark_runtime::{{
    ClipTime, ColorProfile, GlyphProfile, HostTime, Motion, Palette,
    Playback, PlaybackRate, Profile, SampleContext,
}};

// Store this host-owned mapping. Supply elapsed monotonic host time yourself.
pub fn configured_playback(host: HostTime) -> Playback {{
    let mut playback = Playback::new(host);
    playback.set_rate(host, PlaybackRate::new({rate}, 1000).unwrap()).unwrap();
    playback.seek(host, ClipTime::from_nanos({nanos})).unwrap();
{pause}    playback
}}

// One explicit-time frame. In a live app, store design/palette/profile once,
// sample with playback.sample(now), and render on the host's ordinary redraws.
pub fn snapshot() -> Buffer {{
    let design = {constructor};
    let palette = Palette::{palette}();
    let profile = Profile {{
        glyphs: GlyphProfile::{glyphs:?},
        colors: ColorProfile::{colors:?},
        motion: Motion::{motion:?},
    }};
    let host = HostTime::ZERO;
    let playback = configured_playback(host);
    let context = SampleContext::new(playback.sample(host), &palette, &profile);
    let area = Rect::new({x}, {y}, {aw}, {ah});
    let mut buffer = Buffer::empty(Rect::new(0, 0, {tw}, {th}));
{base}    design.sample(&context{input}).with_protected(&protected).render(area, &mut buffer);
    buffer
}}

fn main() {{
    // No terminal or output side effects. Embed the sample in your own host.
    let _buffer = snapshot();
}}
"#,
        id = app.selection.id(),
        width = app.width,
        height = app.height,
        tw = terminal.width,
        th = terminal.height,
        embedded = app.embedded,
        inspector = app.inspector,
        rate = app.rate_milli,
        nanos = clip.as_nanos(),
        palette = if app.amber { "amber" } else { "default" },
        glyphs = app.profile.glyphs,
        colors = app.profile.colors,
        motion = app.profile.motion,
        x = area.x,
        y = area.y,
        aw = area.width,
        ah = area.height,
    )
}

// This is illustrative HOST content, not another art renderer. The semantic
// export tests compare it with design::base, including empty and tiny geometry.
const BORDER_BASE: &str = r#"    let mut protected = Vec::new();
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
"#;

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
