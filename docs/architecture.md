# Architecture

Spark is an explicit-time Rust/Ratatui cell-art library. The reviewed F01–F06
slice has been extended locally through the remaining foundation implementation.
The six designs are implementation candidates, not a published or universally
qualified release. See [foundation status](foundation-status.md) for evidence and
remaining gates, and [API review](foundation-api-review.md) for independent review.

## Four package boundaries

| Package | Owns | Never owns |
| --- | --- | --- |
| `spark-runtime` | Time values, optional playback mapping, repaint advice, palette/profile, permission-limited writers, static frame selection | Clocks, IO, threads, domain state, registry, assets or parsing |
| `spark-catalog` | Six typed immutable designs and borrowed samples | Terminal lifecycle, input/data acquisition, timer tasks, global state |
| `spark-gallery` | Terminal setup, input, host timing, inspection controls | A parallel renderer or required consumer framework |
| `xtask` | Asset validation/codegen, exact cells, previews, measurements, package audits | A runtime dependency of either library |

Exactly four workspace packages remain. Standalone downstream fixtures are excluded
from the workspace and prove application/extension boundaries. The runtime depends
on `ratatui-core` and `unicode-width`; catalogue depends on core and runtime.
Tokio is an **example/dev dependency only**, used by the async host. JSON, hashing,
terminal backends and image encoders remain tooling/application concerns.

## Sampling, ownership and styles

The host stores a typed design and supplies `SampleContext { time, palette,
profile }`, caller geometry and typed data. A sample implements ordinary `Widget`
by value and reference. Equal semantic inputs, base content and decorator order
produce equal cells on the qualified target. Sampling never advances state.

`OpaqueSurface` replaces complete safe single-column cells in an owned rectangle.
`MaskedSurface` further limits replacement to an explicit region/border mask.
`StyleSurface` only patches explicitly granted foreground/background/modifier/
underline-color fields. It has no glyph-writing capability. All mutable buffers
remain private; requested-local coordinates are preserved under clipping.
`Keep`, explicit `Erase`, and full `Replace` remain distinct.

All surfaces conservatively skip every part of pre-existing wide glyphs and
special diff cells, including clipped origins, forced widths and continuation
cells. This preserves whole host glyphs even when an effect has rights to the
whole region; it is intentionally not a general grapheme replacement engine.
Explicit protected rectangles exclude titles, selection, caret and media.
Style fallbacks preserve wide/combining symbols rather than substituting them.

Render base content freshly, apply decorators in a declared order, then render
host overlays. A mutated previous frame is not the same base input. No surface
allocates memory. Wide-footprint detection scans row prefixes and exclusion
checks scale with the supplied list; bounded art geometry is not a constant-total-
cost promise for arbitrary host buffers.

## Six intentionally different designs

| Design | Driver and surface | Fit and fallback |
| --- | --- | --- |
| Segmented Comet | Repeating segmented trail; widget and owned `Line` | Adaptive 8–256×1–3; tiny static point |
| Assembly Progress | Caller known/unknown progress; widget | 12–80×3 gantry/conveyor; truthful static meter |
| Border Runner | Repeating style-only border highlight | Explicit 3–256×3–128 border; otherwise no-op |
| Telemetry Embers | Caller level plus direct seeded analytic particles; widget/scene | 8–120×4–40; tiny static level point |
| Pocket Mascot | Unequal-hold authored complete frames; widget | Native 12×6 centered without stretching; compact poster |
| Prism Reveal | Finite masked text; style-only unsupported-text fallback | At most 256 text scalars; final readable pose |

These are direct typed constructors, not a public heterogeneous registry. A scene
uses the ordinary widget contract. Comet inline output is exactly the requested
0–256 columns, including padding; it allocates an owned one-row buffer/spans.
It does not claim allocation-free text. Inline output has its own owned coordinate
space, so absolute host protected rectangles do not apply to that adapter.

Palette roles remain ordinary styles; unknown background stays unknown. Explicit
ASCII/Unicode, truecolor/ANSI16/monochrome and full/reduced/static profiles do not
auto-detect terminals. Semantic user text remains meaningful in fallbacks; unsupported
wide text receives conservative styling only. No special font or AI service is required.

## Time domains and playback

`ClipTime` is an unsigned nanosecond visual position, up to `u64::MAX`.
`Period` is strictly positive and validates `Duration` conversion. Loop arithmetic
reduces first and uses wide intermediates. `FiniteClip` clamps to the terminal
pose and returns queryable completion; it never fires a callback or completes a
business operation. `FrameSequence` validates 1–256 strictly increasing positive
cumulative frame ends and uses direct binary selection, with absolute next boundaries.

`HostTime` is monotonic elapsed time supplied by the application. Optional
`Playback` maps it to clip time using a positive rational `PlaybackRate`, anchors,
play/pause state and explicit hidden Freeze/CatchUp policy. Freeze is the default.
Pause/resume, seek, restart, rate and visibility commands reject backwards host
timestamps before mutation. Redundant commands preserve fractional anchor phase.
A real state/rate change reanchors at integer nanosecond precision (less than one
clip-nanosecond fractional truncation). Queries do not mutate state; before-anchor
host queries return the anchor pose. Arbitrary visual seeking uses `seek`.

`ChangeHint` is None, an absolute future clip boundary, or a preferred continuous
host cadence. `RepaintPlanner` uses ceiling conversion for discrete boundaries,
respects a host presentation cap, returns strictly future deadlines, and suppresses
paused/hidden requests. Unrepresentable deadlines return None. Hosts merge advice
with their own input/data deadlines, coalesce stalls, and paint frozen/completed
art on unrelated redraws. A finite visual needs its terminal frame before dropping
its time-driven requests. Helpers own no waiting or task/thread.

## Authored assets and review

The mascot source is versioned tooling-only JSON with complete coverage operations,
glyph/role references and positive durations. Central source/geometry/frame/glyph/
cell/duration limits are checked before expanding cells. Generated Rust tables are
checked in and regenerated deterministically; parsing is absent from consumers.
See [authoring](authoring.md) for limits and workflow.

Shared contracts and expected-image changes require independent review. New content
cannot grant itself wider paint rights, add dependencies or silence failures. The
F21 downstream holdout uses public extension points rather than catalogue internals.
No license, package publication or cross-platform execution is implied by local work.
