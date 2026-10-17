# Shared UI workshop

Run `cargo run -p ui-preview -- --story farm`. The packaged `ui-preview` binary
uses the adjacent `assets` directory. No game simulation, saves, purchases or
network calls run in this application. The desktop and preview import the same
`forge-ui` crate; these are real Bevy widgets, not an HTML recreation.

Stories: farm, chef, carpenter, leatherworker, empty, full, insufficient, sold-out,
long-label, controls, confirmation. Buttons across the top change story.
Tab / Shift+Tab moves focus; Enter / Space activates. Confirmation defaults to
Cancel, Escape cancels and restores surviving previous focus. Preview purchase
buttons are explicitly fixtures. Existing title settings/load/quit retain their
own state machine and persistence ownership (#35); their controls use the shared skin.

```
cargo run -p ui-preview -- --story long-label --width 640 --height 480
python scripts/capture-ui.py target/debug/ui-preview
python scripts/validate-ui.py
```

Linux capture requires a display, e.g. `xvfb-run -a`. Capture generates all stories,
small viewport, 200% X11 DPI and missing-art fallback. Missing fonts retain Bevy's
built-in fallback. `--assets PATH` allows isolated asset-failure tests.

## Game integration

M opens the canonical village map. 1–8 visits real destinations; arrows select
map regions. Tab opens management from ambient, then traverses controls.
Escape: shop → map → forge management → ambient. Close collapses the overlay;
it does not quit. F9 opens the existing title/shell route.

Crafting, Materials and Provisions share the overlay root. Queue and needs are
read from `State.life`; purchases/use/crafting go through existing atomic commands
and persistence. No per-job cancellation, invented catalog items or research
features were added. The legacy ledger stays hidden. Merchant NPCs are separate
128×160 main-branch assets, not painted into backgrounds.

## Acceptance status

This is an integration candidate, **not completed epic #58**. Source contact sheets
are temporarily sampled by explicit rectangles to review composition. They are
not validated native-pixel exports. The required art-first gate in #59–#62 has
not been passed; do not merge as final accepted art or close those issues.
`python scripts/validate-ui.py --release` intentionally rejects this state.

See `assets/ui/README.md` for source provenance and remaining export requirements,
and `ACCEPTANCE.md` for actual evidence and limitations.
