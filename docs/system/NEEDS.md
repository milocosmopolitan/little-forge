# Needs — fatigue, hunger, thirst and recovery

[Economy](ECONOMY.md) · [Crafting](CRAFTING.md) · [Consumables](../items/CONSUMEABLES.md) · [Architecture](../../ARCHITECTURE.md)

## Accepted player direction (2026-10-15)

The **blacksmith** has three independent ongoing needs: fatigue, hunger and thirst. This is a cozy idle work/rest loop, **not** a survival, mortality or real-time combat simulation.

| Need | Increase | Recovery | Effect |
|---|---|---|---|
| Fatigue (0 rested → 100 exhausted) | Performing work, accelerated by unmet hunger/thirst | **Sleep/rest** | Higher fatigue reduces effective crafting speed |
| Hunger (0 full → 100 hungry) | Passage of active game time | **Food** | Higher hunger increases fatigue accumulation |
| Thirst (0 hydrated → 100 thirsty) | Passage of active game time | **Drinks** | Higher thirst increases fatigue accumulation |

Hunger/thirst do **not** directly reduce crafting speed in the initial model; they increase fatigue, which slows work. Keep all rates, thresholds and caps data-driven rather than hard-coded. Eating/drinking does not directly restore fatigue; sleep does. Buffs may exist but **food and drinks always serve their baseline restorative purpose even without buffs**.

### Supply and economy

- The **well always provides free water**. Never soft-lock hydration behind currency, stock, travel or NPC relationship.
- The **farmer sells affordable basic food**, making recurring trade useful.
- The **tavern sells premium meals and drinks**, optionally with temporary bonuses; ordinary food/drink still restores needs.
- Do not force cooking, farming, food spoilage, nutrition macros or NPC survival simulation into the MVP.
- **Confirmed idle automation:** when fatigue reaches the configurable sleep threshold, the blacksmith automatically sleeps/rests. On waking, the blacksmith automatically eats from a dedicated food pantry if hungry and food is available, and hydrates using an available drink or free well water as needed. Preserve explicit player controls for rest and consumables. Never automatically spend gold or consume a premium buff meal unless separately permitted.
- An inventory item may have both `hunger_restore` or `thirst_restore` and an optional independent buff. Apply costs and recovery atomically.
- Sleep is an activity state/timer rather than a full day/night simulation requirement; it should be visually represented.

## Initial balance model (proposal; not approved numbers)

Use saturating values in [0,100] to simplify UI and save logic.

```text
hunger += hunger_rate * eligible_elapsed
thirst += thirst_rate * eligible_elapsed
fatigue += (base_work_fatigue_rate * work_intensity
            * hunger_multiplier(hunger)
            * thirst_multiplier(thirst)) * eligible_elapsed
effective_craft_duration = recipe_base_duration / work_speed_multiplier(fatigue)
```

Constraints: hunger_multiplier and thirst_multiplier >= 1, monotonic; work_speed_multiplier in (0,1] and non-increasing with fatigue. No zero speed, death or infinite negative spiral. At maximum fatigue, stop or gently cap slowdown according to future UX decision; the system must remain recoverable with free water, affordable food and sleep. Avoid fine-grained simulation of each hammer swing.

**Do not select rates, buff percentages, sleep durations, hunger thresholds or price points without balancing work.** Prior proposed 30–60 minute request cadence does not define metabolism.

## Time and idle safety

- Needs progress only according to the **defined eligible simulation/activity clock**, not directly from wall-clock render frames; no penalty based on dropped FPS.
- On app sleep, shutdown, sleep/wake and offline catch-up, use the same capped time policy as crafting and orders. No starvation spiral while absent.
- Fatigue modifies *future remaining work* at explicit simulation event boundaries; do not retroactively revoke already finished items or repeat rewards.
- Sleeping/rest should not fail accepted orders without a clearly visible deadline policy; obey activity-time vs offline-time rules defined in ARCHITECTURE.
- Prevent repeated food/drink/sleep commands from double-consuming goods or granting duplicate recovery.

## UX requirements

- Compact ambient UI: unobtrusive three-state indicator with tooltips (fatigue/hunger/thirst); no intrusive alarms or forced focus.
- Expanded management UI: numeric values, recovery actions and clear impact on crafting speed; show free-well action.
- Explain why work slowed and how to recover. Avoid requiring micromanagement every few minutes.
- Optional idle animation: drink water, eat, rest; these are presentation of confirmed simulation events.

## Storage, test requirements and pending tuning

Persist values, current action/rest timer, last eligible simulation time, pending consumable effects and stable event IDs. Add migrations from saves that lack needs values (initialize to healthy defaults, not exhausted).

Tests: bounded invariants; monotonic fatigue rates; sufficient rest/food/water recovery; free water even at 0 gold; tavern buffs not required for recovery; identical time-step partition outcomes at event boundaries; no forced focus; offline cap and idempotent consumption; no soft lock at max needs.

### Still open — explicitly NOT decided

- Base rates and threshold curves.
- Exact sleep duration, interruption policy and fatigue threshold (automatic sleep itself is confirmed).
- Food/drink quantities, prices, buff tables.
- Whether needs accumulate while idle (not crafting), during exploration screen or while app closed.
- Pantry item priority, premium-food opt-in, automatic drinking thresholds and notification frequency (automatic food use after waking is confirmed).
- Whether additional comfort/furniture effects exist.

## Auto-rest and food pantry — accepted (2026-10-15)

The blacksmith resumes normal automatic work after waking and attending to hunger/thirst. No player prompt or forced focus is required. If food is unavailable, do not deadlock or repeatedly open menus: the blacksmith can continue at reduced effectiveness, and the player sees a quiet low-stock notification. Free water is always obtainable independently of stocked drinks.

Provide a **dedicated food pantry** separate from material inventory and weapon display. Its capacity should be generous enough for idle use, without inventory-Tetris or per-slot shape constraints. **Initial proposal:** 12 stackable food/drink slots, each stack holding a configurable amount; pantry capacity is not yet a final balance value. Distinguish cheap staple food (eligible for unattended eating by default) from premium buff meals (not automatically spent by default).

Deterministic sequence: eligible work time accrues needs → if fatigue threshold reached, finish/pause work at a well-defined boundary and enter sleep → rest finishes → consume eligible pantry food if hunger warrants → hydrate if thirst warrants (free well fallback) → resume queued work. Do not trigger double consumption, double completion or unintended order deadline failure on restart. Keep the exact time/threshold formulas tunable.
