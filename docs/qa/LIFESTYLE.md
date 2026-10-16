# Lifestyle loop — epic #18

The product now exposes queued crafting, bounded fatigue/hunger/thirst, automatic rest, staple food, free well water, and explicit farmer purchases. The pure Rust state owns all inventory, work and recovery; Bevy displays state and routes commands. No background food purchases or premium consumption occur.

## Controls

Tab opens the management panel with clickable actions and all 12 pantry slots. Q queues a dagger (reserving 2 ore); C starts one immediately when no job is active. B buys 2 ore for 4 gold; F buys one staple; E eats a staple; W drinks free water even at zero gold; R rests. Existing F8/Ctrl+Alt+F8/Esc recovery, D decorations, T topmost and F12 diagnostics remain. The ambient strip displays fatigue, hunger, thirst and current activity without stealing focus. Sleep dims the idle smith; confirmed working state drives hammer animation. Eating and drinking have distinct activity labels and persisted short timers; no new art sheets are invented.

## Deterministic clock and provisional balance

`assets/needs.json` is a **baseline tuning candidate, not final approved balance**. It is loaded for new games and persisted in each save so patches do not silently change an in-flight simulation. Need values are integer hundredths of a percent, bounded 0–10000. Hunger/thirst accrue each eligible simulation second, including idle and capped offline time. Work alone accrues fatigue, multiplied by unmet needs. Work speed is full below 25% fatigue, then decreases monotonically to 25% at exhaustion. Work is stored as integer thousandths of effective seconds; hunger and thirst affect fatigue rather than directly affecting speed.

At the start of a work second, fatigue at the sleep threshold pauses remaining work. Sleep completion resets fatigue; eligible staples are consumed once if hungry, followed by free well hydration if needed, then work resumes. Empty pantry emits a quiet diagnostic event and resumes without a menu or hard lock. Food/drink do not heal fatigue. Premium stacks are preserved. Pantry insertion and currency changes are atomic; command IDs reject duplicates. Craft completion precedes visitor purchase at tied timestamps.

One-second boundaries make arbitrary `advance()` partitions equivalent. Offline catch-up uses exactly the same clock and the existing eight-hour cap; clock rollback does not repeat time. There are no order-deadline mechanics in this epic. Queue capacity is a provisional 32 jobs; pantry has 12 slots with provisional 20-item stacks. Seeded food is three staples.

## Save migration

Schema 3 retains the schema-2 life contract and adds persisted village commerce; schema 1/2 migrate without resetting gameplay. The life contract persists the action timer, remaining work, queue, pantry, need values and configuration. Loading schema 1 initializes healthy needs and three staple meals, preserving gold, ore, stock, counters, IDs, RNG and pending craft progress. It upgrades in memory and writes a normal journal generation; no reset. Corrupt-slot recovery, future-schema refusal, process locks and the offline high-water timestamp remain. Schema 2 cannot be read by old builds: keep the previous save copy when downgrading.

## Reproduction

- `cargo test --locked --workspace`
- `cargo run --locked --release -p forge-qa -- life-loop`: complete loop trace, then three days of repeated work/rest/food/sales with scripted player purchases and periodic save round trips. Production never performs these scripted purchases.
- `python scripts/capture-lifestyle.py target/release/little-forge` (Windows: `.exe`): actual Bevy ambient/management captures, sleeping/eating/drinking snapshots, event trace and final-state assertions. Uses isolated QA saves, elevated initial needs and an eight-second QA sleep. Rendering uses a fixed eligible clock; production timers are unchanged.
- Linux: `xvfb-run -a -s '-screen 0 1280x800x24' python scripts/soak-lifestyle.py target/debug/little-forge --seconds 1800 --output qa-output/life-soak`. Uses the normal app, production monotonic clock, disposable save, real input events, periodic Tab/queue/purchase and one manual rest. Reports CPU/RSS, journal ticks and final state. This is Linux software-renderer evidence, not Mac or NVIDIA GPU measurements.

The 30-minute desktop soak is opt-in through workflow_dispatch (`soak: true`), so routine PRs do not all pay a 30-minute delay. Run it for lifecycle acceptance and relevant timing/persistence changes. Windows WARP capture has a 600-second per-scene ceiling: the software adapter renders the expanded scene at about six frames/second in CI; this is not native GPU performance. This changes only the harness timeout, not game timers or image tolerance.

CI retains the unchanged Foundation pixel references and captures the added lifecycle separately. macOS/Windows capture jobs use their native renderer on CI hosts; this does not qualify owner hardware, Spaces, mixed DPI or multi-monitor behavior. Raw artifacts include environment logs, traces, screenshots and the soak report. No golden images are automatically updated.

Foundation hardware limitations were accepted by the owner on 2026-10-16 in closing comments on #1/#3/#4/#10/#11. They remain untested, not passed, and should be revisited before broad public release. This implementation does not require the owner to obtain additional hardware.
