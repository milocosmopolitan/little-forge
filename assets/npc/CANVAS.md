# NPC animation canvas

Farmer, leatherworker, chef and carpenter publish animation frames and pose layers on **128×160** native pixels (Grid B, 8×10 cells), with foot anchor **(64,140)**. Character pixel scale stays unchanged.

The 96×128 base originals and expression parts remain in their authoring coordinates. Animation export embeds them at **(16,16)**. Absolute sockets and atlas trim offsets receive the same shift; movement vectors and clip timings stay unchanged. Trimmed atlas pages may retain their original dimensions because the added area is transparent.

Farmer and leatherworker builders invoke `scripts/expand-npc-canvas.py`; chef and carpenter already compose on the expanded canvas. Rebuild farmer with `node assets/npc/farmer/source/build-all.mjs` and leatherworker with `node assets/npc/leatherworker/source/build-all.mjs`. The farmer full builder also refreshes `runtime-atlas.png`, `character.json` and the asset inventory. `python3 scripts/export-farmer-runtime.py --check` checks that runtime output is current.

Farmer runtime catalog cells are 128×160. Runtime head/hand/foot coordinates move together, and desktop/preview render sizing preserves the original native pixel scale. Animation HTML and 4× GIFs show 512×640 frames. Source comparison crops remain at their original authoring coordinates.
