# Workshop atlas v2: 1:1 characters, redrawn props, owner layout

Supersedes the 3/4 character scale in [v1](WORKSHOP_ATLAS_V1.md). At 3/4 the props fit the room,
but nearest reduction dropped character detail. The owner chose 1:1 characters and props drawn
at that density.

## Scale

- Smiths, the Farmer and the legacy visitor render at the canonical 128×160 canvas, foot pivot
  (64,140), `interior::CHARACTER_SCALE = 1`. Smith pages (`atlas-v2/{sex}.png`) are untrimmed
  canonical frames. They are not resampled.
- Props are **re-reduced from the large source art** at 4/3 of their v1 size. They are not
  upscaled from v1. Each native pixel takes the dominant colour of the inner 60% of its source
  block. Alpha is binary (`scripts/export-workshop-atlas.py`).

## Sources

| Prop | Source | Native art |
|---|---|---|
| Furnace | Owner's front-view furnace (`little-forge-assets/interior`, Oct 19 12_20_01) | 112×175 |
| Anvil, tub, desk, counter | `generated-atlas-large.png` from `little-forge-atlas-v1.zip` | 64×53, 75×37, 171×91, 128×80 |
| Display shelf | Generated for this change (GPT Image 2.5), styled on the furnace and counter; black background cut out and 1px outline redrawn | 146×81 |
| Coal bin ×3 | Generated from the owner's reference crop: full / half / empty of one barrel in a single image, so the states align | 44×61 each |

The furnace is drawn in front view. Its chimney top sits above the viewport (origin y = -30),
so only the hood and the lower chimney are visible.

## Layout (owner reference)

Counter with the wall display shelf behind it on the left. The furnace stands against the back
wall (floor y = 146). The anvil sits in front of its left half and the quench tub in front of
its right half (floors y = 170). The research/salvage desk is on the right.

| Fixture | Grid origin | Floor |
|---|---|---|
| display-shelf | (99, 20) | 101 (wall, behind every walker) |
| display (counter) | (96, 64) | 160 |
| coal-bin | (440, 77) | 138 (back wall, left of the furnace) |
| anvil | (464, 106) | 170 |
| forge | (520, -30) | 146 |
| quenching-tub | (592, 106) | 170 |
| workbench | (768, 64) | 160 |

Smith rest point: (360, 166). Walks between furnace → anvil → tub stay short, about 165px at
most. Longer walks consumed whole work stages: the D01 QA failed at 289px and 314px.

`coal-bin-half` and `coal-bin-empty` are exported with the same origin and size but are not in
the catalog. The game has no fuel stock yet, so the runtime always shows the full bin. A fuel
rule can later swap the image.

The display shelf opens display management, like the counter. Four runtime display slots sit
on its empty pegs (`shop_scene::SHELF_SLOTS`). Icons are 14px on the shelf and 20px in hand.
The customer stops in front of the counter at foot y = 168.

`interior::tests::runtime_feet_match_exported_layout` checks that runtime feet equal the
exported `layout.json`, so the offline composites show the real positions.

## Open review

- While forging, the anvil in front hides the smith's lower legs. This follows the reference.
- The display shelf covers part of the background's left window.
- Generated shelf art is a candidate. Its painted tools are decoration. Only runtime slots
  reflect stock.
