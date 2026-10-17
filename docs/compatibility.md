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

## Configured, not executed remotely

The CI scaffold targets Linux, macOS and Windows native checks, all three host
smokes, asset/snapshot validation and downstream consumers; WASM has its own job.
A workflow file does not establish a passed run. Publication remains blocked,
therefore no remote CI result is claimed for this local candidate.

## Unqualified routes and visual limits

No named native terminal/font matrix, macOS/Windows actual execution, tmux/SSH
route, native/WASM cell equality, browser playback, universal Unicode width or
real terminal-emulator CPU/output latency is qualified. The preview font lacks
some CJK glyphs; exact-cell preservation tests remain authoritative for those
host characters. Bold/dim are approximated in images; italics/blink are not simulated.

Choose the explicit ASCII/monochrome/static profile when appropriate. Baseline art
requires no emoji, Nerd Font or image protocol. Semantic user text is preserved
rather than replaced merely to force an ASCII appearance.

Every support claim is scoped to actual evidence. Licenses/public package names,
release approval and external platform checks remain open; no package is published.
