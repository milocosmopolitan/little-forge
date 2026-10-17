# Tavern madam animation assets

Open `previews/animation.html` for normal/slow playback, key stepping, independent eye/mouth combinations, base/equipment, light/dark and 1x/4x views.

All 23 frame and pose canvases are 128x160 with foot pivot (64,140). The approved 96x128 authoring layers are embedded once at (16,16); the fourteen approved neutral images remain pixel-identical. 8 expressions, nod, shake_head, give and six-key walk are complete. The 64-color shared palette uses binary alpha and nearest sampling.

`animation.json` owns clip timing, repeat policy and the one `transfer` event on give key 3. Give ends empty-handed. Preview replay restarts the demonstration with the original tankard; this does not create an inventory item. Walk frees both arms and omits the tankard and wrist towel. Anatomical near side is right; the large boot buckle belongs to that leg as it crosses the screen. All frame layers, sockets and occlusion orders are in `poses/recipes.json`.

Build: `python3 source/build-animation.py`. The neutral builder also regenerates animations automatically after the baseline. Check: `python3 source/verify-character.py` then `python3 source/verify-animation.py`. Generated candidates, exact prompts, registration, baseline hashes and gait phases are retained in `source/animation/`.

`atlas-manifest.json` is an authoring manifest. No game catalog or runtime asset has been replaced, and game runtime performance has not been measured.
