# Crafting engine v1: materials → parts → finished goods (#74, #119 stage 1, #132)

This replaces the Ore → Dagger-only economy with a data-driven processing and assembly chain. It uses
the approved asset catalogs: 40 materials, 23 parts and 22 finished goods.

Status: core and content are implemented and tested. The desktop shows the Chef's parts and part buttons in Orders. The full Crafting screen is still #110 / #64.

## Owner decisions (2026-10-20)

| Decision | Choice |
|---|---|
| Inventory (#119 / D05) | Stage 1: one canonical **stackable** inventory keyed by catalog ID (materials, parts, ordinary finished goods). Crafted weapons with quality and history, and showroom goods, keep their stores until the instance stage. |
| Legacy aggregate stock | `ore` → `iron_ore`, `commerce.timber` → `ash_log`, `commerce.leather` → `deer_leather`. These are the parts catalog's base materials. |
| Numbers the docs do not give | Draft test data in `assets/crafting.json`, validated with `crates/economy-sim`. |
| Chef's kitchen knife | Assembled from a blade and a wooden handle (the new chain). Saves that already queued the old ore-only knife finish it under the old rule. |

## Data: `assets/crafting.json` (version 1, `draft-test-data`)

**Recipes** (22, all available from the start; owner decision 2026-10-20 added the six P0 tools: wooden shield, hoe, sickle, woodcutting axe, cleaver, hand saw). Inputs are debited when the job is queued; the output enters the inventory once, when the job completes.

| Recipe | Inputs → output | Seconds | Station |
|---|---|---:|---|
| `smelt_iron_ingot` | iron_ore ×2 + charcoal ×1 → iron_ingot | 10 | forge |
| `forge_straight_blade` | iron_ingot → straight_blade | 8 | anvil |
| `make_wooden_handle` | ash_log + pine_resin → wooden_handle | 6 | workbench |
| `make_wooden_pole` | ash_log → wooden_pole | 6 | workbench |
| `assemble_kitchen_knife` | straight_blade + wooden_handle → kitchen_knife | 20 | workbench |
| `carve_wooden_club` | ash_log → wooden_club | 6 | workbench |
| `make_wooden_staff` | wooden_pole → wooden_staff | 4 | workbench |
| `forge_frying_pan` | iron_ingot ×2 → frying_pan | 10 | anvil |
| `cut_wooden_plank` | ash_log → wooden_plank ×2 | 6 | workbench |
| `forge_metal_rivet` | iron_ingot → metal_rivet ×8 | 6 | anvil |
| `cut_leather_strap` | deer_leather → leather_strap ×2 | 4 | workbench |
| `forge_hoe_head` / `forge_sickle_blade` / `forge_wide_blade` / `forge_saw_blade` | iron_ingot → part | 8 | anvil |
| `forge_axe_head` | iron_ingot ×2 → axe_head | 10 | anvil |
| `assemble_wooden_shield` | wooden_plank ×2 + leather_strap + metal_rivet → wooden_shield | 10 | workbench |
| `assemble_farming_hoe` / `assemble_woodcutting_axe` | head + wooden_pole → tool | 6 | workbench |
| `assemble_sickle` / `assemble_cleaver` / `assemble_hand_saw` | blade + wooden_handle → tool | 6 | workbench |

Notes on the table:
- **Quantities** come from `parts-catalog.json` and `finished-goods-catalog.json`. A content test fails if a recipe's inputs differ from the catalog's own recipe line.
- **Draft values:** the smelting ratio, all seconds, prices and stock.
- **Knife time:** 20 s matches the existing approved Kitchen Knife time in `progression.json`.
- **Station** picks the smith's animation stage: forge = heating, anvil = hammering, workbench = the bench.

**Vendors** (`Buy { vendor, item, quantity }`, at most 99 per purchase). **Stock is unlimited for now** (owner, 2026-10-20). Finite stock stays supported: set a number on an offer, and its remaining count becomes saved state (`commerce.vendor_stock`).

| Vendor | Offers (price) |
|---|---|
| Blacksmith supply (the existing Buy-ore channel, no new NPC) | iron_ore 2 (`BuyOre` = 2 for 4g, unchanged), charcoal 1, coal 2, iron_ingot 7 |
| Carpenter: every wood material | ash 3 (was "Seasoned timber"), pine 2, oak 3, birch 2, walnut 4, cherry 4, mahogany 6, yew 5 logs; wood_branch 1, pine_resin 2, charcoal 1; wooden_plank 2, wooden_pole 4, wooden_handle 5 |
| Leatherworker: every hide and leather | deer_hide 3, boar_hide 3, wolf_pelt 4, bear_hide 6, tiger_pelt 9; deer 5 (was "Leather straps"), boar 5, wolf 6, bear 9, tiger 14 leather; leather_strip 3, leather_strap 4 |
| Farm | wood_branch 1, pine_resin 2 (food stays in the pantry shop) |

**New-game kit (draft).** A new game starts with iron_ore 10 (as before) plus charcoal 1, ash_log 1 and pine_resin 1. That is exactly one kitchen knife's purchased inputs. Without it the first commission would need 6 gold that a new player cannot have. The existing "normal start can fund research" test proves the start still works. Migrated saves get no kit.

## Code

- **`forge_core::items`**
  - `Id` is an inline, `Copy` catalog ID, serialised as a string.
  - `Registry` is built from the three catalogs. Entries "on hold" (`shield_core`) are skipped.
  - `Inventory` uses checked add/take, never stores zero stacks, and caps each stack at 1,000,000.
- **`forge_core::crafting`**
  - `Book` loads and validates the recipes, vendors and kit.
  - `QueueRecipe`, `Buy`, `quote_recipe` and `quote_buy` (pure), `finish_recipe`.
  - `make_chef_parts` is a fixture helper that drives real commands.
- **Jobs:** `WorkKind::Recipe(Id)` and `WorkKind::ChefOrder`. The Chef's knife is held for the Chef, never stocked.
- **Event:** `EventKind::Produced { item, qty }`.
- **Schema 8:** an explicit decoder moves `ore`, `timber` and `leather`. The old remaining timber/leather shop stock is dropped, because those offers are now unlimited. No serde default fills the new inventory.

## Not done here (needs a decision or another issue)

- **Selling new goods.** Kitchen knives, clubs, staffs and pans enter the inventory but cannot be displayed or sold. The showroom accepts `Good::IronDagger` only, and showing new archetypes needs D05's container compatibility rules.
- **Material variants.** Log species are fixed to ash, and the pole has no length attribute. Both wait for attributes in the item model.
- **XP for new recipes.** Technique XP for the six starter items is #75. Only the Chef's knife grants XP, through the existing recipe 0.
- **Research gates.** Every v1 recipe is `starter` (owner confirmed 2026-10-20). Weapon recipes behind research come with #75 / #76.
- **Crafting and shop UI.** The new offers and recipes exist in the core, but the UI only shows the Chef part buttons, the ore button and the old timber/leather buttons. That is #110 / #64 / #65.
- **Cancellation, refunds and salvage:** none, by rule.
