# Source collar repair, v15

The rejected v14 compositor measured a mean of skin near the shirt, fabricated an 8×6 skin rectangle and compensated with a global 3px head shift. The inferred anchor could lie above the shirt, producing a narrow detached stem; gesture anchors moved with face/skin clustering.

`collar.mjs` replaces that path for all 18 female/male walk and give poses. Each source's actual jaw exit and last exposed collar row are traced explicitly. Between those rows, native skin runs and their original contour are retained directly from the generated underlayer. Only shirt components reaching the chest seed upper garment coverage, so pale ear/headband fragments cannot become shoulders. There is no hull, nearest-color skin fill, or global head offset.

The approved head owns hair, ears, face and jaw only. Root neck exit coordinates are female `(60.5,63)` and male `(55,62)`; the half-pixel x is a measurement of the span center, and the resulting translation is rounded once to a native integer. Head, eyes, mouth and ornament share that translation. Neck and shirt belong to the body and retain original source colors.

The current compositor entry remains `../mobile-poses-v14/motion.mjs`; it imports this collar assembler. Registration, original hand contours and planted give feet retain the previous repair. `before/` records the rejected v14 pose folders; `retained-frames.json` protects every other frame PNG.

Build: `node assets/characters/source/animation/build.mjs`.

Validation: `python3 assets/characters/qa/verify-source-collar.py` checks source color/coverage, jaw-exit binding and negative v14 fixtures. Existing animation checks cover approved identity, recomposition, native grid and atlas. `review-source-collar.py` makes separate body/head/composite close-ups on cyan, dark and light backgrounds.

Source art is the existing generated walk/opposite/gesture sheets and approved character layers; no new generation or rescaling was performed. Structural checks do not establish visual acceptance. Browser playback could not be inspected because the native tool returned `Sky Computer Use native pipe startup failed`.
