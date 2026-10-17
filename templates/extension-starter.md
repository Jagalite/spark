# External extension review checklist

Copy the runnable [starter](../fixtures/extension-starter/README.md) and follow
[the extension guide](../docs/extensions.md). Keep this checklist with your own
project; it is not a request to add an entry to Spark's catalogue or workspace.

## Identity and meaning

- Design name / stable project ID:
- Visual revision (independent of API/package version):
- Contributor and date:
- Status: proposal / local implementation / independently reviewed / qualified
- Host use case and decorative or semantic meaning:
- Which labels/data remain authoritative in the host:
- How the silhouette/motion differs from existing designs:

## Copy and build

- Standalone package location; runtime checkout and revision:
- Cargo package and Rust crate names after rename:
- Exact dependency pins, lockfile and `publish = false` retained:
- Own `[workspace]`; Spark's four package boundaries unchanged:
- Reproducible build/test commands; external-copy check actually run:
- MIT notice retained; different licensing questions recorded explicitly:

## Pure sampling and host responsibilities

- Immutable configuration and borrowed sample type:
- Explicit inputs: clip time, seed if used, palette/profile, geometry, host data:
- Direct-seek rule and full supported time range; no historical replay:
- Repeating period or finite completion behavior, exact boundaries and overflow:
- Host-owned clocks, IO, data, input, task state and presentation scheduling:
- `change_hint` behavior; which quiet/tiny/finished cases return no request:
- Reduced-motion time mapping is identical for frames and deadlines:
- Paused/hidden/clipped/occluded handling in the host; unrelated redraw behavior:

## Paint and geometry

- Public writer and narrowest allowed operations/style fields:
- Native/minimum/capped sizes; fit and compact fallback:
- Empty/offset/partly clipped/fully off-buffer/extreme geometry behavior:
- Requested coordinates survive clipping without re-layout:
- Explicit keep vs styled erase vs full replacement semantics:
- Absolute protected rectangles, whole wide glyphs and special diff cells:
- Fresh base draw, declared decorator order and host overlay ordering:
- Art-work bounds; host scan/exclusion costs; measured allocation limits if any:
- ASCII/Unicode, ANSI16/truecolor/monochrome, reduced/static presentations:
- Unknown background remains unknown; caller palette roles are respected:

## Evidence, not assumed guarantees

- Independent expected-cell assertion; owned and borrowed render equivalence:
- Repeated/shuffled/extreme sampling and hold/seam/finite boundary tests:
- Static/reduced/tiny/wide/protected/clipped coverage and style preservation:
- Real Rust-consumer preview command, input times/geometry/profiles and output:
- Preview rendering limitations (e.g. glyph-only omits all colors/modifiers):
- Preview reproduction check passes without rewriting expected files:
- Original visual source separate from implementation and derived preview:
- Independent reviewer, date, observations and unresolved issues:
- Actual target/toolchain checks; unrun platforms visibly listed:
- Actual performance/allocation measurements, or explicitly unmeasured:
- No existing catalogue golden replacement or runtime permission widening:

## Provenance and release

- Original code sources and terms, including retained upstream notices:
- Original artwork sources/creator/terms; no source inferred from appearance:
- Tool/AI assistance actually used; no automatic legal-clearance claim:
- Derivative preview provenance and any assets/fonts used:
- Unresolved permissions or licenses:
- Publication destination and explicit owner approval, if any:
- Package release, tags, binaries or deployment remain separate decisions:
