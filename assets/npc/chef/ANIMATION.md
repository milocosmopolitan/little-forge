# Chef expressions and poses

Eight expressions and three pose clips are generated from registered layers. The original 96×128 neutral assets remain pixel-identical. Animation frames use a common 128×160 canvas, offset `(16,16)`, foot anchor `(64,140)`. This is an authoring export; no game catalog or runtime has been changed.

## Preview

Open `previews/animation.html` in a browser. It embeds all images and works offline. Choose the clip, 1×/4× pixel scale, normal/0.25× speed and light/dark background. The left panel shows the complete equipment-free underlayer; the right panel shows the equipped frame. The preview repeats non-looping clips after a 400ms review pause; clip metadata retains their non-looping behavior.

`previews/expressions-4x.png` compares the faces. `previews/eye-mouth-combinations.png` shows all 35 eye/mouth combinations. `previews/poses-3x.png` and `previews/poses-base-3x.png` compare equipped and bare poses. Animated `nod.gif`, `shake_head.gif`, `give.gif` and their `-slow.gif` variants show the actual key sequences side by side. `previews/frames/` contains exact nearest 4× PNGs of every frame.

## Expressions

Eyes: `open`, `half`, `closed`, `happy-closed`, `wide`. Mouths: `rest`, `small-open`, `wide-open`, `round`, `round-talk`, `smile`, `laugh`. Happy uses a wide open laugh with teeth and a visible mouth cavity. Talking mouth states are 2 native pixels above the initial export; surprise uses its own rounded mouth, 1 pixel above the initial export. Wide eyes retain the reference pupil width and rightward gaze.

Recipes: `neutral`, `talk-small`, `talk-wide`, `talk-round`, `blink`, `happy`, `surprised`, `relaxed`. Exact parts and combinations are in `expressions/recipes.json`. Brows belong to the eye part. The neutral base is never repainted to make an expression. The permitted change mask consists only of actual eye/brow and mouth pixels.

Blink: open → half → closed → half → open, with 40/80/40ms closure phases. Talk uses rest → small-open → wide-open → small-open → round → rest at 90–120ms per key. These are independent channels; surprise locks automatic blink. The asset clips demonstrate these channels without adding a runtime controller.

## Poses

`nod` has five keys at `[120,100,140,100,120]ms. A one-pixel transition moves the original complete head and facial parts together; the deepest key uses a separately generated downward perspective.

`shake_head` has five keys at `[100,140,100,140,100]ms. The left and right keys use actual alternate head perspectives, not horizontal stretching. Eyes and mouth are split and registered for each new direction. The original clean upper layer remains beneath nod/right-turn heads. Both turning keys reuse the exact neutral shirt and equipment pixels. A separate short neck bridge fills only the transparent gap beneath the left-turned jaw, without painting over the shirt.

`give` has six keys at `[120,120,180,120,120,120]ms: prepare, extend, present, transfer, retract, return. The `transfer` event occurs once on key 4, zero-based index 3. Spoon pixels are absent from that key onward, and the final return is `neutral-empty`. The arm is the screen-right arm, anatomical left in this three-quarter view. The opposite hand remains on the waist. Open hands remain complete with equipment disabled.

The presenting spoon is turned exactly 90 degrees with an integer pixel transpose. It rests on the open palm; occlusion is computed for that pose rather than copied from the standing grip. The former bent arm is removed from a complete torso underlay before the new sleeve, forearm and palm are added. Occluded torso and far-collar reconstruction are recorded in `poses/recipes.json`.

Walking is not generated. There are 17 distinct full-body frames and eight clips including idle, blinking, talking and expression holds. `animation.json` is the timing/event source; `poses/recipes.json` records every actual layer, socket and composition order.

## Build

From the repository root, with Pillow installed:

```sh
rtk proxy python3 assets/npc/chef/source/build-animation.py
rtk proxy python3 assets/npc/chef/source/verify-animation.py
```

The historical `source/build-character.py` entry point also builds the full result, so it cannot leave the old single-frame atlas over the animation atlas. `CHEF_BASE_ONLY=1` is an internal guard for building the foundation without recursive dispatch.

New head and arm candidates were generated with built-in image_gen. Prompts and retained images are in `source/animation/`. They are candidates, not native production sprites. Native registration, fixed masks, shared-palette conversion and transparent edge cleanup are deterministic in `source/build-animation.py`. The original neutral layers are retained in `source/animation/baseline/` and protected by pixel hashes.

## Atlas and verification

`atlas.png` is a 1056×496 RGBA page with 17 fixed 128×160 rects. Each rect reserves one extruded pixel and one transparent gutter pixel on each side. `atlas-manifest.json` records rects, original size, pivot, clips and source hashes. No trimming or UV rotation is used. All native production art shares 32 opaque colors and binary alpha.

`qa/animation-validation.json` records layer recomposition, zero base-pixel loss, expression invariance, baseline locks, the single transfer event, atlas pixel round-trips and explicit extrude/gutter pixel checks. `qa/playback-review.json` distinguishes the current static correction review and normal-speed nod check from the archived earlier playback review. The complete normal/slow replay matrix was not repeated during this correction pass. `qa/sprite-audit.json` records static atlas inspection. The single page needs 2,095,104 bytes for an uncompressed RGBA8 base level, excluding CPU copies and driver overhead. Actual Bevy draw calls, overdraw and GPU timing remain unmeasured.

The repository pins Bevy 0.20.0. Its existing NPC consumer has a different face/clip contract; this export needs an adapter before runtime adoption. No runtime performance or mobile-support claim is made.

## User correction pass

`previews/revision-faces-4x.png` and `previews/revision-joints-4x.png` compare the retained previous export with this revision. The happy opening is confined to its actual mouth mask beneath the moustache; the outer beard and head silhouette remain unchanged. Give-arm upper rims have local native 1px contour repairs; the torso shoulder uses a sloping join instead of the former vertical spike. Mouth sockets follow the expression offsets. Previous frames and eye/mouth parts are retained in `source/animation/revision-before/`.

The subsequent happy-only correction uses low, shallow smiling eyelids instead of pointed arcs. The laugh mouth moves 1 native pixel left and 1 up relative to the preceding export; its socket follows the same offset. `previews/happy-correction-8x.png` compares that correction.

The left-turn collar patch was removed after user review: upper/lower clothing and every equipment layer must now compare pixel-identical to neutral in both head-turn keys. The verifier enforces that rule and zero neck-bridge overlap with the upper layer.
