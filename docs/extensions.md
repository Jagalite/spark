# Build a design outside the catalogue

Spark's public runtime is sufficient for a small custom design. You do not need
catalogue internals, a registry, a fifth workspace package, an async runtime or a
terminal backend. Start with the runnable
[extension starter](../fixtures/extension-starter/README.md). The independent
[public-API holdout](../fixtures/holdout/README.md) remains a separate consumer
exercise; the starter adds a copy/rename path, editable original art, a reproducible
preview and a contributor checklist rather than replacing that evidence.

## Copy, rename, build

From the Spark repository root, with Rust 1.90.0 available:

```sh
# Inspect the unmodified starter first.
cargo run --manifest-path fixtures/extension-starter/Cargo.toml --locked
cargo test --manifest-path fixtures/extension-starter/Cargo.toml --locked

# Copy source only to a new sibling directory. Do not choose an existing path.
bash fixtures/extension-starter/copy.sh ../my-lantern my-lantern
cd ../my-lantern
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt -- --check
cargo run --locked -- --check
cargo run --locked
```

The helper uses Bash and standard shell utilities. It copies the original MIT
notice, exact dependency pins and lockfile, rewrites the package name and Rust
imports, and sets an absolute path to this checkout's `spark-runtime`. It refuses
an existing destination. It does not copy build outputs or fetch/build/publish
anything. It also rejects reserved names and dependency-name collisions. Keep the
complete Spark checkout: the runtime's manifest inherits its
parent workspace metadata. Moving the checkout requires updating the dependency
path. There is no assumption of a published registry package. The starter README
also gives manual-copy steps if a Bash helper does not suit your environment.

The new package contains its own empty `[workspace]` table and `publish = false`.
Do not add it to Spark's root workspace or remove the publication guard as part
of copying it. An ordinary application can instead copy the design module into
its own crate and use the same dependencies. New dependencies, wider runtime
permissions or public runtime changes require separate review.

## A complete, deliberately small example

`HangingLantern` is an immutable original 13×7 design. Its borrowed
`LanternSample` implements Ratatui's `Widget` by value and shared reference and
has a `change_hint(Size)` method. The six authored holds total 1.12 seconds and
are selected directly with the public `FrameSequence`; there is no replay or
mutable animation state. No seed is needed because the motif has no randomness.

The light grows inside a stationary cage. It is decorative and conveys no task
status. The host owns any real task data, labels, stable identity, elapsed clock,
visibility, input, terminal lifecycle and scheduling. The executable demonstrates
`Playback`, `SampleContext` and `RepaintPlanner` with synthetic host times. It
prints the resulting buffers; the library never reads a clock or performs IO.

Behavior is explicit:

- Native or larger: center the 13×7 footprint without stretching; fully replace
  safe footprint cells, including intentionally erased spaces; keep outer margins
- Smaller in either dimension: one stationary centered `*`, keeping other cells;
  zero width or height: no-op
- Clipped buffers: retain the requested-local coordinates, never recenter on the
  intersection; fully clipped writes have no effect
- Static: open-light poster; reduced: quarter speed in the same caller clip-time
  domain; both glyph profiles: the same one-column ASCII composition
- Palette: caller foreground/accent/highlight plus optional known background,
  resolved for truecolor/ANSI16/monochrome; no guessed background color
- Permission: only public `OpaqueSurface`; whole pre-existing wide footprints,
  special diff cells and absolute protected rectangles remain untouched
- Advice: exact absolute next clip boundary, checked for overflow; none for
  static/compact/empty. The host/planner suppress hidden or paused playback. A
  size-only query cannot detect clipping or occlusion; suppress these in the host

Draw base content freshly, apply the design, then draw host overlays. A previous
decorated frame is not the same base input, and leaving old art in a reused buffer
can create trails on resize. `Keep`, explicit styled `Erase` and full `Replace`
are different operations. For your own design choose the narrowest public writer:
`OpaqueSurface` for owned cells, `MaskedSurface` for an explicit region/border
mask, or `StyleSurface` with only the style fields you need. Never expose or bypass
the writer's mutable buffer.

The starter visits at most 91 authored cells and keeps static pose data. Runtime
wide-footprint scanning also depends on host row prefixes, and exclusion checks
depend on the supplied list. This is a bounded art-work statement, not a universal
allocation or total-cost measurement.

## Actual preview and reviewable output

The checked [text preview](../fixtures/extension-starter/preview.txt) comes directly
from the same Rust widget used by downstream consumers. It includes every hold
boundary and loop seam, reduced/static/compact output, a protected host label,
geometry, times and host repaint deadlines. Inspect it directly or reproduce it:

```sh
cargo run --manifest-path fixtures/extension-starter/Cargo.toml --locked --quiet \
  > /tmp/spark-extension-preview.txt
diff -u fixtures/extension-starter/preview.txt /tmp/spark-extension-preview.txt
cargo run --manifest-path fixtures/extension-starter/Cargo.toml --locked -- --check
```

The text preserves actual cell glyphs and widths, with host-added row delimiters.
It omits foreground/background/underline colors and modifiers by design. It is
neither terminal-emulator output nor a raster screenshot and does not qualify
color, font coverage, animation pacing or terminal behavior. Styled-cell
preservation and palette behavior are tested directly. There are no added fonts,
third-party images, promotional renders or separate art implementation.

When changing art, inspect a temporary capture and all poses first. Update only
your extension's expected preview and independent expected-cell tests deliberately;
do not replace catalogue goldens. Broader publication or qualification still
requires the relevant review and owner approval.

## Validation and CI integration

Copy [the extension checklist](../templates/extension-starter.md) into your own
project and fill in the actual evidence. Keep the original MIT notice and record
code and artwork sources separately. The source and lantern motif here were
authored for this example with AI assistance and no external artwork; assistance
does not itself establish legal clearance.

The fixture is intentionally outside the parent workspace, so workspace checks
alone do not run it. CI should execute these explicit commands from Spark's root:

```sh
cargo fmt --manifest-path fixtures/extension-starter/Cargo.toml -- --check
cargo clippy --manifest-path fixtures/extension-starter/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path fixtures/extension-starter/Cargo.toml --locked
cargo run --manifest-path fixtures/extension-starter/Cargo.toml --locked -- --check
```

Tests cover an independent open pose, complete style reset, shuffled and extreme
direct seeking, exact boundaries and overflow, borrowed rendering, reduced/static
and native-subsize combinations, wide/tall compact requests, wide viewports,
clipping and off-buffer requests, protected
and whole-wide/special footprints, profiles and known/unknown backgrounds, and
host pause/visibility/deadline behavior. The preview check is deterministic and
read-only. Copy the package outside the checkout and rerun the same checks to
verify the recipe, rather than assuming a correct in-tree build proves copying.

These checks do not substitute for independent visual acceptance, supported-target
execution, allocation/performance measurement or package-publication approval.
See [authoring](authoring.md) for the wider review policy and
[architecture](architecture.md) for the host/runtime boundary.
