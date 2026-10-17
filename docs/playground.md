# Live playground and Rust export

The gallery is a local host application using the same typed catalogue designs as
an embedding application. It does not introduce a renderer, public registry,
clipboard service, network dependency, or extra package.

```sh
cargo run -p spark-gallery
cargo run -p spark-gallery -- --inspect --design telemetry-embers --palette amber
cargo run -p spark-gallery -- --help
```

## Inspect and tune

Press `i` to open or close the Rust inspector. It shows the selected live sample
and the actual source that export will produce. Wide terminals use side-by-side
panels; smaller terminals stack them. Tiny or empty layouts clip safely. The
initial source viewport starts at the design, palette and profile configuration;
`Home` exposes imports and playback setup. Closing it restores the ordinary
isolated/embedded gallery without resetting selection, playback or configuration.

- `1`–`6` / `Tab`: select a design; reset clip time and requested dimensions to its preferred size
- `Space`: pause at the last displayed pose, or resume from that pose
- `r`: restart at clip zero, preserving whether playback is paused
- `[` / `]`: seek backward/forward by 100ms, also while paused
- `,` / `.`: bounded speed steps, 0.25×, 0.5×, 1×, 2×, 4×
- Arrow keys: requested width 1–256, height 1–64, fitted to the current viewport
- `c`: default/amber palette; `a`: Unicode/ASCII; `m`: truecolor/ANSI16/monochrome
- `p`: reduced/full motion; `s`: static/full motion
- `n`: next seed (the four seeded designs); `-` / `+`: caller input in 0–100
- `u`: known/unknown assembly progress; `v`: isolated/synthetic host presentation
- In the inspector, `PgUp` / `PgDn` scroll, `h` / `l` pan horizontally, `Home` resets to the start, `End` reaches the end
- `e`: export current configuration at the displayed clip position, then exit
- `q` / `Esc` / `Ctrl-C`: quit without exporting

The selected design alone requests animation. Paused, static, completed and tiny
fallback samples retain the existing no-animation behavior; input and resize can
still redraw them. Source inspection never advances the library's state. A
terminal resize can change fitted geometry, so the inspector and export show the
current actual rectangle in addition to the requested dimensions.

## Export without a terminal

Use `--export-rust` for clean stdout suitable for shell redirection. This path
never initializes raw mode or an alternate screen. The interactive `e` path exits
the alternate screen and restores terminal settings before writing any source.
No export happens just because the inspector was opened or the app was closed.

```sh
cargo run -q -p spark-gallery -- --export-rust \
  --design telemetry-embers --size 52x17 --terminal 140x42 \
  --palette amber --glyphs ascii --colors ansi16 --motion reduced \
  --speed 0.5 --seed 42 --input 40 --paused --clip-nanos 750000000 \
  > example.rs
```

`--snapshot` prints gallery cells and accepts the same configuration, including
`--inspect`. Both offscreen modes default to a 100×32 terminal and a 750ms clip
position; live mode defaults to clip zero. Live mode takes its terminal size from
the actual terminal and rejects `--terminal`. `--smoke` does not accept overrides.

The parser rejects unknown, repeated, missing and malformed options rather than
silently dropping settings. Dimensions are bounded before allocation: requested
art is 1–256×1–64, the offscreen terminal is 0–256×0–128. Seeds and clip nanoseconds
accept the full unsigned 64-bit range. `--input` is an integer 0–100; `--unknown`
selects unknown assembly progress instead. `--speed` accepts the five steps above.
`--paused`, `--embedded` and `--inspect` are explicit switches. `--help` lists the
complete vocabulary and stable design IDs.

## What the Rust program contains

The output is a complete Rust program for a host with `spark-catalog`,
`spark-runtime`, and `ratatui` 0.30.2 dependencies. It contains:

- The selected typed constructor and applicable seed or authored reveal text
- The selected palette, glyph/color/motion profiles and current typed input
- A `configured_playback(host)` mapping with the exact clip nanoseconds, speed and pause state
- A `snapshot()` returning an ordinary `Buffer` with the terminal extent and actual fitted art rectangle
- The synthetic host border and protected title rectangle for Border Runner

For example, place the output at `apps/gallery/examples/playground.rs` in this
checkout and run `cargo run -p spark-gallery --example playground`. The program deliberately renders an
offscreen buffer without printing it or taking over a terminal. To embed the
artwork, store the design, palette and profile once, use your host's geometry and
base content, and render the sample through your ordinary Ratatui widget path.
The crates are source dependencies, not published registry releases.

For a separate Cargo application beside a `spark` checkout, the matching dependency
section is (adjust only the two local paths):

```toml
[dependencies]
spark-catalog = { path = "../spark/crates/catalog" }
spark-runtime = { path = "../spark/crates/runtime" }
ratatui = { version = "=0.30.2", default-features = false, features = ["std", "crossterm"] }
```

Use Rust 1.90 or newer. Save the export as `src/main.rs`; `cargo run` compiles and
renders the buffer without displaying a terminal UI. For a minimal one-design
library consumer without a terminal backend, see the existing
[consumer fixture](../fixtures/consumer/Cargo.toml).

The export's clip position and speed are independent of reduced/static motion.
Reduced motion is a rendering profile; pause belongs to the host playback mapping.
The host still supplies monotonic time to `Playback`, samples that mapping, owns
input/data changes and scheduling, and renders finite final poses. The generated
mapping does not read a clock or spawn a task. See [integration](integration.md).

Requested dimensions and isolated/embedded/inspector layout choices appear in
source comments; the actual rectangle appears as compilable `Rect` values.
Gallery chrome, its dark panel backgrounds and status text are not exported as
library settings. The snapshot uses a fresh default buffer. Border Runner needs
fresh host base content before its style-only sample, so its illustrative border,
text and title exclusion are included. Other hosts should replace that base with
their own and preserve the appropriate protected regions. Designs without seed or
data inputs do not acquire artificial parameters just to mirror unused controls.

## Verification

```sh
cargo test -p spark-gallery --all-targets --locked
cargo clippy -p spark-gallery --all-targets --locked -- -D warnings
cargo run -p spark-gallery -- --smoke
cargo run -p spark-gallery -- --snapshot --inspect
# Optional Unix terminal-lifecycle checks, after building the gallery:
python3 apps/gallery/tests/terminal.py target/debug/spark-gallery
```

`src/export_tests.rs` includes ten exact generated Rust programs from
`tests/fixtures/` as compiler-checked modules. It verifies byte equality with the
current exporter, cell equality against the existing typed live adapter, and
playback position/rate/pause semantics. Cases cover all six designs, known and
unknown progress, nondefault palettes/profiles/inputs/seeds, maximum clip/seed,
embedded inspector geometry, and empty/tiny border hosts. These are source-export
fixtures, not replacements for renderer goldens or visual qualification.

Additional tests exercise invalid CLI options, source-only stdout without a TTY,
repeated inspector opening/closing, bounded scroll and controls, paused seeking
and restart/resume, small-terminal resizing, and cleared code after dismissal.
The optional Unix PTY script also checks repeated inspector changes, paused
seek/back, resize, quit and export, exact restoration of terminal flags, and the
absence of terminal escapes after exported source. No captures are saved. These
checks do not qualify every native terminal, font, or operating system.
