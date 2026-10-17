# Foundation status: local candidate

## Current source publication, 2026-10-17

The owner has selected MIT and authorized public source publication to Jagalite/spark.
The old license/publication restrictions below describe the October 14 handoff
only. [Publication verification](publication.md) records current validation;
[artifact policy](../ARTIFACTS.md) distinguishes Git files from archival evidence.
Registry publication and untested platform claims remain separate gates.

## Historical foundation evidence (2026-10-14)

Updated 2026-10-14. This is an unpublished implementation and evidence handoff,
not a claim that all release gates or remote CI checks have passed. The supplied
[foundation plan](foundation-plan.md) remains the reference acceptance specification.

## Implementation and local evidence

| Gate | Result |
| --- | --- |
| F01–F06, original vertical slice | Retained and independently reviewed; original R01 exact fixture preserved |
| F07–F11, breadth | All six distinct designs implemented; inline/widget/decorator/scene and procedural/authored-frame paths represented |
| F12–F13, playback/advice | Rational playback, freeze/catch-up, finite completion and absolute discrete/continuous advice implemented and independently tested |
| F14–F15, integration | Three executable host styles, four layouts, stable entity/generation lifecycle; local smoke, lifecycle stress and real-process PTY checks pass |
| F16–F17, gallery/authoring | Six-design controls, manifests, bounded authored JSON/static generator and drift checks implemented |
| F18, testing and previews | Property/generated-sequence tests, exact styled cells, full/slowed previews and local PTY protocol evidence; fixed-font review is explicitly bounded |
| F19, platform/performance | Linux and renderer-only WASM compilation verified; measured local timing/allocation/output matrix and bounded real-host resource workload. Other native platforms and physical emulator routes remain unqualified |
| F20, downstream audit | Locked one-design/all-six consumers built and run; dependency, package-content, binary-size and symbol observations recorded. Symbol absence is not proof about all anonymous/inlined bytes |
| F21, public extension | Independent runtime-only CrossWeave design built/run/tested without runtime changes |
| F22, release candidate | Source/docs/changelog and local handoff prepared; package publication, licensing/public naming and outstanding platform acceptance remain open |

The library still owns no clock, input/data acquisition, terminal IO, thread,
network or AI service. Four workspace packages remain. Tokio, encoders, JSON,
hashing and allocation instrumentation stay outside production renderer dependencies.

## Final aggregate verification

The final Linux run passed **115 workspace tests, one rustdoc test and five
independent holdout tests (121 Rust tests total)**, strict Clippy and formatting. The gallery
smoke drew 768 offscreen frames. Each of the three host smokes covered 24 design/
layout combinations, recorded 96 independent data polls and presented four final
reveal frames. The paced host added zero animation-only wakes. Exact all-six
goldens cover 77 bounded cases, alongside the unchanged original R01 fixture.

Source fingerprint: `ade8eed73e7f1dd73c4be24d8a8fd77b6242083802c8ce47a4d53784c837bb47`.
The four new lifecycle tests cover 32,256 command checkpoints and 36,560 actual
TestBackend presentations. Seven separate Python harness guard cases also pass.
The real release hosts completed 96 measured cycles plus 12 warm-up cycles, 687
verified checkpoints and 864 resize requests. All exited cleanly. Detailed process
and resource measurements are private archival evidence; this was bounded Linux
work, not a multi-hour soak or physical terminal qualification.

Full command results: `qualification/endurance-linux-x86_64/checks.json` and
`final-checks.log`. These numbers are local evidence, not remote CI or release approval.

## Historical evidence locations

Detailed qualification records and unlisted preview derivatives below refer to
the private archive, not files distributed in this source-only checkout. See
[ARTIFACTS](../ARTIFACTS.md) for retained public inputs and regeneration routes.

- [Independent API and integration review](foundation-api-review.md)
- [Independent fixed-font visual review](visual-review.md)
- [Bounded Linux host endurance](endurance.md) and [independent evidence review](endurance-review.md)
- [Owner release decisions](release-decisions.md)
- [All-six contact sheet](../previews/catalog-contact-sheet.png)
- `previews/<design>.cells.json`: exact cell symbols, foreground/background,
  underline colors, modifiers, diff policy and requested/capture geometry
- Per-design full/slowed GIFs, seam/end boundaries, static/minimum and embedding images
- `qualification/foundation-linux-x86_64/`: original candidate source-stamped measured reports, retained unchanged
- `qualification/endurance-linux-x86_64/`: repaired host source/binary snapshot, PTY recordings, resource samples and current aggregate checks
- `qualification/baseline-linux-x86_64.json`: historical F06-only baseline, retained
- `fixtures/consumer`, `fixtures/consumer-all`, `fixtures/holdout`: separate locked workspaces

See [performance](performance.md), [compatibility](compatibility.md) and
[integration](integration.md) for measurement scope and commands. Final command
logs and an explicit machine-readable check summary accompany the local handoff.

## Limits that prevent a blanket release claim

1. Remote repository publication remains blocked. No push, remote commit, PR,
   package publication or remote CI run was performed in this continuation
2. No project license or final public package-name decision has been made.
   Cargo manifests remain `publish = false`; source packaging does not grant a license
3. macOS/Windows native execution, named terminal/font combinations on each OS,
   tmux/SSH sessions and real terminal-emulator CPU/latency remain unverified
4. WASM is a renderer compilation smoke only; browser playback and native/WASM
   exact-cell equality are separate, unrun checks
5. Fixed-font images approximate styles and show missing-glyph boxes for some CJK
   host text. Exact-cell tests prove preservation; the images do not qualify those fonts
6. Timing/allocation observations apply to the recorded Linux build, fixtures and
   instrumented allocator. Bounded real-host process RSS/CPU observations supplement
   them; neither establishes universal latency, fragmentation, multi-hour/day
   endurance or zero allocation for every geometry/profile
7. The async host's input adapter polls at 25 ms, and paced hosts keep their existing
   presentation clock. Zero animation-only wakeups never meant zero process-wide work
8. Conservative wide-footprint skipping is intentional; general grapheme replacement,
   historical Ratatui compatibility, no_std, reverse playback and a web editor are absent

No implementation count overrides these limitations. The remaining useful steps are owner decisions on licensing/public naming, physical
platform qualification, and separately authorized publication when appropriate.
External checks must identify the exact source revision they exercise.
