# ADR 0003: Start with a bounded opaque cell writer

- Status: Accepted for F04; decorator interfaces deferred to F08
- Date: 2026-10-13

## Context

Rendering into a shared Ratatui buffer can damage adjacent text even when the artwork uses only single-column symbols. A clipped rectangle and a multi-column glyph may overlap at either edge. Space is an erase operation, not transparent content.

## Decision

F04 exposes `OpaqueSurface`, a small single-column writer over an owned rectangle with optional caller-supplied no-write rectangles. Underlying mutable-buffer access stays private. Writes intersect the requested rectangle with actual buffer bounds, respect those absolute-coordinate exclusions, and use safe cell access. Preserve the original requested geometry for art positioning; intersect only the permitted output.

Make no-write and explicit erase different operations. Reject unsupported art glyphs and conservatively skip writes that could split pre-existing wide host glyphs. Do not add a general grapheme rewriter or infer permission over the rest of a wide footprint.

R01 is a bounded widget rendered into an area the host gives it. This writer is not a decorator capability. Style-only operations, masks, border exclusions, and protected-region transformations will require separate enforceable rights based on the actual R03/R06 use cases.

## Consequences

- Empty or out-of-buffer areas are safe no-ops.
- Offset and clipping behavior can be tested independently of the comet.
- Conservative skips may leave host cells visible inside an otherwise opaque area; that is preferable to corrupting a wide glyph.
- Hosts should supply fresh well-formed base content and owned art regions.
- No claim is made to repair arbitrary malformed host buffers.
- Future transforms must include original base content and paint order in their determinism contract.

## Verification

Test sentinels outside the requested/intersected area, requested-origin stability under clipping, no-write versus space, empty/tiny/offset rectangles, combining marks, and wide glyphs crossing both edges. The permission surface must prevent a caller from obtaining unrestricted mutable-buffer access through it.
