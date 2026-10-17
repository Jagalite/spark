# Spark expansions

Authorized 2026-10-17 after the six-design MIT foundation source publication.
The work keeps explicit time, host-owned data/state/IO, scoped painting, four
workspace packages, pinned dependencies and existing exact-cell fixtures.

## Bounded increments

1. **Composition:** finite borrowed sequences, delayed parallel tracks and integer
   linear/smoothstep transitions. Ordinary widgets remain the composition surface;
   explicit host status drives loading-to-success/error examples. Independent
   contract tests cover overflow, seams, seeking and deadline mapping.
2. **Live playground:** extend existing gallery controls with inspectable,
   compilable Rust configuration export. Implemented.
3. **Host adapters:** pure caller-fed counted progress/task snapshots and bounded
   explicit-time streamed activity, with reset/generation safeguards. Implemented.
4. **External starter:** copyable runtime-only design template, public-API tests,
   docs and reproducible actual-cell preview. Implemented.
5. **Coordinated status theme:** distinct loading/empty/success/error presentations
   sharing geometry/palette/fallback rules and explicit host selection. In progress.

## Composition

See [the contract and runnable consumer](composition.md). No existing golden was
replaced, no paint permission widened and no production dependency added.
Samples own no clocks, IO, threads or callbacks. A visual completion query is
separate from completion of a task. Physical terminal/font qualification and
package release decisions remain outside these source increments.

Validation results are added only after the corresponding source check completes.
CI on a published increment must be verified against its exact commit before a
cross-platform passing result is claimed. Historical machine/PTY evidence is not
published with these increments; [ARTIFACTS](../ARTIFACTS.md) still applies.

### First increment validation

The composition source increment passed all 17 local validation commands:
formatting, strict all-target Clippy, workspace tests and doctests, catalog
validation, unchanged exact snapshots/assets, gallery and four host/composition
smokes, two standalone consumers, public holdouts, runtime/catalog WASM compilation,
and seven harness guard tests. Rust results: 126 workspace tests, one doctest and
five external holdout tests. The 11 new composition contract tests were authored
independently of the implementation; the composition smoke checks 180 combinations
of host-selected state, geometry, time and quiet motion. These are bounded checks,
not a physical-terminal or universal performance claim.

## Live playground

The [playground guide](playground.md) covers the inspector, tuning controls and
both Rust export paths. Ten generated source fixtures compile as tests and compare
cells/playback with the same typed live adapters. The focused suite passes 28 tests
and strict Clippy; the existing 768-frame smoke passes. Four bounded Unix PTY
flows pass, including repeated inspector changes, pause/seek, resize, quit/export,
restored terminal flags and clean source after terminal teardown. The PTY regression
is reproducible source with no saved captures. This is not native-font or other-OS
terminal qualification.

The clean integrated playground increment also passed all 18 local validation
commands: 145 workspace tests, one doctest, five external holdout tests, existing
snapshot/asset checks, consumers/smokes, WASM, seven harness guards and the four
PTY flows. CI includes the PTY lifecycle check on Linux only.

## Caller-fed host adapters

[Adapter contracts and the runnable example](adapters.md) cover count snapshots,
explicit task status and exact bounded stream decay. The 13 focused tests include
conservative integer-ratio conversion, atomic rejection, generation replacement,
maximum values, and 2,500 events with 12,500 pure queries against an independent
small-integer reference. Strict Clippy, doctests and the adapter smoke pass.
Count completion remains distinct from the application's chosen status.

The clean adapter increment passed all 19 integrated local commands: 158 workspace
tests, two doctests, five public holdout tests, unchanged snapshots/assets,
consumers/smokes, WASM, seven harness guards and four Linux PTY flows.

## External design starter

The [runtime-only starter](extensions.md) includes original Hanging Lantern art,
a source-only copy/rename helper, public-API tests and exact text preview. Its 16
tests, formatting, strict Clippy and read-only preview reproduction pass both
in-tree and after copying/renaming outside the checkout. Invalid names and existing
destinations are rejected. The standalone fixture owns its own workspace and
remains `publish = false`; the root still contains exactly four packages.

The clean starter increment passed all 23 local integration commands. Counts are
158 workspace tests, two doctests, five holdout tests and 16 standalone starter
tests, plus the snapshot/asset checks, consumers/smokes, WASM, seven harness guards
and four Linux PTY flows. CI runs the starter explicitly on each native target;
workspace tests alone do not include it.
