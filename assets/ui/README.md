# Little Forge UI art preparation

Epic #58; asset tasks #59–#61. These files prepare the requested visual direction. **Source sheets are not runtime atlases.** The game loads the independent exports in `runtime/` (see below); visual approval is still pending.

## Sources

| File | Contents | Status |
|---|---|---|
| `source/controls-v1.png` | Wood/iron frames, button variants, tabs, inventory slots, progress bars, tooltip and scrollbar | Candidate; needs isolated exports and slice validation |
| `source/icons-v1.png` | 24 navigation, utility and current-game item icons | Candidate; needs alpha cleanup assessment, registration and small-size checks |
| `source/merchant-headers-v1.png` | Carpenter, leatherworker, farm and tavern backdrops, top to bottom | Candidate; needs individual exports and NPC composition checks |
| `references/forge-management.png` | Owner-supplied management visual target | Visual reference, not a menu/gameplay specification |
| `references/merchant-panels.png` | Owner-supplied shop visual target | Visual reference, not a catalog/gameplay specification |

Generated with built-in imagegen on 2026-10-16 using the supplied references. No third-party asset pack was introduced. Hashes, dimensions and image modes are in `source-manifest.json`. Owner reference rights are not independently assessed. The generator returned RGBA for controls/icons, but alpha presence alone does not establish clean transparency: their alpha maximum is 254 and compositing/edge tests are still required.

## Runtime export contract (proposed targets)

| Group | Logical target | Required metadata |
|---|---|---|
| Window/inset/tooltip | Nine-slice; final source dimensions follow export | Per-edge integer insets, minimum size, opaque center |
| Button | 160 × 40, all states identical | normal/hover/pressed/disabled/selected; independent focus ring |
| Tab | 144 × 40 | normal/selected; focus indication |
| Item slot | 64 × 64 | normal/selected/locked; content safe rect |
| Navigation icons | 32 × 32 | common baseline and transparent padding |
| Item icons | 48 × 48 inside 64 × 64 slot | common visual scale, no baked counts |
| Merchant header | 640 × 160 | crop/safe area; live title left, existing NPC separate |
| Progress track/fill | 256 × 16 | fixed caps, stretch-safe middle, clip-driven fill |

These are target export sizes, not a claim that source-sheet components already have those sizes. Never infer atlas rectangles from equal division: generated spacing varies. Do not stretch a whole source sheet into a window. Export to `assets/ui/runtime/` only after inspection, with a separate validated runtime manifest and 2px atlas extrusion if packed. Use nearest sampling and integer scaling for UI art; retain map's existing sampling policy.

## Icon order

Six columns, four rows: craft, materials, provisions, map, commissions, research; codex, settings, close, back, plus, minus; coin, lock, warning, ore, dagger, timber; leather, staples, meal, water, rest, queue. Provisions and staples deliberately share a grain motif. A research/commission icon is not proof its gameplay exists.

## Remaining asset gate

- Isolate assets, normalize state dimensions, remove unwanted fringe and inspect on light/dark checker backgrounds.
- Validate nine-slice seams at small/default/large sizes and preserve thin edges.
- Finish focus ring, separators and any missing hover/pressed variants consistently.
- Export four header images; inspect existing NPC overlays at actual game size. Do not regenerate NPC identities.
- Bundled Noto Sans KR under SIL OFL 1.1; ASCII and all 11,172 precomposed Hangul syllables pass glyph coverage. Long-label captures use the actual font.
- Validate icons at 32/48px; simplify if needed rather than blindly shrinking high-detail art.
- Record export hashes, alpha bounds, insets, provenance and preview evidence before marking #60/#61 complete.

## Product boundaries

Opaque charcoal/brown panel interiors; transparent desktop outside. Cream text is rendered by Bevy, never embedded in textures. Selected controls use muted blue. Existing map and farmer animation assets are reused. Farm offers provisions/basic materials, not seed/livestock gameplay. Home is forge management. Price, quantity, progress and stock come from Rust state.

## Generation briefs

1. Controls: reference-matching text-free source sheet; thin wood/iron framing, opaque dark interiors, five wide button states, tabs, slots, progress tracks/fills, tooltip and scrollbar; isolated transparent gutters; orthographic pixel art.
2. Icons: exactly 24 isolated icons in the order above; cream navigation glyphs, colored current-game items, dark pixel outlines, transparent background; no labels or panel art.
3. Headers: four stacked wide text-free and character-free workshop/farm/tavern backdrops; dark left title area and right foreground for separately composited NPCs; consistent warm pixel-art palette.

The exact generated files are retained unmodified. Generation requested transparent controls/icons; final export QA must verify compositing rather than trust the request.

## Runtime exports (#59–#61)

`python scripts/export-ui-assets.py` regenerates `assets/ui/runtime/` from the
source sheets; it never edits the sources. Components come from alpha
connectivity (`scripts/ui_components.py`), not equal division. Alpha is
binarised at 96 to drop generator halos, state variants share one canvas, icons
sit centred on a 256×256 canvas, and every file is registered with size, alpha
bounds, source box and slice insets in `runtime/manifest.json`.

| Group | Files | Slice insets L/T/R/B |
| --- | --- | --- |
| Window | `controls/window.png` (1011×378) | 53/88/54/38, corner scale 0.35 |
| Modal / tooltip frame | `controls/modal.png` | 40/60/40/40 (not yet used) |
| Inset panel | `controls/inset.png` | 18 each |
| Buttons | `button-normal/hover/pressed/disabled/selected` (288×85) | 20 each, corner scale 0.5 |
| Tabs, slots | `tab-*`, `slot-*` | exported, not yet used by components |
| Progress, scrollbar | `progress-track(-alt)`, `progress-fill-amber/blue`, `scrollbar-*` | none |
| Icons | `icons/<name>.png`, 24 files | none |
| Headers | `headers/{carpenter,leatherworker,farm,tavern}.png` (1250×200) | none |

`scripts/review-ui-assets.py` renders every export on light and dark
checkerboards (`docs/ui/evidence/runtime-assets-review.png`); no halos or seams
were seen at that size. `scripts/validate-ui.py` checks hashes, hard alpha,
identical state dimensions, icon names against `forge-ui::ICONS` and that every
`ui/runtime/...` path referenced by `forge-ui` is registered.

Known limits: sources are generated upscaled art, not native-grid pixel art, so
exports are not integer-scale clean; focus ring and separators have no art (Bevy
borders); tab/slot/modal/scrollbar/progress exports are registered but components
still draw slots and bars with Bevy nodes. `runtime_ready` stays `false` until
the owner approves native-size review.

## Font provenance

`fonts/NotoSansKR.ttf` is the variable Noto Sans KR font from
https://github.com/google/fonts/blob/main/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf
with adjacent `OFL.txt` (SIL Open Font License 1.1). No system font is required.
`python scripts/validate-ui.py` verifies source hashes and font glyph coverage;
`--release` rejects candidate art rather than reporting it as finished.
