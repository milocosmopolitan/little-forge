# Village Commerce & Commission Contract (MVP)

> Status: gameplay/content specification. This document does **not** change the Bevy/Rust Foundation epic, platform policy, runtime implementation, or release gates.
>
> Canonical companion documents: [Economy](ECONOMY.md), [NPC relationship](NPC_RELATIONSHIP.md), [Experience](EXPRIENCE.md), [NPC and shops](../npc/README.md), [Crafting](CRAFTING.md), [MVP](../mvp/README.md).

## Product intent

Little Forge is an unobtrusive desktop idle blacksmith game, not an inventory-Tetris, full farming simulator, or shop-production simulator. The player owns the forge; it is a **home/management view**, never a normal NPC merchant store. A living village depends on the blacksmith for repairs and occasional new tools; the blacksmith depends on village shops for materials, furniture, work gear and provisions.

The core village loop is **repair request → accept → source materials (shop or expedition) → repair/craft → deliver within accepted deadline → gold and relationship → NPC usage feedback → understanding/research → tool upgrade → shop unlock**. Automatic display sales are a secondary income stream: buyers leave holding the same item image.

## Distinct shop panels

Use one restrained, thin wood-framed UI shell with a dark, readable content surface and optional merchant artwork. Do **not** clone a generic store panel for every building.

| Destination | Main tabs | Primary actions | What it is not |
|---|---|---|---|
| Blacksmith / Home | Craft, Repair, Research, Display, Commission log | Queue and manage own forge | Not a merchant purchase panel |
| Carpenter | Furniture, Materials, Commissions | Buy wood/handles and forge furnishings; accept tool orders | No simulated woodworking production line |
| Leatherworker | Work Gear, Materials, Commissions | Buy leather and practical blacksmith clothing/accessories | No full outfit simulation in MVP |
| Farm | Provisions, Basic Materials, Commissions | Buy staples, fibers and expedition supplies | No seeds, livestock or farming controls |
| Tavern | Meals, Recruit | Buy restorative meals/drinks or recruit expeditions | No identical three-tool commission progression |
| Outside village | Expeditions | Prepare and dispatch expedition | Not a generic shopping screen |

Furnishings are investments (e.g. display/bench/storage slot improvements); work clothing provides a limited gear effect, with visible cosmetic layering deferred where necessary; food/provisions are recurring consumables. Food/rest/hydration behavior follows [Needs](NEEDS.md). Avoid introducing a player-managed crop or livestock economy.

## Three recurring partner shops

Each partner owns exactly **three signature tool types and one new product unlock** in content MVP. Proposed names (tunable, not immutable balance data):

| Partner | Three signature tools for repair/new manufacture | First unlockable product |
|---|---|---|
| Carpenter | Saw blade, plane blade, woodworking chisel | Improved weapon display |
| Leatherworker | Cutting knife, leather awl, metal punch | Artisan work gloves |
| Farmer | Sickle, hoe head, shovel head | Improved expedition provisions |

A **repair** is repeatable income and relationship maintenance. A **new tool commission** is milestone progression and may unlock a shop product. A shop can both purchase tools from the player and sell the player useful materials/equipment. Supplier stock and production capacity need not be simulated: shop items unlock by explicit quest/relationship flags.

## Scheduling and anti-pressure contract

- **Confirmed:** village events arise approximately every **30–60 minutes**; this is not a physical item wear simulation.
- **Proposed interpretation:** a *shared village commission channel*, not one independent 30–60 minute timer per shop. Keep generation jitter/weights data-driven.
- Distinguish repair requests, new tool commissions and **eligible usage feedback**. Feedback requires a prior delivered tool and is not a random fake repair request.
- A request that has not been accepted has **no running delivery deadline**. A deadline is attached to an accepted commission; proposed starting values are repair 2h, new tool 4h. Optional urgent orders are opt-in.
- Unaccepted requests may expire without relationship loss. Failure after acceptance can affect reputation **once**, with no repeated penalty upon restart. Keep pending/active queue caps tunable (initial proposals: 3 / 2).
- Do not force windows to foreground, interrupt work or spam notifications; use an inbox, batch summaries and non-intrusive world animations.
- Offline, suspend/resume and catch-up policy is owned by the Foundation time model; do not invent a second clock or silently determine expiry semantics here.
- **Never** implement per-NPC durability, physics, precise work cycles, wear ticks, complex supply chains or generated daily production rates.

## Relationship, pricing and feedback

- Each partner relationship is a bounded 0–100 value. Successful on-time deliveries raise it; accepted failed deadlines may lower it. Numerical deltas, discounts and thresholds remain balancing values.
- Reputation affects **future** purchase quotes, perks and eligibility. Prevent punitive price spirals or locking basic survival inputs. Accepted commission terms stay snapshotted.
- For each partner, the first unlock requires an appropriate new-tool milestone and may additionally require relationship rank (e.g. relationship 60 is only a tuning proposal).
- Delivering a tool creates a **feedback eligibility** record. A delayed report may provide crafting experience, related tool/material insight and/or research experience or a clue. No simulated equipment lifespan.
- Award feedback/commission rewards **once per immutable event/delivery ID**. Repeated feedback should have limited or diminishing research rewards; a new insight must not be farmable by reopening a panel.
- Tool knowledge can transfer into weapons: curved sickles suggest curved blades; carpenter precision cutting suggests dagger edges; leather punches suggest fine metal shaping.

## Tutorial and scope

Tutorial commissions are scripted, not gated behind the first 30–60 minute scheduled event. Three initial crafts can guarantee the first relevant insight level; ordinary discovery probabilities remain tunable. Content MVP proves **9 signature tools, 3 shop unlocks, modest feedback/research examples, and four differentiated shop UIs** before expanding to the 60-weapon catalog, cards, art packs, multiplayer or elaborate NPC progression.

## Acceptance scenarios (gameplay contracts)

1. Accept a carpenter repair; purchase missing materials; fulfill; pay once and increase relation once.
2. Miss an accepted order; apply the configured relation change once; restart without duplicate loss.
3. Ignore an offered order; allow quiet expiration with no penalty.
4. Supply an upgrade tool to leatherworker; unlock just the associated work gear when prerequisites are met.
5. Receive feedback only for a previously completed delivery; gain insight/research exactly once.
6. Validate 30–60 minute village-level request scheduling with fake clock and deterministic seed; no NPC durability fields or wear simulation.
7. Visit home from village map and land on forge management rather than a shop checkout.
8. Verify carpentry furniture, leather gear, farm supplies, tavern meals/recruitment have distinct primary actions and no crop/livestock management.
9. Confirm no active game event steals focus from the user's desktop and no missed notification blocks progression.

## Coordination

The active Bevy/Rust Foundation epic owns code, engine choice, OS-specific QA and test scaffolding. This document defines **content contracts only**. Foundation consumers can implement these rules incrementally without importing speculative feature/asset expansion into its acceptance criteria.
