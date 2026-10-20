# Finished goods v1: 22 icons (22 done)

Production spec for the finished goods in catalog v0.3. They follow the [material atlas](MATERIAL_ATLAS_V1.md)
and [crafting parts](CRAFTING_PARTS_V1.md) contract:
- 32×32 RGBA, binary alpha
- ≤ 16 colours per icon, 1px transparent rim
- top-left light, shared warm near-black outline
- 2× in item slots

The machine-readable spec is [`assets/items/finished/finished-goods-catalog.json`](../../assets/items/finished/finished-goods-catalog.json).
It holds the ids, names, category, recipe (in parts), tier, atlas cell, status, material regions and visual cue.

## Scope

| Tier | Items | Status |
|---|---|---|
| P0 | wooden_club, wooden_staff, wooden_shield, kitchen_knife, frying_pan, farming_hoe, sickle, woodcutting_axe | done |
| P1 | cleaver, hand_saw, shears, fleshing_knife, leather_awl, leather_punch, wood_chisel | done |
| P2 | dagger, hunting_knife, hand_axe, spear, pike, halberd, arrow | done |
| P3 (later) | shortbow, reinforced_shield, magic_wand | needs new parts or systems |

P3 waits for its parts
(bow stave and string, a finished shield plus `shield_boss`, special branches from rare expeditions).

## Atlas layout

`finished-v1.png`, 8×4 cells = 256×128. One row per tier; row 3 is reserved for P3.

| Row | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 0 P0 | wooden_club | wooden_staff | wooden_shield | kitchen_knife | frying_pan | farming_hoe | sickle | woodcutting_axe |
| 1 P1 | cleaver | hand_saw | shears | fleshing_knife | leather_awl | leather_punch | wood_chisel | — |
| 2 P2 | dagger | hunting_knife | hand_axe | spear | pike | halberd | arrow | — |
| 3 P3 | (reserved) | | | | | | | |

## Materials

Finished goods use the same base materials as their parts, so a knife and its blade and handle match:

| Region | Material ramp | Where |
|---|---|---|
| wood | ash heartwood (`WOOD("ash")` from recolor-parts.py) | handles, hafts, staff, shield planks, club grip |
| bark | `ash-bark` | the club head |
| metal | iron, plus the iron highlight | blades, heads, pan, rivets |
| leather | darker half of deer leather plus a shadow | shield straps, dagger and hunting-knife grips, arrow cord |

`scripts/recolor-finished.py` splits each icon into these regions by colour (grey → metal or bark,
warm → wood, saturated mid-brown → strap leather, dark brown → grip leather where an item lists `warm_dark`)
and re-samples each region by lightness into its ramp. Pale ash shafts (spear, pike, halberd) and the dagger's dark
grip are nearly grey, so those items set a lower `grey_sat` in the catalog and are also split by hue (blue-grey
steel is always metal). The other items keep the shared threshold, so P0 and P1 are unchanged.
Material variants of finished goods (a bronze-riveted shield, a walnut-handled knife) are not built yet;
they need the same two-ramp pass that mixed parts need.

## Owner decisions

- **Wooden shield (2026-10-20):** square. Vertical ash planks, two horizontal leather straps, iron rivets at the strap ends.
- **No more generation (2026-10-20):** new and revised icons are drawn by hand in `scripts/draw-finished.py`.
  Items marked `"source": "hand"` in the catalog are skipped by the clean and recolor steps, so the pipeline never overwrites them.
- **Budget (2026-10-20):** P0, P1 and P2 each on the owner's go-ahead, one call per item (330 credits). The Sprite Fusion balance is now 0; P3 or any retry needs new credits.
- **Frying pan (2026-10-19):** made from iron ingot ×2; there is no `pan_body` part.
- **Pole length (2026-10-19):** one `wooden_pole` with a `length` attribute. In the icons the staff, spear,
  pike and halberd still need distinct silhouettes; the pike runs past the cell corner to read as longer.
- **Fleshing knife:** drawn as a two-handled drawknife so it does not read as the cleaver.
- **Leather punch:** uses `wooden_handle` (the recipe names no handle; same assumption as the parts doc).

## Shape reference (researched 2026-10-20)

- **Frying pan:** wrought-iron pans have a shallow round body and a long forged handle, often with a hanging
  ring or eye at the end ([Concord Museum](https://concord.emuseum.com/objects/32649/frying-pans),
  [Huntington](https://emuseum.huntington.org/objects/55671/fry-pan-with-long-handle),
  [Lochac SCA: Dark Ages skillet, after UiO C16512](https://cockatrice.lochac.sca.org/?p=373)).
- **Sickle:** one piece of forged iron, a thin curved blade with a long tang set into a wooden handle
  ([Szczecin, 10th–12th c.](https://wmuzeach.pl/all-objects/6iNHLjqVGyG3qVtlPBTB_iron-sickle-),
  [Penn Museum](https://collections.penn.museum/collections/object/239939)).
- **Felling axe:** a socketed iron head with a broad bit on a wooden haft; the type lasted through the
  medieval period ([Pitt Rivers, Merovingian axe](https://odp-test.prm.ox.ac.uk/item/prm-object-152303/iron-axe-head-1-with-a-fragment-of-a-wooden-haft-2),
  [DIA felling axe head](https://dia.org/collection/head-felling-axe-26959)).
- **Hoe:** an iron blade socketed or tanged onto a straight wooden haft, set across the haft
  ([Pitt Rivers hoe](https://odp-test.prm.ox.ac.uk/item/prm-object-179106/hoe-with-an-iron-hoe-head-socketed-onto-a-straight-wooden-haft-the-head-is-oval-in-shape-with-a-pointed-tip-it-is-vertically-curved-ab-ops-move-2-12-2016)).
- **Plank shield:** shields were built from narrow wooden boards glued edge to edge, then faced or bound
  ([Szczecin, 12th-c. slat shield](https://wmuzeach.pl/all-objects/RC6GwjBCBFoAiOhDSVTS_wooden-shield-),
  [British Museum buckler](https://www.britishmuseum.org/collection/object/H_1884-0410-2)). No surviving
  medieval square plank shield was found; the square shape is a game decision.
- **Drawknife (fleshing knife):** a long, gently curved blade with a wooden handle at each end, angled to the blade
  ([Clare Museum](https://claremuseum.ie/explore/objects/drawknife-gently-curved-metal-blade-with-wooden-handles-at-angle-to-blades),
  [Pitt Rivers wheelwright's drawknife](https://odp-test.prm.ox.ac.uk/item/prm-object-133765/wheelwright-s-draw-knife-with-a-curved-iron-blade-and-two-turned-wooden-handles-used-for-trimming-felloes-spokes-etc-sm-verve-02-03-2015)).
  Real fleshing knives are a related two-handled beam knife; the drawknife silhouette was chosen for readability.
- **Scissors:** late-medieval iron scissors have two blades pivoting on a central rivet and loop handles
  ([Gilbert collection, Blackfriars](https://rosaliegilbert.com/thegilbertcollection_scissors.html),
  [Timeline Auctions, Thames find](https://timelineauctions.com/lot/medieval-thames-iron-scissors/216599/)).
- **Awl:** a slender round tapering iron blade in a wooden handle, used to pierce leather before sewing
  ([Museum Wales stabbing awl](https://museum.wales/collections/online/object/bca90a1d-5f05-35e0-93c3-b34e9fa98f43?page=475),
  [Collections WA leatherwork awl](https://collectionswa.net.au/items/2074c7de-29bb-4e53-bf51-96c3c40ae3f5)).
- **Chisel:** a flat iron blade on a tang or socket into a wooden handle
  ([Colonial Williamsburg socket chisel](https://emuseum.colonialwilliamsburg.org/objects/30897/hexagonal-socket-mortise-chisel),
  [Winterthur socket chisel](https://museumcollection.winterthur.org/object/4197)).
- **Hand saw:** a tapering toothed blade with a closed wooden handle ([Winterthur hand saw](https://museumcollection.winterthur.org/object/3968)).
  No medieval hollow punch was found in museum records; the punch follows later leatherworking tools.
- **Dagger:** a narrow double-edged blade of diamond section with a small guard and a disc or ring pommel
  ([London Museum rondel dagger](https://londonmuseum.org.uk/collections/v/object-29505/dagger-rondel-dagger),
  [Higgins ring-pommel dagger](https://worcester.emuseum.com/objects/49629/)).
- **Hunting knife:** single-edged with a clip point, where the spine dips toward the tip ([Wikipedia: Clip point](https://en.wikipedia.org/wiki/Clip_point)).
  The clip point is old, but the Bowie-like proportions in the icon are a game choice.
- **Hand axe:** a small axe with a flat butt (a hatchet has a hammer face instead)
  ([Science Museum Group hand axes](https://collection.sciencemuseumgroup.org.uk/search/object_type/hand-axe)).
  Redrawn by hand (owner, 2026-10-20) because the generated head flared both ways and read as a double-bit axe.
  The head is square to the 45° haft, as on a real axe. It is drawn in head coordinates (along the head, along the haft)
  so it stays at a right angle without rotating a bitmap. The haft runs through the eye, and its tip shows past the top.
  The poll is a shallow flat butt, and the bit flares on one side only, to a convex honed edge. Against the woodcutting axe it is smaller and paler, with no beard and a thinner, shorter haft.
- **Pike:** a small lozenge head on a tall socket with a pair of long, narrow iron langets down the shaft
  ([Higgins pike](https://worcester.emuseum.com/objects/48799/pike), [Wikipedia: Ahlspiess](https://en.wikipedia.org/wiki/Ahlspiess)).
  The spear keeps the broad leaf head from the parts doc, so a large head on a short shaft vs a small head on a long one tells them apart.
- **Halberd:** a thrusting spike on top, an axe blade on one side, a fluke (hook) opposite, and langets down the staff
  ([Higgins halberd](https://worcester.emuseum.com/objects/48670/halberd), [another](https://worcester.emuseum.com/objects/48694/halberd)).
- **Arrows:** iron heads (bodkin or broadhead), shafts of ash or similar wood, three feathers, usually goose
  ([Wikipedia: Arrowhead](https://en.wikipedia.org/wiki/Arrowhead), [Wikipedia: Bodkin point](https://en.wikipedia.org/wiki/Bodkin_point)).

## Pipeline

```sh
python3 scripts/clean-material-candidates.py --set finished   # raw picks -> 32x32 icons
python3 scripts/draw-finished.py                               # hand-drawn icons (hand_axe)
python3 scripts/recolor-finished.py                            # snap regions to material ramps, rebuild atlas
python3 scripts/qa-finished-goods.py                           # rim/alpha/colour checks + QA images
```

Always run both in order: the snap reads the cleaned colours, so running it twice drifts.

Raw Sprite Fusion picks and their asset ids are in `assets/items/finished/source/spritefusion-p0/`.
QA images: `docs/qa/evidence/finished-goods/` (atlas 3×, 1×/2×/3× on dark and light, P0 goods next to their parts).

## Open review

- At 1× the hoe and the axe share a haft and a head on the upper right; the hoe blade hangs straight down, which
  is the only cue. Check it in the inventory slot.
- The leather punch and the wood chisel share a silhouette (handle, ferrule, short iron end); the punch's round hollow tip is the cue.
- The arrow bundle is the busiest icon (three heads, three fletchings, a cord); check it at 1×.
- The icons are not loaded by the game yet; the in-game check needs the item UI to read `finished-v1.png`.
