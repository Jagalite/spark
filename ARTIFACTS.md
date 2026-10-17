# Source repository and archival evidence

The Git repository retains source, tests, authoring inputs, generated Rust,
pinned lockfiles, both exact regression fixtures in `fixtures/cells`, six full
preview GIFs, the overview and directly linked review images, render manifests,
and the small fresh publication-check summary. These support normal builds and tests.

Large generated `previews/*.cells.json`, other preview derivatives, verbose
`.log` files, raw `.cast.gz`/`.output.bin.gz` recordings, `.samples.jsonl` resource
streams, all detailed historical machine/PTY/performance qualification records,
archived input snapshots and historical measured ELF binaries are deliberately ignored. No source,
regression golden or authoring input is excluded by this policy. Measured binaries
are not redistributed because their full dependency/platform notice obligations
were not reviewed for binary distribution.

The complete October 14 evidence remains in the owner's Library archive
`spark-foundation-endurance-2026-10-14(1).zip` (8,473,544 bytes), SHA-256:
`ce9bbfe016b66ed55f979a6487e3919b72fc6b94e9a664f38ffe7998b809fcf2`.
This is an archival provenance reference, not a public download URL. Git users
can regenerate captures using the documented `cargo xtask render`, `record` and
qualification commands; reruns are new evidence and need their own source stamp.

The original `HANDOFF.json`, dated qualification reports, their verifier and nested
endurance `SHA256SUMS` are historical records archived privately.
Their hashes describe the October 14 archive, not the licensed publication tree.
The historical endurance verifier requires that complete archive; it is not a
fresh-checkout CI command. The root `SHA256SUMS` instead covers exactly
all tracked files except itself. [Publication verification](docs/publication.md)
records the fresh source checks. Verbose publication logs are local ignored output.

A source-publication review required this narrower scope on 2026-10-17. Detailed
process IDs, sampling/checkpoint traces and environment records are not needed
to build or test the library and are intentionally excluded. The complete earlier
reviewed handoff remains separately preserved; removed data was not rerouted or
split into other public files. Historical documentation summarizes bounded findings
without claiming that the private raw evidence is available in Git.
