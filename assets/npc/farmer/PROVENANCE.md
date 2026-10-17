# Farmer 2

Current output is a modular character authoring asset at Grid B 6×8 cells, 96×128 RGBA pixels, binary alpha and a shared 64-color palette. It includes 8 expression recipes, 14 pose recipes (including source upper-body helpers), and 11 timed clips. A separate fixed-cell export connects these frames to the game's runtime catalog.

## Current sources and build

The user's original `../farmer/source/reference.png` defines full-figure proportions, pose and material detail. Built-in image generation produced the current empty-handed farmer with charcoal hair, the basic-clothes underlayer, and an apron underlayer with no belt/tool pouch. Exact prompts are in `source/body-rebuild-prompts.json`. The source files are `source/body-rebuilt-master.png`, `source/body-rebuilt-base.png`, and `source/body-rebuilt-apron.png`; the corresponding `generated-*.png` files are current copies for inspection.

Run `node assets/npc/farmer/source/build-character.mjs` from the repository root. The builder uses one source registration (scale 0.1, offset [-11,-10]) for all body/equipment artwork. It preserves authored contours and material shadows without global erosion, skeletonization or outline-color replacement. The feet remain registered at [48,124]. Source dimensions and authoring bands are not runtime auto-fit rules.

`source/approved-face/` retains the improved native facial donor and eyes/mouth. The donor is translated by [-2,+6]; only its facial interior is copied onto the complete underlayer. Its obsolete hat-clipped hair silhouette is never used as the head foundation. Eyes and mouth retain their geometry and RGB values under the same translation. The donor's 34 colors are reserved; remaining colors describe the source materials.

The complete `body-rebuilt-base.png` underlayer supplies head, shirt, bare hands, trousers and boots. It is sampled once into `source/native-underlayer.png`, then the facial interior is patched into `source/foundation-native.png`. Head/upper/lower partition this foundation without dropping pixels. Equipment only adds opaque pixels. The builder fails if any of the underlayer's 4,226 opaque native pixels disappear from either the basic or equipped composite; `source/coverage-check.json` records the result. This supersedes the earlier incomplete reconstruction from body/gear region masks. Apron, belt/tools, gloves and hat remain independent layers; `equipment/apron-belt.png` is a derived combined export.

## Inspection and limits

`previews/underlayer-check.png` shows the sampled underlayer, basic character with facial features, equipped character without a hat, and fully equipped character, left to right. `previews/body-comparison.png` shows original/before/after at the same source scale and canvas, including the original's carrots for reference. `previews/head-check.png` shows hat-on, hat-off and head-base. `previews/layers.png` shows decomposition. The builder regenerates these previews alongside native exports. Coverage checks prove that export did not remove underlayer pixels; they do not prove perfect style, clean equipment masks or absence of defects already present in a source. Sampling and palette reduction still differ from the high-resolution reference.

Earlier prompts and candidates remain for provenance. Superseded builders and source images are in `source/archive/`; compatibility entry points now invoke the current builder so they cannot reintroduce the earlier fragmented exports.

## Expressions and posed parts

`source/expression-prompts.json` and `source/pose-prompts.json` record built-in image_gen prompts and generated source files. Expression candidates were localized edits of a fixed 64×64 crop enlarged 16×. Only eye/eyebrow and mouth feature pixels are extracted into the existing shared palette; the approved neutral base is unchanged. The 5 eye states and 4 mouth states form 8 named expressions plus 20 inspection combinations.

Posed head sources supply complete hatless skulls and direction-specific blank facial bases, eyes and mouths. Existing hat artwork is reused with small integer offsets; it has not been newly painted for every angle. Nod uses one authored down-pitch key plus an integer-shifted intermediate; shake uses toward/away keys. These head poses currently provide `open`/`rest` only, rather than every neutral-face expression combination.

Giving and walking upper-body sources replace the whole upper-body pose, with separate front-arm occlusion and glove layers. The standing hip underlayer comes from the extended-give source where the previously occluding hand is absent. Six authored walking lower-body keys reuse the two arm swing sources and a neutral arm position at passing keys. The near anatomical right leg carries the gold buckle; its forward phase pairs with the near arm swinging back. Foot ground is registered at native y=124 and a complete shared hip strip bridges the small source-key registration differences. Only tiny detached source specks are dropped; larger detached body components cause the builder to fail.

The first turn-away blank-face candidate erased the far-eye edge into transparency. It is marked rejected in the prompt ledger. `turn-away-blank-fixed.png` fills the eye location with skin, retaining the original far-eye silhouette. The insufficient first opposite-arm candidate and the walking key with a misplaced buckle are also marked superseded.

Run `node assets/npc/farmer/source/build-all.mjs` to rebuild base, parts, composites, previews, clips and atlas. Run `python3 assets/npc/farmer/source/verify-animation.py` for read-only checks. `animation.json` owns frame paths, durations in milliseconds and events; `give` emits `transfer` exactly once even though two timeline entries share the extended-give image.

## Authoring atlas and review

`atlas.png` is a baked, trimmed authoring export derived from the part compositions. `atlas-manifest.json` preserves each frame's 96×128 logical size, trim offset, [48,124] pivot, clips and events. Pixel-identical frames share payload rectangles while their IDs/timeline entries remain distinct. The page has nearest sampling, no mipmaps/rotation, 1px extrusion and an additional 1px transparent gutter per side. This manifest is not the existing Bevy `Catalog` schema; the game uses the separate untrimmed export below.

`python3 scripts/export-farmer-runtime.py` packs the authored `frames/` into `runtime-atlas.png` and writes `character.json` for Bevy. It preserves native RGBA pixels and the foot anchor, converts milliseconds to seconds, and reads posed hand sockets from `poses/recipes.json`. Timeline entries retain their own duration and transfer event even when sharing an image. Baked faces replace the obsolete face overlays. The standalone `carrot.png` prop is retained byte-for-byte from Git HEAD when integrating this export; the deleted legacy farmer artwork is not used. `build-all.mjs` also runs this exporter. Use `python3 scripts/export-farmer-runtime.py --check` to verify generated files and the integrity inventory without changing them. The obsolete `scripts/pack-npc.py` still targets deleted source sheets and must not be used for this character.

`previews/animation.html` opens locally without a build server and supports clip selection, pause/step/scrub, speed, bright/dark background and equipment toggling. `previews/*.gif` are enlarged review loops; non-looping clips intentionally repeat in these GIFs for inspection. Named contact sheets include `expressions-8.png`, `expressions-20.png`, `head-poses-no-hat.png`, `give-poses.png` and `walk-6-no-equipment.png`.

`qa/animation-check.json` records native dimensions/alpha/palette, unchanged neutral baseline, expression-mask invariance, pose coverage, corrected eye silhouette, opposite hand/boot travel at contact/support keys, ground anchor, and atlas round-trip/extrude/gutter checks. `qa/atlas-audit.json` records static sprite-optimization metrics. These do not measure GPU draw calls, overdraw or runtime performance. Source-key continuity and acting quality remain visual judgments; structural checks alone do not establish them.
