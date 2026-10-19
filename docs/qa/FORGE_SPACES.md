# Forge Spaces QA — Epic #86

> **Screen audit supersession (2026-10-19 UTC):** Historical captures below qualify their original full-height/exterior layout only. The new lower180 live-indoor contract is not implemented. Research DaggerStudy is real now; salvage remains unavailable. Current native acceptance is listed in the new audit matrix. [Current audit](SCREEN_ACCEPTANCE_MATRIX.md).

Contract: [docs/system/FORGE_SPACES.md](../system/FORGE_SPACES.md). This page separates
**independent visual checks** (preview fixtures, no simulation) from **actual game / save /
offline checks** (real `State`, real commands, real persistence). Generated previews and
structural scripts never count as art approval.

## Reproduction

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python scripts/build-space-assets.py        # byte-identical re-export of the supplied art
python scripts/validate-spaces.py
cargo build --locked -p little-forge -p npc-preview -p ui-preview

# Independent: player motion (both smiths, 17 clips, base/equipped, mirrored, light/dark, stations off)
python scripts/capture-player.py target/debug/npc-preview qa-output/player
cargo run -p npc-preview -- --character female --story hammering     # interactive

# Independent: showroom fixtures (empty / partial / full / reserved / held / upgraded / long labels / small)
python scripts/capture-ui.py target/debug/ui-preview

# Actual app: exterior, workshop stages, showroom, full-and-waiting, character select, both smiths
python scripts/capture-forge.py target/debug/little-forge
# Linux only: real X11 keys through workshop → showroom → exterior → queue → collapse → expand
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/test-forge-input.py target/debug/little-forge
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/test-title-input.py
```

QA scenes (`--qa`): `forge-exterior`, `forge-workshop` (extra captures at frames 45/100/140 =
heating / hammering / quenching of one real job), `forge-showroom` (2 for sale, 1 held, 1
reserved), `forge-full` (every slot taken, two queued jobs waiting), `forge-select` (title on
the Choose route), `forge-input`; append `-female` for Eira. All use an isolated save
directory and the fixed 1/30 s QA clock.

## Deterministic tests (Rust)

| Area | Test | Proves |
|---|---|---|
| Capacity | `showroom::reserve_place_sell_and_hold_are_single_shot` | reserve once, place once, hold excluded from sale, full → `DisplayFull` |
| Growth | `showroom::racks_grow_capacity_without_moving_goods` | slots appended per rack; goods and reservations stay in place; limits enforced |
| Migration | `showroom::legacy_migration…`, `save::schema_three_stock_beyond_capacity…` | 7 legacy daggers + in-flight job → 3 displayed, 1 reserved, 4 overflow; overflow sells first; new craft blocked; schema-4 file without showroom rejected |
| Waiting | `spaces::activity_follows_authoritative_state_and_full_showroom_waits` | queue waits at full showroom without a second ore debit; action returns to Idle |
| Stages | `craft::stage_is_derived_from_authoritative_work_only` | heating → hammering → quenching follow work, slower when tired, never ahead |
| Partition | existing `partition_and_seed_replay`, `loop_partition_resume_food_water_sale`, `save_every_boundary…` | time splitting and per-second saves still produce identical state/events with slots |
| Commands | `commands_are_atomic_and_idempotent`, carpenter/furniture observer test in `village.rs` | duplicate IDs rejected; rack purchase/upgrade debits once; UI observers respect expanded/home/dialog guards |
| Selection | `title::picking_records_the_smith_once_and_backing_out_writes_nothing` | Back writes no save; Pick records smith and persists; Continue asks once for an unchosen save |
| Views | `spaces::space_actions_need_an_expanded_home_and_toggle_cleanly` | collapsed window ignores view actions; hold toggles stock without gold change; reserved slot unchangeable |
| Playback | `player::*` (5 tests), `stations::floor_plan_keeps_every_station_inside_the_scene` | both manifests parse (17 clips, 5 stations), loop-range hold/release, markers once per pass, seek determinism, walker handoff, floor plan bounds |
| Headless | `forge-qa` craft-and-sale + `life-loop` 3-day run | sold 10 722 over 3 simulated days with 4 slots; saves every 600 s equal |

## Evidence captured on this branch (macOS, Metal, debug build, 2026-10-18)

| Capture | What it shows | Status |
|---|---|---|
| `qa-output/forge/forge-exterior.png` | Bram outside the idle forge, dark window, one-line status chip, `Enter` over the door | rendered |
| `forge-workshop-045/100/140.png` | one real job: heating at the forge, hammering at the anvil, walking to the tub | rendered, both smiths |
| `forge-showroom.png` | supplied interior + shared slot window (FOR SALE / HOLD badges) + detail panel | rendered |
| `forge-full.png` | 4/4 occupied, status "Waiting · Showroom full…" while two jobs stay queued | rendered |
| `forge-select.png` | two native 2× portraits with blink beside the Choose menu | rendered |
| `qa-output/player/*` | 17 clips × 2 smiths, base variant (`frames/scene-base` for quenching), mirrored give, light background, walk-to-anvil | rendered |
| `qa-output/ui-showroom-*.png` | fixtures incl. upgraded 8-slot rack, Korean long labels at 640×480 | rendered |

Linux llvmpipe captures, Windows DX12 captures and the X11 input scripts run in CI
(`.github/workflows/ci.yml`); they were **not** executed locally.

## Not verified here (keep open)

- Native transparency / always-on-top / click-through with the new exterior chip and door
  button on macOS and Windows desktops (the QA runs use `--safe-window`).
- Mixed DPI and the 2× X11 scale path for the interior scene (the scene snaps to integer 1×
  when the window is smaller than 960×540; verify legibility of the 1× case on a real display).
- The Linux golden images `docs/qa/golden/linux-llvmpipe/{ambient,sale,expanded}.png` still
  show the retired legacy smith sprite. They need a reviewed re-approval on the Linux runner;
  this branch does not loosen the comparison thresholds.
- Art direction sign-off: the two interiors, window frame and detail panel are supplied
  references reduced to native size; the window glow is a 24×20 procedural sprite. None of
  this is an approval of style or motion quality.
- Research and salvage remain preview-only until #72/#76 land; the workshop menu says so.
- Energy/frame cost of the expanded interior on battery (sprites are re-placed only when
  the window changes; UI rebuilds are keyed on state, but no profile was taken).
