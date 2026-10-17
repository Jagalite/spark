# RXX — Design title

- Stable ID: `choose-a-stable-id`
- Visual revision: positive integer; separate from runtime API version
- Status: brief only / local implementation / independently reviewed / qualified
- Authorized milestone/task:
- Contributor and dates: identify accurately; do not invent a rights holder
- Manifest path: `catalog/<id>/manifest.json`

## Composition and motion

Describe a recognizable silhouette, staged motion, unequal holds if applicable,
loop seam or finite final pose, and how the design differs from existing entries.
Explain why this motion is useful in its intended host layout. A palette/speed
variant is not another design.

## Driver, input, and lifecycle

Specify explicit time, authored frames, or caller-supplied validated typed values.
State seed/configuration defaults, finite/repeating semantics, direct seeking,
equal-input determinism, and time units/range. For data-driven art, distinguish
unknown from zero and identify what remains host-owned. Document replacement,
restart, cancellation, and stable entity/generation responsibilities.

Describe `change_hint`/cadence behavior, exact authored boundaries when applicable,
and conditions with no animation-only wakeups. Static, paused, hidden, and complete
are different states; explain the appearance on unrelated redraws.

## Placement, fit, and geometry

- Supported surfaces: inline / widget / decorator / scene / masked text
- Preferred, native, minimum-full, and capped dimensions:
- Fit policy: adaptive / centered / cropped; authored cells are never stretched
- Empty, compact fallback, offset, and partly/fully clipped behavior:
- Requested-coordinate behavior under clipping:
- Inline declared width and padding policy, when applicable:
- Unsupported placement/text fallback:

## Paint rights and bounded work

Name the public surface and exact allowed operations/fields. Define keep/erase/
replace semantics, mask/title exclusions, protected content, and whole-wide-glyph
policy. State the host's base-redraw/composition ordering requirements. A proposed
capability is not permission to bypass the scoped writer.

State art cell/particle/asset limits and any scratch/storage allocations. Separate
art bounds from host-buffer row scanning, text validation, and exclusion counts.
Do not claim measured zero allocation or constant total cost without evidence.

## Palette and profiles

List remappable roles, optional known-background use, validated glyph repertoire,
ASCII/ANSI-16/monochrome choices, reduced motion, and authored static pose. State
which differences change structure and which only change style.

## Authored source, if applicable

- Source schema/version, path, generator, and checked-in output:
- Complete frame dimensions/count and positive integer holds:
- Cumulative period, quiet poster index, and loop seam:
- Explicit cell coverage, glyph references, and palette-role references:
- Source/dimension/frame/cell/glyph/duration limits and checked arithmetic:
- Validation-before-expansion and deterministic generation checks:

Delta playback, runtime parsing/downloads, and arbitrary unbounded glyph strings
are not part of the current authored-asset contract.

## Realistic embedding

Name a host layout, authoritative labels, adjacent/protected content, stable
identity, and host-owned data cadence. Explain the visual's meaning without making
it the authority for progress, task completion, or business state.

## Evidence and remaining gaps

Link implementation, manifest, source assets, direct and independent tests, exact
styled-cell captures, isolated/embedded previews, and qualification reports.
Record source/configuration hashes, time/input traces, geometry, seed if applicable,
palette/profile, and renderer/font/terminal settings. State passed, failed, unrun,
and unmeasured checks separately. A linked future output path is not evidence.

- Boundary and shuffled/extreme-time tests:
- Geometry/protected/wide/style preservation tests:
- Invalid-input and deterministic regeneration tests:
- Independent visual reviewer, scope, date, and findings:
- Native/terminal/WASM/platform checks actually run:
- Allocation/performance/package measurements actually run:
- Release gates still open:

## Provenance and terms

Use [the provenance entry template](provenance-entry.md). Record code and artwork
separately, actual sources, assistance, transformations, and unresolved permission
questions. Generated previews retain source restrictions. Project terms and any
publication remain owner decisions; implementation/review is not authorization to
publish.
