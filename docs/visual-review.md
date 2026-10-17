# Independent fixed-font visual review

These observations concern actual Rust cell captures and their fixed-font image
and GIF derivatives. They are not terminal-emulator/font qualification, legal
clearance, or blanket foundation release acceptance. Each section names the
review scope and remaining limits. Reviews are independent of the named designs'
implementation author, with AI assistance.

## R03 — Border Runner

Reviewed 2026-10-14 by a contributor who did not implement R03. Visual revision 1;
main capture source hash at inspection:
`a5c114b54a74a0b265c37622468b7210df78a312fcc35b2f10369313dbd51ee5`.
Later source changes need either a fresh capture review or an explicit finding
that the drawing output is unchanged.

### Evidence actually inspected

- The [actual contact sheet](../previews/border-runner-contact-sheet.png), every
  frame of the [70-frame full loop](../previews/border-runner.gif), and the
  [slowed sequence](../previews/border-runner-slow.gif) as full-frame contact grids
- Programmatic verification that full/slow GIFs contain identical frame pixels,
  with 40 ms / 160 ms holds respectively: 2.8-second / 11.2-second playback
- [Boundary captures](../previews/border-runner-boundaries.png) at 0, 1, 2799,
  2800, and 2801 ms; [quiet ASCII poster](../previews/border-runner-fallback.png);
  [reader embedding](../previews/border-runner-reader.png)
- The exact capture's `smallest` 3×1 no-op view, rendered for inspection, plus a
  separate actual public-API 3×3 minimum-full sample at eight perimeter phases

This is frame-sequence/pixel inspection plus verified playback timing, not an
observation of a native terminal animating in real time. Rendering uses installed
DejaVu Sans Mono with the preview renderer's documented style approximations.

### Findings

The short cyan/white head and narrow fading trail travel clockwise without
changing border geometry or creating a broad flashing panel. The interior labels
stay dominant. The head rounds all four corners coherently, returns along the
left edge, and leaves no accumulating trail in the fresh-base sequence.

The head deliberately disappears while traversing the excluded `HOST TITLE`
segment, including much of the start/seam views. This is visible and expected from
the no-write region, not a broken loop or skipped-time correction. It makes the
motion less continuous than an uninterrupted rectangular circuit, but preserves
the more important title. Do not remove that exclusion merely to make the loop
look continuous.

The quiet poster remains an unobtrusive border, with only a restrained static
modifier emphasis. In a 3×3 panel the small highlight remains confined to the
outline and the single center cell stays empty. The 3×1 fallback correctly leaves
the host's tiny border fragments unchanged rather than inventing a glyph spinner.

The reader fixture retains the selected line, caret, media box, inner title, and
outer labels. Its wide host character appears as a missing-glyph box in this
installed font, so the image is not evidence of successful CJK font rendering;
complete-cell/wide-footprint tests carry the preservation claim.

**Local visual finding:** no blocking geometry, contrast, trail, or protected-text
issue in the inspected fixed-font captures. Terminal-specific rendering,
accessibility preference fit, and other fonts remain separate qualification.

## R06 — Prism Reveal

Reviewed 2026-10-14 by a contributor who did not implement R06. Visual revision 1;
main capture source hash at inspection:
`a5c114b54a74a0b265c37622468b7210df78a312fcc35b2f10369313dbd51ee5`.

### Evidence actually inspected

- The [actual contact sheet](../previews/prism-reveal-contact-sheet.png), every
  frame of the [45-frame finite replay](../previews/prism-reveal.gif), and the
  [slowed replay](../previews/prism-reveal-slow.gif) as full-frame contact grids
- Programmatic verification of identical full/slow frame pixels and 40 ms /
  160 ms delays: 1.8-second / 7.2-second observation windows
- [Boundary captures](../previews/prism-reveal-boundaries.png) at 0, 1, 1199,
  1200, and 1201 ms; [quiet final pose](../previews/prism-reveal-fallback.png);
  [reader embedding](../previews/prism-reveal-reader.png)
- The exact capture's 3×1 `SIG` crop, plus separate actual public-API 1×1 samples
  at 0, 400, 1199, 1200, and 2000 ms and unsupported-width text fallback samples

The extra minimum/fallback sampling called the ordinary public designs from a
small independent consumer. It was an inspection probe, not a new production
implementation or a claim of broader package compatibility.

### Findings

A short empty anticipation precedes a readable left-to-right reveal. The bright
single-cell leading edge and small cyan following band identify the transition
without a full-line flash. Previously revealed characters settle behind it; the
space between words remains correctly placed. The result is restrained rather
than a multicolor prism spectacle.

At 1199 ms the final `D` still carries a residual cyan accent. At 1200 ms the whole
`SIGNAL RESTORED` line is stable foreground text, and subsequent frames keep that
same final appearance. The preview GIF repeats its inspection window for viewing;
that replay must not be presented as the component restarting itself after
completion.

Quiet/static starts with the full readable final text. The 3×1 crop shows `SIG`;
the actual 1×1 request reveals only `S` and leaves it settled. This is honest
requested-width clipping, not a guarantee that a one-cell host area communicates
the full phrase. Hosts still need sufficient room for their authoritative text.

The reader layout retains selected host content, caret, media, and surrounding
labels while the reveal is mid-phrase. A separate wide/combining-text probe keeps
its original glyph structure and only changes eligible styles. The wide character
again renders as a missing-glyph box with the installed font; exact cells and
wide-footprint tests, rather than these pixels, establish content preservation.

**Local visual finding:** no blocking ordering, final-state, clipping, or neighbor
legibility issue in the inspected fixed-font captures. No native-terminal timing,
universal font support, or business-operation completion claim is made.

## R02 — Assembly Progress

Reviewed 2026-10-14 by the shared-runtime owner, who did not implement R02.
Visual revision 1, source capture `a5c114b54a74a0b265c37622468b7210df78a312fcc35b2f10369313dbd51ee5`.
Inspected the six-pose contact sheet, all 45 frames as a contact grid, boundary
0/1/1799/1800/1801 ms, quiet/static, unknown-progress and request/result images.
Verified that full/slow GIF pixels match exactly, at 40/160ms frame holds.

The moving gantry and rolling lower belt have a recognizably different silhouette
from the comet. The solid module count stays fixed at the caller target while the
carrier moves; unknown progress clearly shows a question-mark workpiece with no
invented filled percentage. The gantry reverses smoothly at its endpoints, and
the loop seam keeps the assembled modules fixed. One-cell step changes in rollers
are visible and appropriate for the coarse cell grid.

The ASCII/monochrome poster retains the machinery and assembled/unassembled
contrast. The request fixture keeps its 65% authoritative host label separate and
legible. No blocking clipping, legibility or pose issue appeared in this fixed-font
review. Full minimum-size combinations are covered in exact-cell tests; the visual
review does not establish every font, terminal or host layout.

## R04 — Telemetry Embers

Reviewed 2026-10-14 by the shared-runtime owner, who did not implement R04.
Visual revision 1 and same capture source hash as R02. Inspected the contact sheet,
all 200 frames in an eight-second observation contact grid, 0/1/7999/8000/8001ms
boundaries, quiet poster and dashboard embedding. Full/slow image sequences were
verified pixel-identical at 40/160ms holds.

Sparse upward particles, small dim trails and a quiet hearth leave considerable
negative space. The constellation remains coherent in monochrome. The larger
embedding does not obscure host level/data-cadence labels. Individual particles
expire and reappear according to separate analytic lifetimes; there is deliberately
no common eight-second seamless loop. GIF replay of this observation window has a
visible reset and must not be advertised as a native period seam.

No accumulating residue or runaway density appears in the inspected sequence.
The field is relatively energetic at level 0.65; reduced/static profiles remain
important for readers who prefer less motion. Minimum geometry, zero-level and
protected cells are tested independently; terminal-specific perception remains
unqualified. No blocking visual implementation defect found.

## R05 — Pocket Mascot

Reviewed 2026-10-14 by the shared-runtime owner, who did not author the creature
or its frame schema. Visual revision 1 and same capture source hash as R02.
Inspected all 60 frames of the 2.4-second capture, its contact sheet, 0/1/2399/2400/
2401ms seam, quiet poster and request/result embedding. Full/slow GIF pixels match
exactly at 40/160 ms holds.

The original antenna creature has a clear held attentive silhouette, a short blink,
a compressed anticipation, a small lean, then a soft return to its stable pose.
Unequal holds carry its personality rather than color alone. The first/last seam
is the same poster, with no residual art in the fresh-base frames. Its thin ASCII
outline remains clear in monochrome and does not compete with the host task label.

This proves a distinct authored-frame path and recognizable local art candidate,
not universal font appearance or external character/IP clearance. The compact
`o-o`/`o` fallback is intentionally much simpler than the full creature; exact
centering and keep/erase coverage are verified separately. No blocking fixed-font
pose, silhouette or embedding issue found.

## R01 and final-source linkage

R01's earlier independent full-sequence, seam, quiet/minimum and embedding review
is recorded in [the vertical-slice review](api-review.md). Its expected cell fixture
is retained unchanged. The current overview was also inspected beside the five
new designs and the six silhouettes/compositions are distinct rather than color
variants. Final qualification regenerates captures against the final source hash;
cell/pixel equality checks preserve these findings when nonvisual code changes.

Final-source validation: `qualification/foundation-linux-x86_64/preview-consistency.json`
confirms all six semantic capture sets and full/slow frame pixels are unchanged
at source `0c9fd43a2bb9b799b1cce95f40dcdafb4d0d581e9aa353e5c26b5e725ed1eaf2`.
The original F06 preview files and their verified manifest/capture hash are also
preserved under `previews/f06/`, separate from current six-design captures.

The refreshed R02 12×3, R04 8×4 and R05 12×6 minimum-full PNGs were
inspected after final regeneration. R02 retains its gantry/modules/two rollers;
R05 retains its full native silhouette. R04 is necessarily sparse at 8×4, with a
single visible spark and short dim trail in the inspected pose, rather than the
full-sized field density. No clipping or invented semantic state was observed.
