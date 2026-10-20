# Crafting parts v1: 23 part icons

Production spec for the parts in catalog v0.3. The finished goods (22 plus 3 later) are a
separate atlas. Parts follow the [material atlas](MATERIAL_ATLAS_V1.md) contract:
- 32×32 RGBA, binary alpha
- ≤ 16 colours per icon, 1px transparent rim
- top-left light, shared warm near-black outline
- 2× in item slots

The machine-readable spec is [`assets/items/parts/parts-catalog.json`](../../assets/items/parts/parts-catalog.json).
It holds the ids, names, recipe, yield, uses, tier, atlas cell, base material and visual cue.

## Materials and variants

Each part is drawn once, in its base material:

| Family | Base material | Why |
|---|---|---|
| wood | ash | Ash is the traditional wood for tool handles and spear shafts (tough and shock-resistant). |
| metal | iron | The starting metal. |
| leather | deer | Light buckskin, so the surface marks stay readable. |

Material variants are palette swaps, not new art:
- `scripts/recolor-parts.py` re-samples every non-outline pixel by lightness into a ramp from `material-catalog.json`.
- Wood parts use heartwood, without the pale sapwood, plus one darkened shadow tone. Bark colour is never used, so white-barked birch still reads as wood.
- The silhouette edge is always redrawn in the shared outline colour.
- Example: `variant("metal_rivet", "bronze")` writes `icons/variants/bronze_metal_rivet.png`.
- Samples: `docs/qa/evidence/parts-p0/variant-samples-3x.png`.

Rules from the catalog:
- Steel goes only to cutting parts (blades, spearheads, axe heads), and only after research.
- Bronze and brass go to rivets, fittings and decorative parts.
- Not every ingot becomes a blade.

## Atlas layout

`parts-v1.png`, 8×4 cells = 256×128.

| Row | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 0 wood | wooden_pole | wooden_plank | wooden_rod | arrow_shaft | wooden_handle | leather_wrapped_handle | (shield_core, on hold) | — |
| 1 metal | straight_blade | wide_blade | awl_point | axe_head | hoe_head | saw_blade | sickle_blade | arrowhead |
| 2 metal | spearhead | shear_blade | metal_rivet | metal_fitting | shield_boss | — | — | — |
| 3 leather | leather_strip | leather_grip | leather_strap | leather_reinforcement | — | — | — | — |

## Tiers

Each part takes the tier of the earliest finished good that needs it.

| Tier | Parts |
|---|---|
| P0 (done) | wooden_pole, wooden_plank, wooden_handle, straight_blade, axe_head, spearhead, metal_rivet, leather_strip |
| P1 | hoe_head, sickle_blade |
| P2 | wide_blade, awl_point, saw_blade, shear_blade, arrowhead, metal_fitting |
| P3 | wooden_rod, arrow_shaft, leather_wrapped_handle, leather_grip, leather_strap, leather_reinforcement, shield_boss |

## Shape reference (researched 2026-10-19)

- **Spearhead:** leaf-shaped blade with a raised midrib that runs into a round, tapering socket. The socket is pinned to the shaft through a rivet hole ([British Museum ML.2451](https://www.britishmuseum.org/collection/object/H_ML-2451), [Cambridge MAA](https://collections.maa.cam.ac.uk/objects/579986), [Timeline Auctions](https://timelineauctions.com/auction/168/93/)).
- **Axe head:** a round socket (eye) for the haft, with a curved cutting bit ([Timeline Auctions](https://timelineauctions.com/auction/168/93/)).
- **Knife blade:** the tang is the part of the blade that runs into the handle, so loose blades are drawn with a bare tang and handles with a tang slot ([Wikipedia: Tang](https://en.wikipedia.org/wiki/Tang_(blade))).
- **Shield boss:** a domed boss with a flange riveted to the board ([Cambridge MAA](https://collections.maa.cam.ac.uk/objects/579988)).

## Pipeline

```sh
python3 scripts/clean-material-candidates.py --set parts   # raw picks -> 32x32 icons
python3 scripts/recolor-parts.py                           # snap to material palette, rebuild atlas
python3 scripts/derive-parts.py                            # rod, arrow shafts, wrapped handle, grip
```

Four parts are derived locally from approved icons, with no generation:
- `wooden_rod`: the pole with its middle removed.
- `arrow_shaft`: three thin shafts cut from the pole's centre line, each with a nock slit.
- `leather_wrapped_handle`: the handle with a spiral deer-leather wrap, one tone darker than the ash so it reads.
- `leather_grip`: the wrapped section on its own.

The wrapped handle mixes two materials. `variant()` recolours a single material, so variants of it need a two-ramp pass later.

Raw Sprite Fusion picks and their asset ids are in `assets/items/parts/source/spritefusion-p0/`.

## Owner decisions (2026-10-19)

- **Pole length:** `wooden_pole` carries a `length` attribute (`short` | `long`). A spear uses a short pole; a pike and a halberd use a long one. Both lengths share one icon.
- **Frying pan:** made directly from iron ingot ×2. `pan_body` is removed from the catalog. Its generated candidate was not used.
- **Shield core:** on hold. The wooden shield is likely square, but other shapes are possible, so the core waits for that decision.
- **Shield shape (2026-10-20):** square. The finished shield is drawn from planks, a strap and rivets (see [FINISHED_GOODS_V1](FINISHED_GOODS_V1.md)); whether `shield_core` is still needed is open.

## Open question

- **Leather punch:** its recipe says "handle" without naming one. `wooden_handle` is assumed.
