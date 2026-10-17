# Signal seal: coordinated status theme

Status: implemented source candidate, visual revision 1. The existing six-design
gallery selection is unchanged. This separate showcase demonstrates a coordinated
family, not four new registry entries or a new package. Source:
[`status.rs`](../crates/catalog/src/status.rs). Public API lives under
`spark_catalog::status`.

## Design brief

A chamfered seal, quiet side rails, inset symbol, and centered literal label make
the four states recognizable as one family. The native composition is **23×7**,
centered without stretching. Color, shape, and pacing share the same vocabulary:

- **Loading:** paired packets gather through an upper channel, meet at the core,
  and release through the lower branch. Eight discrete poses repeat over 2400 ms
- **Empty:** an open, unoccupied cradle sits inside a quietly interrupted seal.
  Its dotted rails and empty space remain still at every supplied time
- **Success:** a check joins and the enclosure closes in four arrival steps over
  960 ms. A brief pair of small seam accents resolves into a stable final check
- **Error:** the enclosure fragments settle into a split seal around an
  exclamation. Four arrival steps over 960 ms finish with a stable, visible fault
  symbol, with no perpetual shaking or flashing

The full-size labels `LOADING`, `EMPTY`, `SUCCESS`, and `ERROR` are present from
the first frame. Arrival motion never delays the host-selected meaning. The
distinct silhouette and literal label avoid relying on red/green color alone.
This is original procedural cell artwork made for Spark with AI assistance; no
external images, character references, generated raster assets, or asset services
were used. Repository licensing and third-party terms continue to apply.

## Host-selected data, immutable sampling

```rust
use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::status::{StatusKind, StatusTheme};
use spark_runtime::{ClipTime, Palette, Profile, SampleContext};

let theme = StatusTheme::new(); // Store in host state.
let palette = Palette::default();
let profile = Profile::default();
let area = Rect::new(2, 1, 23, 7);
let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));

// The application chooses both state and its visual age.
let status = StatusKind::Success;
let context = SampleContext::new(
    ClipTime::from_nanos(480_000_000), &palette, &profile,
);
let protected = [Rect::new(0, 0, 30, 1)];
let sample = theme.sample(&context, status).with_protected(&protected);
let next_visual_change = sample.change_hint(area.as_size());
let visually_settled = sample.transition_complete(area.as_size());
let borrowed_sample = &sample;
borrowed_sample.render(area, &mut buffer);
```

`StatusKind` is an explicit presentation choice. Loading does not become success
when enough time passes; empty is not inferred from a collection; error does not
retry or disappear. The library stores no operation identity, previous state,
completion callbacks, elapsed accumulator, timer, clock, or task data. Rendering
the same semantic inputs and base buffer produces the same cells, including
late, repeated, shuffled, and maximum timestamps.

`context.time` is the visual age supplied by the host. To replay a success/error
arrival, supply zero when selecting the new state and advance your own visual
clock. A state change does **not** implicitly reset the time. Supplying an old or
late timestamp to a newly selected success/error state may show its settled pose
immediately. Stable entity/generation handling belongs to the host. The separate
host-adapter `TaskStatus` does not implicitly map into this theme: queued and
cancelled, for example, need an application-specific presentation decision.

### API

- `StatusTheme::new()` / `Default`: reusable, immutable configuration
- `with_loading_period(Period)` / `loading_period()`: positive full-motion period
- `with_transition_duration(Period)` / `transition_duration()`: positive arrival
  duration shared by success and error
- `StatusTheme::WIDTH` / `HEIGHT`: native width 23 and height 7
- `sample(&SampleContext, StatusKind)`: borrowed `StatusSample`
- `StatusKind::label()`: the canonical uppercase ASCII label
- `StatusSample::status()`: exactly the host-supplied selection
- `with_protected(&[Rect])`: absolute host-owned no-write regions
- `change_hint(Size)`: next absolute clip-time boundary or `ChangeHint::None`
- `transition_complete(Size) -> Option<bool>`: success/error **visual** arrival
  completion only. Loading and empty return `None`
- `Widget` for both `StatusSample` and `&StatusSample`

Only a validated positive `Period` can configure timing. There is no floating
point duration conversion, implicit clamping, or dependency on optional timeline
composition helpers.

## Geometry and paint ownership

Requests at least 23×7 center the native 23×7 composition. Larger requests leave
all cells outside that rectangle untouched. Smaller requests center a static
fallback inside a rectangle capped at 23 columns and 7 rows:

| Available width | Static fallback |
| --- | --- |
| 11 or more | A marker plus full label: `[~] LOADING`, `[-] EMPTY`, `[+] SUCCESS`, `[!] ERROR` |
| 7–10 | The full literal state label |
| 3–6 | `[~]`, `[-]`, `[+]`, or `[!]` |
| 1–2 | `~`, `-`, `+`, or `!` |
| Zero width or height | No writes |

Compact and empty geometry bypass arrival animation. `transition_complete(size)`
therefore returns `Some(true)` for success/error in those presentations. This
does not claim that a label fits, the buffer is visible, or a business operation
is complete. Narrow markers need surrounding host labels for accessibility;
they are deliberately not truncated words. Clipping or protection can hide any
part of the composition, including its label.

The sample owns its bounded composition and explicitly erases unused cells
there, making state replacement independent of the previous theme pose. Always
redraw host base content as usual. All writes use the public `OpaqueSurface`:
requested-local coordinates survive clipping; protected rectangles, existing
whole wide-glyph footprints, continuation cells, forced-width footprints, special
diff cells, and neighbors remain untouched. A large request never expands the
art beyond 161 cell-write attempts. The widget allocates no intermediate buffer
or heap-owned animation state; writer row-prefix scans and exclusion work still
scale with the caller's buffer and protection list. No general constant-total-
cost or measured-performance claim is implied.

## Palette and motion profiles

The supplied palette supplies all style roles. `muted` draws the enclosure,
`accent` marks gates and moving seams, `highlight` carries the bright core, and
bold `foreground` keeps the literal label readable. No extra semantic colors,
background assumptions, or theme-specific palette globals are introduced.

ASCII uses ordinary punctuation. Unicode replaces one-column rails, diagonals,
corners, dots, and core accents; the same silhouette and words remain. Truecolor,
ANSI16, and monochrome all resolve through `Palette::resolve`, including underline
colors. Unknown backgrounds stay unknown. Monochrome retains geometry and bold
label/core accents.

- **Full:** eight loading poses; four success/error arrival steps and terminal pose
- **Reduced:** loading glyphs remain fixed while the core cycles through four
  restrained roles over four times the configured period. Success/error glyphs
  are their final silhouettes immediately, with two subdued style steps before
  the terminal style at the same configured duration. Visual completion remains
  false until that style settle finishes
- **Static:** loading uses its deliberate parked channel/core poster, empty is
  unchanged, and success/error immediately use their final glyphs and styles
- **Quiet:** `Profile::quiet()` combines static, ASCII, and monochrome

These are explicit host choices, not terminal or accessibility-setting detection.
For no motion at all, choose static/quiet. A reduced-motion role change may become
visually indistinguishable under a custom palette or monochrome; hints remain
conservative rather than analyzing whether two resolved styles happen to match.

## Exact boundaries and final-frame ownership

`change_hint` reports discrete boundaries using integer arithmetic and ceiling
division. Short positive periods may skip poses; they never repeat history or
divide by zero. A one-nanosecond loading loop is necessarily constant at integer
nanosecond sample precision and returns no animation hint. Reduced loading uses
checked wide arithmetic when mapping the next boundary back to clip time. If the
next boundary exceeds `ClipTime::MAX`, it returns no hint.

Success/error hints include their terminal timestamp. Render that final sample
before dropping time-driven redraws; subsequent samples keep the same final
appearance forever. Empty, static, compact, empty-geometry, and settled samples
request no animation-only wake. Hints use requested size; the host still owns
hidden/paused suppression, scheduling, data invalidation, and visibility. Querying
completion does not paint the final frame or notify an external system.

## Reproducible previews and focused verification

The standalone example renders actual widgets into Ratatui buffers and serializes
those cells. It uses no parallel art implementation, screenshots of invented
mockups, image generator, terminal state, network, or system clock. Nine sheets
show all four states, every loading pose, every finite arrival step, reduced-role
settles, quiet/compact views, and a normal host-content embedding.

```sh
cargo run -p spark-catalog --example status_theme --locked -- --text
cargo run -p spark-catalog --example status_theme --locked -- --html > /tmp/spark-status-theme.html
cargo run -p spark-catalog --example status_theme --locked -- --smoke
cargo test -p spark-catalog --test status --locked
cargo clippy -p spark-catalog --test status --example status_theme --locked -- -D warnings
```

Open the HTML locally for Unicode and exact palette-role styles. The HTML page's
dark background is the preview host's choice, not a background chosen by the
theme. Plain-text output preserves ASCII cells but cannot show color/style changes.
Outputs include a deterministic non-cryptographic FNV-1a64 fingerprint of the
compiled `status.rs` source. For an archived review, record source hashes and the
toolchain alongside the capture:

```sh
sha256sum crates/catalog/src/status.rs crates/catalog/examples/status_theme.rs
rustc --version --verbose
```

Focused tests independently assert terminal silhouettes, all state labels, exact
short-period boundaries, end-minus-one/end/MAX samples, shuffled state changes,
reduced/static profiles, palette resolution, compact fallback truthfulness,
requested-coordinate clipping, protected/wide/special cells, and extreme geometry.
The example smoke check compares repeated captures and exercises Unicode glyph
validation. These checks are not a terminal-font, cross-platform, benchmark,
package-publication, or independent aesthetic qualification claim. Root workspace
checks and publication remain separate integration gates.
