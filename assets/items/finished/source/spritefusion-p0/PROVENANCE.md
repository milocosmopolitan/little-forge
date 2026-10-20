# Finished-good candidates (P0, P1, P2) — Sprite Fusion

Generated 2026-10-20 with Sprite Fusion (MCP), text → sprite at 32×32, tag `little-forge`.
One call (12 variations) per item: P0 8 calls (120 credits), P1 7 calls (105 credits), P2 7 calls (105 credits). These are raw downloads; cleaned and
palette-snapped icons are in `../../icons/`.

| Item | Pick | Asset id |
|---|---|---|
| wooden_club | 7 of 12 | 864aed48-5b3b-43da-95c4-cd9dee9f5ad1 |
| wooden_staff | 12 of 12 | f7ee97fb-86fe-4646-bbef-53553757d999 |
| wooden_shield | 8 of 12 | f3cdc1de-3454-430f-9087-72fdd5cb270e |
| kitchen_knife | 5 of 12 | 99217577-877a-47c3-ac93-7e73449b99ce |
| frying_pan | 7 of 12 | c9c32c19-0d53-4aa0-bd08-6ab16a40452f |
| farming_hoe | 5 of 12 | 92eb42bb-9391-49b5-a2e8-c65ed381d9a8 |
| sickle | 1 of 12 | cf244899-ea69-4d10-932f-5a2b82ffc47f |
| woodcutting_axe | 5 of 12 | 0a98f551-5166-45f5-9321-0c87d8997d1a |
| cleaver | 11 of 12 | 44b99ea4-4b88-4de4-b7c3-6025d18252ab |
| hand_saw | 3 of 12 | 585b7ff0-930b-4dc4-8495-61e494d5025b |
| shears | 12 of 12 | 0f7b44da-9d0d-48fd-b320-65d7cecafaf5 |
| fleshing_knife | 3 of 12 | 6c36fdd5-ad0a-4400-b20e-65d0c68023b4 |
| leather_awl | 5 of 12 | b46caf80-a80a-4d9b-9d17-2ae210b10725 |
| leather_punch | 12 of 12 | d0489110-0af8-40bd-a0a3-36396facda57 |
| wood_chisel | 7 of 12 | c06b758e-bd29-4c28-8fca-176ddc38f4a5 |
| dagger | 6 of 12 | b791c67b-ea52-407b-8fbd-b6e10eb9f307 |
| hunting_knife | 6 of 12 | 9c4358f5-aa80-4436-93eb-b5f94b754f80 |
| hand_axe | 9 of 12 | 707f2859-c231-4291-8d5b-3702e6c976ce |
| spear | 11 of 12 | 1d87d0d5-3050-4416-b9c7-be49b21d608b |
| pike | 1 of 12 | a9e604c1-6993-4371-9da2-11f2a5696d62 |
| halberd | 10 of 12 | 08961aad-5536-4cd4-8386-d2236b2b5a5f |
| arrow | 7 of 12 | 46f2233c-1448-44bf-a53d-ee79e90fbf5b |

Why these picks:
- kitchen_knife: the narrow blade, so it matches `straight_blade` and never reads as the cleaver (`wide_blade`).
- frying_pan: the forged handle ends in a hanging hole, as on surviving long-handled iron pans.
- farming_hoe: the blade hangs at a right angle from the haft, unlike the axe.
- hand_saw: the only variation with a visible row of teeth.
- shears: grey loop handles; the all-black loops in other picks would merge into the outline.
- fleshing_knife: drawknife with a curved blade and a handle at each end, so it never reads as the cleaver.
- leather_awl: pear-shaped handle with an iron ferrule.
- leather_punch: round hollow tip, ferrule ring, straight handle.
- dagger: double-edged blade, small guard and ring pommel, dark grip; no fantasy curl on the guard.
- hunting_knife: clear clip point, ash bolster and butt with a wrapped grip between.
- hand_axe: plain grey head on a short ash haft. Superseded: the shipped icon is hand-drawn (`scripts/draw-finished.py`); this raw pick is kept for the record only.
- spear: leaf head with no crossbar or tassel; its large head sets it apart from the pike.
- pike: small head with long iron langets, the thin shaft runs to the corner.
- halberd: spike, axe blade and hook in one head, langet down the staff, no ribbon.
- arrow: three separate heads and fletchings, cord around the middle.

Prompts follow each item's `cue` in `finished-goods-catalog.json`, with the shared style suffix used for
the material and parts atlases ("Lit from the top-left, warm near-black 1px outline, muted medieval
fantasy blacksmith game palette, centered.").
