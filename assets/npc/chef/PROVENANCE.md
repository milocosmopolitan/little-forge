# Chef basic pose

Current scope: eight expressions and nod/shake-head/give pose clips have been added. See `ANIMATION.md`, `animation.json`, `poses/recipes.json` and `atlas-manifest.json`. The original neutral PNGs and layers are preserved; `atlas.png` now contains the 17-frame animation export. The basic-pose notes below document the foundation and its earlier corrections.

Latest correction: talking mouths moved up 2 native pixels; surprise mouth moved up 1 pixel with reference-derived eyes; happy now uses an open laugh. Removed the invented neck skin patch, repaired the far collar on the left turn, and restored the presenting-arm upper outline and sloping shoulder join. See `previews/revision-faces-4x.png`, `previews/revision-joints-4x.png` and `qa/revision-checks.json`.

Reference: `source/reference.png`, copied from the user-provided `ChatGPT Image Oct 15, 2026, 10_47_02 AM.png`. The reference defines the bald head, brown beard, stocky proportions, outfit and spoon pose. This is a new native interpretation, not a pixel-identical reduction.

Generated with built-in image_gen. Exact prompts: `source/generation-prompts.json`. Retained candidate: `source/generated-candidate.png`. Both generation and background extraction retained background glow. Explicit native silhouette masks remove it during export. Hidden shirt, trousers and hands are reconstructed, not directly observed in the reference.

## Files

- `neutral.png`: equipped basic standing pose, RGBA 96×128.
- `base-neutral.png`: complete shirt/trousers/bare-hands underlayer with neutral face.
- `head-base.png`, `body/`, `eyes/open.png`, `mouth/rest.png`: registered base layers.
- `equipment/`: scarf, straps, apron, belt/tools, towel and spoon.
- `atlas.png`: the same single frame. No animation states are implied.
- `previews/`: exact nearest previews, face close-up, light/dark layer checks and equal-height reference comparison.
- `character-spec.yaml`: JSON-compatible YAML with coordinates, order and scope.

## Rebuild and verify

Run from the repository root:

```sh
rtk proxy python3 assets/npc/chef/source/build-character.py
rtk proxy python3 .agents/skills/pixel-art-gen/scripts/validate_pixel_asset.py assets/npc/chef/neutral.png --grid B --cells 6 8 --max-colors 32 --preview assets/npc/chef/previews/neutral-4x.png --scale 4
```

Pillow is required. Build inputs are the retained candidate, reference and native masks/palette in the builder. The builder regenerates layers, composites and previews and fails on base coverage loss or nonmatching recomposition. `source/coverage-check.json` records actual color count and pixel hashes. The original generated alpha is not suitable for export; native layers use binary alpha.

Visual review covers face, full body, equipment-off state and light/dark backgrounds. The reference has no hat, so hat toggling is inapplicable. One neutral expression exists, so cross-expression invariance is inapplicable. No runtime/catalog/manifest integration or animation was requested or performed.

## Outline revision

The user identified rough spoon/hand edges, a stepped scalp, an interrupted towel hem and scarf contours. These four regions now use local native pixel construction in the builder, with complete equipment outlines and a rounded scalp arc. This revision edits the existing code-authored native shapes. The hand remains complete when the spoon is hidden; visible shaft sections do not paint over the fist. No blur, antialiasing or extra palette colors beyond the original 32-color contract are used.

Previous images are retained in `source/review-before/`. `previews/outline-before-after.png` compares all four regions. The builder asserts that the final composite outside those review regions remains pixel-identical, that base pixels are retained, and that the pre-split base is restored by recomposition. All 14 native layers/composites passed structural validation; all PNG outputs were identical on repeat rebuild. The revised layers use 32 opaque colors in total.

## Reference and raised-arm correction

The parallel scalp highlight read as a second head outline, so it was replaced by a compact light patch. The earlier hand-authored spoon had changed the reference bowl shape, and the generated boots had drifted toward red. Spoon and boots now sample `source/reference.png` at the same uniform body scale, 118/1342, with native integer registration to the hand and leg sockets. Explicit masks preserve their contours. A restricted subset of the shared palette keeps the reference leather brown rather than red. Trousers continue beneath the boot cuffs without transparent gaps.

The raised forearm and fist now share one native contour, eliminating the internal dark seam. The builder checks zero dark outline pixels in the strict forearm interior. `previews/reference-details-4x.png` compares the spoon and boots against the reference. `source/reference-repair-before/` preserves the state before this correction. Comparison against that state confirms the final composite changed only within scalp, spoon/arm and boot regions. The outline review now includes boots as a fifth region. Repeated build, 14 native structural checks, exact nearest preview and zero base-pixel loss all passed again.

## Grip detail

The seam cleanup made the fist too flat. Local native skin clusters now distinguish the thumb crossing the curled fingers, three short finger creases and knuckle highlights. Only fist RGB pixels changed; the composite alpha silhouette is unchanged. No dark forearm seam was reintroduced. The saved state in `source/hand-detail-before/` feeds the reproducible `previews/hand-detail-before-after.png`. All 14 structural checks and the deterministic rebuild passed again.

## Reference grip rebuild, replaces the preceding grip detail

The user rejected the added finger bands. That approach preserved the wrong mitten silhouette and put the thumb on the wrong side of the grip. The current upper layer instead samples the original reference forearm, palm and folded-finger color clusters at the common character scale. Explicit native masks place the thumb opposite the fingers around the handle. A small native skin fill keeps the narrow thumb readable inside its outline. The spoon moved two native pixels right to share the hand's reference registration; its bowl geometry and scale are unchanged.

`source/grip-rebuild-before/` retains the rejected version. `previews/grip-reference-check.png` is regenerated by the builder and shows reference, rejected version and current grip at the same body scale. Both the bare underlayer and equipped composite have a single connected silhouette. Visual review covered that comparison, the complete 4x character and the light/dark equipment toggle. All 14 native validations, repeat-build hashes and zero base coverage loss passed; composite changes are confined to the grip and prop region. Earlier grip-detail notes describe superseded versions, not the current build.

## Complete empty-hand underlayer, replaces the reference grip rebuild

The reference grip was still broken when the spoon was hidden: equipment occlusion had been mistaken for bare anatomy. The current arm is traced from the complete equipment-free panel of `source/generated-candidate.png`, preserving one palm/wrist/forearm silhouette. A local subset of the shared skin palette removes accidental red cloth colors from the skin. The hand occludes the spoon shaft, while the spoon cannot erase base pixels. Reference spoon-bowl geometry and the other character parts remain unchanged.

`source/bare-hand-repair-before/` preserves the rejected state. The builder regenerates `previews/empty-hand-toggle-check.png`, showing old/current empty hands and old/current equipped hands on light and dark backgrounds. Visual review covered this toggle, the full underlayer and the full equipped pose. Both current sprites have one connected silhouette. All 14 native structural checks, exact nearest preview, zero base coverage loss and repeat-build hashes passed. Pixel changes from the preceding version are confined to the hand/forearm region. The preceding reference-grip notes are historical and superseded.

## Complete spoon and foreground fingers

The user identified that the prop PNG itself had a hand-shaped cutout. The current `equipment/spoon.png` is a complete connected spoon, with wood reconstructed along the previously occluded shaft. It is no longer cut against the hand mask. `body/hand-front-spoon.png` copies only the overlapping finger pixels from the existing upper layer. Composition is upper/base hand, complete spoon, then foreground fingers. The foreground layer is enabled only with the spoon; the bare underlayer is pixel-identical to the preceding version.

`previews/spoon-layer-order.png` shows the complete prop, bare hand, prop over hand and final finger overlap on light/dark backgrounds. `source/spoon-order-before/` retains the previous state. Checks confirm one connected prop with no missing rows, matching foreground-hand pixels, all 15 native assets, exact nearest preview and deterministic rebuild. `character-spec.yaml` records the conditional equipment recipes and foreground layer. This replaces the earlier masked-shaft approach.

## Grip-only shaft cutout, current composition

The user clarified that the shaft must be hidden where the hand actually grips it. `equipment/spoon.png` now has a cutout only behind the curled fingers, excluding the lower forearm. The bare-hand underlayer is unchanged. Active composition is the complete bare hand followed by this masked prop. `source/spoon-complete.png` retains the uninterrupted source prop. The optional `body/hand-front-spoon.png` records the equivalent occluder but is not applied in the active masked-prop recipe.

The builder asserts that masked prop over bare hand equals complete prop plus foreground fingers, pixel for pixel. Removed shaft pixels all lie behind opaque hand pixels, and all other prop pixels remain unchanged. `previews/spoon-layer-order.png` now compares masked prop, empty hand, unmasked overlap and final grip on both backgrounds. The preceding complete-prop rendering recipe is superseded. All 16 native assets and repeat-build hashes passed.

## Happy eye reference correction

The user supplied a closed-eye laugh reference at 5:30 PM. Happy eyes now use shallow upper arcs at native y24–25 (near eye) and y23–24 (far eye), with light lower creases. The laugh mouth moves (-1,-1) from the immediately preceding export, under the moustache. The prior happy frame and components are retained in `source/animation/happy-revision-before/`. Other expressions, poses and the neutral foundation retain their preceding construction.

## Fixed clothing during head shake

Removed the left-turn shirt/collar repaint. Both turn keys now reuse the neutral upper/lower and equipment PNG pixels exactly. The neck is a separate layer filling only transparent pixels between jaw and original upper layer. `source/verify-animation.py` checks clothing/gear equality and zero neck overlap with the upper layer. Previous left-turn frames are in `source/animation/shake-clothes-before/`.
