# Independent public-API holdout

`CrossWeave` is a separate consumer's small two-axis lattice design. It depends
only on `spark-runtime` and pinned `ratatui-core`, and uses ordinary borrowed
`Widget` samples, explicit context/time, public scoped painting, direct authored
frame holds, and public repaint advice. No runtime or catalogue source changes
were made to support this extension.

This is an independent F21 API exercise, not a seventh qualified catalogue entry
or a publication claim. The tiny example prints sampled glyphs; it is not a
terminal-emulator or font qualification. Its source and invented motif were
written for this local exercise with AI assistance; no external art was used.
Project source and authored motif are covered by [MIT](../../LICENSE).

```sh
cargo test --manifest-path fixtures/holdout/Cargo.toml --locked
cargo clippy --manifest-path fixtures/holdout/Cargo.toml --all-targets --locked -- -D warnings
cargo run --manifest-path fixtures/holdout/Cargo.toml --locked
cargo tree --manifest-path fixtures/holdout/Cargo.toml --edges normal --locked
```

Tests exercise deterministic direct seeking, borrowed rendering, clipped requested
coordinates, wide/protected content, static/tiny advice, and host deadline mapping.
Memory is bounded by a 17×7 authored region; no universal allocation claim is made.
