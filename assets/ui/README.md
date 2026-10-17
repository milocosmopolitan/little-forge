# Little Forge UI art preparation

Epic #58; asset tasks #59–#61. These files prepare the requested visual direction. **Source sheets are not validated runtime atlases.** The draft integration samples them using explicit UV rectangles to review real Bevy composition; that does not pass the art-first acceptance gate.

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

## Draft integration rectangles (source pixels)

These are temporary source crops, not a native atlas contract. Rectangles use
exclusive right/bottom bounds. There is no atlas extrusion. All live labels,
counts, focus rings and progress fills remain Bevy nodes.

| Component | Rectangle | Slice insets L/T/R/B |
| --- | --- | --- |
| Window | 16,80 → 1024,454 | 52/86/52/36; corner scale 0.35 |
| Inset | 1174,734 → 1400,838 | 14 each |
| Button states | x=16/322/628/934/1240, y=500; 280×78 | 16 each; corner scale 0.5 |
| Icons | 6×4 grid of 256×256 crops, named order above | none |
| Header | x=0..1250; y=0/226/446/666; height=200 | none; fill header region |

Button order: normal, hover, pressed, disabled, selected. Focus is an independent
cream border. Temporary noninteger scaling is **not** pixel-grid acceptance.
The native export contract above remains the target and must replace these crops.

## Font provenance

`fonts/NotoSansKR.ttf` is the variable Noto Sans KR font from
https://github.com/google/fonts/blob/main/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf
with adjacent `OFL.txt` (SIL Open Font License 1.1). No system font is required.
`python scripts/validate-ui.py` verifies source hashes and font glyph coverage;
`--release` rejects candidate art rather than reporting it as finished.
