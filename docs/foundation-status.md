# Foundation status: implementation and acceptance

## Current acceptance audit — 2026-10-27

The dependency-ordered implementation F01–F22 exists on `main`; the plan's
unchecked original checklist is not a list of absent features. A fresh source
review verified the six designs, all four surfaces, both authoring paths, three
host loops, lifecycle/property coverage, tooling and independent runtime-only
holdout. This does **not** mean every release acceptance gate is complete.

At source revision [`d1de335`](https://github.com/Jagalite/spark/commit/d1de335eab945a63dd711210e01a97a48e4bc5a0),
the [foundation CI run](https://github.com/Jagalite/spark/actions/runs/39069923630)
passed Linux, macOS and Windows native jobs and renderer-only WASM compilation.
This supersedes the historical assertions below that publication and remote native
execution had not happened. MIT source publication is also complete; registry
publication, release tags and deployment are neither performed nor authorized here.

| Milestone | Current acceptance disposition |
| --- | --- |
| M0–M3 / F01–F15 | Implemented; native CI exercises runtime, designs, examples and consumers |
| M4 / F16–F18 | Implemented; gallery, authoring, cell regression and bounded mutation/lifecycle tests exist; historical visual reviews remain explicitly scoped |
| M4 / F19 | Native build/test and WASM smoke verified remotely; named physical terminal/font review on each OS and emulator measurements remain unqualified |
| M4 / F20 | Downstream dependency/package/binary audit implemented; observations do not prove anonymous asset-byte elimination |
| M5 / F21 | Independent runtime-only CrossWeave holdout demonstrates public extension without runtime modifications |
| M5 / F22 | Source candidate and documentation exist; blanket foundation release acceptance remains blocked by the external visual/platform gates below |

The current continuation repairs qualification-tool defects rather than inventing
missing renderer features: the documented allocation entry point, portable audit
executable discovery, complete hard-boundary checks in qualification, and fresh
source/binary provenance for actual PTY captures. No renderer contract, paint
permission or expected-art fixture is changed.

### Qualification repair verification

The repaired candidate passes 175 workspace tests, two rustdoc tests, five
independent holdout tests and 16 starter tests (198 Rust tests total), formatting,
strict Clippy, asset/cell drift checks, all six local qualification reports,
gallery/three-host and expansion smokes, downstream consumers and renderer-only
WASM compilation. Nine capture regression tests, seven endurance-harness guard
cases and four gallery PTY lifecycle flows pass. A fresh actual PTY capture also
passes all selection/control events, clean exit and terminal restoration.
The custom-target-directory package audit and standalone allocation command pass;
their local raw reports remain excluded under the artifact policy.

Independent review covered the tooling changes and found/fixed a latent UTF-8
chunk-splitting error in terminal recordings. No new visual golden was accepted.
Native CI now also runs package-audit, while the new capture regressions run on
Linux. A final revision's remote result must be checked separately; these local
results are not a predeclared green remote build.

### Remaining acceptance evidence and execution blocks

| Plan requirement | Result / smallest remaining action |
| --- | --- |
| F19 native Linux/macOS/Windows build and test matrix | Passed at the linked main revision; this repair also runs exact-commit native CI |
| F18 actual recordings, exact cells and independent art review | Existing reviewed full/slowed offscreen previews retained; fresh Linux PTY recording and integrity tests pass; neither establishes emulator/font appearance |
| F19 named terminal/font combination on each native OS | **Blocked, not passed.** Linux graphical startup was attempted using installed Xorg dummy, Xfce Terminal and DejaVu Sans Mono; the sandbox could not create local display sockets. The supported escalation attempt failed before execution. No graphical recording, screenshot or visual result was obtained. An approved graphical-enabled executor is needed. macOS/Windows graphical hosts are unavailable in this task |
| Section 11 actual terminal-emulator CPU measurement | **Blocked with the graphical route.** Offscreen renderer/backend and PTY host measurements exist, but no new emulator process ran. Run the same no-art/static/animated fixture in the approved named emulator and record scoped process measurements |
| tmux/remote-session route | **Not claimed.** The plan requires a smoke before claiming such a route; no tmux/SSH route has been qualified. Tool availability discovery was blocked by system package-index access, and no restriction was bypassed |
| F22 full foundation release acceptance | **Open on the named-emulator gates above.** Source candidate, public docs and holdout exist; source completion is not blanket platform acceptance |
| Registry/package release, tags, deployment | Not performed; separate explicit approval and public-name/package decisions are required, not inferred from permission to commit source |
| Browser playback, native/WASM cell equality | Optional unqualified routes; compilation does not establish either |

This is the minimal remaining platform distinction: CI can run native tests, and
a cloud PTY can verify protocol/control behavior, but neither demonstrates a named
emulator's font shaping or measures that emulator. A graphical-enabled cloud host
could meaningfully close the Linux route; physical display hardware is not itself
a requirement. The attempted environment cannot currently start that display.
No unrun gate is checked off and no permission restriction was worked around.
Historical reports below retain their dates and scope rather than being rewritten
as fresh measurements.

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

## Historical limits recorded on 2026-10-14

Items 1–2 and the native build/test portion of item 3 are superseded by the
current acceptance audit above. The other scope limitations still apply.

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

No implementation count overrides physical platform qualification. The historical
license/source-publication decisions above have since been resolved; package
publication and unsupported platform claims remain separate.
External checks must identify the exact source revision they exercise.
