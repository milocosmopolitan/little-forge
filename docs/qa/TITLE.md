# Cinematic title acceptance

The approved 2026-10-16 reference now drives the actual Bevy title. The hammer/arm
are part of a single static plate. Only blade heat and six-pose pixel sparks move.
See `assets/title/README.md` for source provenance, image-edit prompt, registration,
frame timing and font license. This replaces the old translated hammer cutout.

## Reproduction

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p little-forge
xvfb-run -a -s '-screen 0 1280x800x24' python scripts/test-title-input.py
xvfb-run -a -s '-screen 0 1280x800x24' target/debug/little-forge --safe-window --qa title --screenshot qa-output/title.png
```

In containers without Unix X sockets, start Xvfb with `-ac -listen tcp` and set
`DISPLAY=127.0.0.1:<display number>` on a disposable local display.

## Verified here

- Linux llvmpipe/Vulkan, actual executable, 960×540 title: scene and material load,
  static background, live menu, independently moving sparks.
- Fresh title writes no save. New Game confirmation creates a valid journey;
  Continue/Load stay disabled before that. Existing progress cannot be overwritten
  by New Game. A current recovery journal is not presented as multiple user slots.
- Real OS keys: New → InGame → management/village → F9 → title → Settings → still
  effects → Escape → QuitConfirm → Escape cancel → Save and Quit.
- Workspace tests cover transition rejection, reversible submenus, disabled entry,
  keyboard selection skips, six-pose periodicity and existing simulation/save tests.
- Strict clippy and asset validation. CI repeats real title keyboard/save acceptance.
- Every visual layer uses the same centered 960×540 stage, uniformly scaled to fit;
  no independently stretched arm, background, menu or heat mask.

## Controls and boundaries

Up/Down: selection; Enter/Space: activate; Escape: back, never immediate exit;
F9: explicitly recover title from gameplay/management. F10 remains the existing
QA screenshot shortcut. Pointer buttons use Bevy activation and hover borders.

Title pause resets the monotonic gameplay timestamp when reentering the game, so
menu dwell time cannot become an unexpected simulation burst. No focus request is
issued. Black background/live menu remain if artwork is unavailable; the static
plate remains if the heat shader cannot load. Still mode hides sparks and holds heat.

#36 is routing, and #42 is the replacement art pipeline. Independent user save
slots/migration (#37/#39), complete persistent device/audio/settings (#40), full OS
close/failure lifecycle (#41), and comprehensive cross-platform acceptance (#43)
remain the separate parent #35 work. This change does not claim those complete.

Native macOS/Windows visual behavior and hardware DPI/transparency are not locally
tested by a Linux software renderer. Platform CI must be read separately.
