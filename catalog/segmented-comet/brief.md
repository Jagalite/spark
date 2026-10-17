# R01 — Segmented Comet

- Stable ID: `segmented-comet`
- Visual revision: 1
- Status: **Implemented bounded-widget slice**, F05; see [source](../../crates/catalog/src/comet.rs). Qualification evidence and limitations are recorded separately in [compatibility](../../docs/compatibility.md) and [performance](../../docs/performance.md).
- Driver: repeating explicit clip time, validated positive period, caller seed

## Visual idea

A luminous pointed head pulls a broken spine of bright segments and dim gaps. Detached sparks occupy the outer rows; sparse fixed marks give the strip a track. The composition is a compact glowing trail rather than a stock radial spinner. The default period is 1,600 ms and seed is 7.

## Contract

Store `SegmentedComet`; borrow a `CometSample` for ordinary `Widget` rendering. Preferred views are 16×1 and 24×3. Widths 8–256 adapt; widths 1–7 use a quiet static point. Empty requests draw nothing. Larger requests center a strip capped at 256 columns and three rows, leaving cells outside the strip untouched. Clipping preserves the requested composition origin.

Paint through the opaque writer with optional protected rectangles. Never split existing wide host glyphs. Palette roles supply glow contrast without assuming a known background. ASCII and monochrome are available; reduced motion is four times slower without detached sparks; static motion fixes the head at a deliberate three-quarter position.

## Embedding and evidence

Use a dedicated status strip beside host-owned labels. Test repeated/shuffled timestamps, seed consistency, period edges, offsets/resizes/clipping, large times, wide boundaries, and protected neighbors. Review isolated and embedded captures before calling the design visually qualified.

The F07 inline adapter returns an owned styled `Line` with exactly 0–256 requested columns, including padding. It allocates; the widget path remains direct. No typed progress or business-completion semantics are implied. Provenance and unresolved licensing: [project record](../../PROVENANCE.md).
