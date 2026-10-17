# Leatherworker modular baseline, revision 2

The user supplied `/Users/minhokang/Work/games/little-forge-assets/npc/ChatGPT Image Oct 15, 2026, 10_54_10 AM.png`. Its unchanged copy is `source/reference.png`. No farmer/chef body or face is reused.

## Current outputs

`neutral.png` is the equipped reference pose. `neutral-no-prop.png` hides the leather sheet. `base-neutral.png` removes occupational gear. Parts share a 96×128 RGBA canvas, binary alpha and a palette of at most 80 colors. Nearest previews are in `previews/`.

The head, eyes/brows, mouth, upper body, lower body, hairpin, scarf, apron, belt/tools, gloves and leather hide remain separate. The raised empty hand retains the gripping pose. Existing NPC runtime/catalog configuration is unchanged.

## Corrections requested by the user

Revision 1 clipped material colors and object edges with approximate polygons and color filters. The saved original outputs and builder are in `source/revision-1-before/`. Those polygon/filter operations are removed from the current garment construction.

Complete generated apron, scarf and belt/tool sources are registered to reference landmarks before native sampling. This preserves leather surfaces, gold buckles, stitching, pouches and scissors without cutting their highlights into holes. The side roll/pouches sit below the hand. Apron strap/buckle alignment and skirt length use separate vertical guides. `source/corrections/registration.json` records the mappings. The old `source/ownership-regions.json` is historical revision-1 evidence, not the active garment mask.

Built-in image_gen edited a new closed folded leather sheet. Its top pinch edge, solid fold interiors and dark contour are preserved. Source and exact prompt are in `source/corrections/`. It renders in front of the lower grip; knuckles above the sheet remain visible. The final image must exactly equal the sheet at every opaque sheet pixel, so fingers cannot show through the fold. The empty-hand body itself was not edited.

Seven original cheek-pigment pixels restore the blush, with their mask in `source/corrections/blush-mask.png`. Head alpha, hair, remaining face pixels, eyes and mouth retain the accepted revision-1 baseline. Upper/lower body, bare hands, eyes and mouth are checked byte-for-byte against that baseline.

The original 64 palette entries remain available. Sixteen extra shared slots preserve original blush, scarf reds, leather/gold/metal details while keeping the accepted body pixels unchanged. Maximum is now 80 across all layers combined. The scarf maps to reference reds and dark contours, avoiding hair-brown highlights. This is a native reconstruction rather than lossless equivalence to the high-resolution reference.

## Sources and reconstructed regions

The original image_gen prompt ledger is `source/generation-prompts.json`; correction prompts are `source/corrections/prompts.json`. Full-figure source registration samples nominal source center `(95+10x,115+10y)` at native coordinate `(x,y)`. Isolated items have explicit registration maps rather than assuming matching canvas size means matching object scale.

The complete equipment-free foundation restores the hidden shirt, waist, trouser hips, wrists/hands and hair below the clip. These occluded shapes are inferred. The accepted complete native foundation is rebuilt from the archived head/upper/lower baseline, with only the explicit blush correction. It is never reconstructed from equipped body fragments. Restored body contours and hand placement have small differences from the supplied reference.

## Review and checks

`previews/reference-base-equipped.png` shows reference, empty base, current equipped character from left to right. `previews/correction-before-after.png` shows revision 1 and current output. Face close-ups and individual layers on light/dark backgrounds are also saved.

```sh
rtk proxy node assets/npc/leatherworker/source/build-character.mjs
rtk proxy python3 assets/npc/leatherworker/qa/verify-character.py
```

The verifier checks dimensions, RGBA, binary alpha, the shared palette, all 64 gear toggles, foundation reconstruction and lost pixels, unchanged body/hands/eyes/mouth, face changes confined to the blush mask, no enclosed transparent hole in the hide, correct sheet/hand occlusion, exact nearest previews and byte-identical PNG rebuilds. Reports are in `qa/`. These structural checks are separate from visual review.

The neutral modular baseline is now extended with expressions, gestures, six-key walking and a composed authoring atlas. See [ANIMATION.md](ANIMATION.md) for current scope, build commands, playback and validation. Runtime catalog integration remains outside this asset-authoring pass.
