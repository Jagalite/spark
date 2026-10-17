#![forbid(unsafe_code)]
//! Reproducible cell captures, not a second renderer or a running business task.
//! `--text` (default) prints ASCII; `--html` emits a standalone styled capture.
use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::Widget,
};
use spark_catalog::status::{StatusKind, StatusTheme};
use spark_runtime::{ClipTime, GlyphProfile, Motion, Palette, Profile, SampleContext};
use std::{fmt::Write as _, io};

const STATES: [StatusKind; 4] = [
    StatusKind::Loading,
    StatusKind::Empty,
    StatusKind::Success,
    StatusKind::Error,
];
struct Preview {
    title: &'static str,
    buffer: Buffer,
}

fn main() -> io::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "--text".into());
    if std::env::args().len() > 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "choose one of --text, --html, --smoke, --help",
        ));
    }
    if matches!(mode.as_str(), "--help" | "-h") {
        println!(
            "status_theme\n--text   deterministic ASCII contact sheets (default)\n--html   standalone Unicode/color cell captures to stdout\n--smoke  verify all generated captures without terminal setup"
        );
        return Ok(());
    }
    if !matches!(mode.as_str(), "--text" | "--html" | "--smoke") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unknown option; use --help",
        ));
    }
    let html = mode == "--html";
    let previews = captures(html);
    if mode == "--smoke" {
        let repeated = captures(false);
        assert_eq!(previews.len(), repeated.len());
        for (first, again) in previews.iter().zip(&repeated) {
            assert_eq!(first.buffer, again.buffer);
            assert!(!first.buffer.area.is_empty());
        }
        // Exercise the Unicode profile and its single-column glyph validation.
        assert_eq!(captures(true).len(), previews.len());
        println!(
            "status theme smoke: {} reproducible native, transition, fallback, and embedded cell sheets passed",
            previews.len()
        );
        return Ok(());
    }
    let fingerprint = fingerprint(include_bytes!("../src/status.rs"));
    if html {
        println!(
            "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Spark signal-seal status theme</title><style>body{{background:#101820;color:#b9cddd;margin:40px;font:15px/1.6 system-ui}}main{{max-width:1500px;margin:auto}}h1{{color:#e6fff6}}h2{{font-size:18px;margin-top:36px}}p{{max-width:900px}}pre{{font:17px/1.2 ui-monospace,monospace;overflow:auto;padding:20px;border:1px solid #304158;border-radius:12px}}.note{{color:#9bb5d0}}</style><main><h1>Signal seal</h1><p>Four host-selected states, one visual family. These are exact cells produced by StatusTheme through its ordinary Widget implementation. Each panel is a separately supplied status and time; the gallery never infers a business result.</p><p class=\"note\">Visual revision 1 · native 23×7 · full loop 2400 ms · arrival 960 ms · default palette with unknown background · status.rs FNV-1a64 {fingerprint:016x} (non-cryptographic source fingerprint)</p>"
        );
        for preview in &previews {
            println!(
                "<h2>{}</h2><pre aria-label=\"{}\">{}</pre>",
                escape(preview.title),
                escape(preview.title),
                html_cells(&preview.buffer)
            );
        }
        println!(
            "<p class=\"note\">The page background is this preview host's choice. The design does not assume a black or known terminal background. Reduced views keep glyphs fixed and settle palette roles; monochrome/static views retain shape and labels. Tiny markers are not substitutes for accessible host labels. Font appearance varies by terminal.</p></main></html>"
        );
    } else {
        println!(
            "Spark signal-seal status theme | visual revision 1\nExact Widget cells, ASCII; default palette (styles omitted by plain text)\nNative 23x7; loading 2400ms; arrival 960ms\nstatus.rs FNV-1a64 {fingerprint:016x} (non-cryptographic source fingerprint)\nUse --html for Unicode/color captures, including reduced-motion style changes"
        );
        for preview in &previews {
            println!("\n{}", preview.title);
            for row in preview
                .buffer
                .content
                .chunks(usize::from(preview.buffer.area.width))
            {
                println!(
                    "{}",
                    row.iter().map(|cell| cell.symbol()).collect::<String>()
                );
            }
        }
    }
    Ok(())
}

fn captures(unicode: bool) -> Vec<Preview> {
    let profile = Profile {
        glyphs: if unicode {
            GlyphProfile::Unicode
        } else {
            GlyphProfile::Ascii
        },
        ..Profile::default()
    };
    let reduced = Profile {
        motion: Motion::Reduced,
        ..profile
    };
    let palette = Palette::default();
    let state_entries: Vec<_> = STATES.into_iter().map(|s| (s, 960_000_000)).collect();
    let loading: Vec<_> = (0..8)
        .map(|i| (StatusKind::Loading, i * 300_000_000))
        .collect();
    let success: Vec<_> = (0..5)
        .map(|i| (StatusKind::Success, i * 240_000_000))
        .collect();
    let error: Vec<_> = (0..5)
        .map(|i| (StatusKind::Error, i * 240_000_000))
        .collect();
    let reduced_arrivals = [
        (StatusKind::Success, 0),
        (StatusKind::Success, 480_000_000),
        (StatusKind::Success, 960_000_000),
        (StatusKind::Error, 0),
        (StatusKind::Error, 480_000_000),
        (StatusKind::Error, 960_000_000),
    ];
    let reduced_loading = [
        (StatusKind::Loading, 0),
        (StatusKind::Loading, 2_400_000_000),
        (StatusKind::Loading, 4_800_000_000),
        (StatusKind::Loading, 7_200_000_000),
    ];
    vec![
        sheet(
            "The four states at 960 ms",
            &state_entries,
            4,
            &palette,
            &profile,
        ),
        sheet(
            "Loading loop: eight discrete packet poses, wraps at 2400 ms",
            &loading,
            4,
            &palette,
            &profile,
        ),
        sheet(
            "Success arrival: 0 / 240 / 480 / 720 / 960 ms, then still",
            &success,
            5,
            &palette,
            &profile,
        ),
        sheet(
            "Error arrival: 0 / 240 / 480 / 720 / 960 ms, then still",
            &error,
            5,
            &palette,
            &profile,
        ),
        sheet(
            "Reduced arrivals: fixed glyphs, palette-role settle at 0 / 480 / 960 ms",
            &reduced_arrivals,
            3,
            &palette,
            &reduced,
        ),
        sheet(
            "Reduced loading: fixed glyphs, gentle core-role cycle over 9600 ms",
            &reduced_loading,
            4,
            &palette,
            &reduced,
        ),
        sheet(
            "Quiet profile: ASCII, monochrome, static",
            &state_entries,
            4,
            &palette,
            &Profile::quiet(),
        ),
        compact(&palette, &profile),
        embedded(&palette, &profile),
    ]
}

fn sheet(
    title: &'static str,
    entries: &[(StatusKind, u64)],
    columns: u16,
    palette: &Palette,
    profile: &Profile,
) -> Preview {
    let rows = entries.len().div_ceil(usize::from(columns)) as u16;
    let mut buffer = Buffer::empty(Rect::new(0, 0, columns * 26 - 3, rows * 10 - 1));
    let design = StatusTheme::new();
    for (index, &(state, time)) in entries.iter().enumerate() {
        let x = index as u16 % columns * 26;
        let y = index as u16 / columns * 10;
        buffer.set_string(x, y, format!("{} ms", time / 1_000_000), palette.foreground);
        let context = SampleContext::new(ClipTime::from_nanos(time), palette, profile);
        design
            .sample(&context, state)
            .render(Rect::new(x, y + 2, 23, 7), &mut buffer);
    }
    Preview { title, buffer }
}

fn compact(palette: &Palette, profile: &Profile) -> Preview {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 64, 9));
    let design = StatusTheme::new();
    let context = SampleContext::new(ClipTime::ZERO, palette, profile);
    for (column, width) in [11, 7, 3, 1].into_iter().enumerate() {
        let x = column as u16 * 16;
        buffer.set_string(x, 0, format!("width {width}"), palette.foreground);
        for (row, state) in STATES.into_iter().enumerate() {
            design
                .sample(&context, state)
                .render(Rect::new(x, row as u16 * 2 + 2, width, 1), &mut buffer);
        }
    }
    Preview {
        title: "Compact: static labels or distinct markers; narrow views cannot fit full words",
        buffer,
    }
}

fn embedded(palette: &Palette, profile: &Profile) -> Preview {
    let area = Rect::new(0, 0, 80, 11);
    let mut buffer = Buffer::empty(area);
    for x in 0..area.width {
        for y in [0, area.height - 1] {
            buffer[(x, y)].set_char('-').set_style(palette.muted);
        }
    }
    for y in 0..area.height {
        for x in [0, area.width - 1] {
            buffer[(x, y)].set_char('|').set_style(palette.muted);
        }
    }
    buffer.set_string(
        3,
        0,
        " Import preview / host-owned details ",
        palette.foreground,
    );
    buffer.set_string(
        32,
        3,
        "The host explicitly selected Success",
        palette.highlight,
    );
    buffer.set_string(
        32,
        5,
        "Visual age: 960 ms, arrival settled",
        palette.foreground,
    );
    buffer.set_string(
        32,
        6,
        "Operation data and retry controls belong",
        palette.foreground,
    );
    buffer.set_string(
        32,
        7,
        "to the host; this example performs no task",
        palette.foreground,
    );
    let context = SampleContext::new(ClipTime::from_nanos(960_000_000), palette, profile);
    StatusTheme::new()
        .sample(&context, StatusKind::Success)
        .render(Rect::new(3, 2, 23, 7), &mut buffer);
    Preview {
        title: "Ordinary Widget embedding beside authoritative host content",
        buffer,
    }
}

fn html_cells(buffer: &Buffer) -> String {
    let mut output = String::new();
    for (index, cell) in buffer.content.iter().enumerate() {
        if index > 0 && index % usize::from(buffer.area.width) == 0 {
            output.push('\n');
        }
        let weight = if cell.modifier.contains(Modifier::BOLD) {
            "bold"
        } else {
            "normal"
        };
        write!(
            output,
            "<span style=\"color:{};background:{};font-weight:{weight}\">{}</span>",
            css_color(cell.fg, "inherit"),
            css_color(cell.bg, "transparent"),
            escape(cell.symbol())
        )
        .unwrap();
    }
    output
}
fn css_color(color: Color, reset: &str) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Reset => reset.into(),
        // The capture profiles use the default RGB palette or monochrome Reset.
        // Other colors remain valid should the host preview palette be changed.
        Color::Black => "#000000".into(),
        Color::Red => "#800000".into(),
        Color::Green => "#008000".into(),
        Color::Yellow => "#808000".into(),
        Color::Blue => "#000080".into(),
        Color::Magenta => "#800080".into(),
        Color::Cyan => "#008080".into(),
        Color::Gray => "#c0c0c0".into(),
        Color::DarkGray => "#808080".into(),
        Color::LightRed => "#ff0000".into(),
        Color::LightGreen => "#00ff00".into(),
        Color::LightYellow => "#ffff00".into(),
        Color::LightBlue => "#0000ff".into(),
        Color::LightMagenta => "#ff00ff".into(),
        Color::LightCyan => "#00ffff".into(),
        Color::White => "#ffffff".into(),
        Color::Indexed(_) => reset.into(),
    }
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
