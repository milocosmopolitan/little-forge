# NPC animation evidence — Epic #45

## Executed locally

Linux Xvfb, Vulkan llvmpipe (Mesa 25.2.8, LLVM 20.1.2), Rust 1.99.1, Bevy 0.20.0. This is software rendering, not a native macOS/Windows GPU acceptance claim.

- Workspace build: desktop game and independent `npc-preview` executable.
- `cargo fmt --all -- --check` and strict workspace/all-target Clippy: pass.
- `cargo test --locked --workspace`: **32 tests pass**. Includes clip validation, partitioned timing, pause/seek, backwards handoff scrub, neutral return, mirrored sockets, alternate catalog timing, rejected/duplicate commands and legacy scene isolation.
- Existing asset validator: all PNG dimensions, hashes, atlas divisibility and meaningful alpha pass.
- Actual standalone captures: all 8 actions, talk + automatic blink at 3.33 s, mirrored give, before/after visual handoff, carrot/dagger swap. Checked at rendered scale for frame registration, seams, hat/feet and item grip.
- Six timed walk frames exported by the **actual Bevy preview renderer**, assembled into [walk.gif](walk.gif). This is not an animation assembled directly from source artwork.
- Actual desktop app `--safe-window --qa npc`: successful authoritative food purchase followed by grounded farmer handoff, [capture](game-npc.png).
- Existing ambient/sale/expanded golden comparisons: **0.0 changed-pixel ratio** for each, unchanged 0.5% tolerance. No golden updates.

[Runtime story contact sheet](runtime-stories.png) · [talk + blink](talk-blink.png) · [mirrored give](give-left.png) · [dagger replacement](give-dagger.png)

## Bugs found during executable QA

1. Legacy `animate` visited every sprite and reset newly added NPC/layer positions to its fallback ground offset. It now excludes shared NPC players, attachments and visual layers. A headless test protects the isolation boundary.
2. NPC animation used real-time delta in the fixed-clock game QA scenario. The shared plugin now accepts an optional deterministic presentation clock, set by the QA adapter; ordinary play uses Bevy time.
3. Backwards seek retained the previous visual handoff flag. Seeking now recomputes that flag from the sampled timeline. Domain inventory/gold were never driven by this visual flag.
4. Generated source-sheet neighbors left tiny detached fragments in cells. Mechanical source packing isolates the primary character component and preserves sheet-wide scale and boot registration; offsets are retained in `registration.json`.

## Reproduction

```sh
cargo build --locked -p npc-preview -p little-forge
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
python scripts/validate-assets.py
xvfb-run -a python scripts/capture-npc.py target/debug/npc-preview qa-output/npc
xvfb-run -a target/debug/npc-preview --story walk --sequence --capture qa-output/npc/motion.png
xvfb-run -a target/debug/little-forge --safe-window --qa npc --screenshot qa-output/npc/game-npc.png
```

This hosted sandbox rejects Unix X11 sockets. Executed captures used `xvfb-run -a -l -s '-screen 0 1280x900x24 -ac -listen tcp'` and `DISPLAY=127.0.0.1$DISPLAY` inside its child shell. Normal desktop/CI Xvfb does not need this workaround. The local Rust build initially produced invalid dependency metadata/empty object files; a clean of `bevy_asset` and `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_CODEGEN_UNITS=1` produced working binaries. This environment workaround is not committed as a project-wide profile change.

## Resource baseline and limits

Runtime holds one body atlas, three face/grip textures and the chosen prop; authoring-only PNGs are not loaded by the player. The clips contain 22 registered full-body frames; playback checks at most a few clip entries per update and has no simulation ticking dependency. PNG bytes/decoded RGBA dimensions can be inspected from `assets/manifest.json`. This software-renderer run establishes correctness; desktop GPU frame-time/idle energy performance remains unmeasured. Preview updates are bounded to a requested 30 Hz, including unfocused mode.

Left-facing farmer mirrors asymmetric hat/belt art as an explicit MVP choice. Fixture B reuses farmer art with independent timing and tint to exercise shared playback; it is not a second completed NPC art set. PNG layer sources are supplied; no native Aseprite source project is claimed. Transparent PNG inspection uses contrasting backgrounds; OS-transparent/click-through window QA remains a game-shell concern.

Three-OS CI builds/tests and package production are recorded on the implementation PR. Native human-driven macOS/Windows preview GUI tests are not performed in this hosted Linux environment. Do not label cross-compilation as native GUI acceptance.
