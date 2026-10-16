# Village UI & Navigation — Epic #27

## Implementation

The actual Bevy desktop app now has a common thin wood/charcoal navigation shell,
one canonical Ashbrook map, eight destinations and four local vendors. The forge
simulation continues while browsing. Home retains the Living Forge controls,
including free water and the existing staple pantry; meals require explicit use.

| Destination | Implemented behavior |
|---|---|
| Home | Existing crafting, needs and pantry management |
| Carpenter | Timber purchase; Furniture tab explains commission requirement |
| Leatherworker | Leather purchase; Work gear tab explains commission requirement |
| Farm | Existing staple provisions; Basic materials tab explains locked supply |
| Tavern | Premium meal purchase; Recruit tab explains missing expedition system |
| Chief | Read-only progression availability |
| Guild | Read-only commissions/trade availability |
| Outside | Expedition entry and truthful unavailable reason |

No quest, recruitment, dispatch, furniture unlock or shop production is simulated.
Stock is a finite, persisted milestone fixture (40 timber, 30 leather, 60 staples,
12 meals); no replenishment timer has been invented. Materials persist without
pretending their future recipes have been implemented. Prices are provisional.

## Controls

- `M`: open village explicitly; no passive event opens/focuses it.
- Map: pointer hover/click, `Tab`/arrows to cycle focus, `Enter` to visit,
  `1`–`8` for the ordered destination list. The public well is not clickable.
- Vendor: Left/Right selects category, `-`/`=` adjusts quantity (1–99),
  Enter buys. Price, stock, ownership and unavailable reason remain visible.
- `Esc`: vendor → map → home → ambient; header Back/Close are pointer alternatives.
- At home, `Tab` retains expand/collapse; existing life controls remain.
- `F12`: diagnostics include domain state, screen, FPS, adapter and window state.
  `--screenshot PATH` plus `F10` records the current running screen for QA.

## Boundaries and persistence

- `forge-core::village`: typed navigation, stable locations, item catalog, quote hook,
  stock/material state and atomic purchase validation. Quote is read-only.
- `State::command`: shared monotonic IDs, cloned-state commit, balance/stock/pantry
  checks, quantity/vendor checks, overflow protection. `BuyFood` uses this same
  farm transaction; it is not a second food economy.
- Buy widgets retain the transaction ID rendered with them; duplicate activation
  of that widget cannot debit twice. A new explicit action receives the next ID.
- Schema **3** adds commerce; schema 1/2 migrate in memory while retaining needs,
  gold, inventory, jobs and RNG. A schema-3 save missing commerce is invalid.
  Previous binaries refuse schema 3 instead of silently dropping materials.
- `forge-content::village`: validated normalized polygons, exact eight-ID registry,
  image letterboxing and picking. Points off-image or on unassigned geography
  cannot navigate. Tests exercise matching 100%, 150%, 200% and narrow rectangles.
- `apps/desktop/src/village.rs`: Bevy rendering and user intents. Map linework is
  immutable, overlay textures are cached, only one wash is visible, no texture is
  regenerated on hover. Cartography uses linear sampling; sprites retain nearest.

## Reproduction

```sh
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
python scripts/validate-assets.py
python scripts/validate-village.py
cargo build --locked -p little-forge
# On a graphical desktop (including native macOS/Windows CI):
python scripts/capture-village.py target/debug/little-forge
# On disposable Linux X11 only:
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/test-village-input.py target/debug/little-forge
```

`capture-village.py` executes the actual app for map, four vendors, chief, guild,
outside and a packaged missing-art fallback. It validates PNGs; images are evidence,
not exact cross-GPU goldens. `test-village-input.py` uses real XTest key/pointer events
and reads app diagnostics to assert map → carpenter → buy → farm → buy → tavern →
buy → home → close while ticks advance. Its 24-gold starting fixture is QA-only,
fixed-clock, isolated, and never loaded from or written over a player save. Rust
integration tests independently store/reload a real journal and reject replay.

Local Linux llvmpipe acceptance passed, including real pointer picking and purchases;
32 Rust tests passed. The software-rendered input run reported approximately 12.5 FPS;
this is not a hardware performance estimate. Native macOS/Windows build/test and
screen captures run in the existing OS CI matrix; inspect the linked PR checks and
`evidence-*` / `visual-candidates` artifacts for the exact commit's results.

## Limits retained from Foundation

CI OS screenshots are actual native app observations, not physical monitor testing.
Retina/Windows display composition, monitor changes, taskbar interaction, transparent
surfaces and application focus on a user's hardware are not newly certified by this
milestone. Preserve the Foundation hardware limitations/waivers and safe opaque
interactive fallback. No user multi-monitor setup is required to run the automated
suite. Coordinate/letterbox scale tests do not substitute for OS hardware DPI QA.

Missing map/mask files produce an explicit destination-list fallback. Missing vendor
portraits show a readable substitute. Missing required forge atlases still fail with
the established recovery message, so incomplete packages do not silently pass.
