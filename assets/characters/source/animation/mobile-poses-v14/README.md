# Walk and give repair, v14

Historical rejected procedure. The current compositor now imports `../collar-v15/collar.mjs`; see that revision for the source-owned neck replacement. The 3px offset and fabricated connector described below have been removed.

Scope: female/male `walk-0` through `walk-5`, `give-half`, `give-full`, and `give-release`. Neutral endpoints, expressions, research and smithing actions are retained. Canvas is 128×160; foot pivot is (64,140).

`motion.mjs` assembles the existing generated walk/opposite/gesture underlayers with the approved root head and features. It registers the body by the trouser waist, retains complete shirt/arm silhouettes, removes generated ear/jaw fragments, and seats the approved head three native rows below the fabric-near-skin centroid. The short neck belongs to the body. Head ornaments share the head translation.

Apron and belt layers follow the shirt and waist anchors. Upper apron edges stay inside the source body's silhouette. Source forearms render in front of the apron; gloves recolor the original hand pixels with the existing leather palette. No neutral glove is stretched over a moving hand. Give uses the approved neutral legs and boots.

Each pose has separate body/head/eye/mouth layers and a separately reconstructable equipped variant. Equipment layers are pose-specific; these are not freely interchangeable with neutral gear. `source_partition_roundtrip` is false because generated heads are intentionally replaced and give legs are retained from neutral.

Source art: existing `../{female,male}-{walk,walk-opposite,gestures}.png`, their generation records in `../prompts.json`, and the approved root character layers. No new image generation was performed for this repair. `before/` preserves the starting pose folders; `retained-frames.json` protects the 204 other frame PNGs.

Build: `node assets/characters/source/animation/build.mjs`.

Checks: `python3 assets/characters/qa/verify-mobile-poses.py`, `python3 assets/characters/qa/verify-animation.py`, and `node assets/characters/qa/check-rebuild.mjs`. `review-mobile-poses.py` makes read-only before/after and light/dark inspection sheets.

Static visual inspection and structural checks do not establish playback or user acceptance. Browser inspection was unavailable in this run: the native tool returned `Sky Computer Use native pipe startup failed`, and the browser inventory was empty. Runtime integration and GPU performance were not measured.
