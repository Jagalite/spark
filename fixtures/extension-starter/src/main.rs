#![forbid(unsafe_code)]
//! Tiny host: explicit capture times, fresh buffers, and stdout owned here.

use std::{fmt::Write as _, io::Write as _, time::Duration};

use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use spark_runtime::{
    GlyphProfile, HostTime, Motion, Palette, Playback, Profile, RepaintPlanner, SampleContext,
};

use spark_extension_starter::{HangingLantern, NATIVE_SIZE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => std::io::stdout().lock().write_all(preview().as_bytes())?,
        [flag] if flag == "--check" => {
            if preview() != include_str!("../preview.txt") {
                return Err("preview drift: render to a temporary file and review the diff".into());
            }
            println!("extension-starter preview matches preview.txt");
        }
        [flag] if flag == "--help" => {
            println!("Usage: cargo run --locked -- [--check | --help]");
            println!("Default: print actual sampled cell glyphs. --check: verify preview.txt.");
        }
        _ => return Err("expected no arguments, --check, or --help".into()),
    }
    Ok(())
}

fn preview() -> String {
    let mut output = String::from(
        "Hanging Lantern | original downstream Spark design | visual revision 1\n\
         Actual Rust Buffer glyphs; each row is enclosed by host-added | delimiters.\n\
         Native 13x7; ASCII; default palette; truecolor unless marked quiet.\n\
         Text-only capture omits foreground, background, underline and modifiers.\n\
         No font/terminal, color, motion-smoothness or performance qualification.\n\
         Host supplies synthetic elapsed time; Playback rate 1; presentation cap 16ms.\n",
    );
    let design = HangingLantern::new();
    let palette = Palette::default();
    let full = Profile {
        glyphs: GlyphProfile::Ascii,
        ..Profile::default()
    };
    let native = Rect::new(0, 0, NATIVE_SIZE.width, NATIVE_SIZE.height);
    for time_ms in [0, 240, 360, 520, 840, 1_000, 1_120] {
        capture(
            &mut output,
            &design,
            &palette,
            &full,
            time_ms,
            "full / native",
            native,
        );
    }
    capture(
        &mut output,
        &design,
        &palette,
        &Profile {
            motion: Motion::Reduced,
            ..full
        },
        2_080,
        "reduced / native (quarter speed)",
        native,
    );
    capture(
        &mut output,
        &design,
        &palette,
        &Profile::quiet(),
        9_000,
        "quiet / native (ASCII monochrome static poster)",
        native,
    );
    capture(
        &mut output,
        &design,
        &palette,
        &full,
        520,
        "full / compact (stationary fallback)",
        Rect::new(0, 0, 4, 2),
    );

    // Host content and absolute exclusions belong to the application. This
    // intentionally overlapping label demonstrates that a design cannot erase it.
    let area = Rect::new(0, 0, 21, 11);
    let requested = Rect::new(2, 2, 17, 9);
    let protected = [Rect::new(8, 8, 5, 1)];
    let profile = Profile::quiet();
    let playback = Playback::new(HostTime::ZERO);
    let context = SampleContext::new(playback.sample(HostTime::ZERO), &palette, &profile);
    let mut buffer = Buffer::empty(area);
    buffer.set_string(1, 0, "Host-owned panel", Style::new());
    buffer.set_string(8, 8, "OWNER", Style::new());
    design
        .sample(&context)
        .with_protected(&protected)
        .render(requested, &mut buffer);
    writeln!(
        output,
        "\nEmbedding: request=(2,2,17,9), buffer=(0,0,21,11), protected=(8,8,5,1)"
    )
    .unwrap();
    rows(&mut output, &buffer);
    output
}

fn capture(
    output: &mut String,
    design: &HangingLantern,
    palette: &Palette,
    profile: &Profile,
    time_ms: u64,
    label: &str,
    area: Rect,
) {
    // A real application obtains elapsed time from its own clock and merges
    // this deadline with input/data deadlines. This reproducible host never waits.
    let now = HostTime::from_nanos(time_ms * 1_000_000);
    let playback = Playback::new(HostTime::ZERO);
    let context = SampleContext::new(playback.sample(now), palette, profile);
    let sample = design.sample(&context);
    let planner = RepaintPlanner::new(Duration::from_millis(16));
    let next = planner.next_deadline(now, now, &playback, sample.change_hint(area.as_size()));
    let next = next.map_or_else(|| "none".to_owned(), |time| time.as_nanos().to_string());
    writeln!(
        output,
        "\n{label} | time_ms={time_ms} | size={}x{} | next_host_ns={next}",
        area.width, area.height,
    )
    .unwrap();
    let mut buffer = Buffer::empty(area);
    sample.render(area, &mut buffer);
    rows(output, &buffer);
}

fn rows(output: &mut String, buffer: &Buffer) {
    for row in buffer.content.chunks(usize::from(buffer.area.width)) {
        output.push('|');
        for cell in row {
            output.push_str(cell.symbol());
        }
        output.push_str("|\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_preview_is_exact_reproducible_consumer_output() {
        assert_eq!(preview(), include_str!("../preview.txt"));
    }
}
