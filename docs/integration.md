# Integration without handing over your application

The host keeps terminal initialization/restoration, input/focus, layout, application
state, semantic labels, data acquisition and when frames are presented. Spark samples
cells. It never completes a request, fetches a metric or owns a timer.

## Minimal use

Store one typed design; create its borrowed view during a draw with explicit
`SampleContext`. Pass the same requested size to `change_hint` and `render`.
`CometSample::inline(width)` returns an owned padded Line for widths 0–256; it is
an allocating convenience path. `fixtures/consumer` is a compiled standalone
one-design application; `fixtures/consumer-all` uses all six without gallery code.

Use `Progress::known(value)` or `Progress::unknown()`, and `Level::new(value)`.
Nonfinite/out-of-range values are rejected. The library never guesses a percentage;
authoritative labels remain host-rendered. The synthetic examples supply their own
illustrative data and explicitly count their own producer polls.

## Composition and protected content

1. Draw base host content freshly
2. Render owned art rectangles and explicitly permitted decorators in your chosen order
3. Keep title, selection, caret and media rectangles protected; draw host overlays

`BorderRunner` grants only border foreground/modifier patches. For unsupported
wide/combining `PrismReveal` text, draw the original text first; its fallback only
styles eligible cells inside that text's bounded footprint. All scoped surfaces
preserve existing whole wide/special-cell footprints conservatively. Symbols outside
an owned glyph mask and unapproved style fields remain unchanged.

## Playback and advice

Direct sampling only needs `ClipTime`. Optional `Playback` maps a monotonic elapsed
`HostTime` using a positive rational rate. Call pause/resume/seek/restart/set_rate/
set_hidden with current host time; backwards command timestamps are errors with no
mutation. Hidden policy defaults to Freeze; CatchUp is explicit. Redundant commands
and equivalent fractions preserve anchor phase. Real state changes reanchor at
integer nanosecond precision.

`ChangeHint::At` is an absolute future clip timestamp. `Continuous` is preferred
host presentation cadence; `None` waits for host invalidation. `RepaintPlanner`
converts discrete boundaries with ceiling arithmetic and applies a host frame cap.
It creates no task or wait. Coalesce missed presentation deadlines and draw the
latest pose rather than replaying a backlog. Recompute after size, profile, input,
visibility, operation or playback changes.

No advice means no animation-only wake, not no painting. Paused/static/finished
views still render when the host redraws for another reason. Render a finite
visual's last frame before discarding its time-driven deadline.

## Three executable host styles

All three reuse `crates/catalog/examples/support/mod.rs`, which holds synthetic
layouts and ordinary typed designs, not a required public registry.

```sh
cargo run -p spark-catalog --example sync_host
cargo run -p spark-catalog --example async_host
cargo run -p spark-catalog --example paced_host
```

Use `--help` for controls or `--smoke` for deterministic offscreen coverage.

- **Synchronous dirty/deadline:** polls terminal input until the earliest independent
  data or presentation deadline; blocks on input when neither exists
- **Async event:** Tokio belongs only to this example/dev dependency graph. One
  bounded input adapter and cancellable `select!` waits replace obsolete deadlines;
  there is no task per animation or queued obsolete-frame stream. The synchronous
  Crossterm adapter polls at 25 ms to allow orderly shutdown; that is host input work,
  not a renderer animation wake or a claim of a fully wake-free process
- **Already paced:** keeps its pre-existing presentation cadence and samples at that
  time. Art adds no presentation task or animation-only wake. The slower data producer
  remains independent of rendering

## Four synthetic layouts and lifecycle identity

Table/status keeps authoritative percentages beside inline/bounded art. The art
follows a stable selected entity ID after row sorting. Request/result handles working,
complete, cancelled and replaced generations without letting stale results attach
to a new operation. Dashboard separates producer and presentation cadence while
embedding a larger scene. Reader preserves selected text, caret, wide content and
reserved media.

Use `(entity_id, operation_generation)` for visual lifecycle identity. Replacement
increments generation and resets its optional visual playback; cancellation rejects
late updates. Rendering cannot transition business state. Tests exercise obsolete
updates, rapid replacement, row reorder, resizing, repeated show/hide, long stalls,
finite terminal presentation and static/paused/hidden/complete inactivity.

These are reusable synthetic examples, not claims of integration with an external
application. The independent `fixtures/holdout` CrossWeave design proves public
runtime extension without catalogue or gallery access.

## Compound controls and displayed-pose preservation

Example hosts retain the actual clip positions submitted at presentation. Pausing
freezes those stored values, even if a rate change has already re-anchored playback
between draws. Reconstructing the previous pose from its host timestamp and the
new mapping is incorrect. Explicit restart, replacement and new completion clear
the applicable saved reveal position so obsolete results cannot return.

[Lifecycle and actual-process endurance evidence](endurance.md) covers these
compound commands, bounded burst draining, resize storms and producer recovery.
