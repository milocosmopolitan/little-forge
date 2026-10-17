# Female research material ownership, v13

The v12 white-plate check passed while leather strap ends were missing. It checked opacity and brightness, not garment identity. The user rejected that result.

Scope: female equipped research keys only. Restore measured leather source strips, including dark seams and buckles, as an explicit garment layer. Continue the top of each strip with its first complete native source row. Do not classify leather as white shirt or extend skin over it.

Reposition each source collar around the existing short body-owned neck, keeping both head and torso fixed. Neck pigments come from the approved neutral head's original neck. The thinking hand stays in front of the collar. Exclude the original source face edge from the transferred collar so an old jaw outline cannot become a new neck mark. Remove six known old-source hair pixels at the thinking shoulder; they are not a moving-head outline.

`materials.mjs` contains native ownership guides. Diagnostics are `rendered/body/restored-straps.png`, `restored-collar.png`, and `removed-seam.png`. The source garment check permits only these explicit female equipped research relocations and the existing male cloth removal. `verify-research-material.py` independently checks leather continuity from shoulder to buckle, exact source pixels, approved neck pigments, rejected v12 gap rows, unchanged face layers, and unrelated frames.

No new generated artwork or facial redraw is used. The repair recomposes the existing generated native source pixels through the established JavaScript raster builder. Canvas 128×160, pivot (64,140), palette and sampling remain unchanged.

Build with `node assets/characters/source/animation/build.mjs`, then `node assets/characters/source/animation/scenes.mjs`. The prototype stays isolated until inspection. Structural checks do not establish visual acceptance.
