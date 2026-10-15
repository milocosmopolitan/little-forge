# Decision Audit — conversations vs current docs (2026-10-15)

This audit distinguishes **confirmed user choices**, **implementation candidates**, and **still-open questions**. Do not convert proposals into fixed economy values. Implementers should defer to the latest explicit user decision.

## Previously discussed and accepted, but not fully reflected in current docs

| Topic | Decision | Documentation follow-up |
|---|---|---|
| Engine | **Bevy / Rust** preferred; agent-first development, no Godot/GDExtension | README, ARCHITECTURE, MVP still mention Godot/POC; align in independent implementation PR or docs rebase |
| OS | macOS first, Windows Shadow PC manual desktop QA, Linux later | [PLATFORMS](PLATFORMS.md) is current QA policy |
| UX | Bottom floating ambient window with management opening upward; side-view 2D | Window dimensions 960×180→960×540 are **candidates**, not performance facts |
| QA | Automated headless unit/ECS/visual tests; manual Shadow PC release acceptance; no disposable POC | Foundation epic #1, issues #2–#11 |
| Direct control | Blacksmith not freely walking map or fighting in MVP; choose destinations in management map, dispatch expeditions | Keep future direct travel/combat out of MVP |
| Social | No realtime multiplayer/chat in MVP; optional async friend trade later | Server/ownership verification deferred |
| Sales | NPC visits, purchases displayed goods and **leaves holding the same weapon icon**; suggested automatic pricing with optional player override | Economy doc calls pricing undecided: **policy is decided; numeric curve is not** |
| Progression | Skill XP on production; probability-based insight with beginner guarantee by three tutorial crafts; research discoveries separate from unlock | Curves, targets, thresholds remain tuning |
| Card direction | Duplicate-card merging; perk persistent vs enhancement consumed is design proposal; timer/card collection later | Do not force cards into Foundation |
| Asset research | 60 physical weapon types and 3 core visual tiers proposed for long-term art; 96px per logical slot, variable grid, art packs/uniques | Do not confuse with initial content minimum |
| Living economy | Farmer, carpenter, leatherworker shops send repairs, supply items and give usage feedback; no item durability simulation | Existing NPC/economy/crafting docs carry most of this |
| New needs | Fatigue sleep, hunger food, thirst drinks; hunger/thirst accelerate fatigue; fatigue reduces crafting speed; well always free; farmer low-cost food, tavern premium | [NEEDS](NEEDS.md), consumables/economy/crafting cross refs |

## Deliberately open questions (do not silently decide)

1. Numeric need rates, food/drink recovery, rest duration and auto-use.
2. How needs advance while app closed or work not active.
3. Specific ticket/order timing, offline clock and deadline semantics.
4. Research XP/probability curves and exact tier distribution.
5. Expedition party recruitment and equipment ownership.
6. NPC relations discount curve and reward amount.
7. Actual render resolution, OS compatibility/performance baselines; measured acceptance required.
8. Precise NPC animation frames and placement, map overlay assets and click polygons.
9. First Steam content counts beyond agreed minimal foundation; monetization undecided.

## Scope guard

**Foundation now:** Bevy shell and window recovery, minimal living forge NPC sale, simulation boundary, save/offline QA, asset integration.

**Later product content:** 3×3 repair tools, shop unlocks, food/drink/rest, village UI, research, exploration.

**Later expansion:** 60-weapon catalog, 3-tier asset generator, art packs, unique items, card systems, online friends/trading and chat.

Introducing needs should not force full survival simulation into the Foundation spike. Implement incremental data/UI contracts, then fit recovery to the economy milestone. Update README/ARCHITECTURE/MVP only after reconciling concurrent Astra branch changes.
