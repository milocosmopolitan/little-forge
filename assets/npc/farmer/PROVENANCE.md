# Farmer assets

Original: user-supplied `ChatGPT Image Oct 15, 2026, 10_42_02 AM.png`, 1122×1402 RGBA, copied byte-for-byte to `source/reference.png`.

Generated with the built-in image generation tool, using that image as identity/style reference. No external art pack was downloaded. User supplied the visual reference for this project; this records provenance rather than making an independent copyright/licensing determination.

## Prompt set

1. **Expression/gesture sheet** (`source/expressions.png`): preserve straw hat/leaves, gray beard, cream shirt, olive apron, brown gloves/boots and pixel-art proportions. Exactly 4×4 transparent full-body cells: neutral, three speaking mouths; half/closed eyes and two surprises; two nods and two real yaw head turns; prepare/extend/hold/retract empty-handed offer. Same scale/ground alignment, no carrots or text.
2. **Walk sheet** (`source/walk.png`): six right-facing three-quarter walking poses in 3×2, alternating contact/recoil/passing, same appearance, empty hands, no labels. Follow-up extraction request removed dark backdrop/glow while retaining characters. Zero-alpha pixels retain source RGB; runtime alpha compositing must honor PNG alpha.
3. **Carrot item** (`carrot.png`): isolate the original bunch of three carrots/leaves, no farmer/gloves, matching pixel-art, genuine transparent background.

Runtime export is mechanically packed by `scripts/pack-npc.py`. Source expressions are not uniformly gridded despite the generation request; explicit source rectangles and connected-component isolation remove neighboring-cell fragments. Registration uses boot bounds. One shared scale per sheet: expressions 1.0; walking 0.625. No per-frame resizing. Resulting 384×352 cells preserve motion margins and source pixel ratios.

`head.png`, `body.png`, `eyes.png`, `mouth.png` are registered neutral authoring layers. `mouths.png` and `eyes-atlas.png` contain independently composited dialogue patches. `hands.png` restores foreground grip pixels over held items. The original full-body source remains available for future redrawing; these are PNG sources, not a claim of a native Aseprite project.
