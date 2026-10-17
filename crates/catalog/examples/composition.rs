//! Host-selected outcomes composed from ordinary catalogue widgets.
use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::{BorderRunner, PrismReveal, SegmentedComet};
use spark_runtime::composition::{ClipTrack, Easing, Parallel, Phase, Sequence, Transition};
use spark_runtime::{ClipTime, Palette, Period, Profile, SampleContext};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
enum Outcome {
    Loading,
    Success,
    Error,
}

fn period(ms: u64) -> Period {
    Period::new(Duration::from_millis(ms)).unwrap()
}

fn render(outcome: Outcome, elapsed: ClipTime, size: (u16, u16), quiet: bool) -> Buffer {
    let area = Rect::new(0, 0, size.0, size.1);
    let mut buffer = Buffer::empty(area);
    let palette = if matches!(outcome, Outcome::Error) {
        Palette::amber()
    } else {
        Palette::default()
    };
    let profile = if quiet {
        Profile::quiet()
    } else {
        Profile::default()
    };
    if area.is_empty() {
        return buffer;
    }
    // The host owns this base; every catalogue layer still uses scoped writers.
    for x in 0..area.width {
        buffer[(x, 0)].set_symbol("-");
    }
    let protected = [Rect::new(0, 0, area.width.min(16), 1)];
    buffer.set_stringn(
        0,
        0,
        "HOST owns result",
        usize::from(area.width),
        palette.foreground,
    );
    match outcome {
        Outcome::Loading => {
            // The host explicitly repeats this finite arrangement. An animation
            // boundary never changes the operation's Loading domain state.
            let clock = period(3_000);
            let time = ClipTime::from_nanos(if quiet {
                clock.as_nanos() - 1
            } else {
                clock.phase_nanos(elapsed)
            });
            let tracks = [
                ClipTrack::new(ClipTime::ZERO, clock).unwrap(),
                ClipTrack::new(ClipTime::from_nanos(150_000_000), period(2_850)).unwrap(),
            ];
            let parallel = Parallel::new(&tracks).unwrap();
            let comet = SegmentedComet::new().with_seed(7);
            let border = BorderRunner::new().with_seed(11);
            for (layer, sample) in parallel.sample(time).enumerate() {
                if sample.phase() == Phase::Before {
                    continue;
                }
                let context = SampleContext::new(sample.local_time(), &palette, &profile);
                if layer == 0 {
                    let rect = Rect::new(0, 1, area.width, area.height.saturating_sub(1));
                    comet
                        .sample(&context)
                        .with_protected(&protected)
                        .render(rect, &mut buffer);
                } else {
                    border
                        .sample(&context)
                        .with_protected(&protected)
                        .render(area, &mut buffer);
                }
            }
        }
        Outcome::Success | Outcome::Error => {
            // elapsed starts at the *host-reported* result, not when progress
            // happens to reach 100% or when the loading animation loops.
            let durations = [period(150), period(700)];
            let sequence = Sequence::new(&durations).unwrap();
            let time = if quiet {
                ClipTime::saturating_from_duration(sequence.duration())
            } else {
                elapsed
            };
            let segment = sequence.sample(time);
            if segment.index() == 0 && area.height > 1 {
                buffer.set_stringn(
                    0,
                    1.min(area.height - 1),
                    "...",
                    usize::from(area.width),
                    palette.muted,
                );
            } else if segment.index() > 0 {
                let transition = Transition::new(durations[1]).with_easing(Easing::SmoothStep);
                let pose = transition.sample(segment.local_time());
                let y = pose.interpolate(3, 1).min(area.height - 1);
                let label = if matches!(outcome, Outcome::Success) {
                    "SUCCESS: saved"
                } else {
                    "ERROR: retry"
                };
                let reveal = PrismReveal::new(label).unwrap().with_duration(durations[1]);
                let context = SampleContext::new(segment.local_time(), &palette, &profile);
                reveal
                    .sample(&context)
                    .with_protected(&protected)
                    .render(Rect::new(0, y, area.width, 1), &mut buffer);
            }
        }
    }
    buffer
}

fn smoke() {
    let mut frames = 0;
    for outcome in [Outcome::Loading, Outcome::Success, Outcome::Error] {
        for size in [(0, 0), (1, 1), (3, 2), (16, 4), (40, 7)] {
            for quiet in [false, true] {
                for nanos in [
                    0,
                    149_999_999,
                    150_000_000,
                    849_999_999,
                    850_000_000,
                    u64::MAX,
                ] {
                    let time = ClipTime::from_nanos(nanos);
                    let first = render(outcome, time, size, quiet);
                    assert_eq!(first, render(outcome, time, size, quiet));
                    if quiet {
                        assert_eq!(first, render(outcome, ClipTime::ZERO, size, true));
                    }
                    if size.0 > 0 && size.1 > 0 {
                        assert_eq!(first[(0, 0)].symbol(), "H");
                    }
                    frames += 1;
                }
            }
        }
    }
    println!(
        "composition smoke: {frames} deterministic frames; explicit outcomes, parallel layers, seams, terminal holds, quiet/tiny geometry"
    );
}

fn main() {
    if std::env::args().any(|arg| arg == "--smoke") {
        smoke();
        return;
    }
    for (outcome, millis) in [
        (Outcome::Loading, 750),
        (Outcome::Success, 500),
        (Outcome::Success, 850),
        (Outcome::Error, 850),
    ] {
        println!("{outcome:?}, {millis} ms since host state change:");
        let buffer = render(
            outcome,
            ClipTime::from_nanos(millis * 1_000_000),
            (40, 7),
            false,
        );
        for row in buffer.content.chunks(40) {
            println!(
                "{}",
                row.iter().map(|cell| cell.symbol()).collect::<String>()
            );
        }
    }
}
