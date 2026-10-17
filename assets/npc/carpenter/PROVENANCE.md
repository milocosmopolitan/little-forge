# Carpenter assets

Current export: eight expressions and nod/shake-head/mallet-give animations, 17 frames in a 128×160 common canvas. The latest local outline repair changes 40 bare-base hand/wrist pixels, five visible equipped. The preceding baseline and every delta are retained in `source/animation/outline-review-before/` and `outline-review-migration.json`. See `ANIMATION.md`, `animation.json` and `poses/recipes.json`. The neutral-production history below is retained as provenance; its original single-frame scope describes that earlier phase.

The user-provided `Pixel Carpenter with Dirty Blonde Hair.png` is retained as `source/reference.png`. It defines the dirty-blond tousled hair, young face, short jaw stubble, slim-to-average proportions, cream rolled sleeves, olive trousers, brown boots with dark toe caps, leather work apron, boards and wooden mallet. Only one neutral standing pose is produced.

Built-in image_gen created the complete equipment-free candidate in `source/generated-underlayer.png`; the prompt and input registration are in `source/generation-prompts.json`. Hidden shirt, trousers and bare hands are reconstructed rather than observed. Both source images register on a fixed 10-source-pixels-per-native-pixel grid with source origin `(60,130)`. The generated supporting hand needed a local native shift to meet the reference glove. Its old location is restored as shirt before the complete base is split. The body is not assembled from unrelated NPC parts.

The native export is 96×128 RGBA, Grid B 6×8, origin `(0,0)`, foot baseline `(48,124)`. It shares 32 opaque RGB colors and binary alpha. The palette comes from this carpenter's base and equipment, reserving off-white eye highlights and two neutral toe-cap grays. This is a native interpretation with reconstructed hidden anatomy, not a pixel-identical reduction of the reference.

## Files

- `neutral.png`: complete equipped standing pose.
- `base-neutral.png`: complete equipment-free body and neutral face, with bare hands.
- `head-base.png`, `body/upper-base.png`, `body/lower-base.png`: complete registered base parts. Eyes and mouth have opaque skin backing.
- `eyes/open.png`, `mouth/rest.png`: original relaxed expression only. No alternate expressions are implied.
- `equipment/`: apron with its attached pocket tools, straps, belt/pouches, towel, boards, fingerless gloves, mallet. The attached tool pocket is intentionally kept with the apron.
- `character-spec.yaml`: JSON-compatible YAML with actual layers, palette, sockets, reconstruction notes and authored native measurements. Width measurements are horizontal native samples at the recorded rows, not reference anatomy measurements.
- `atlas.png`, `atlas-manifest.json`: one untrimmed frame in a 112×144 page, rect `[2,2,96,128]`, 1px extrusion plus 1px transparent gutter. The source's boundary pixels are transparent, so the extrusion is transparent too.
- `previews/index.html`: offline equipment-free/equipped comparison at nearest 4× and native 1×, with a light/dark background toggle.
- `previews/`: exact 4× PNGs, face 8×, fixed-scale reference comparison, light/dark and individual equipment review.

The mallet shaft contains a pose-specific gap behind the gripping fingers. The complete bare hand remains in the underlayer when the mallet and gloves are disabled. Gear alpha is applied without clearing any underlying base pixels. Equipment source polygons, local contours, feature masks and hand registration are deterministic in the builder. No global outline erosion or smoothing is used.

## Rebuild

From the repository root with Pillow installed:

```sh
rtk proxy python3 assets/npc/carpenter/source/build-character.py
rtk proxy python3 assets/npc/carpenter/source/verify-character.py
```

`qa/validation.json` records fixed canvases, shared palette, binary alpha, feature backing, exact layer recomposition, zero base loss, exact nearest previews and atlas round-trips/boundaries. `qa/pixel-validation.json` applies the repository pixel validator to every native production image. `qa/rebuild-check.json` checks repeat export bytes. Static atlas inspection is recorded separately in `qa/sprite-audit.json`.

Visual inspection covers hair silhouette, brows/eye whites/gaze, smile and sparse beard, shoulders/collar, complete bare hands, glove/board/mallet overlap, towel hem, apron side and toe-cap colors on light and dark backgrounds. The reference has no hat; no hat toggle is applicable. Only neutral exists, so cross-expression invariance and animated playback are inapplicable.

This export does not change the NPC catalog, existing NPCs, runtime code, or the asset manifest. The repository's existing face/clip contract needs a separate adapter before runtime adoption. No GPU timing, draw-call reduction or device qualification is claimed.

## Hand, cloth and equipment repair

The user identified the rough supporting hand, a protruding shirt patch, broken towel/belt coverage, a clipped mallet and irregular apron edges. The approved 32-color palette is now locked in `source/palette-locked.png`, preventing these local repairs from recoloring the head or face. Prior native layers are retained in `source/cleanup-before/`.

The supporting palm now has a continuous wrist, grouped fingers and a shallow thumb fold. Its shirt edge follows the torso instead of the old translated-hand patch. The empty gripping hand is compact and aligned to the reference shaft. All hand pixels remain owned by the upper-body layer even where they cross the nominal waist band.

The towel has a complete cloth outline, two green stripes and a folded inner shadow. Its hanging leather pouch is complete behind it. The apron retains visible pocket/tool detail inside a closed leather silhouette, without the old copied shirt fringe or punched-out side notches. The mallet includes the previously clipped left face and a complete shaft in `source/mallet-complete.png`; `equipment/mallet.png` cuts only two explicit front-finger bands from that shaft. The glove is composed above the mallet, as recorded in equipment order.

`previews/cleanup-hands-8x.png`, `cleanup-towel-8x.png`, `cleanup-mallet-8x.png`, and `cleanup-apron-8x.png` show the preceding native export beside the repairs. Structural validation and the equipment-free/equipped light/dark review are repeated after this revision. No alternate expression, animation or runtime integration is added.

## Supporting-hand anatomy and full towel drape

A subsequent user review found that the simplified native hand had lost anatomy and the towel silhouette still missed visible cloth. A focused image_gen edit rebuilt only the supporting hand/forearm crop; the exact prompt, crop registration and retained candidate are in `source/hand-repair-prompts.json`. Only the isolated native hand mask is adopted, against the locked existing palette. The face, shirt body, boots and other equipment do not use the regenerated crop.

The towel's upper-left side and folded hem are now filled as a broader complete drape. The obsolete partial-pouch source polygon that left a dangling dark line beside the towel is removed; the complete leather pouch behind the towel is retained. Native 8× before/after evidence from the immediately preceding export is in `previews/hand-towel-hand-8x.png` and `previews/hand-towel-towel-8x.png`. The immediate pre-edit assets are retained in `source/hand-towel-before/`.

## Animation review repairs

The user reported a moving collar/shoulder, outlined neck stumps that made the head appear detached, a surprised mouth overlapping the nose, unfamiliar surprised eyes, a fork-shaped offering palm, an interrupted supporting-forearm edge, and residual white in a closed eye. Movable head masks now exclude fixed collar pixels. Candidate neck stumps are removed and a skin support joins the head to the existing neck while retaining visible shirt pixels. Closed eyes occlude the complete original eye interior; surprised eyes retain the original pupils and eye width. Mouth openings sit below the fixed nose/cheek pixels.

Only the requested inner forearm edge changes the approved base: nine opaque pixels become dark contour pixels, without changing alpha or hand silhouette; two are visible equipped. `source/animation/review-baseline-migration.json` records every changed coordinate. A localized image_gen palm repair is retained in `source/animation/candidates/give-hand-repair.png`; its prompt and native registration are in `review-repair-prompt.json`. The source sleeves and proximal forearms are reused; the previous blanket arm-edge darkening is removed. All expression/pose layers, frames, atlas and normal/slow GIF previews are regenerated from the builders.

## Second animation review: upright grip and neck registration

The user rejected the palm-up give motion, an exposed neck wedge during turns, and the surprised eyes. The give pose now retains the approved curled hand, glove and upright mallet together; only a localized generated distal forearm joins them to the retained sleeve. The generator's new fingers are not used. Exact prompt, retained candidate and source crop are in `source/animation/grip-repair-prompt.json`. Prior frames and builders are retained in `source/animation/review-round2-before/`.

Both turn jaws move down two integer pixels to the neutral chin height. Their fixed neck now reuses the original neck cross-section instead of the diagonal skin wedge. Exposed cloth stays fixed; a turned chin can occlude the uppermost collar. Surprised brows move upward with the original shape, with warm enlarged whites and right-looking pupils. `previews/review-round2.png` and its dark variant compare the specific rejected keys with this revision.

## Neutral hand outlines and supporting wrist, October 17

The user's two screenshots identified a protruding/doubled grip outline and an indented supporting wrist in the neutral base. The legacy grip fragment above the former clear rectangle is removed, its internal wrist line is omitted, and its bottom contour is rounded. The supporting inner wrist uses a gradual contour instead of a four-pixel step. Changes are limited to two hand/wrist regions; equipment, head/face and lower-body pixels are retained exactly. The approved baseline hashes are updated only after comparing every changed pixel against those regions. The revised grip mask is shared by the give poses and all frames/atlases/GIFs are regenerated. The source remains native-grid code; no new raster candidate or body generation is used for these contour corrections.
