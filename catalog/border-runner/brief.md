# R03 — Border Runner

- Stable ID: `border-runner`; visual revision 1
- Status: implemented local candidate; independent visual/platform qualification tracked separately
- Driver: repeating explicit clip time, positive period (default 2,800ms), seed
- Placement: style-only decorator over an existing host border

A short bright head and restrained fading tail travel around a panel perimeter.
The composition highlights existing glyphs; it never draws a replacement box.
Foreground and modifier rights are explicit. Background, underline color, glyphs,
title exclusions, selected rows, caret, reserved media, wide footprints and all
non-border cells remain unchanged. Apply after freshly rendered host content.

Requested geometry is the motion coordinate system. Supported borders are
3–256 columns by 3–128 rows; empty, degenerate and oversized requests deliberately
do nothing and request no animation wakes. Static motion accents one fixed span;
reduced motion slows by four. Monochrome uses approved bold rather than inventing
color. No glyph repertoire is required by the decorator.

Independent tests cover protected titles, wide cells, field permissions, clipping,
static/oversized advice and repeated sampling. Actual captures are generated from
this same implementation; see manifest/evidence reports for their current review
status. This is not an emulator/font or published-release guarantee.

Code and visual provenance: [project record](../../PROVENANCE.md).
