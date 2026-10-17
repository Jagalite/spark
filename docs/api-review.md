# F01–F06 API review

Independent implementation/test review, 2026-10-13. This is a vertical-slice review,
not acceptance of the later complete foundation release.

## Result

Stored immutable design + cheap borrowed context/sample works in a separate
consumer. The locked downstream application uses Ratatui 0.30.2 with one shared
core 0.1.2, renders borrowed samples through `Terminal<TestBackend>`, and implements
a different `TravelingMarker` using public time/context/surface APIs. No private
catalogue access, dynamic registry, unstable Widget trait, or renderer clock is
needed. This early extension smoke is not the full F21 holdout/release exercise.

13 independent catalogue contract tests include a 192-case seed/period/profile/
geometry sweep, shuffled timestamps, both sides of loop seams, protected styles,
clipping, wide footprints, tiny layouts, and extreme requested rectangles on small
buffers. Runtime tests separately cover validation, surface operations, and colors.

## Defects discovered and fixed

1. A forced-width cell containing actual control/escape sequences was skipped at
   its origin, but its continuation footprint could be overwritten. Width lookup
   now honors `CellDiffOption::ForcedWidth` before rejecting ordinary control
   content. Regression covers a clipped origin and out-of-order writes
2. Caller RGB underline-color overrides escaped ANSI-16/monochrome conversion.
   Core's pure underline-style feature is now explicit; fg, bg, and underline
   colors are all resolved. Regression preserves requested modifiers and checks
   actual sampled cells
3. Borrowed Widget usage was made explicit in tests/consumer so strict Clippy
   checks verify that implementation instead of suggesting a by-value call

## Visual review

An independent reviewer inspected the actual contact sheet, synthetic embedding,
quiet fallback, every frame of the 40-frame loop, and 1599/1600/1601 ms snapshots.
The bright head, broken fading spine, and sparse sparks remain legible. Two-edge
wrap is intentional; host labels and borders remain distinct. Full/slow loops use
the same pixels at 40/160 ms delays (1.6/6.4 seconds). Minimum 8-column strips and
1–7-column static-point fallbacks are also tested.

This is frame-sequence/fixed-font review. The separate Linux PTY smoke checks real
terminal protocol, controls, clean exit, and terminal restoration; it is not an
emulator/font rendering qualification. Preview BOLD approximation and unknown
terminal background are recorded limitations.

## Decisions before F07+

- Retain `ClipTime`, positive `Period`, and requested-coordinate clipping
- Keep `Keep`, explicit `Erase`, and full-cell `Replace` distinct
- Document the conservative wide-footprint policy: underlying host glyphs can
  remain visible even inside an owned region; no partial grapheme replacement
- Keep the art cap distinct from row-prefix and exclusion-list scanning costs
- Avoid generalizing the one-design cadence hint into a public scheduler now
- Design style-only/masked operations from R03/R06 requirements in the next stage
- Measure the first inline adapter before promising allocation-free text

No blocking issue remains in the reviewed F01–F06 contract.

## Continuation review, 2026-10-14

Local continuation was authorized after a second independent contract review.
That review found no blocker in explicit-time sampling, borrowed samples, clipping,
or conservative wide-footprint preservation. F07–F11 proceeds with distinct
field-limited style and masked-glyph writers, bounded inline widths, clamped finite
clips, and centrally bounded assets with direct frame selection. Shared interfaces
remain centrally owned and independently tested. Publication remains separate.
