# R05 — Pocket Mascot

- Stable ID: `pocket-mascot`
- Visual revision: **1**
- Status: **Implemented local candidate; independent visual review and release qualification pending**
- Driver: authored complete cell frames, eight positive unequal holds
- Placement: bounded widget with optional absolute protected rectangles
- Native/preferred geometry: **12×6**, centered without stretching
- Full loop: **2400 ms**; reduced loop: **9600 ms**

## Visual idea

An original asymmetric seed-pod creature with a curled antenna, bright pair of
eyes, diamond-shaped snout, short belly bands, and a small loose tail. It watches,
blinks, gathers itself, leans to peek sideways, then softly returns. Deliberately
unequal holds give a long attentive pause and a short gesture rather than constant
mechanical motion. Silhouette and timing carry the expression in monochrome.

| Stage | Hold | Cumulative end |
| --- | ---: | ---: |
| Attentive | 900 ms | 900 ms |
| Blink | 120 ms | 1020 ms |
| Gather | 160 ms | 1180 ms |
| Tilt | 100 ms | 1280 ms |
| Peek | 140 ms | 1420 ms |
| Return | 200 ms | 1620 ms |
| Settle | 180 ms | 1800 ms |
| Rest | 600 ms | 2400 ms |

The final rest and initial attentive poses match at the seam. The quiet poster is
frame zero. Reduced motion retains the same authored sequence with fourfold holds.
Both ASCII and Unicode profiles use these ASCII cells; no alternate font is needed.
Colors resolve through the host's foreground, muted, accent, and highlight roles.
The generator source is [frames.json](frames.json).

## Public consumption

Store `PocketMascot::new()` and borrow `design.sample(&context)` at host-supplied
clip times. `MascotSample` implements ordinary `Widget` by value and by reference.
`with_protected` supplies absolute no-write rectangles. `period()` reports the
full-motion duration. `change_hint(size)` returns the next absolute authored frame
boundary, with checked reduced-motion mapping; it returns no hint for static,
compact, or empty views, or when a future boundary cannot be represented.
`frame_interval(size)` remains a conservative 100 ms advisory cadence (400 ms
reduced). These helpers own no clock or scheduler.

## Geometry and coverage

Center the native 12×6 composition in larger requested areas, leaving every cell
outside it alone. Composition coordinates are determined by the original request,
not the clipped buffer, so clipping does not recenter or stretch the creature.
Below either native dimension, use the static centered `o-o` face if at least three
columns are available, otherwise a single `o`. Empty areas draw nothing. Compact
art never grows to fill the panel and never requests animation-only wakeups.

Every frame has complete 12×6 coverage instructions. `keep` preserves the entire
host cell; `erase` deliberately writes a styled space inside the silhouette;
`replace` writes one validated ASCII glyph and its palette role. Transparent outer
cells are not a space-based clearing trick. Render fresh host content before every
sample; keeping a previous animation frame as the next base would leave trails.
The scoped writer preserves protected rectangles, existing wide-glyph footprints,
and special diff cells. A native sample considers 72 authored cells. Host row-prefix
scanning and exclusion-list costs remain separate from that art-work bound.

## Tooling-only schema v1

The top-level fields are `schema`, `design`, `visual_revision`, `width`, `height`,
`poster`, `glyphs`, `cells`, and `frames`. `glyphs` is a unique printable non-space
ASCII glyph table. The cell table maps unique one-byte printable ASCII `key`s to:

- `keep`: no glyph or role
- `erase`: a palette `role`
- `replace`: an in-range `glyph` table index and palette `role`

Each frame contains a unique lowercase `name`, positive integer `hold_ms`, and
exactly `height` strings of exactly `width` cell-table keys. Each row is complete;
there are no deltas or implicit transparent cells. Missing/unknown fields, unknown
roles/operations, controls, multi-column/combining glyphs, invalid references, and
noninteger or overflowing durations are rejected.

Central ceilings in `xtask/src/assets.rs` apply before expanded frames are emitted:
1 MiB source bytes, width 128, height 64, 256 frames, 262144 total frame cells,
94 glyphs, 95 cell keys, and 24 hours cumulative duration. The file reader is bounded
before JSON parsing; every structural/reference check completes before expansion.
Duration multiplication and sums use checked arithmetic. Runtime `FrameSequence`
separately validates nonempty, bounded, strictly increasing cumulative ends.

`cargo xtask generate-assets` emits checked-in static Rust arrays.
`cargo xtask generate-assets --check` validates source and checks byte-for-byte
deterministic regeneration. It does not rewrite drift during a check. The catalogue
contains no file access, parser, asset download, decompression, or frame-zero replay.
Runtime selection uses binary search on cumulative ends and direct complete-frame
lookup, including shuffled timestamps and very large times.

## Evidence and provenance

Focused executable tests cover all eight poses with independent complete-cell
expectations, both sides of every hold boundary, direct/shuffled seeking and the
loop seam, distinct keep/erase behavior on styled host content, quiet and reduced
profiles, centering, clipping, tiny/extreme geometry, wide glyphs, exclusions, and
strictly future repaint advice. Tooling tests exercise malformed shapes, byte and
resource limits, glyphs, coverage, references, durations, deterministic output, and
systematic malformed-byte/truncation mutations. Mutation tests are bounded test
coverage, not a claim of exhaustive fuzzing.

The frame drawings and timing were newly authored for this repository with AI
assistance on 2026-10-14, from the original R05 brief. No external character artwork,
sprite sheet, font asset, or third-party image was copied or imported. Source code
provenance and artwork provenance remain distinct; the owner selected MIT on 2026-10-17; see [LICENSE](../../LICENSE). See [the project policy](../../PROVENANCE.md).

Actual generated previews and synthetic panel embedding require independent visual
review. This implementation is not evidence of real-terminal/font qualification,
allocation measurements, all-platform support, or a complete foundation release.
