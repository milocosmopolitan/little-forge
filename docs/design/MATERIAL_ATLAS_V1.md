# Material atlas v1: 40 raw and processed materials

Production spec for catalog v0.3: v0.2 with the branches merged into one item and a birch log added. It covers ores, ingots, wood, hides and fuel.
Finished tools and intermediate parts are a separate atlas.

The machine-readable spec lives in
[`assets/items/materials/material-catalog.json`](../../assets/items/materials/material-catalog.json).
It holds the ids, Korean and English names, tier, atlas cell, palette ramps, cue and source.
This document holds the rules the JSON cannot express. Hex values exist only in the JSON.

## Contract

| Item | Value |
|---|---|
| Canvas | 32×32 native, Grid B 2×2 cells, RGBA, binary alpha (0/255) |
| Content box | x 1–30, y 1–30. The 1px transparent rim doubles as the atlas gutter; nearest sampling at integer offsets needs no extrusion. |
| Ground row | y = 30. Objects that rest on something (ore, ingot, log, leather, fuel) put their lowest opaque row here, so a row of slots shares one baseline. |
| Light | Top-left. Highlights on the upper-left edges, shadows lower-right. |
| Outline | 1px native, shared warm near-black `outline` ramp (matches the existing UI icons). Never double it to make a shadow. |
| Colours | Each icon uses ≤ 16 opaque colours. There is no atlas-wide cap: each material keeps its own ramp. |
| Shadow | No baked drop shadow or ground ellipse. The slot frame supplies context. |
| Display | 2× in 64×64 item slots (up to 60px visible), 3× in 96×96 detail. Integer nearest only. |
| Atlas | `materials-v1.png`, 8×6 cells = 256×192. Cell (col,row) comes from the JSON, at origin (32·col, 32·row). |

At 2× these icons show 2px blocks. The 256px UI icons render at 64 native, so material icons look
coarser beside navigation icons. This is the accepted cost of 32 native.

The proposed "48 inside 64" item-icon target in `assets/ui/README.md` needs 1.5×. That is not
an integer scale, so this spec replaces it with 2× and the 1px rim.

Not for the world layer: the shop hand icon (20px) and shelf icon (14px) are non-integer sizes.
If materials ever appear in the scene, they need their own world sprites.

## Atlas layout

Columns line up by kind, so each ore sits above its own ingot.

| Row | Col 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 0 ore | iron | copper | tin | lead | silver | gold | zinc | — |
| 1 ingot | iron | copper | tin | lead | silver | gold | zinc | steel |
| 2 alloy/fuel/branch | bronze_ingot | brass_ingot | charcoal | coal | wood_ash | pine_resin | wood_branch | — |
| 3 log | pine | oak | ash | walnut | cherry | mahogany | yew | birch |
| 4 raw | deer_hide | boar_hide | wolf_pelt | bear_hide | tiger_pelt | — | — | — |
| 5 leather | deer | boar | wolf | bear | tiger | — | — | — |

## Branches

There is one ordinary branch, `wood_branch`. It covers clubs, arrow shafts and fuel, whatever
the tree. The v0.2 catalog had seven species branches (14 wood items); this was cut by the
owner on 2026-10-19:
- At 32×32 a bare twig is only 1–2px thick, so species bark cannot be told apart.
- Special branches for wands come from rare mission loot. They are designed later as their own items, not as the species variants of this atlas.

## Metal reference (researched 2026-10-19)

| Item | Real appearance | Icon |
|---|---|---|
| Iron ore | Hematite: steel-grey to rust-red, with a rust-red streak | Grey rock, rust-red vein |
| Copper ore | Chalcopyrite: brassy yellow. It weathers to malachite, which is green and banded. | Grey rock, banded green vein, brassy grains |
| Tin ore | Cassiterite: mostly black or brown, adamantine lustre | Grey rock, black-brown vein, 1px glints |
| Lead ore | Galena: lead-grey to silver, metallic, cubic | Grey rock, metallic grey vein |
| Silver ore | Native silver: silvery metallic blebs | Near-black rock, silver vein |
| Gold ore | Native gold, usually in quartz | White quartz host, gold vein |
| Zinc ore | Sphalerite: yellow to brown to black, resinous lustre | Grey rock, amber-brown vein, pale highlight |
| Copper | Reddish-orange | Red-orange ingot |
| Tin | Silvery-white with a faint yellow hue | Light warm grey |
| Lead | Shiny bluish-silver when cut; tarnishes to dull grey | Dull blue-grey, dented corner |
| Zinc | Bluish-silver, duller with age | Pale silver-grey, faint blue |
| Bronze | Copper-toned reddish brown | Brown, darker than copper |
| Brass | Yellow; more zinc makes it lighter | Pale greenish yellow |

The v0.2 zinc "spangle" cue was dropped: spangle is a feature of galvanized coatings, not of
zinc ingots.

Sources:
- Minerals:
  - [Tulane mineral study sheet](https://www2.tulane.edu/~geol113/minerals.htm)
  - [MIT 12.001 mineral guide](https://ocw.mit.edu/courses/12-001-introduction-to-geology-fall-2013/ca9d1d21763c564ad4fd651054c3d738_MIT12_001F13_Lab1_M_Guide.pdf)
  - Wikipedia: [Hematite](https://en.wikipedia.org/wiki/Hematite), [Malachite](https://en.wikipedia.org/wiki/Malachite), [Cassiterite](https://en.wikipedia.org/wiki/Cassiterite)
  - [RRUFF: native silver with malachite](https://rruff.info/S/R070463)
- Metals:
  - [Wikipedia: Lead](https://en.wikipedia.org/wiki/Lead) and [Tin](https://dlab.epfl.ch/wikispeedia/wpcd/wp/t/Tin.htm)
  - [Copper, brass and bronze colours](https://materialwelding.com/?p=516430)
  - [Lewis Brass colour matching](https://lewisbrass.com/pages/color-matching-chemical-finishes)
  - [SHCG metal identification](https://shcg.surfaceimpression.digital/Metal)

### Derived ores and ingots (no generation)

`scripts/recolor-metals.py` derives the six other ores from `iron_ore` and the nine other
ingots from `iron_ingot`. It splits each base into three regions:
- outline: kept unchanged
- body: re-sampled by lightness from the metal or host ramp
- accent: the vein on ores; on ingots, the rust specks, which are folded into the body because they are iron only

Rule-based marks add the mineral cues:
- malachite bands and chalcopyrite grains (copper)
- glints (tin, lead)
- a resin highlight (zinc)
- the dented corner (lead)

All seven ores share the iron-ore silhouette. The planned `ore-B/C` masks were not made, so
ores differ by host rock and vein only.

## Hide and leather reference (researched 2026-10-19)

| Animal | Raw coat | Leather icon |
|---|---|---|
| Deer | Reddish-brown summer coat (grey-brown in winter), white belly and tail underside | Honey-beige buckskin |
| Bear (brown) | Brown to blond or black; long guard hairs with white or silver tips give a grizzled, shaggy look | Dark brown |
| Tiger | Orange (yellowish to reddish); black or dark-brown stripes, mostly vertical; white underside and inner legs | Golden orange with faint stripe ghosts |
| Wolf, boar | Existing P0 icons | Existing P0 icons |

Smoke-tanned buckskin is dark honey; unsmoked buckskin is lighter, even white.

How they are made:
- Raw hides are generated: the head (deer ears, bear's round head and claws, tiger stripes) cannot come from a recolour.
- Leathers are recoloured from `wolf_leather` by `scripts/recolor-leathers.py`. The cord stays shared.
- Edge thickness is the base roll's for every animal, so bear leather is not thicker yet.

Sources:
- [White-tailed deer coat (Brainerd Dispatch / MN DNR)](https://www.brainerddispatch.com/sports/why-does-the-fur-coat-of-a-deer-change-colors)
- [USFWS: brown bear](https://fws.gov/species/brown-bear-ursus-arctos-arctos)
- [Wikipedia: Tiger](https://en.wikipedia.org/wiki/Tiger)
- [Wikipedia: Buckskin (leather)](https://en.wikipedia.org/wiki/Buckskin_(leather))
- Birch:
  - [SAPS: silver birch](https://saps.org.uk/trees/birchs.htm)
  - [USU: European white birch](https://extension.usu.edu/treebrowser/catalog/birch-european-white.php)
  - [Wikipedia: Birch wood](https://en.wikipedia.org/wiki/Birch_wood)
  - [The Wood Database: yellow birch](https://www.wood-database.com/yellow-birch/)

## Wood reference (researched 2026-10-19)

Wood colours follow published descriptions, not guesses. A log icon shows the **lower trunk**
bark and a cut end with a sapwood ring around the heartwood.

| Species | Bark (lower trunk) | Heartwood | Sapwood |
|---|---|---|---|
| Pine (소나무, *P. densiflora*) | Brown to grey-brown thick plates. Orange-red only on young upper trunk. | Light brown to yellow-brown | Pale yellow |
| Oak | Grey-brown, deep vertical furrows and ridges | Light–medium brown, reddish cast, ray flecks | Paler, not sharply separated |
| Ash | Grey-brown, diamond-shaped furrow net | Light–medium brown | Wide, beige or light brown |
| Walnut | Dark brown to black, deep narrow furrows | Pale to chocolate brown | Pale yellow-grey to near white |
| Cherry | Dark red-brown, horizontal lenticel stripes. Old bark breaks into black curled plates. | Pinkish → medium reddish brown | Pale yellowish |
| Mahogany | Brownish-grey to reddish-brown, scaly, long furrows | Pinkish to reddish brown | — |
| Yew | Thin reddish-brown, flaking in small strips | Orange-brown | Thin pale yellow or tan band |
| Birch (silver) | White, with black diamond marks near the base, horizontal lenticels and papery strips | Light reddish to pale yellow-brown | Nearly white |

Sources:
- Wood colours: [The Wood Database](https://www.wood-database.com/), pages for eastern white pine, red oak, white ash, black walnut, black cherry, Honduran mahogany and yew.
- Pine: [Wikipedia: *Pinus densiflora*](https://en.wikipedia.org/wiki/Pinus_densiflora) and the [Korean pine wood-colour study](https://www.earticle.net/Article/A214911).
- Bark:
  - [How to identify a tree by its bark](https://www.discoverwildlife.com/plant-facts/plant-id/how-to-identify-a-tree-from-its-bark/)
  - [Tree bark identification](https://www.goodandbeautiful.com/tree-bark-identification/)
  - [Wikipedia: *Taxus baccata*](https://en.wikipedia.org/wiki/Taxus_baccata)
  - [NParks: *Swietenia macrophylla*](https://www.nparks.gov.sg/florafaunaweb/flora/3/1/3150)

Rule: research every new species or material before generating it, and record the reference
here.

## IDs

- Ids are `snake_case`, `<kind>_<form>`.
- `ash_log` is the wood. `wood_ash` is the burn by-product. A bare `ash` id is forbidden.
- Raw forms keep the catalog's words: `wolf_pelt` and `tiger_pelt` are fur pelts; deer, boar and bear are `_hide`.
- The existing village `Item::Timber` and `Item::Leather` are coarse trade goods. Mapping them to these ids is an open gameplay decision; this atlas does not change them.

## Shared templates

The bounds below were the pre-pilot targets. The approved P0 icon of each family becomes the
real mask, and later variants reuse it.

One mask per family. Variants change colour and surface marks inside the mask; they never redraw
the outer shape. The only exceptions are listed per family.

| Template | Bounds (x, y) | Shape |
|---|---|---|
| `ore-A` | 4–27, 8–28 | Irregular rock; all seven ores share it (B/C masks dropped, see Derived ores). Mineral inclusions take 25–40% of the face. |
| `ingot` | 3–28, 16–28 | Trapezoid in 3/4 view. Top face y 16–20 (20 wide, x 6–25), front face y 21–28 (26 wide). Lead alone dents the top-right corner by 1px. |
| `log` | 2–29, 9–28 | Log lying diagonally, end grain facing the viewer at left (oval about 10×12, 2–3 growth rings). Bark texture is per species. |
| `branch` | 3–28, 3–28 | Single `wood_branch`. Bare stem from lower-left to upper-right with one fork, neutral brown, pale cut tip. **No leaves, needles or fruit** (owner decision). |
| `raw-hide` | 2–29, 3–28 | Flat stretched pelt: head flap, four leg flaps, tail. The fur side faces up. The lower-right corner turns up to show the pale `flesh` side. That turned corner is what marks a raw hide. |
| `leather-roll` | 3–28, 9–28 | Half-rolled sheet. The spiral end is on the left, a flat tongue on the right with a straight cut edge, and a `cord` tie. No fur pixels. |
| `fuel-chunks` | 4–27, 12–28 | Three lumps. Charcoal is split and square-ended with grain cracks. Coal is angular and faceted with glints. |
| `fuel-heap` | 5–26, 16–28 | Low cone. |
| `fuel-drop` | 8–23, 8–28 | Teardrop on a small bark chip. |

### Raw hide vs. leather

The catalog requires the two forms to read apart. This spec makes that structural, not just a
colour change:

- **Raw:** the animal's fur colour and pattern, the irregular pelt silhouette and the turned flesh corner.
- **Leather:** the roll shape, an even edge thickness, a cut edge and the cord. The animal reads from edge thickness and tone:
  - deer: 1px
  - wolf, boar, tiger: 2px
  - bear: 3px
  - boar: pore dots
  - tiger: faint stripe ghosts

## Identifiability

Each item's `cue` in the JSON is the non-colour feature that must survive at 1×. Known
greyscale-only collisions, each resolved by a shape or surface cue:

| Pair | Luminance | Separator |
|---|---|---|
| iron / lead ingot | close | Iron: neutral grey and rust specks. Lead: blue tint and a dented corner. |
| zinc / steel ingot | close hue | Zinc: paler, faint blue. Steel: darker, bluer, brightest bevel. |
| copper / bronze ingot | close | Copper: red-orange. Bronze: brown. |
| gold / brass ingot | 135 / 142 | Brass: paler and greener, lower saturation. |
| charcoal / coal | — | Charcoal: matte with grain cracks. Coal: faceted with glints. |

## Production order

Follows the catalog tiers (P0 14, P1 12, P2 13). The tier of each item is in the JSON.

1. **Templates first.** Draw one base per family as the P0 pilot:
   - iron_ore and iron_ingot
   - pine_log and wood_branch
   - wolf_pelt and wolf_leather
   - charcoal, coal, wood_ash and pine_resin

   Review these at 1×, 2× and 3× on light and dark backgrounds before making any variant.
2. **P0 variants.** Oak and ash logs, plus boar hide and leather.
3. **P1, then P2.** Recolour and re-mark inside the approved masks. Add new masks only for `ore-B/C`.

### P0 pilot (Sprite Fusion)

Candidates come from Sprite Fusion at native 32×32, tagged `little-forge`.
- Iron ingot and iron ore were generated first as the style anchors.
- The other eight were generated in their style.
- The picked asset ids are listed in `assets/items/materials/source/spritefusion-p0/PROVENANCE.md`.

`scripts/clean-material-candidates.py` turns the raw picks into `assets/items/materials/icons/`
and the partial `materials-v1.png`. It does three things:
- Reduces each icon to 16 colours with frequency-damped k-means snapped to real source colours, so small accents such as the hematite vein, the cord and the glints survive.
- Re-places each icon by integer shifts only.
- Trims one column on each side of any pick wider than 30 (the leather rolls) and redraws that edge as outline.

Sprite Fusion's style mode sometimes returns 33–57px results. Only 32×32 results are used.

### Derived logs (no generation)

Walnut, cherry, mahogany and yew logs are recoloured locally from approved logs by
`scripts/recolor-logs.py`. This costs no generation credits. Each species takes the base whose
bark texture matches the research:

| Log | Base | Why |
|---|---|---|
| walnut | oak | deep narrow furrows |
| yew | oak | fluted trunk |
| mahogany | pine | scaly, plated bark |
| cherry | ash | finer bark, with added pale horizontal lenticel dashes |
| birch | ash | bark mapped to white; the deepest furrows become black marks, plus sparse short black streaks across the log |

The script splits each base log into three regions and re-samples colour by relative
lightness from the species ramps:
- outline: kept unchanged
- cut end: its outer ring becomes sapwood and the rest heartwood; mahogany has no separate sapwood
- bark

Run `clean-material-candidates.py` first, then `recolor-logs.py`, `recolor-metals.py` and `recolor-leathers.py`. Both rebuild the atlas from
every icon in `assets/items/materials/icons/`.

Other generated art (high-resolution or off-size) is a candidate only. Rebuild each icon on the native grid, then
clean the palette, alpha and stair-steps by hand. A reduce-and-quantize pass alone is not
acceptable.

## Acquisition gaps

- Wolf and boar map to the current expedition enemies.
- Deer, bear and tiger have no source yet and stay unobtainable until a monster or region is added.
- Ores, coal, logs, the branch and resin need gathering activities that do not exist yet.

## Validation

```sh
python .claude/skills/pixel-art-gen/scripts/validate_pixel_asset.py \
  assets/items/materials/materials-v1.png --grid B --cells 16 12 --max-colors 640
```

Per icon:
- ≤ 16 colours
- 1px empty rim
- resting families end on y = 30
- a greyscale contact sheet passes the pairs above

The validator checks structure only. Visual review decides identifiability.
