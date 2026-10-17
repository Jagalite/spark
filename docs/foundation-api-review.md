# Independent foundation API review

Date: 2026-10-14. Reviewed the local working tree with Rust 1.90.0
(`1159e78c4`), `ratatui-core` 0.1.2, and application Ratatui 0.30.2 on
`x86_64-unknown-linux-gnu`. This review is an API/contract and downstream extension
exercise, not approval to publish or a claim that all foundation release gates
have passed.

## Decision

**Go for continued local integration and qualification.** The original F01–F06
vertical slice was reviewed before broadening it. Its immutable design/borrowed
sample pattern, positive period validation, direct explicit-time sampling,
requested-coordinate clipping, and conservative wide-footprint behavior remained
suitable for the additional surfaces and designs.

The new runtime remains usable independently of the catalogue. Style-only,
masked glyph, finite-time, frame-hold, playback, and repaint APIs were exercised
through public interfaces. The reviewer made no runtime or catalogue source
changes; implementation owners fixed the issues below, and independent tests
then verified the fixes.

## Defects found and fixed during review

1. **Border fit and advice disagreed.** `BorderRunner` painted nothing beyond its
   256×128 cap but still requested animation frames. Advice now uses the same
   supported-size bounds. Regression checks oversized, tiny, empty and maximum
   requested dimensions on a small actual buffer.
2. **Unsupported-text fallback exceeded the text footprint.** A short wide or
   combining string in a wider requested rectangle could recolor neighboring host
   text. `PrismReveal` now stores the text's display width and restricts styling
   to that bounded footprint. Regressions retain symbols, backgrounds, underline
   colors, protected last cells and all neighboring cells.
3. **Redundant playback commands discarded fractional phase.** Repeated `resume`
   at rate 1/3 could keep clip time at zero indefinitely because each no-op
   reanchored rounded integer time. Same-state play/pause/visibility/policy/rate
   commands now preserve the anchor while checking command ordering. Rational
   rates are normalized by their greatest common divisor, so alternating 1/3
   with 2/6 or a maximum-component equivalent is also a true no-op. Tests issue
   redundant and equivalent-rate commands at every host nanosecond and assert
   continued progress and unchanged absolute deadline phase.
4. **Zero-column text requested invisible animation.** A string consisting only
   of unsupported zero-width scalars had no paint footprint but requested frames.
   Reveal advice now requires positive display width. Regressions cover a lone
   combining mark and a zero-width joiner.

No test or expected image was disabled or replaced to conceal a failure.

## Independent evidence

The reviewer-owned suites add **40 tests**: 18 runtime tests, 17 catalogue tests,
and five tests in the separate holdout consumer. These supplement, rather than
replace, the original contract tests and each implementation owner's tests.

### Runtime: `crates/runtime/tests/extension_contract.rs`

- Repeated/reordered opaque and style writes over clipped wide glyphs, a
  one-column combining grapheme, escaped forced-width content, special diff cells
  and explicit exclusions
- All 16 style-permission combinations, checking complete resulting cells rather
  than only symbols or foreground colors
- Border masks in requested coordinates when actual buffer bounds clip the panel
- Exact duration maximum and overflow rejection; 2,688 extreme/shuffled period
  lookups with wide-integer expectations
- Finite beginning, final and beyond-end poses, including maximum duration and
  maximum step count
- Empty, excessive, zero and non-increasing frame tables; unequal frame holds,
  cycle seams and strictly future absolute boundaries near maximum time
- Pause, resume, seek, restart, rate change, freeze and catch-up; backwards
  commands reject without mutating state
- Fractional-rate deadline ceiling from the anchor phase, paused/hidden/no-change
  suppression, frame caps, overdue coalescing and arithmetic saturation
- A deterministic, bounded **4,096-transition command trace**, checking visible
  continuity, reads that do not mutate playback, seek/restart behavior, and
  backwards-command nonmutation after every transition. This is generated
  sequence coverage, not a claim of exhaustive or coverage-guided fuzzing

### Catalogue: `crates/catalog/tests/breadth_contract.rs`

- Inline output has its exact declared width and full styled-cell parity with
  direct one-row widget output; 257-column and maximum-width requests reject
- Border decoration preserves symbols, backgrounds, underline colors, titles,
  protected cells and full existing wide footprints even with hostile role styles
- Reveal validation, authored-text final pose, settled redraws, unsupported-text
  fallback boundaries and zero-width no-work behavior
- Typed progress/level validation; no silent clamping; unknown progress is visibly
  indeterminate; known whole-cell progress does not exceed binary-f64 targets
- Data-driven samples are history-free, and static poses still reflect current
  inputs; complete progress and zero-level views paint without animation requests
- **All six designs** preserve requested coordinates under clipping and retain
  protected/wide host content; huge requested geometry uses six-cell actual
  buffers instead of allocating pathological screens
- Mascot unequal holds, repeated cycles, reduced motion, exact absolute discrete
  hints, centering and distinct keep/erase operations
- A finite reveal at playback rate 3/2 requests and paints its terminal frame,
  then returns no further time-driven deadline

### Separate public-API extension: `fixtures/holdout`

A new `CrossWeave` design was authored by the independent reviewer in its own
workspace. It draws a small interlaced two-axis lattice, distinct from the six
catalogue entries. It uses only:

- Public `SampleContext`, `ClipTime`, palette/profile and `FrameSequence`
- Public `OpaqueSurface`, validated `Glyph`, explicit erase/replacement
  operations and caller-protected rectangles
- Ordinary owned and borrowed Ratatui-core `Widget` implementations
- Public `ChangeHint`, `Playback` and `RepaintPlanner` integration

Its manifest has only `spark-runtime` and pinned `ratatui-core` as direct
dependencies. Its inspected normal dependency tree contains no `spark-catalog`,
gallery, JSON parser, terminal backend or async runtime. No runtime modification,
private catalogue access, registry or unstable widget trait was required.

The standalone example ran and emitted the expected 17×7 lattice with an absolute
440 ms next-change hint. Its five tests cover direct shuffled seeking, borrowed
rendering, clipping, wide/protected host content, static/tiny behavior and deadline
integration. Strict Clippy passes for this separate fixture. This establishes the
public extension exercise, not the holdout motif's visual-release qualification.

## Independent F14/F15 host integration review

The reviewer subsequently inspected the actual `sync_host`, `async_host`, and
`paced_host` loops, their shared `examples/support/mod.rs` model/draw path, and
`tests/host_lifecycle.rs`. No additional integration blocker was found in this
scope. The implementation owner's 14 lifecycle tests were independently rerun
and passed; they are separate from the 40 reviewer-owned tests counted above.

- The synchronous loop merges independently tracked data and presentation
  deadlines. Data collection is outside drawing, and late wakes produce one
  current sample rather than replaying missed frames or data polls
- The async loop selects between terminal input and separate data/presentation
  waits. Losing sleep futures are dropped when an event wins. The only bounded
  channel holds up to 32 input events, never rendered frames; there is no task
  per design or obsolete-frame queue. Shutdown drops the receiver before joining
  the input thread, releasing a producer blocked by backpressure
- The paced loop retains the host's existing frame cadence. It adds no
  animation-specific timer and polls the data producer by its own deadline
- Presentation bookkeeping runs after the draw. A finite reveal's terminal pose
  is painted before its animation deadline disappears, and unrelated redraws do
  not count a second completion for the same entity/generation
- Updates locate rows by stable entity ID and reject obsolete generations or
  cancelled operations. Replacement increments generation and restarts reveal
  playback; sorting moves placements with entity identity rather than reusing
  row-index state
- Each draw reconstructs base content before applying decorators. The reader
  fixture supplies explicit title, selection, wide-text, caret and media
  exclusions

All three executable smoke commands passed. Each covered six designs in four
layouts and recorded 96 independent data polls and four final reveal paints.
The synchronous smoke rendered 980 frames, the async smoke 1,124, and the paced
smoke 3,120. The paced smoke recorded zero added animation-only wakes. Async's
real Tokio select cancellation smoke also passed.

**Idle-wakeup qualification:** These checks establish suppression of
animation-only deadlines, not zero process-wide wakeups. The synchronous loop
can block in terminal input when both producer and art are idle. The async
example's terminal-input adapter deliberately polls at 25 ms even then, and an
already-paced host deliberately keeps its existing presentation cadence. No
power/CPU measurement or universal idle-wakeup claim follows from these tests.
The fake-clock tests and the bounded select smoke are not a real-terminal stress
or cross-platform input-adapter qualification.

## Commands actually run by the independent reviewer

```sh
cargo test -p spark-runtime --test extension_contract --locked
cargo test -p spark-catalog --test breadth_contract --locked
cargo clippy -p spark-runtime --test extension_contract --locked -- -D warnings
cargo clippy -p spark-catalog --test breadth_contract --locked -- -D warnings
cargo test --manifest-path fixtures/holdout/Cargo.toml --locked
cargo clippy --manifest-path fixtures/holdout/Cargo.toml --all-targets --locked -- -D warnings
cargo run --manifest-path fixtures/holdout/Cargo.toml --locked
cargo tree --manifest-path fixtures/holdout/Cargo.toml --edges normal --locked
cargo test -p spark-catalog --test host_lifecycle --locked
cargo run -p spark-catalog --example sync_host --locked -- --smoke
cargo run -p spark-catalog --example async_host --locked -- --smoke
cargo run -p spark-catalog --example paced_host --locked -- --smoke
```

The holdout was formatted and checked with its own manifest. The reviewer also
ran the original runtime/catalogue all-target tests before the breadth additions.
Final aggregate workspace, host, preview and package results belong in the
qualification reports; focused reviewer checks are not a substitute for them.

## Deliberate limits and remaining qualification

- Conservative writers skip all existing wide/special footprints, even when the
  caller owns the whole footprint. They do not implement arbitrary grapheme
  replacement. Host buffers must be well formed
- Style-only fallbacks retain host glyphs. They must follow freshly drawn base
  content in a host-declared composition order. Reusing already decorated content
  is a different input, not automatic trail removal
- Real playback transitions reanchor at integer nanoseconds and can discard a
  fractional remainder below one nanosecond. Redundant commands no longer do so.
  The review does not promise exact rational residue across real transitions
- Inline output deliberately allocates owned text. This review makes no zero-
  allocation claim for adapters, Ratatui cell storage or the overall host
- Runtime repaint advice is pure. It does not prove a host correctly separates
  producer cadence, input handling and presentation; executable host tests and
  actual traces are required for that claim
- A compact/bounded design API does not by itself measure render p95, memory
  growth, linked-asset elimination, backend output or terminal-emulator CPU
- This review does not establish macOS/Windows terminal support, tmux/remote
  behavior, browser playback, native/WASM sample equality, universal glyph widths
  or all-six visual acceptance. Consult separate evidence and retain missing
  results as missing
- Publication, project naming and license selection remain owner decisions.
  A local passing API exercise is not a foundation release approval
