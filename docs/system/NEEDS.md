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
- Optional auto-consumption and resting/sleep schedules are *proposals*, not agreed behaviors. Offer safe manual actions first; design future automation to be predictable and configurable.
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
- Sleep duration, interruption and whether rest automatically starts at fatigue cap.
- Food/drink quantities, prices, buff tables.
- Whether needs accumulate while idle (not crafting), during exploration screen or while app closed.
- Auto-consumption preferences and notification frequency.
- Whether additional comfort/furniture effects exist.
