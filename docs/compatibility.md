# Compatibility and qualification boundaries

## Exact selected versions

| Component | Version | Use |
| --- | --- | --- |
| Rust/Cargo |1.90.0 | Edition 2024, pinned toolchain |
| ratatui-core |0.1.2 | Production library API; std and underline-color, no default features |
| ratatui |0.30.2 | Applications, examples and tooling |
| Crossterm |0.29.0 | Native application backend, absent from renderers/minimal consumers |
| unicode-width |0.2.2 | Validated art scalars; host widths use Ratatui CellWidth |
| Tokio |1.48.0 | Async example/dev dependency only |
| stats_alloc |0.1.10 | Qualification-tool allocation instrumentation only |

Lockfiles record transitive versions. The standalone consumers share one compatible
core instance with the full application API. No broad historical compatibility range,
MSRV matrix beyond the pinned toolchain, unstable widget traits or no_std claim.

## Locally verified routes

- Linux x86_64: aggregate formatting, strict Clippy, unit/property/integration tests,
  rustdoc, manifests/assets/exact-cell drift checks, gallery and three host smokes
- One-design, all-six and independently authored runtime-only holdout consumers
- `wasm32-unknown-unknown`: runtime/catalogue compilation without terminal backends
- Linux PTY: actual native controls, clean exit, alternate-screen and terminal-state
  restoration; final six-design protocol capture preserved separately from F06
- Linux release host PTYs: bounded control storms, resize/replacement/stall recovery,
  687 checked lifecycle/layout checkpoints, process-resource samples and clean exit
  across all three scheduling styles; no physical terminal-emulator inference
- Fixed DejaVu Sans Mono offscreen previews, exact font hash and Pillow version in
  per-design render manifests; independent frame-sequence/embedding review

Final commands, source fingerprints and limitations are recorded in
`qualification/foundation-linux-x86_64/` (original candidate) and
`qualification/endurance-linux-x86_64/` (current checks and repaired hosts). Details: [status](foundation-status.md),
[API review](foundation-api-review.md), [visual review](visual-review.md),
[measurements](performance.md), [bounded endurance](endurance.md).

## Verified remote native and WASM checks

The [foundation run for `d1de335`](https://github.com/Jagalite/spark/actions/runs/39069923630)
completed successfully on 2026-10-17 and was rechecked on 2026-10-27:
Linux, macOS and Windows native jobs, plus renderer-only WASM compilation.
The native jobs cover formatting, Clippy, tests, host smokes, asset/snapshot
validation, consumers and public extension fixtures. Linux additionally runs the
gallery PTY lifecycle check. These results qualify those source/build/test routes,
not a physical terminal or font. Later commits require their own CI evidence.

## Unqualified routes and visual limits

No named native terminal/font matrix, macOS/Windows interactive terminal review, tmux/SSH
route, native/WASM cell equality, browser playback, universal Unicode width or
real terminal-emulator CPU/output latency is qualified. The preview font lacks
some CJK glyphs; exact-cell preservation tests remain authoritative for those
host characters. Bold/dim are approximated in images; italics/blink are not simulated.

Choose the explicit ASCII/monochrome/static profile when appropriate. Baseline art
requires no emoji, Nerd Font or image protocol. Semantic user text is preserved
rather than replaced merely to force an ASCII appearance.

Every support claim is scoped to actual evidence. MIT source licensing is resolved.
Public registry-name decisions, package release approval and external physical
platform checks remain open; no package is published.
