# R06 — Prism Reveal

- Stable ID: `prism-reveal`; visual revision 1
- Status: implemented local candidate; visual/platform qualification recorded separately
- Driver: explicit finite clip time; default duration 1,200ms
- Placement: masked one-row authored text, or style-only unsupported-text fallback

An anticipatory gap precedes a narrow bright highlight that crosses the text,
leaving readable foreground characters behind. The settled pose is stable and
complete; rendering never emits business callbacks. Static motion shows final text.

The constructor accepts at most 256 non-control scalars. Validated single-column
text is stored as authored glyphs and replaced only inside the requested mask.
Wide/combining text uses a style-only fallback over its measured text footprint;
the host must draw that text before applying the decorator. Symbols and protected
wide footprints remain intact. Unsupported text never grants permission to style
neighboring host labels in a wider requested rectangle.

Clipping retains authored coordinates. Protected regions preserve selection,
caret, title and media. Frame advice uses the finite terminal boundary near the end,
then becomes idle after the final frame. A host retains its entity+generation key,
so cancelled/replaced operations cannot attach old visual state to new work.

Independent tests cover end−1/end/MAX, shuffled sampling, controls/length limits,
terminal readability, Unicode fallback boundaries and protected cells. Captures
come from the Rust pipeline; current review and unqualified routes are separate.
No general grapheme-rewriting, completion callback, or terminal-font guarantee.

Provenance and MIT licensing: [project record](../../PROVENANCE.md).
