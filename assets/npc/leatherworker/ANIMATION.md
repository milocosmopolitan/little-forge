# Leatherworker expressions and poses

Open [previews/animation.html](previews/animation.html) for playback controls: clip, gear, 1×/4× native scale, light/dark background, 0.25×/0.5×/1×/2× playback, scrub and single-key stepping. The 0.25× option is playback speed, not fractional pixel scaling.

## Delivered parts

Five eyes: open, half, closed, happy-closed, wide. Four mouths: rest, small-open, wide-open, round. Eight expression recipes are in `expressions/recipes.json`. Expression frames change only eye/brow and mouth pixels; the approved head, copper bob, cheek blush, body and equipment remain fixed. The existing neutral render is reconstructed exactly. `previews/expressions-8.png` shows the selected recipes and `expressions-20.png` reviews all combinations.

Twelve pose recipes supply their own complete equipment-free foundations, head/upper/lower parts, face features, gear, sockets and composition order. The individual layers are in `poses/<id>/`. `poses/recipes.json` links the exact files. Head turns have angle-specific skin-filled head bases and separate eyes/mouth, and currently lock to open/rest. The current eye/mouth library can be translated with the unchanged-angle heads in walk/give keys. Do not paste neutral features onto a rotated face.

## Clips and prop ownership

`animation.json` defines eleven clips: idle, talk, blink, happy, surprised, relaxed, nod, shake_head, give, walk, talk_blink. Times are milliseconds. Blink uses half/closed/half at 40/80/40ms, separate from idle hold durations. Explicit head turns and surprised expressions override automatic face channels.

Nod has five keys: neutral, preparation, pitched-down head, preparation, neutral. Preparation reuses the exact neutral head/eyes/mouth with a 1px downward translation; the down key changes pitch with its own coherent face. Shake_head has five keys with distinct toward/away head perspectives. Give has six keys: neutral holding hide, preparation, extension with hide, extension empty, retraction empty, neutral empty. The fourth key emits `transfer` once. The hide stays gone after transfer. An application that wants the hide back must explicitly restore it. The initial near hand moves from hip toward a supporting gesture; the far hand carries the hide's upper pinch edge.

Walk is six 120ms keys, loops in place, and carries no hide so both arms can swing. Near limbs are anatomical right, far limbs anatomical left. The gold buckle stays on the near right boot. Far-left-foot contact goes with near-right-arm forward; near-right-foot contact goes with near-right-arm backward. The support keys begin recovery and reduce the previous arm extreme; passing keys use lowered transitioning arms. The plan records intended phases, while final recipes record inspected hand/socket positions. No horizontal mirroring swaps the asymmetric boot.

## Authoring and registration

The built-in image_gen sources and exact prompts are preserved in `source/animation-prompts.json`. They are production candidates rather than final sprite sheets. High-resolution candidates are sampled onto the shared 96×128 canvas and mapped to the accepted shared palette of at most 80 colors. Generated character heads are discarded for walk/give; the approved head/eyes/mouth/blush are reused. New gestures come from complete bare-body sources, not a second arm pasted over the neutral pose.

Glove source plates are separate. Their leather pixels are restricted to the corresponding bare hand/wrist silhouette and hand region, preventing background or shirt fragments from entering the glove layer. The generated give-glove plate contains a background despite the transparent request; only its hand colors are consumed through the bare-hand mask. Arm foreground layers provide correct ordering over apron/belt. Every pose has a full body underneath removable equipment.

Soles align at authoring y=124, animation y=140. Shirt, shoulders, arms, head and equipment move by rigid integer translations. Only trouser hip rows 71..82 absorb the difference between upper-body and foot registration; calves and boots retain their source rows. Resampling the whole torso previously duplicated contour pixels, and a neck-width clipping mask erased shoulder edges. Both operations have been removed. The hide follows its hand socket and remains in front of the lower grip.

At the source-head replacement boundary, disconnected source-hair fragments in rows 45..48 are excluded using 8-connected occupancy from the shirt. The connected shoulder contour is retained; this is not a width or brightness cutoff.

The outline correction uses built-in image_gen with its exact request in `source/outline-corrections/prompts.json`. Only the toward-camera jaw/neck join `(41,43)..(54,48)` still consumes that candidate; its eyes use the original unscaled design. Nod and away-facing heads use the coherent identity corrections below. Approved neutral frames and expression recipes are unchanged. Active source ownership is recorded in `source/outline-corrections/feature-registration.json`; previous outputs are archived in `source/outline-before/`.

## Head identity correction

Independent resizing of eyes from a shared head plate changed their shape and expression. The toward-camera pose now restores its original eye component exactly. The away pose was redrawn as one coherent head from the approved neutral head, preserving the copper bob, rounded cheek, calm vertical brown eyes, blush and small mouth. Its generated candidate is registered once onto the native canvas using the measured head size, not fitted independently on playback frames. Registered bounds are `(17,8)..(68,46)`, 51×38 pixels; the neutral head is 51×39. Eyes and mouth are split at their source positions with no individual transforms. A second image_gen edit supplies solid skin under their exact feature masks; only a local dark nose-dot cleanup also uses that skin. Head alpha is unchanged during separation.

Sources and exact built-in image_gen requests are in `source/face-consistency/`; `registration.json` records the candidate sampling and feature windows. `previews/face-consistency-review.png` shows only the currently applied neutral/toward/away faces from left to right. Historical comparison is separate in `previews/face-consistency-before-after.png`, with the preceding render above and the current render below. Nod, body, walking, give and the approved expression frames are unchanged by this revision. Old away-pose sources and the prior registration remain archived, not active.

### Nod follow-up

The old nod independently transformed rounder eyes from another face, making the character's expression jump. Its intermediate key was the same pitched face moved upward, so the changed eyes appeared at the first transition. Both keys are superseded. The pitched head now comes directly from the approved neutral reference with the same calm vertical brown eye design, round cheeks, blush and small mouth. Candidate registration is a 64×64 source tile at native offset `(11,-2)`, giving a 52×38 visible head against the neutral 51×39. No eye/brow or mouth is independently resized. The skin-filled edit supplies only the exact feature underlay and a local dark nose-dot cleanup; recomposition matches the complete candidate elsewhere and preserves head alpha. The short neck remains connected to the original upper body.

Exact built-in image_gen prompts, full and blank candidates, registration and the previous nod are preserved in `source/nod-consistency/`. `previews/nod-heads-current.png` shows neutral/preparation/down from left to right. `previews/nod-base.gif` and `nod.gif` replay the current five keys. Every non-nod frame is unchanged by this follow-up.

`previews/outline-head-before-after.png` compares nod-down, turn-toward and turn-away. `previews/outline-shoulder-before-after.png` compares give-mid, walk-0 and walk-3. Both show the previous render above the corrected render at exact 6× nearest scale.

## Atlas and checks

`atlas.png` is a 512×496 authoring export, 25 sprite IDs with 24 unique payloads. Baked frames are packed deterministically with trim offsets, original 128×160 animation size, pivot `(64,140)`, no rotation, 1px extrusion and 1px transparent gutter. The atlas is generated from composed parts rather than image_gen. The duplicate half-blink/relaxed image shares a rect while keeping separate state IDs.

This is not integrated into the existing game catalog. Its trim/state/face-channel contract needs an adapter before runtime use. GPU draw calls, overdraw and device compression support are not measured. RGBA8 base texture allocation is 1,015,808 bytes; PNG file size is a separate metric. Static sprite audit and explicit extrusion/gutter checks pass without implying a runtime performance improvement.

```sh
rtk proxy node assets/npc/leatherworker/source/build-all.mjs
rtk proxy python3 assets/npc/leatherworker/qa/verify-character.py
rtk proxy python3 assets/npc/leatherworker/qa/verify-animation.py
```

Verification checks shared dimensions/palette/binary alpha, distinct eye/mouth states, expressions unchanged outside their feature masks, complete foundation reconstruction and coverage, arm/leg opposition from boot/hand measurements, all soles grounded, transfer once, atlas round-trip and boundary pixels, and deterministic PNG/JSON/GIF rebuilds. Reports are in `qa/animation-validation.json` and `qa/sprite-audit.json`.

`qa/playback-report.json` records actual requestAnimationFrame playback in an isolated headless Chrome preview. All six walking keys were captured in eight combinations of 1×/4× scale, bare/equipped, and normal/0.25× speed; other gesture/face clips were replayed as well. All 48 browser captures are checked pixel-for-pixel against the corresponding native frame with its background and exact nearest scale. GIF pixels and frame times are also checked. The captures provide visual evidence separately from static source sheets and timing/order checks. This previews the authoring images, not the Bevy game renderer.

The outline revision also captures all 16 nod/shake-head/give keys during bare-body 4× playback at 0.25× speed and compares their rendered pixels. Regression checks preserve raw-source shoulder occupancy, rigid upper-body pixels and approved expression renders. Toward head pixels outside the local neck patch stay unchanged; rebuilt nod/away heads reconstruct their registered complete sources outside the nose cleanup and retain their silhouettes. Both eyes retain sclera and iris. The nod preparation exactly translates approved neutral face parts, and all non-nod frames match the preceding revision.

## Common animation canvas

Frames and pose layers publish on 128×160 with a `(16,16)` offset from the retained 96×128 authoring originals. Sockets, trim offsets and pivots use the expanded coordinates. `scripts/expand-npc-canvas.py` runs after the animation builder; no image is resampled. GIFs and animation HTML use 512×640 at 4×. Diagnostic source crops retain their authoring coordinates. Archived browser captures cover the prior native view; current GIF pixels and crop roundtrips are verified separately.
