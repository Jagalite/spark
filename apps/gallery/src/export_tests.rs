//! Compile the exact generated programs as ordinary Rust modules. Byte equality
//! makes these compiler checks evidence for the current exporter, not stale examples.
use super::*;
use crate::{cli, design};
use ratatui::{buffer::Buffer, layout::Rect};
use spark_runtime::{HostTime, Playback, PlaybackRate, SampleContext};

fn check(args: &str, source: &str, snapshot: fn() -> Buffer, playback: fn(HostTime) -> Playback) {
    let options = cli::parse(
        format!("--export-rust {args}")
            .split_whitespace()
            .map(str::to_owned),
    )
    .unwrap();
    let app = options.app;
    assert_eq!(rust(&app, app.displayed, options.terminal), source);
    assert!(source.chars().all(|c| c == '\n' || !c.is_control()));
    let context = SampleContext::new(app.displayed, &app.palette, &app.profile);
    let area = view::sample_area(options.terminal, &app);
    let mut expected = Buffer::empty(Rect::new(
        0,
        0,
        options.terminal.width,
        options.terminal.height,
    ));
    let protected = design::base(app.selection, area, &context, &mut expected);
    app.designs.render(
        app.selection,
        &context,
        app.input,
        app.unknown,
        area,
        &protected,
        &mut expected,
    );
    assert_eq!(
        snapshot(),
        expected,
        "exported art differs from the live typed adapter"
    );
    let host = HostTime::from_nanos(100);
    let mapping = playback(host);
    assert_eq!(mapping.sample(host), app.displayed);
    assert_eq!(mapping.is_playing(), !app.paused);
    assert_eq!(
        mapping.rate(),
        PlaybackRate::new(app.rate_milli, 1000).unwrap()
    );
    let later = HostTime::from_nanos(1_000_000_100);
    let advance = if app.paused {
        0
    } else {
        u64::from(app.rate_milli) * 1_000_000
    };
    assert_eq!(
        mapping.sample(later).as_nanos(),
        app.displayed.as_nanos().saturating_add(advance)
    );
}
macro_rules! fixture {
    ($name:ident, $path:literal, $args:literal) => {
        mod $name {
            include!($path);
            #[test]
            fn generated_program_compiles_and_matches_live_config() {
                super::check($args, include_str!($path), snapshot, configured_playback);
                main();
            }
        }
    };
}
fixture!(
    comet,
    "../tests/fixtures/comet.rs",
    "--design segmented-comet"
);
fixture!(
    assembly,
    "../tests/fixtures/assembly.rs",
    "--design assembly-progress --size 71x11 --palette amber --glyphs ascii --colors ansi16 --motion reduced --speed 0.25 --seed 18446744073709551615 --input 35 --paused --clip-nanos 1234567890 --embedded"
);
fixture!(
    unknown,
    "../tests/fixtures/unknown.rs",
    "--design assembly-progress --unknown --input 100 --speed 4 --clip-nanos 18446744073709551615 --inspect"
);
fixture!(
    border,
    "../tests/fixtures/border.rs",
    "--design border-runner --size 61x9 --palette amber --glyphs ascii --colors ansi16 --motion reduced --seed 18 --speed 0.5 --paused"
);
fixture!(
    border_quiet,
    "../tests/fixtures/border_quiet.rs",
    "--design border-runner --glyphs unicode --colors monochrome --motion static --terminal 140x42 --inspect"
);
fixture!(
    border_tiny,
    "../tests/fixtures/border_tiny.rs",
    "--design border-runner --terminal 1x1 --speed 2"
);
fixture!(
    border_empty,
    "../tests/fixtures/border_empty.rs",
    "--design border-runner --terminal 0x0"
);
fixture!(
    embers,
    "../tests/fixtures/embers.rs",
    "--design telemetry-embers --size 52x17 --palette amber --glyphs ascii --colors monochrome --motion static --input 40 --seed 18446744073709551615 --speed 2 --paused --clip-nanos 18446744073709551615"
);
fixture!(
    mascot,
    "../tests/fixtures/mascot.rs",
    "--design pocket-mascot --colors ansi16 --motion reduced --speed 0.5 --paused --embedded --inspect --terminal 128x42"
);
fixture!(
    prism,
    "../tests/fixtures/prism.rs",
    "--design prism-reveal --size 19x1 --palette amber --motion static --speed 4 --clip-nanos 777777777"
);
