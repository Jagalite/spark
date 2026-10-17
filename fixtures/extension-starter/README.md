# External design starter: Hanging Lantern

A small original decorative lantern, built outside Spark's catalogue with only
public `spark-runtime` and pinned `ratatui-core`. This standalone workspace is a
copyable example, not a fifth Spark workspace package or a qualified catalogue
entry. `publish = false` remains set.

## Run and inspect

From this directory, with Rust 1.90.0 installed:

```sh
cargo run --locked
cargo run --locked -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt -- --check
```

The first command prints the actual sampled Rust buffer's glyphs at all six hold
boundaries and the loop seam, plus reduced, static, tiny and protected embedding
examples. [preview.txt](preview.txt) is that exact output. `--check` and a test
compare it byte for byte without rewriting it. Preview generation uses explicit
synthetic host times; there is no wall-clock dependence or terminal setup.

This is a text capture, not an illustration. Host-added `|` delimiters preserve
visible row width. It deliberately omits cell foreground/background, modifiers,
and underline color. Tests compare actual styled cells, but the preview makes no
claim about color appearance, fonts, terminal compatibility, motion smoothness,
allocation count or performance. No third-party art or font is included.

## Copy and rename

In the original Spark checkout, run (destination must not already exist):

```sh
bash fixtures/extension-starter/copy.sh ../my-lantern my-lantern
cd ../my-lantern
cargo test --locked
cargo run --locked -- --check
cargo run --locked
```

The Bash helper copies source, tests, lockfile, license and the checked preview;
renames the Cargo package and Rust imports; and points `spark-runtime` at the
current complete Spark checkout. It never copies `target/`, downloads code, runs
a build, or publishes a package. It needs ordinary shell utilities, with no extra
Rust dependency. Run the copied package from its own directory so its pinned
toolchain is used. You can remove the original fixture after copying, but retain
the Spark checkout that contains `crates/runtime` and its parent workspace.

For a manual copy, copy the same files plus `src/` and `tests/`, update the runtime
`path` in `Cargo.toml`, rename `spark-extension-starter` in the manifest and
lockfile, and replace `spark_extension_starter` imports in `src/main.rs` and
`tests/public_contract.rs`. Hyphens in Cargo names become underscores in imports.
Retain `[workspace]`, exact dependency pins, `publish = false`, and the MIT notice.
After moving the Spark checkout, update the runtime path explicitly. This example
does not assume a crates.io release exists.

## Change the design

- `src/lib.rs`: immutable `HangingLantern`, borrowed `LanternSample`, complete
  original ASCII poses and unequal frame holds, scoped writes, and repaint advice
- `src/main.rs`: host-owned explicit times, buffers, playback mapping, scheduling
  advice and text preview output; all IO stays in this executable
- `tests/public_contract.rs`: independent expected pose, direct/extreme seeking,
  borrowed rendering, reduced/static/tiny behavior, geometry, preserved host
  content and styles, and host scheduling integration

The native 13×7 lantern grows a light inside its stationary cage over 1.12 seconds
(holds of 240, 120, 160, 320, 160, 120 milliseconds). It conveys no progress,
health or completion. Larger regions center its unchanged footprint. Either
dimension below native size uses one stationary center `*`; an empty request is
a no-op. Both ASCII and Unicode profiles use the same safe ASCII silhouette.
Reduced motion runs at quarter speed; static uses the open-light poster.

Use a fresh host base buffer every draw, then paint this design, then host
overlays. Native spaces are explicit styled erases. Margins outside the centered
native footprint are untouched; compact mode changes only its center. Draw
authoritative labels in the host and pass absolute protected rectangles when
they overlap. `OpaqueSurface` preserves whole existing wide-glyph footprints and
special diff cells, including clipped origins. No mutable buffer escapes it.

The renderer visits at most 91 authored cells (one in compact mode), stores its
poses statically, and uses direct frame selection without replay. That does not
bound the runtime's host-row-prefix scans or the caller's exclusion list, nor
prove zero allocations or performance. The text-preview host allocates buffers
and strings. `change_hint` returns an absolute clip boundary, or no request for
static/compact/empty/unrepresentable cases. `RepaintPlanner` handles hidden and
paused playback. Size-only hints cannot know clipping or complete occlusion; the
host suppresses those time-only requests and merges any advice with input/data
deadlines. Quiet/frozen art still paints on unrelated redraws.

After intentionally editing artwork, render to a temporary file, inspect it,
review the independent pose assertions, then deliberately update `preview.txt`.
Do not update established Spark goldens or widen paint rights to make tests pass.

## Provenance and scope

Code and the small lantern motif were written for this starter on 2026-10-17 with
AI assistance, following the existing public-API holdout's integration pattern.
The lantern is original to this exercise; no third-party artwork, stock character,
generated promotional image or other external asset was used. Code, motif and
derived text preview use the included [MIT license](LICENSE); dependency licenses
remain their own. Tool assistance is not a separate legal-clearance claim.

This starter is local implementation and test evidence. It is not independent
visual approval, a package release, universal platform qualification or permission
to publish your derivative. In the original Spark checkout, see
`docs/extensions.md` and `templates/extension-starter.md` for the extension review
workflow; the independent `fixtures/holdout` remains a separate API test.
