# Ashbrook asset provenance

Imported for Little Forge from the owner's `little-forge-assets.zip` (2026-10-15).
The source contains artwork generated for this project in the accompanying ChatGPT conversation;
it is not a third-party asset pack. This records provenance, not a separate legal certification.

- `base.png`: byte-for-byte `map/Ashbrook Village Parchment Map.png`, 1122 × 1402.
- Seven color references: `map/ChatGPT Image Oct 15, 2026, 12_54_{20,30,36,40,49,58} PM.png`
  and `12_55_25 PM.png`. These full-map illustrations are references, not runtime layers.
- `overlays/*.png`: normalized region masks traced against the base. Extracted chroma washes only;
  original reference ink/paper is never composited. Color alpha is capped at 35%, zero RGB at zero
  alpha; feathered edges. The canonical base remains the only source of geography and ink.
- The bundle has no independent southern-exit color reference. `outside.png` uses a 13% feathered
  earth wash over the exit road, sampling the established parchment/earth palette. It adds no
  building. All eight regions use the same base coordinate system.
- `preview/all-highlighted.png`: base plus all eight layers; review-only, never used for picking.
- `locations.json`: normalized image coordinates, stable domain IDs, labels and overlay paths.
  The central public well is not selectable. No arbitrary destination brand colors.

Vendor sprites under `assets/village/` are alpha-bounds crops scaled proportionally from:
carpenter `npc/...10_54_02 AM.png`, leatherworker `...10_54_10 AM.png`, farmer
`...10_42_02 AM.png`, tavern `...10_46_54 AM.png`. Art replacement has no domain effect.
Checksums and dimensions are recorded in `assets/manifest.json`.
