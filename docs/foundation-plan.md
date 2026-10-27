# Ratatui Animated Art — Foundation-First Project Plan

**Version:** 1.0  
**Date:** October 12, 2026  
**Status:** Original proposed acceptance specification (historical). For implemented
work, verified checks and remaining external gates, see [current foundation status](foundation-status.md).
The unchecked checklist and original handoff below are preserved planning history,
not current implementation status.
**Working title:** Ratatui Animated Art. Repository and crate names below describe roles, not verified available public names.  
**First release:** A useful, qualified foundation with six distinctive reference designs—not a large catalogue.

## Executive decision

Build a standalone Rust library of highly customized terminal animations that applications can adopt without replacing their event loop, state management, input handling, or theme. AI will help author the designs and tests; consumers will use ordinary offline Rust code and assets.

Start with a small runtime, six deliberately different designs, three executable integration examples, a native gallery, and reproducible qualification tools. The foundation is complete when an additional design can be implemented through the public extension points without modifying the runtime, and the existing designs remain useful in real application layouts.

This plan consolidates the earlier research and ecosystem integration addendum [P1, P2]. It changes their release sequence: **six qualified designs are sufficient for foundation v0.1**. A collection of roughly 24 is the next workflow-validation exercise, not a prerequisite for completing the foundation. Hundreds remain a later content-production decision.

The project has two equally important success criteria: the artwork must be distinctive, and embedding it must be straightforward. Neither a generic engine without good art nor attractive recordings without reliable components meets the goal.

## 1. Scope and deliberate exclusions

| Build for foundation v0.1 | Defer until there is implementation evidence |
| --- | --- |
| Explicit-time rendering into Ratatui cells | A general timeline editor or scene-graph framework |
| Procedural Rust and authored cell-frame playback | General simulation, arbitrary scripting, shader languages |
| Inline text, bounded widgets, safe decorators, and a larger scene example | Terminal image protocols, GPU-specific rendering, 3D engines |
| Small typed inputs for progress and live values | Audio capture, networking, domain-specific data collection |
| Optional playback controls and repaint planning | A mandatory runtime, scheduler thread, or application framework |
| Six polished reference designs | Hundreds of candidates, category packs, a marketplace |
| Native gallery, exact snapshots, actual recordings | A full live web editor or user accounts |
| Author template, manifest validation, reproducible previews | Autonomous multi-agent orchestration and automatic publication |
| Minimal dependency and binary-size checks | A compatibility layer for every historical Ratatui release |

Do not start by porting Charm, reimplementing TachyonFX, or building a generic widget toolkit. This project can serve those ecosystems later without becoming one of them. The baseline must not require AI services, telemetry, network access, a special font, or a background thread.

A small amount of bespoke Rust is intentional. Do not make every design fit a declarative recipe merely to simplify generation.

## 2. Foundation decisions

### 2.1 The application owns the application

The host owns terminal initialization/restoration, input, focus, layout, data acquisition, application state, and when to present a frame. The library owns drawing its declared region, interpreting its declared visual inputs, and advising when another sample would be useful.

This follows the upstream boundary: Ratatui leaves event handling to applications, and its component template is explicitly one opinionated organization rather than a universal contract [S4]. The prior survey found multiple loop and component arrangements [P2, sections 2.2–2.4].

No design may require a prescribed `Action` enum, Tokio, Crossterm events, a global animation manager, or the host's complete `AppState`.

### 2.2 Rendering is sampling, not advancing

The logical contract is:

```text
implementation + design configuration + seed
    + clip time + requested size + palette/profile + typed input
    + explicit current base content, when transforming content
        -> permitted cell writes and sampling advice
```

Rendering twice must not advance an animation twice. Sampling timestamps out of order must work. A large elapsed time must not require replay from the start. Reconstructing a lightweight widget in a draw function must not reset the animation.

Exact reproducibility initially means equal cells for equal inputs under the same qualified build/target. Cross-target floating-point equality and identical appearance across fonts/terminals are separate claims requiring separate evidence.

### 2.3 Keep state only where it has a purpose

Separate reusable design/configuration, optional persistent playback or transition state, and temporary render views. Caches and scratch storage may be mutable, but must not change the result associated with the same semantic inputs.

History-dependent smoothing must receive explicit history or use a separately named stateful helper. Do not infer a complete spring trajectory from only a current scalar. Keep arbitrary spring simulation out of the foundation; a bounded start/target/time transition is sufficient.

### 2.4 Adopt Ratatui's library boundary

Use the stable `Widget`/`Buffer`/`Rect`/text/style interfaces through a tested `ratatui-core` dependency. Upstream specifically recommends core for widget libraries and the larger `ratatui` crate for applications [S1]. Upstream also supports implementing `Widget` for references to immutable widgets [S2].

Choose and record the exact compatible Rust/core/application versions during M0. Do not publish a compatibility claim from documentation alone or require unstable widget traits merely to support a catalogue.

## 3. Repository and dependency structure

Begin with four workspace packages. Keep adapters and helper modules inside these packages until a real dependency boundary justifies extraction.

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
AGENTS.md
crates/
  runtime/
    src/
      time.rs             Clip-time values, repeat and finite sampling
      playback.rs         Optional host-time to clip-time mapping
      repaint.rs          Advice and pure deadline planning
      profile.rs          Glyph/color/motion choices
      palette.rs          Small role palette and style overrides
      surface.rs          Clipping and permitted cell operations
      sprite.rs           Validated static frame playback
      primitives/         Helpers extracted from actual designs
  catalog/
    src/
      comet.rs
      assembly_progress.rs
      border_runner.rs
      telemetry_embers.rs
      pocket_mascot.rs
      prism_reveal.rs
      generated/          Checked-in generated asset data
    examples/
      sync_host.rs
      async_host.rs
      paced_host.rs
    tests/
      fixtures/           Synthetic application layouts and traces
apps/
  gallery/
xtask/
catalog/
  <design-id>/
    manifest.json
    brief.md
    frames.json           Only for authored-frame designs
    input-traces.json     When needed
fixtures/
  cells/
  traces/
docs/
  architecture.md
  authoring.md
  integration.md
  compatibility.md
  performance.md
  decisions/
```

`catalog` depends on `runtime`; `gallery` and qualification tools depend on both. The runtime never depends on the gallery or catalogue. Full Ratatui, terminal backends, async runtimes, image encoders, JSON parsing, and CLI support belong in application/dev/tooling dependencies unless a specific runtime requirement is established.

Do not add one crate or Cargo feature per design. The six designs can be ordinary modules. Use direct typed constructors; keep the all-design enumeration inside the gallery/tooling. Test a small downstream consumer to make sure it does not retain unrelated assets. Split packs only when the catalogue and measurements justify it.

Do not promise `no_std` in v0.1. Keep clocks, OS handles, and IO out of the renderer so a WASM compile check is feasible without prematurely designing an embedded platform abstraction.

## 4. Public contracts

### 4.1 Consumer experience

The intended use is a stored design and a temporary sampled view. This is an API sketch, not implemented or compiled code:

```rust
// Once, during UI setup. Names are provisional.
let comet = NeonComet::new()
    .period(Duration::from_millis(1200))
    .seed(7);

// During a redraw; the host supplies time, palette, and capabilities.
let context = SampleContext::new(clip_time, &palette, &profile);
let sample = comet.sample(&context);
let hint = sample.change_hint(area.size());

frame.render_widget(sample, area);

// In the host's loop, map/merge the hint with existing deadlines.
```

The sketch does not prescribe a giant trait shared by every possible design. Start with small sampling and painting interfaces, supported-placement adapters, and typed inputs. Let the gallery use a private enum for heterogeneous browsing until a dynamic public registry is actually needed.

The extension contract must allow a downstream crate to author another design. It must not require access to private catalogue internals. A proposed render view should borrow configuration and assets where practical; it must not allocate a full intermediate screen simply to represent a sample.

### 4.2 Four consumption surfaces

| Surface | Initial API direction | Required behavior |
| --- | --- | --- |
| Inline | Styled `Line`/`Span` output, with declared width | Fits titles/status/table cells; no ANSI escape strings; padding policy is explicit |
| Bounded widget | A sampled view implementing `Widget` | Draws in caller geometry using validated scoped writes |
| Decorator | Explicit application after base content, with a permission-limited surface | Protects text, selection, title, caret, and excluded regions according to supplied rules |
| Scene | The same widget contract at a larger declared size | No separate engine; explicit cost, fit, and fallback policy |

Not every design must support every surface. Unsupported placements and minimum sizes belong in metadata. For inline output, offer a simple allocation-using convenience path first and measure it; add reusable storage if needed. Do not advertise universal zero-allocation text generation without evidence.

A scene is a usage/qualification class, not another public renderer abstraction.

### 4.3 Inputs describe visual requirements, not the whole application

Use design-specific typed input structures. Share only genuinely common primitives, such as validated progress and normalized levels.

- Progress distinguishes unknown from known values. The adapter rejects invalid non-finite values or offers an explicitly named clamping constructor.
- A live level/sample input carries bounded caller-supplied values. The library does not sample audio, poll transfer rates, or invent values.
- A finite visual uses an explicit start/clip time and lifecycle identity. Completion of the visual never completes a business operation.
- A supplied phase such as busy/succeeded/failed changes art only. It is not an authority on task state.

A smoothed percentage may be visually interpolated, but authoritative numeric labels remain host-owned. Do not advance a known percentage above the supplied target or turn unknown progress into a fabricated estimate.

Keep triggers stable across redraws. A host should associate transient state with an entity ID plus an operation generation, not a sorted row index. Replacing or cancelling an operation must prevent its old success effect from appearing on the new operation.

### 4.4 Geometry, style, and capability resolution

Resolve fit/fallback and repaint planning for the same requested size that will be rendered. Publish native/preferred/minimum dimensions and one explicit fit policy: adaptive, centered, or cropped. Never silently stretch authored cell art.

Clipping changes which writes are allowed, not the underlying composition's coordinate system. If part of a panel is clipped, preserve the design's requested origin/size when calculating motion and intersect only its output.

A small palette should provide roles such as foreground, muted, accent, highlight, and an optional known background. Permit ordinary style overrides rather than imposing a theme engine. Defaults should make the first example easy; role remapping should make embedding practical.

Use explicit glyph/color profiles. Baseline assets should use validated single-column ASCII or a selected one-column Unicode repertoire. Provide a quiet ASCII/monochrome fallback for the reference designs; it need not reproduce every visual detail. Avoid mandatory emoji or Nerd Fonts. Unknown terminal background must remain unknown; effects requiring alpha-like blending must use a supplied background or a different fallback.

## 5. Safe painting and composition

### 5.1 Make permissions enforceable

Provide scoped writers with private access to the underlying mutable buffer. Separate style-only operations from glyph-writing operations, ideally through distinct types or capability-specific methods.

| Permission | May change | Must preserve |
| --- | --- | --- |
| Style-only | Approved foreground/background/modifier fields | Symbols and unapproved style fields |
| Masked cells | Glyphs and approved styles in a supplied mask | Every cell outside the mask/protected regions |
| Opaque region | Complete cells in the supplied owned region | Every cell outside that region |

Distinguish no-write, explicit space/erase, foreground-only patching, and full replacement. Never implement transparency as a space that destroys underlying content.

The facade must intersect the requested region with actual buffer bounds and handle empty regions without panicking. Ratatui exposes optional safe cell access; use it appropriately instead of relying on unchecked index assumptions [S5].

Masks may begin as rectangle/border/exclusion helpers. Do not build an arbitrary mask language. Border decorators must accept title exclusions rather than guess that every cell on a border belongs to the effect.

### 5.2 Define how previous content is handled

For each application frame, render base content, then apply approved decorators in an explicit order, with protected overlays/caret handling owned by the host. A decorator operates on the current frame's base, not residue left by its own previous frame.

Sampling determinism for a content transform includes the base content and decorator order. Applying a color multiplication twice to an already transformed buffer is not the same input; do not mislabel it as a sampling failure or promise automatic idempotence.

Prefer direct writes for cell-local transformations. If an effect needs neighbors from an unmodified source, give it an explicit bounded read-only snapshot or reusable region scratch buffer. Do not make every widget allocate a full-screen offscreen surface.

### 5.3 Wide characters are an explicit edge case

Single-column art still needs to coexist with multi-column host text. Do not overwrite only one part of a wide glyph. Skip a glyph-writing operation when its complete footprint cannot be safely updated within the allowed region, or use a qualified whole-grapheme replacement path.

For the first text reveal, support validated one-column authored text. For unsupported user text, fall back to a style-only reveal that leaves glyphs intact. General grapheme rewriting is not a foundation requirement.

Test combining marks and wide glyph boundaries in the host buffer even if our own art repertoire is narrow. Ratatui's buffer-difference documentation explicitly assumes well-formed wide-character cells [S5].

Protected image-protocol rectangles are host-supplied no-write regions. The foundation does not discover or manage image protocols.

## 6. Playback, clocks, and repaint planning

### 6.1 Keep two time domains separate

`HostTime` is a monotonic elapsed duration supplied by the application. `ClipTime` is the time at which a design is sampled. Direct sampling needs only clip time; optional playback maps host time into it.

Playback stores an anchor host time, anchor clip time, positive rate, and play/pause mode. Re-anchor when changing rate or pausing so the visible pose remains continuous. Prefer a validated rational rate for the initial helper. Zero is not a rate: use pause. Reverse playback is deferred.

State-changing playback commands reject backwards host timestamps without mutating state. Direct clip sampling may occur in any order, including seeking backwards. Checked arithmetic and documented saturation at the supported maximum must prevent overflow panics. Repeating renderers reduce time before lossy floating-point conversion.

### 6.2 Sampling advice is not a timer

Use an explicit hint with three meanings:

| Hint | Time domain / meaning |
| --- | --- |
| No time-driven change | Wait for host input, state, size, theme, or visibility invalidation |
| Next discrete boundary | An absolute future clip time, including the correct cycle—not merely a modulo phase |
| Continuous motion | A preferred presentation cadence; the host may render less frequently |

For a discrete boundary, optional playback converts remaining clip duration to a host deadline using the rate. Pause/hidden/static policy removes animation-only requests. Continuous cadence is advisory and subject to the host's cap.

A pure helper may merge requests, but must not spawn tasks or own IO. Already-paced hosts can ignore advisory deadlines and sample at their existing cadence.

Host examples must demonstrate these rules:

1. Process input/data independently of animation cadence.
2. Present only when the host is dirty or a visible active visual is due.
3. Coalesce overdue frame requests and sample the latest pose; do not enqueue missed frames.
4. Recompute advice after drawing, resizing, changing input, replacing a clip, or changing visibility.
5. Keep a positive wake interval and respect the presentation cap so near-boundary rounding cannot produce a busy loop.
6. Schedule a terminal frame for a one-shot before dropping its time-driven requests.

The earliest applicable deadline guides waiting; the frame cap limits drawing, not input processing or data polling. Ratatui's own component example distinguishes tick rate from frame rate [S4].

### 6.3 Lifecycle policies

Support play, pause, resume, restart, seek, rate change, and finite completion. Hidden behavior is explicit: freeze or catch up. Default optional-helper behavior should freeze; callers using direct clip time can choose catch-up.

Paused means retain the current pose. Reduced/static motion means a deliberately authored quieter presentation, not necessarily that pose. Each reference design must supply a meaningful static fallback.

When the app redraws for another reason, a paused/completed/static widget must still paint. No time-driven repaint request is not permission to omit rendering.

Completion is queryable visual state. Never fire callbacks or business effects from `render()`. A caller that wants a once-only notification handles state transitions outside drawing and keys them to the operation generation.

## 7. Authoring paths

### 7.1 Procedural Rust

Implement small pure renderers with a limited helper set: periodic phase, easing used by a reference design, palette lookup/interpolation, line/path positions, bounded seeded variation, masks, and glyph ramps.

Extract helpers from the six designs. Do not prebuild dozens of speculative primitives. Keep custom Rust available when an original composition benefits from it.

Prefer analytic trajectories and indexed seeded particles over simulations that must advance once per frame. A live value can change density, brightness, or displacement without changing the domain state. Cap work by validated counts and supported dimensions.

### 7.2 Authored cell frames

Use a versioned authoring JSON format in tooling, not a required runtime parser. Initial assets use complete frames, positive integer frame durations, dimensions, palette references, glyph references, and explicit coverage operations. Delta frames, compression, and remote loading are deferred.

Tooling validates the asset and emits checked-in Rust/static tables. Runtime frame selection uses cumulative durations and direct lookup/binary search. No per-frame parsing, decompression, or replay from frame zero is needed.

Set central limits for source bytes, dimensions, frame count, cumulative duration, total cells, and glyph table size during implementation. Reject over-limit assets before allocating their expanded form. Validate all multiplications and duration sums.

Own this small asset format rather than serializing Ratatui's internal structs as the public file format. An external editor importer can target it later.

### 7.3 Design package

Every reference entry contains its brief, implementation/editable frames, stable ID, visual revision, driver and placement capabilities, supported sizes, palette/profile requirements, fallback, provenance/license record, tests, actual previews, and qualification results.

Keep a design's visual revision separate from its runtime API version. A palette or speed variant does not automatically become a new design. The first release can pin a visual revision through the package release plus descriptor; a dynamic historical-asset resolver is unnecessary.

Use one author manifest as the tooling source of truth. Generate/check the minimal static descriptor and asset data deterministically; do not hand-maintain conflicting copies. Store measurement results separately, keyed by source and configuration hashes.

## 8. The six reference designs

Names and sizes are proposed briefs, not existing implementations or achieved benchmarks.

| ID / working design | Intended form and starting size | Driver | Why it is in the foundation |
| --- | --- | --- | --- |
| R01 — Segmented Comet | Inline and bounded strip; 16×1 and 24×3 | Repeating time | Tests the user's original glowing-trail direction, fixed width, palettes, and elapsed-time sampling |
| R02 — Assembly Progress | Compact machine/conveyor composition; 24×3 | Known/unknown progress | Tests honest progress semantics, adaptive geometry, and an original progress design |
| R03 — Border Runner | Moving restrained highlight around an existing panel | Time or finite trigger | Tests style-only rights, title exclusions, and content preservation |
| R04 — Telemetry Embers | Sparse particle/ember field; 24×8, with a larger scene profile | Caller-supplied level | Tests reactive inputs, bounded analytic particles, live/static behavior, and scene reuse |
| R05 — Pocket Mascot | Original small creature; native 12×6 | Timed frames | Tests authored assets, frame holds, transparency, centering, and exact seeking |
| R06 — Prism Reveal | Finite reveal of authored text; size declared by content | Explicit start/generation | Tests one-shot completion, masks, protected neighboring text, and replay/reset behavior |

All six need a compelling isolated loop and at least one realistic embedding. Do not accept six barely customized stock spinners to meet the numerical target. Conversely, do not make the foundation wait for a feature film's worth of frame art.

A reference design may simplify at an unsupported size. The fallback must be visible, documented, and truthful; it must not panic or silently overwrite outside its area.

## 9. Integration fixtures

### 9.1 Three executable host loops

**Synchronous dirty/deadline host:** Demonstrates input polling plus the earliest applicable repaint/data deadline. It must return to its idle behavior when animations stop.

**Async event host:** Demonstrates input/data events and a cancellable wait until the next presentation deadline. Tokio is an example dependency only; no task per animation and no unbounded obsolete-frame queue.

**Already-paced host:** Has separate presentation and slow data-update cadence. Adding a design must not change how often the data producer runs.

All six designs must be renderable through each host style without changing their implementation. Shared fixture/layout code is acceptable; the loop ownership must remain explicit.

### 9.2 Four reusable embedding layouts

| Layout | Exercises |
| --- | --- |
| Table/status | Fixed width, labels and percentages, reordering, stable item identity |
| Request/result | Busy/progress/result transitions, cancellation, generation replacement, editable form remains host-owned |
| Multi-rate dashboard | Live values, hidden panes, a larger scene, independent input/data/presentation rates |
| Reader/protected content | Border/text effects next to selection, wide text, caret, and reserved media rectangles |

These are synthetic hosts informed by the earlier survey, not claims of integration with specific third-party projects [P2, section 7]. A later real integration can validate the API with an application such as Superseedr, but modifying another repository is not part of this planning deliverable or an unconditional release dependency.

## 10. Qualification and acceptance tests

### 10.1 Unit and property tests

Test designs directly against buffers; reserve `TestBackend` for whole-UI integration. This distinction follows upstream guidance [S3].

Required properties include repeated/out-of-order sampling, seeded reproducibility, no advancement on redraw, empty/tiny/offset/clipped rectangles, very large times, validated configuration, profile fallbacks, and bounded work. Snapshots must include glyphs, colors, modifiers, and relevant coverage—not plain text alone.

For content transforms, identical-input tests include identical base content and paint order. For loops, inspect timestamps on both sides of the period boundary, not just zero and one full period.

Test the last frame and completion of one-shots, pause/rate continuity, freeze/catch-up, cancellation/replacement, rapid resizing, sorting, and repeated show/hide. Fuzz asset validation and playback command sequences with bounded allocations. Test arithmetic overflow with small actual buffers; do not allocate pathological full-size screens just to test invalid dimensions.

### 10.2 Preservation and scheduling tests

For every decorator, assert that cells outside the allowed mask and protected fields inside it remain unchanged. Include titles, selected rows, multi-column text, and reserved rectangles. Validate overlay movement over freshly drawn content so trails do not persist accidentally.

Use fake host clocks and counted wakeups/data polls. Assert zero animation-only wakeups after the final required frame when all visible designs are paused, completed, hidden, or static. Continue unrelated redraws and assert that frozen art stays present.

Run event storms and long stalls. The final pose must reflect current time, not a backlog, and the data producer's count must be independent of animation cadence. Avoid tight rounding loops at the next discrete boundary.

### 10.3 Visual review

Review full-speed loops, slowed loops, boundary frames, a contact sheet, and the embedding fixture. Check silhouette, motion staging, loop seam, legibility, theme fit, distraction, static pose, and smallest supported size.

Previews must be produced by the same Rust design/asset pipeline the consumer uses. A separately generated promotional image is not qualification evidence. Tie captures to the source/visual revision, seed, dimensions, palette/profile, input trace, and render/capture configuration.

Exact offscreen cells and real-terminal recordings serve different purposes. Fixed renderer/font previews are useful for review but do not establish universal terminal appearance.

### 10.4 Platform checks

Target native Linux, macOS, and Windows build/test coverage. Begin with one named terminal/font combination per OS, and add a tmux/remote-session smoke check before claiming that route. Record versions and limitations.

Run a WASM compile smoke test for the runtime/catalogue early. Actual browser playback and native/WASM sample equality are separately qualified; the live browser gallery is not required for foundation release. A failed optional browser experiment should not force a native app into a web abstraction.

## 11. Performance and dependency qualification

Measure the same fixture with no art, a static pose, and animation. Separate cold construction, preparation, warmed painting, scratch allocation, buffer differences, backend output, application CPU, and terminal-emulator CPU. Ratatui's buffer-difference interface is a reason to inspect changed-cell/output behavior rather than judging only arithmetic cost [S5].

Start with these **provisional engineering targets**, then calibrate them on named hardware before adopting release gates:

| Class | Example area | Proposed warmed release render p95 |
| --- | --- | --- |
| Micro | 32×3 cells | Under 100 microseconds |
| Compact | 60×12 cells | Under 500 microseconds |
| Larger scene | 120×40 cells | Under 2 milliseconds |

These are not benchmark results or universal limits. Record debug results separately. Do not let a fast microbenchmark hide a noisy or permanently active terminal.

Hard architectural gates are stronger than initial numeric targets: no design-owned timers/threads/IO, no elapsed-time-linear replay, bounded memory, no allocation growth over a long run, and no animation-only wakeups when nothing visible moves. Aim for no steady-state design-owned allocations in simple procedural effects; measure Ratatui cell storage and text adapters separately.

Exercise one active design, six mixed designs, a grid of repeated visible instances, and hidden/completed cases. Reusing the six designs for a stress fixture is sufficient; do not generate hundreds of placeholder entries.

Build an external minimal consumer using one typed design, and another using all six. Compare runtime dependency trees, package contents, linked assets, build time, and binary size under the same build settings. The single-design app must not depend on gallery/encoder/AI/async tooling or unintentionally retain unrelated large asset tables. Do not infer code elimination from a small package count.

## 12. Gallery and developer tools

### 12.1 Native gallery

Begin the gallery with the first working comet, then extend it as the other designs arrive. It is an inspection tool, not another major product.

Foundation controls: choose one design; play/pause/restart/scrub; change size, palette, profile, speed, and seed; feed progress or levels where supported; toggle static motion; switch isolated/embedded view; show declared capabilities and qualification details. Keep inactive entries as static posters or labels rather than animating every item.

Usage snippets should come from compiled examples or otherwise be compiled by CI. Documentation must not advertise an API that only exists in the gallery's private wrappers.

### 12.2 `xtask` interface

The following commands are intended deliverables; they do not exist yet:

```text
cargo xtask validate
cargo xtask render <design-id> --fixture <name>
cargo xtask record <design-id>
cargo xtask qualify <design-id>
cargo xtask generate-assets --check
cargo xtask package-audit
```

Keep exact cell snapshots as the first reproducible output. Add a practical recording/contact-sheet path for human review without putting encoders in runtime dependencies. Qualification emits a machine-readable report plus a human summary, including missing checks rather than treating missing data as success.

### 12.3 AI-ready, not AI-dependent

Add an authoring brief template and `AGENTS.md` now. An author submission must include the implementation/asset, metadata, tests, deterministic previews, embedding evidence, and resource/provenance notes.

Content contributors cannot change runtime contracts, grant themselves wider paint permissions, add arbitrary dependencies/build scripts, rewrite goldens without review, or disable failures to make an entry pass. Core changes and expected-image changes need independent review.

Use AI assistance for the six designs and their repairs, but do not build provider integrations, agent scheduling, or automated publishing. Before any later unattended generation, use isolated credential-free workers with bounded resources, approved dependencies, and human-controlled publication. `forbid(unsafe_code)` is a project guardrail, not a sandbox.

## 13. Milestones and implementation backlog

These are dependency-ordered acceptance gates, not calendar estimates. Complete a vertical slice before expanding abstractions.

### M0 — Bootstrap and decisions

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F01 | Four-package workspace, CI scaffold, local build/test commands, licensing/provenance policy | None |
| F02 | Architecture decisions: ownership, time domains, surfaces, initial compatibility matrix, six briefs | F01 |

**Exit:** A minimal external host compiles against the selected core/application pair; runtime dependency boundaries are visible. Record exact toolchain and dependency versions. No claim of broad compatibility is needed.

### M1 — One complete vertical slice

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F03 | Validated clip time, seed/config handling, palette/profile skeleton | F02 |
| F04 | Clipped single-column cell writer, explicit no-write/erase, empty/offset tests | F03 |
| F05 | R01 Segmented Comet, ordinary widget view, fixed-time snapshots | F04 |
| F06 | Minimal gallery preview, first synchronous host, first render/output baseline | F05 |

**Exit:** One attractive design renders live and offscreen, resizes safely, preserves adjacent content, and samples correctly in shuffled time order. The runtime is already exercised by a consumer. Do not continue with a generic engine if this slice is awkward to use.

### M2 — Prove breadth with the remaining five designs

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F07 | Declared-width inline adapter and R01 inline use | F06 |
| F08 | Style-only/masked decorator interfaces; R03 Border Runner | F06 |
| F09 | Typed progress/level inputs; R02 Assembly Progress and R04 Telemetry Embers | F06 |
| F10 | Author asset schema, validation/static code generation, R05 Pocket Mascot | F06 |
| F11 | Finite clip semantics and R06 Prism Reveal with safe text fallback | F08, F03 |

**Exit:** All four surfaces and both authoring paths are represented. The designs demonstrate genuinely different motion/composition. Revise overgeneralized APIs now, before mass content work. A native gallery can inspect all six.

### M3 — Playback and host integration

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F12 | Optional playback clock and tested pause/resume/seek/rate/hidden policies | F03, F11 |
| F13 | Discrete/continuous repaint advice and pure planning helper | F10, F12 |
| F14 | Complete synchronous, async, and already-paced executable examples | F13 |
| F15 | Four embedding layouts, operation-generation and rapid replacement traces | F07–F14 |

**Exit:** The same designs work unchanged in all three loop styles. Completed/paused/hidden/static cases generate no animation-only wakeups after their final update. Host input/data timing and semantic text remain correct.

### M4 — Authoring workflow and qualification

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F16 | Complete gallery controls and compiled consumer snippets | F15 |
| F17 | Author manifest/template, deterministic descriptors and asset drift checks | F10, F16 |
| F18 | Property/fuzz/integration suites, exact cell fixtures, actual recordings | F15, F17 |
| F19 | Native platform matrix, early WASM result, performance/allocation/output reports | F18 |
| F20 | One-design/all-six downstream package and binary audit | F17, F19 |

**Exit:** All six entries have actual previews and named qualification results. Remaining limitations are explicitly documented. The minimal consumer remains minimal. Qualification tools report failures/missing evidence rather than manufacturing a passing badge.

### M5 — Foundation v0.1

| Task | Deliverable | Dependency |
| --- | --- | --- |
| F21 | Independent public-API extension exercise and integration review | F20 |
| F22 | Resolve review defects, public docs, changelog, package/release candidate | F21 |

**Exit:** A maintainer can add a new original test design in a separate consumer crate using documented extension points, without modifying runtime internals. This holdout need not become a seventh published catalogue entry. All hard release checks pass; publish only the tested support claims.

## 14. Work ownership and parallelism

Assign one owner for runtime/public contracts, one for independent tests/integration, one for gallery/tooling, and bounded content owners for the reference designs. These are responsibilities; they do not require a multi-agent orchestration system.

F01–F06 are the main serial path. After the first vertical slice and an interface review, F07/F08/F09/F10 can proceed in parallel with explicit ownership. F11 follows the decorator/time primitives. Playback, scheduling, and host-loop work should have one coordinating owner so time-domain assumptions do not diverge.

Content changes may parallelize; shared primitive and API changes should be reviewed together and revalidate every dependent design. The person or agent authoring a feature should not be the only reviewer of its expected output.

Statelessness is not a production runtime dependency in this plan. Pure playback transition functions plus generated command-sequence tests are enough to begin. A later test-only model can check lifecycle invariants when it adds evidence; it must not delay ordinary rendering or force catalogue consumers to adopt a state-machine framework.

## 15. Foundation release checklist

- [ ] Six original, visually reviewed designs; no variant-count inflation.
- [ ] Inline/widget/decorator/scene uses and procedural/frame authoring are demonstrated.
- [ ] Simple typed construction works without a global registry or async runtime.
- [ ] No hidden clock, timer, thread, terminal IO, or domain-data collection in designs.
- [ ] Same inputs/time/base yield the same qualified cell output; seeking is direct.
- [ ] Zero/tiny/clipped/offset sizes and supported fallback profiles behave safely.
- [ ] Style-only permissions, masks, titles, protected text/media, and wide-cell edges pass.
- [ ] Pause/resume/rate/restart/seek and finite completion have defined tested behavior.
- [ ] Operation replacement/cancellation cannot reuse obsolete visual state.
- [ ] Three host loops demonstrate independent data, input, and presentation ownership.
- [ ] Inactive designs cause no animation-only wakeups but paint on unrelated redraws.
- [ ] Real isolated and embedded previews are traceable to source and configuration.
- [ ] Numeric performance reports name target/build/settings; no unmeasured claims.
- [ ] Runtime and one-design consumer exclude development tooling and unrelated assets.
- [ ] Compatibility documentation lists tested routes and unqualified limitations.
- [ ] A public-API holdout design is possible without a runtime modification.
- [ ] Authoring and consumer documentation examples compile.
- [ ] Provenance and licensing records are present for code and artwork separately.

## 16. What happens after the foundation

First expand toward approximately 24 accepted designs to test the authoring contract. Prefer new compositions and underserved placements rather than cosmetic variants. Track how often a new design requires a runtime change, review/repair effort, defects, visual duplication, and cost per accepted entry.

Only then decide whether to automate larger batches. Before 100, add selected-pack distribution if needed, better nearest-neighbor review, and bounded creation/qualification workers. Add external-editor import or a live web gallery when it solves a measured workflow problem. Do not require either to render an animation.

No fixed design count overrides release quality. Repeated public-API changes, poor embedding, or unreliable previews are reasons to improve the foundation, not to produce more entries.

## 17. Initial implementation handoff

> Create the workspace and implement F01–F06 only. Use provisional local package names. Establish the tested Rust/Ratatui dependency pair and the no-runtime-IO boundary. Implement validated explicit-time sampling, a small palette/profile model, a clipped cell writer, and one original segmented-comet design with a borrowed widget view. Add direct buffer tests, shuffled-timestamp sampling, resize/offset/protected-neighbor tests, a minimal native preview, a synchronous host example, and baseline measurements. Keep animation time host-supplied. Do not add a global scheduler, AI service, broad recipe language, hundreds of designs, or cross-repository changes. Deliver the code, executable examples, tests, actual previews, measured results, and a brief review of the API issues discovered. Revise that contract before proceeding to F07–F11.

This handoff is a proposed next implementation assignment, not work performed by this document.

## Sources and planning lineage

The architectural decisions above are proposals. External facts are grounded in these sources; prior survey observations remain bounded by their original inspection depth. Documentation was checked for this plan, but no dependency matrix was built or benchmarked here.

**P1.** `ratatui_animated_art_research_plan.md` — Earlier supplied research plan, especially runtime, qualification, authoring, and distribution sections.

**P2.** `ratatui_ecosystem_integration_addendum.md` — Supplied ecosystem survey and integration recommendations, especially sections 2–7. Its application-source observations were not re-audited for this implementation plan.

**S1. Ratatui core documentation.** Widget-library dependency guidance and core/application distinction. https://docs.rs/ratatui-core/latest/ratatui_core/

**S2. Ratatui Widget documentation.** The `Rect`/`Buffer` drawing boundary and reference-based implementations. https://docs.rs/ratatui-core/latest/ratatui_core/widgets/trait.Widget.html

**S3. Ratatui TestBackend documentation.** Direct-buffer unit testing versus whole-terminal integration testing. https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html

**S4. Ratatui component template.** Application-owned events, optional opinionated organization, and distinct tick/frame rates. https://ratatui.rs/templates/component/

**S5. Ratatui Buffer documentation.** Safe optional cell access, cell differences, and wide-character invariants. https://docs.rs/ratatui-core/latest/ratatui_core/buffer/struct.Buffer.html
