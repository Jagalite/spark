# MIT foundation source publication verification

This is the historical foundation publication record for commit `f2488f7`. Later expansion
implementation and checks are recorded in [expansion status](expansions.md).
The [current acceptance audit](foundation-status.md) verifies remote native/WASM
CI at `d1de335`; historical unrun-remote statements here describe the earlier handoff.

Prepared 2026-10-17 for the owner's request to review the code, add README and
.gitignore, and commit/push to Jagalite/spark. The owner selected MIT. This scope
covers original source, docs and authored assets under [LICENSE](../LICENSE),
with [third-party terms](../NOTICE.md) retained. All seven package manifests remain
`publish = false`; this is not crates.io, a release tag, binary distribution or deployment.

## Recovery and review scope

The complete October 14 archive was recovered byte-for-byte with SHA-256
`ce9bbfe016b66ed55f979a6487e3919b72fc6b94e9a664f38ffe7998b809fcf2`; all 269
member checksums passed. The earlier temporary licensed checkout was lost when
the execution workspace reset. This publication reconstructs the limited docs,
license metadata and ignore-policy changes; it does not claim the old local Git
commit or tree identity. Runtime, rendering, tests and regression golden bytes
remain unchanged from the verified archive.

A bounded independent source/provenance review found no source-code blocker.
The publication review required omitting detailed historical PTY/environment
evidence; the final public tree follows that narrower source-only scope.
Fresh Linux validation passed all 17 commands: 115 workspace tests, one rustdoc
test, five public-API holdout tests, seven Python harness guards, strict Clippy
and formatting, snapshot/asset checks, gallery/three-host smokes, standalone
consumers and renderer-only WASM compilation. Source fingerprint:
`921488ce301063f716ba5650c24161ae8023c6bc72db5393f8c0dff71a4dcbf6`.
The machine-readable results live in
[checks.json](../qualification/publication-20261017/checks.json).
The source fingerprint covers implementation, tests, manifests, lockfiles, CI,
authoring inputs and scripts using the xtask hashing algorithm; it is distinct
from the root tracked-file checksum list.

The final source-only export also passed validation, both golden checks, generated
asset drift and all seven harness guards with archived reports absent.

## Reproducibility and limitations

See [ARTIFACTS](../ARTIFACTS.md) for the explicit source-versus-archive policy.
The privately archived original `HANDOFF.json` and dated qualification are historical, including their
then-unselected license and unpublished status; they do not override current MIT
terms or establish current remote publication/CI status. Their measured results
have not been rewritten as fresh benchmarks.

The existing GitHub Actions workflow validates native Linux/macOS/Windows source
and renderer-only WASM compilation on pushes/PRs, without deployment credentials.
A configured workflow is not evidence that it has run. Remote commit/ref and exact
commit CI results must be verified separately after upload.

Physical terminal/font/tmux/SSH checks, browser-WASM playback and exact-cell parity,
multi-hour/day endurance, universal resource guarantees, registry/public name
clearance and package/binary releases remain open. Source publication does not
mean all F01–F22 release-acceptance gates are satisfied.
