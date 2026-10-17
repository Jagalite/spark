# Spark contributor guardrails

This repository is continuing the foundation beyond the reviewed F01–F06 slice. Read README.md,
docs/architecture.md, docs/authoring.md, and the relevant design brief before edits.

- The host owns clocks, terminal IO, input, data, and frame scheduling. Runtime and
  catalogue must not introduce those responsibilities or an async dependency.
- Samples are pure functions of configuration, explicit time, seed, requested
  geometry, palette/profile, protected regions, and current base content.
- Keep exactly four workspace packages until evidence justifies another boundary.
- Use the scoped writer; do not bypass it or expose its mutable buffer.
- Protect entire existing wide-glyph footprints and explicit no-write regions.
- No unsafe code, build scripts, extra dependencies, broader paint permissions,
  public contract expansion, golden replacement, or scope expansion without review.
- Never disable a failing test or manufacture qualification evidence. Unrun checks
  and unmeasured targets must stay visible as limitations.
- Code and artwork provenance are separate. The owner selected MIT on 2026-10-17;
  retain third-party terms and do not infer legal clearance from AI generation.
- F07–F22 local work is authorized after the independent 2026-10-14 contract review.
  Runtime/public API changes remain centrally owned and independently tested.
  The owner authorized MIT source publication to Jagalite/spark on 2026-10-17.
  Package publication, release tags and deployment require separate approval.

## Local checks

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo xtask validate
cargo xtask snapshots --check
cargo xtask generate-assets --check
cargo run -p spark-gallery -- --smoke
cargo run -p spark-catalog --example sync_host -- --smoke
cargo run -p spark-catalog --example async_host -- --smoke
cargo run -p spark-catalog --example paced_host -- --smoke
cargo run --manifest-path fixtures/consumer/Cargo.toml --locked
cargo run --manifest-path fixtures/consumer-all/Cargo.toml --locked
cargo test --manifest-path fixtures/holdout/Cargo.toml --locked
cargo check -p spark-runtime -p spark-catalog --target wasm32-unknown-unknown --locked

See docs/performance.md for measured-baseline and actual-preview commands.
