# Compose explicit-time effects

`spark_runtime::composition` maps one caller-supplied clip timestamp into finite
segments or parallel tracks. The host renders ordinary typed widgets in a chosen
order. There is no effect registry, callback engine, hidden clock or task framework.

## Sequences and parallel tracks

- `Sequence::new(&[Period])` borrows 1–256 positive durations and rejects an
  overflowing total. Selection is bounded O(n), allocation-free and stateless.
- At an internal seam, the next segment starts at local zero. At/beyond total
  duration, the last segment holds its exact final pose and `complete` is true.
- `ClipTrack::new(start, duration)` rejects end overflow. `Parallel` borrows
  1–256 tracks and samples them in the supplied order, without sorting or painting.
- Track phase is `Before` before its start, `Active` through its exclusive end,
  and `Finished` at/beyond the end. Before/Finished samples retain zero/final
  local time. The host decides whether to omit or hold a finished layer.
- `Parallel::next_boundary` includes pending starts, so a quiet current layer
  cannot accidentally leave a later layer asleep.

These timelines are finite. Use the existing `Period` explicitly if the host
intends to repeat a visual arrangement. Domain status must not change merely
because a timeline finishes. For static/reduced-motion presentations, choose an
intentional final/quiet composition; timeline helpers do not inspect profiles.

## Repaint advice

Samples expose their outer `start`, `end`, `local_time`, and strictly future
`next_boundary()`. `map_deadline(child_time)` maps a future local `ChangeHint::At`
back into the outer timeline and clamps it to the segment end. Stale local
deadlines, not-yet-started tracks, and finished segments return no mapped deadline.

Keep a child's `ChangeHint::Continuous(interval)` unchanged: its interval describes
host presentation cadence, not clip-time speed. Feed the mapped child hint and
the outer boundary separately into `RepaintPlanner::next_deadline`, then merge
with `RepaintPlanner::earliest`. Also merge host input/data deadlines. The existing
playback object suppresses paused/hidden animation requests; it does not suppress
painting current content on unrelated redraws.

## Finite transitions

`Transition` reuses `FiniteClip` and supports linear or smoothstep easing. Its
integer `weight()` is in `0..=WEIGHT_SCALE` (one million), rounded down. Smoothstep
first quantizes the linear fraction, then evaluates its polynomial with wide
integer intermediates. `interpolate(from, to)` handles ascending and descending
`u16` coordinates; signed displacement rounds toward zero. Endpoints are exact.
Quantized positions can settle early, so use the separate exact-time `complete()`
query when a visual sequence needs its terminal frame. Neither query reports a
business-operation result. No alpha/background-color blending is assumed.

## Runnable host-selected loading → success/error

```sh
cargo run -p spark-catalog --example composition
cargo run -p spark-catalog --example composition -- --smoke
```

The example renders actual Rust cells: parallel comet/border loading layers, then
a short anticipation and smoothly positioned text reveal after an explicit host
success/error choice. Its finite terminal pose stays visible. It protects a host
label and exercises empty/tiny, quiet, repeated and far-future sampling. Printed
glyphs omit styles and do not qualify physical terminal/font behavior.
