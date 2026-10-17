//! Bounded, noninteractive replay/export options, without a command-line dependency.
use crate::{app::App, design::Selection};
use ratatui::layout::Size;
use spark_runtime::{ClipTime, ColorProfile, GlyphProfile, Motion, Palette};
use std::{collections::HashSet, time::Duration};

pub const HELP: &str = "spark-gallery

1-6/tab design | q/Esc quit | space pause | r restart | [ ] scrub 100ms
arrows size | c palette | a ASCII | m colors | p reduced | s static
, . speed | n seed | - + input | u unknown progress | v isolated/embedded
i Rust inspector | PgUp/PgDn scroll | h/l horizontal | Home/End
e export displayed configuration to stdout and exit (after terminal restoration)

--smoke          deterministic offscreen checks, no terminal required
--snapshot       print gallery cells (100x32 at 750ms unless overridden)
--export-rust    print a complete Rust program only; no terminal required
--help, -h       show this help

Live, --snapshot and --export-rust accept these configuration options:
--design ID      segmented-comet, assembly-progress, border-runner,
                 telemetry-embers, pocket-mascot, prism-reveal
--size WxH       requested art size, 1..256 by 1..64
--terminal WxH   offscreen terminal, 0..256 by 0..128 (default 100x32)
--palette NAME  default | amber
--glyphs NAME   unicode | ascii
--colors NAME   truecolor | ansi16 | monochrome
--motion NAME   full | reduced | static
--speed RATE    0.25 | 0.5 | 1 | 2 | 4
--seed INTEGER  unsigned 64-bit seed (used by the first four designs)
--input PERCENT integer 0..100 (assembly progress / embers level)
--unknown       assembly progress is unknown
--paused        freeze the exported/live playback mapping
--clip-nanos N  unsigned 64-bit clip position (default 750000000 offscreen; 0 live)
--embedded      synthetic host layout
--inspect       show the Rust inspector (also changes fitted geometry)

Example: cargo run -q -p spark-gallery -- --export-rust --design telemetry-embers \
  --palette amber --input 40 --seed 42 --paused > example.rs
The export captures fitted geometry and sample configuration, not gallery chrome.
Your app owns clocks, scheduling, current input, placement and base content.
";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Live,
    Smoke,
    Snapshot,
    Export,
    Help,
}
pub struct Options {
    pub command: Command,
    pub app: App,
    pub terminal: Size,
}
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut args = args.into_iter();
    let mut command = Command::Live;
    let mut app = App::default();
    let mut terminal = Size::new(100, 32);
    let mut requested = None;
    let mut clip = None;
    let mut seen = HashSet::new();
    let mut configured = false;
    while let Some(arg) = args.next() {
        if !seen.insert(arg.clone()) {
            return Err(format!("duplicate option {arg}"));
        }
        let mode = match arg.as_str() {
            "--smoke" => Some(Command::Smoke),
            "--snapshot" => Some(Command::Snapshot),
            "--export-rust" => Some(Command::Export),
            "--help" | "-h" => Some(Command::Help),
            _ => None,
        };
        if let Some(mode) = mode {
            if command != Command::Live {
                return Err(
                    "choose exactly one of --smoke, --snapshot, --export-rust or --help".into(),
                );
            }
            command = mode;
            continue;
        }
        configured = true;
        match arg.as_str() {
            "--unknown" => app.unknown = true,
            "--paused" => app.paused = true,
            "--embedded" => app.embedded = true,
            "--inspect" => app.inspector = true,
            "--design" => {
                let value = value(&mut args, &arg)?;
                app.selection = Selection::from_id(&value)
                    .ok_or_else(|| format!("unknown design {value}; use --help"))?;
            }
            "--size" => requested = Some(size(&value(&mut args, &arg)?, false)?),
            "--terminal" => terminal = size(&value(&mut args, &arg)?, true)?,
            "--palette" => {
                app.amber = match value(&mut args, &arg)?.as_str() {
                    "default" => false,
                    "amber" => true,
                    _ => return Err("--palette expects default or amber".into()),
                };
                app.palette = if app.amber {
                    Palette::amber()
                } else {
                    Palette::default()
                };
            }
            "--glyphs" => {
                app.profile.glyphs = match value(&mut args, &arg)?.as_str() {
                    "ascii" => GlyphProfile::Ascii,
                    "unicode" => GlyphProfile::Unicode,
                    _ => return Err("--glyphs expects ascii or unicode".into()),
                }
            }
            "--colors" => {
                app.profile.colors = match value(&mut args, &arg)?.as_str() {
                    "truecolor" => ColorProfile::TrueColor,
                    "ansi16" => ColorProfile::Ansi16,
                    "monochrome" => ColorProfile::Monochrome,
                    _ => return Err("--colors expects truecolor, ansi16 or monochrome".into()),
                }
            }
            "--motion" => {
                app.profile.motion = match value(&mut args, &arg)?.as_str() {
                    "full" => Motion::Full,
                    "reduced" => Motion::Reduced,
                    "static" => Motion::Static,
                    _ => return Err("--motion expects full, reduced or static".into()),
                }
            }
            "--speed" => {
                app.rate_milli = match value(&mut args, &arg)?.as_str() {
                    "0.25" => 250,
                    "0.5" => 500,
                    "1" => 1000,
                    "2" => 2000,
                    "4" => 4000,
                    _ => return Err("--speed expects 0.25, 0.5, 1, 2 or 4".into()),
                }
            }
            "--seed" => app.seed = integer(&value(&mut args, &arg)?, &arg)?,
            "--clip-nanos" => {
                clip = Some(ClipTime::from_nanos(integer(
                    &value(&mut args, &arg)?,
                    &arg,
                )?))
            }
            "--input" => {
                let input = integer(&value(&mut args, &arg)?, &arg)?;
                if input > 100 {
                    return Err("--input expects an integer in 0..100".into());
                }
                app.input = input as u8;
            }
            _ => return Err(format!("unknown option {arg}; use --help")),
        }
    }
    if configured && matches!(command, Command::Smoke | Command::Help) {
        return Err("--smoke and --help do not accept configuration options".into());
    }
    if command == Command::Live && seen.contains("--terminal") {
        return Err("--terminal is only for --snapshot or --export-rust; live size comes from your terminal".into());
    }
    let art = requested.unwrap_or_else(|| app.selection.preferred());
    app.width = art.width;
    app.height = art.height;
    app.designs = crate::design::Designs::new(app.seed);
    app.seek(
        clip.unwrap_or_else(|| {
            if command == Command::Live {
                ClipTime::ZERO
            } else {
                ClipTime::from_nanos(750_000_000)
            }
        }),
        Duration::ZERO,
    );
    Ok(Options {
        command,
        app,
        terminal,
    })
}
fn value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}"))
}
fn integer(value: &str, option: &str) -> Result<u64, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{option} expects an unsigned decimal integer"));
    }
    value
        .parse()
        .map_err(|_| format!("{option} is larger than u64::MAX"))
}
fn size(value: &str, terminal: bool) -> Result<Size, String> {
    let (width, height) = value.split_once('x').ok_or("size expects WxH")?;
    let width = integer(width, "width")?;
    let height = integer(height, "height")?;
    let minimum = if terminal { 0 } else { 1 };
    let maximum_height = if terminal { 128 } else { 64 };
    if !(minimum..=256).contains(&width) || !(minimum..=maximum_height).contains(&height) {
        return Err(format!(
            "size must be {minimum}..256 by {minimum}..{maximum_height}"
        ));
    }
    Ok(Size::new(width as u16, height as u16))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options(args: &str) -> Result<Options, String> {
        parse(args.split_whitespace().map(str::to_owned))
    }
    #[test]
    fn defaults_and_order_independent_configuration() {
        let a = options("--export-rust --size 71x8 --design pocket-mascot --paused --seed 18446744073709551615 --clip-nanos 18446744073709551615").unwrap();
        assert_eq!((a.app.width, a.app.height), (71, 8));
        assert_eq!(a.app.selection, Selection::Mascot);
        assert_eq!(a.app.seed, u64::MAX);
        assert_eq!(a.app.clip_time(Duration::MAX), ClipTime::MAX);
        assert_eq!(
            options("--export-rust").unwrap().app.displayed.as_nanos(),
            750_000_000
        );
        assert_eq!(options("").unwrap().app.displayed, ClipTime::ZERO);
    }
    #[test]
    fn rejects_invalid_unbounded_ignored_and_ambiguous_arguments() {
        for args in [
            "--export-rust --snapshot",
            "--smoke --seed 4",
            "--help --paused",
            "--export-rust --seed 4 --seed 5",
            "--export-rust --seed",
            "--export-rust --size 0x1",
            "--export-rust --size 257x1",
            "--export-rust --size 1x65",
            "--export-rust --terminal 256x129",
            "--export-rust --size 1x2x3",
            "--export-rust --input 101",
            "--export-rust --input -1",
            "--export-rust --speed NaN",
            "--export-rust --speed 0",
            "--export-rust --clip-nanos 18446744073709551616",
            "--export-rust --seed +1",
            "--export-rust --design nope",
            "--export-rust --palette nope",
            "--export-rust --glyphs nope",
            "--export-rust --colors nope",
            "--export-rust --motion nope",
            "--terminal 100x32",
            "--export-rust --nope",
        ] {
            assert!(options(args).is_err(), "accepted {args}");
        }
        assert!(options("--export-rust --terminal 0x0").is_ok());
    }
}
