# Tavern madam neutral asset

Scope: one neutral pose, open eyes and resting smile, a complete equipment-free base and seven removable equipment layers. Authoring parts are 96×128; published frame and pose layers are 128×160 at offset `(16,16)`, foot anchor `(64,140)`. No gesture/walk clips or game catalog replacement are included.

The supplied original is retained as `source/reference.png`. Built-in image_gen supplied a background-extraction candidate, an equipment-free candidate, a corrected complete underlayer, and one isolated raised-arm correction. Exact prompts, input roles and output paths are in `source/generation-prompts.json`. The first bare candidate is retained as superseded, not used by the builder.

The cutout supplies removable reference equipment and a bounded facial-interior graft restoring the original eyes, nose and smile. The corrected complete underlayer supplies the complete outer hair and ear, blouse, bodice, skirt, boots and hip hand. The generated raised arm replaces only the recorded local crop so the empty grip aligns with the tankard handle. The recorded facial-interior polygon matches the registered cutout pixels except the restored single beauty mark `(42,34)`, darkened after review. A one-pixel outline row `(67..70,45)` closes the top of the empty grip. Complete head silhouette and body are reconstructed, not claimed pixel-identical to the supplied high-resolution reference. Hidden garment/hair/hand pixels are reconstructed artwork.

`source/registration.json` records source sampling and arm registration. The palette is shared, at most 64 opaque RGB colors with binary alpha; three greens retain the small bottle. Eyes and mouth are extracted at their native positions with skin beneath. Hair curls remain with the head. Equipment masks exclude copied skin/cloth where material ownership requires it. The tankard front contains finger gaps so the bare hand remains visible at the handle.

Rebuild from the repository root with Pillow and PyYAML installed:

```sh
python3 assets/npc/tavern-madam/source/build-character.py
python3 assets/npc/tavern-madam/source/verify-character.py
```

Open `previews/index.html` for equipment toggles, 1×/4× display and light/dark backgrounds. `neutral-comparison.png`, `reference-comparison.png` and their dark variants compare the actual native output. Face and both hands also have nearest 8× crops. QA reports distinguish structural/pixel checks from visual inspection. The packed atlas is an authoring manifest, not the Bevy Catalog schema; runtime performance and integration are unmeasured.
