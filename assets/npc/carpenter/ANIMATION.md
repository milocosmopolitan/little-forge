# Carpenter expressions and pose animations

Latest supporting-hand revision: 115 base pixels and 49 equipped pixels changed inside native `(24,66)..(37,82)`. A localized generated fist replaces the long hanging fingers; the elbow/board gap and glove edge are completed. The exact local deltas and preceding assets are retained in `source/animation/support-hand-migration.json` and `support-hand-review-before/`. Compare `previews/support-hand-review.png` and its dark variant. Structural checks cover all 17 frames and 16 GIFs; live playback was not repeated for this revision.

This export adds eight expressions, three pose animations and the supporting idle/blink/talk/hold clips. Seventeen distinct frames use one 128×160 canvas, base offset `(16,16)`, foot anchor `(64,140)`. Walking and game integration are outside this phase.

## Review

Open `previews/animation.html` directly or through a local HTTP server. All frames are embedded, so the HTML also works offline. It shows equipment-free and equipped versions together. Controls select the clip, normal or 0.25× speed, native or exact nearest 4× scale, light/dark background, pause and the next key. The eight expression buttons hold a selected face. Non-looping clips repeat after a 400ms authoring pause; exported clip metadata remains non-looping.

`previews/expressions-4x.png` and `expressions-dark-4x.png` compare all eight expressions. `eye-mouth-combinations.png` shows five eyes × four mouths. `poses-3x.png`, `poses-base-3x.png` and their dark variants compare the actual pose frames. `previews/frames/` contains exact 4× PNGs. Each clip also has a side-by-side GIF and a `-slow.gif` version.

## Expressions

The eye states are `open`, `half`, `closed`, `happy-closed`, `wide`; mouths are `rest`, `small-open`, `wide-open`, `round`. Exact component recipes are in `expressions/recipes.json`. The original mouth already smiles, so happy retains that smile with shallow smiling eyelids. Talking uses three openings below the fixed nose/cheek turn, which is restored in every mouth layer; the beard, head silhouette, body and equipment stay fixed.

Blink occludes the complete original eye interior, including warm white pixels retained in the head layer. Half closure retains the original lower pupil instead of inventing a new eye. Blink uses open → half → closed → half → open, with 40/80/40ms closure phases. Talking uses rest → small-open → wide-open → small-open → round → rest at 90–120ms per key. These are separate authoring channels; surprise locks automatic blink. The combined clips demonstrate them without adding a runtime controller.

## Poses

`nod`: five keys at `[120,100,140,100,120]ms`. The middle transition moves only the movable hair/face and its facial parts together by one integer pixel. The original crop also contained collar pixels; these now remain fixed in a separate neck/collar support layer. Generated neck stumps are removed, and a continuous skin underlay joins the rotated jaw to the existing neck without repainting shirt pixels. The deepest key uses a separately generated downward head perspective at the same native volume.

`shake_head`: five keys at `[100,140,100,140,100]ms`. Left and right are actual alternate perspectives, not mirrored or stretched faces. Generated heads are registered at 41×41 native pixels and have separate feature parts. Original upper/lower clothing and every equipment layer are reused pixel-identically for all head gestures. No shirt/collar repaint is substituted during a turn.

`give`: six keys at `[120,120,180,120,120,120]ms`. The screen-right arm, anatomical left, extends a compact curled hand gripping the original upright mallet. The supporting hand and boards remain fixed. `transfer` occurs once on key 4, zero-based index 3; the mallet is absent thereafter and the complete curled hand remains. The former palm-up hand and horizontal tool are superseded.

The registered sleeves are retained. Only the distal forearm uses the localized candidate in `source/animation/candidates/give-grip-arm.png`; the candidate's fingers are not adopted. The bare gripping fingers, glove design, mallet angle and front-finger gaps are copied from the approved neutral, all at the same translated socket. Mid grip is `(100,91)` and full grip is `(110,86)`. Exact source/crop/adoption are in `source/animation/grip-repair-prompt.json`. Thumb/finger core and mallet pixel retention are verified separately. Only the former sloping wrist edge is removed where it becomes an internal attachment seam in the horizontal forearm.

The head-turn jaws now register at the neutral chin height, two pixels below the previous turn registration. The fixed neck uses the existing neck cross-section instead of the exposed diagonal wedge. The very top collar can be naturally occluded by the turned chin; exposed collar landmarks and all clothing layers stay fixed. Surprise raises the original brows and uses warm eye whites with compact right-looking pupils.

Review links may set `clip`, one-based `key`, `paused=1`, `speed=0.25`, `scale=1`, `background=dark`, or a fixed `frame` such as `surprised`. Default controls are unchanged. Example: `animation.html?clip=give&key=3&paused=1`.

## Sources and rebuild

Built-in image_gen produced three isolated head candidates and two bare arm-pose candidates, followed by localized palm and distal-arm candidates. The palm repair is superseded by the approved neutral grip. Exact prompts are in `source/animation/generation-prompts.json`, retained images in `source/animation/candidates/`. Only the needed head/arm parts are adopted. Their full generated bodies never replace the approved body or outfit. Native registration, shared-palette conversion, binary alpha and localized palm adoption are deterministic in `source/build-animation.py`.

The retained neutral layers after the explicitly recorded forearm-contour correction are in `source/animation/baseline/`, protected by fourteen pixel hashes. Rebuild from the repository root with Pillow installed:

```sh
rtk proxy python3 assets/npc/carpenter/source/build-animation.py
rtk proxy python3 assets/npc/carpenter/source/verify-animation.py
```

The older `source/build-character.py` entry point also builds the complete animation export. `CARPENTER_BASE_ONLY=1` is an internal recursion guard. The base verifier resolves the neutral rect from either a single-frame or expanded animation atlas, so it does not depend on sprite ordering.

## Packing and verification

`atlas.png` is 1056×496 RGBA with seventeen fixed 128×160 rects. Each side reserves 1px extrusion followed by a 1px transparent gutter. No trim or UV rotation is used. Sampling is nearest, with no mipmaps. The shared palette remains 32 opaque colors and all native alpha is binary.

`qa/animation-validation.json` checks neutral hashes, layer recomposition, zero base loss, expression changes confined to actual eye/mouth masks, unchanged clothes/gear in head gestures, exposed collar landmarks, skin joins across neck rows, closed-lid eye-white absence, static nose pixels and inner forearm contours, exact 4× previews, transfer timing, atlas round-trip pixels and extrude/gutter contents. `qa/pixel-validation.json` applies the repository's pixel validator; `qa/rebuild-check.json` checks deterministic exports through both entry points. `qa/playback-review.json` separately records native/4× normal/slow browser review. Structural checks alone do not establish art quality.

The atlas occupies 2,095,104 bytes as an uncompressed RGBA8 base level, excluding CPU copies, staging and driver overhead. Static checks are in `qa/sprite-audit.json`. Runtime draw calls, overdraw and GPU timing are unmeasured. The existing NPC catalog/face contract needs a separate adapter before adoption; no game code, catalog or asset manifest has been changed.

## Hand contour revision, October 17

The neutral grip retained old thumb pixels outside the replacement hand, plus a dark internal wrist edge and a pointed lower contour. The old fragment is now cleared completely, the perimeter uses a continuous native one-pixel line, and the curled-finger shading remains. The supporting wrist's four-pixel horizontal step is replaced by a gradual curve, filling the indentation and removing the isolated elbow-side tooth. The same revised grip mask feeds the give poses. `previews/outline-wrist-review.png` and its dark variant compare the rejected base with this revision.
