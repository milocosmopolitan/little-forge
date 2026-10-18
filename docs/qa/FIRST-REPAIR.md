# First farmer repair

The first commission is available immediately in new and migrated games. It is a
single introductory order, not recurring requests or the three-order research tutorial.

## Play

Start or continue a game. Open **Manage → Farmer** (O opens Chef in Alpha 01).
Accept the sickle repair (**A**), then queue it (**S**). Earlier dagger jobs finish
first, followed by the repair and any later jobs in the same FIFO workbench.
Sleep, fatigue and background progression use the existing simulation.
When ready, choose **Deliver** (**L**) to receive gold and farmer favor.

The compact forge shows the order step. The farmer waits for acceptance and
collection, nods on acceptance and thanks the player on delivery. Animations cannot
award rewards. The entrusted sickle is order state, never sale stock; no new sickle
sprite is generated or claimed here.

## Provisional first-play balance

- **2 ore**, consumed once on enqueue, like existing dagger jobs.
- **20 effective seconds**, slowed by fatigue and paused by sleep.
- Deadline: **2 simulation hours from acceptance**; unaccepted offers never expire.
  Offline progression retains the existing **8-hour cap**.
- Finish by the deadline, inclusive: **24 gold and +5 farmer favor**, on delivery.
- Finish late: **12 gold and no favor gain**, on delivery, no further penalty.
- Completion time fixes the reward. Reading or delivering later cannot reduce it.

These are first-play values, not final economy approval. Favor is recorded on this
one order. Broader relationships, research, repeat orders, cancellation and queue
reordering are outside this change.

## Persistence and checks

Save schema **5** retains typed FIFO jobs, active work kind, order phases, timestamps
and settlement, and adds Alpha 01 progression. Schemas 1–4 migrate without recharging
existing queued daggers. Older binaries refuse schema 5; returning to an older build
requires an older save.
Malformed current-format order/work metadata is rejected rather than reset.

Core regressions cover mixed FIFO jobs, customer-item isolation, invalid/duplicate
actions, resource shortage/full queue, inclusive deadlines/delayed delivery,
sleep/offline completion, journal restart, overflow rollback, deterministic time
partitioning, legacy migration and missing metadata.

```sh
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo run --locked -p little-forge -- --safe-window --save-dir /tmp/forge-repair-play
```

Actual-app screenshot fixtures: `--qa repair-offered|repair-working|repair-ready|repair-delivered`
with `--screenshot PATH`. `repair-input` starts an isolated offer with a longer input
window. Fixtures never load or write a user's gameplay save. Native platform
acceptance remains distinct from deterministic core tests.
