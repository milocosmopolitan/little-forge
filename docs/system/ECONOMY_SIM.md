# Headless economy simulator (#131)

`crates/economy-sim` runs Little Forge's economy with no Bevy and no window. It is the base for the
automation epic #130.

The rule it exists to keep: **every state change goes through `State::command`, `State::advance`
or `Save::catch_up`**, the same transitions the desktop uses. There are no simulator-only rules.

## Run it

```sh
cargo run --release -p economy-sim -- run crates/economy-sim/scenarios/player-day.json --out day.json
cargo run --release -p economy-sim -- matrix crates/economy-sim/scenarios/matrix-v0.json --out out/v0 [--threads N] [--full]
```

- **`run`** writes one report (JSON).
- **`matrix`** writes:
  - `summary.csv`, one row per run;
  - `failures.json`;
  - with `--full`, every report under `runs/`.
- **Exit status** is 1 when any run breaks an invariant or fails, so CI can gate on it.

## Scenario file

| Field | Meaning |
|---|---|
| `name`, `seed` | Label, and the seed used for the visitor RNG. The same seed always gives byte-identical reports. |
| `duration_secs` | Wall-clock time simulated, including time offline. |
| `step_secs` | Online step. The driver acts, then the state advances. Default 1. |
| `sample_every_secs` | Metrics sample interval. Default 600. |
| `start` | See below. |
| `driver` | See below. |
| `offline` | `[{at, secs}]`. The save is serialised and decoded, which must round-trip, then settled with the game's capped `Save::catch_up` (8 h). |

**`start`** is `preset` plus optional `gold`, `ore` and `staples` overrides:
- `fresh` is a new game, as the desktop creates it.
- `legacy-dagger` knows the dagger recipe from the start.
- `qa-smith` is `weapon::qa_smith()`.

**`driver`** stands in for the player. None of these is game automation; automation policies (#133–#137) will be added as drivers that issue the same commands.
- `idle` does nothing.
- `script` sends `{at, command}` unit commands at fixed times.
- `player` is a diligent scripted player with these settings:
  - toggles `orders`, `research`, `craft`, `buy_ore`, `buy_food`;
  - `max_queue`;
  - `gold_reserve`;
  - `act_every_secs`, how often it pays attention.

**Matrix file** is a `base` scenario, `seeds` (a list or `{from, count}`), and `vary`, which maps a JSON pointer to a list of values. Every combination × every seed is one run, and runs execute in parallel. Results come back in case order, so the output does not depend on thread timing.

## Measures

Each report has:
- samples of the core state over time;
- a gold **ledger by source**: `cmd:<Command>`, `sale` (visitor sales in `advance`), `offline` and `advance:other`;
- accepted and refused commands, and event counts;
- queue waiting time and the longest wait, meaning jobs were queued but none was at the bench;
- each offline settlement, with its uncredited seconds;
- failures.

## Invariants (checked after every driver action and every step)

- `State::validate()` passes.
- `tick`, `crafted`, `sold`, `event_id` and `command_id` never decrease.
- Each order is delivered (paid) at most once.
- The gold ledger balances: start gold + the sum of the ledger = end gold.
- Offline: the save round-trips byte-for-byte, and the tick moves by exactly the settled time.

Tests, in `cargo test -p economy-sim`, cover:
- determinism;
- that one crafted dagger sells for 12 gold within the hour, as in forge-qa's craft-and-sale;
- that offline settlement equals staying online;
- the 8 h offline cap;
- that a scripted player finishes the first loop (Chef → research → daggers → Farmer);
- matrix expansion;
- a 24-run smoke matrix with zero failures.

## First findings (v0 matrix, 2026-10-20)

`matrix-v0.json` is 48 policy combinations × 10 seeds, one simulated day each. It ran 480 runs in about 8 s on 10 threads, with zero invariant failures.

| Player policy | Median gold change per day |
|---|---|
| Orders only (Chef + Farmer, no ore purchases) | +44 (two 24-gold rewards − 4 research) |
| Orders + craft, no ore purchases | +80 (10 starting ore → 3 daggers, then stops) |
| Orders + craft + buy ore, online all day | **+28,520** (≈3,570 daggers) |
| The same, with 8 h of a 24 h day offline | +19,060 |

**The current rules contain an unbounded gold loop.**
- Two ore cost 4 gold and make a dagger that sells for 12 gold, and a visitor arrives every 18–30 s.
- Buying ore and crafting therefore compounds without limit.
- This is the runaway that the village order loop (#77, #109) and automation epic (#130) must remove before any automation ships.
- It is an input for #132 (vendor stock, prices and resale) and #138 (balance thresholds).
- These are measurements of today's draft numbers, not a balance proposal.

## Crafting chain coverage (2026-10-20, with the crafting engine)

New scenarios:
- `chain-day`: a fresh game making a kitchen knife, club, staff and pan from bought materials.
- `chain-stock-out`: 2 days with 5,000 gold, making knives until a vendor runs out, with 8 h offline.
- `baseline-ore-dagger-runaway`: the historical regression baseline.
- `matrix-crafting.json`: start gold × buying materials × goods to make × offline, 5 seeds, 180 runs.

The scripted player now follows the recipe data. It buys a missing input if a vendor sells it, otherwise makes it first. Every step uses the same `Buy` and `QueueRecipe` commands as the UI. The report adds production by item, end inventory and end vendor stock.

| Measurement | Result |
|---|---|
| Ore → dagger baseline (1 day), main 584a58d vs crafting branch | **Identical:** gold 0 → 28,446, 3,564 crafted, 3,560 sold, same samples. The inventory change did not alter the legacy loop. |
| Fresh game, Chef chain, no purchases, 0 gold | Chef delivered using the new-game kit (all 180 matrix runs deliver). |
| Keep one knife, club, staff and pan (1 day) | −14 gold of materials. 13 jobs: 4 ingots, 2 blades, 2 handles, 1 pole, and 1 each of knife, club, staff, pan. |
| Keep knives without limit (2 days) | Pine resin (farm, 30) ran out after 30 knives, and production stopped cleanly with no queue deadlock. |
| Invariant failures | 0 in all 180 crafting runs and 480 v0 runs. |

What this shows:
- **The chain is sound but closed.** Finished goods have no buyer yet: the showroom sells daggers only, pending D05. So crafting is pure cost, and the only gold loop is still the dagger loop.
- **Vendor stock is the only brake on materials.** With no restock rule (D02), purchases fail permanently after stock-out.

**Performance:** the v0 matrix takes 20 s, up from 8 s. Each simulated second now clones the inventory and vendor-stock maps. That is still ≈0.4 s per simulated day per run.
