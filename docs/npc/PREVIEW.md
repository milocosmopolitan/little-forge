# NPC Preview

A standalone Bevy app with Storybook-style stories. It imports `forge-npc`, the same player used by the desktop game, and never initializes a game session, economy, pantry, simulation clock or user-save IO. Windows uses the same embedded DXC and sprite shader correction as the game.

## Run

From the repository root:

```sh
cargo run --locked -p npc-preview
cargo run --locked -p npc-preview -- --story give
cargo run --locked -p npc-preview -- --story talk --capture qa-output/npc-talk.png
```

Portable packages include `npc-preview` (`npc-preview.exe` on Windows) beside the game executable. On macOS it is inside `Little Forge.app/Contents/MacOS/`. Run that executable directly; the adjacent `assets` directory is selected before the development checkout fallback. This app needs a graphical desktop/GPU; Linux CI uses Xvfb + Mesa.

## Controls

| Input | Behavior |
|---|---|
| Story buttons / 1–8 | Idle, talk, blink, surprised, nod, shake head, walk, give |
| Space / R | Pause or resume / restart selected story |
| Left / right | Pause and seek to previous/next frame |
| F | Mirror facing, including sockets and face/hand layers |
| I | Cycle separate item: carrot → dagger → none |
| B | Dark / light / black backgrounds |
| G | Foot, head and hand markers |
| K / L | Toggle automatic blink / face and foreground hand layers |
| Z | Large inspection / 75 px canvas-size preset |
| N | Switch farmer / fixture B (same artwork, different clip timing and tint) |
| + / - | Change playback speed |
| Escape | Quit preview |

The fixture is a second data configuration, not a newly produced character. No new NPC cast art is implied. `--story` selects a stable reproducible story ID; `--capture` samples 240 ms by default (`--at SECONDS` overrides it; `--left` captures mirrored facing; `--item dagger|carrot|none` selects props; `--sequence` exports six timed motion frames) and exits after screenshot readback. Interactive playback uses monotonic frame time. Transparent sprite pixels are inspected over opaque backgrounds; this preview does not change OS click-through/window transparency.

## Player smith stories

`cargo run -p npc-preview -- --character female|male [--story <clip>]` opens the same
workshop floor plan the game uses (`forge_npc::stations`) with the chosen smith rendered by
`forge_npc::player::Actor`, the component the desktop app drives from `State::activity()`.
Stories: the 17 authored clips (`idle … salvaging`) plus `walk-to-anvil`. Work clips are held
in their loop range at their station; `H` releases the hold to play the outro. `E` switches
equipped / base variants (`base/` atlas entries, `frames/scene-base` for quenching), `S` hides
the stations, `F` mirrors, `B` cycles dark / light / black, `G` shows foot and head guides,
Left/Right seek ±50 ms, Space pauses, `+`/`-` change speed. `--capture PATH [--at s]
[--sequence] [--base] [--left] [--light] [--no-stations]` writes reproducible fixed-clock
frames; `scripts/capture-player.py` covers both smiths and every clip. No simulation, economy
or save IO runs; research and salvage are preview-only until #72/#76.

## Architecture

- `crates/forge-npc`: versioned catalog validation; timing, action priorities/neutral return, sockets and shared Bevy renderer; no dependency on `forge-core`.
- `apps/npc-preview`: minimal window/camera, local mock commands, controls and capture. No import from game startup or save code.
- `apps/desktop/src/npc.rs`: thin adapter for accepted live farmer-food purchases, including the village farm button/keyboard Purchase contract. Repeated IDs are suppressed. Offline simulation/load does not fabricate a burst of presentations.
- Domain transactions commit through existing pure Rust commands. Handoff frames only change item visibility. Completing, skipping or interrupting a clip never grants or debits resources.
- The app and preview reuse `apps/desktop/src/rendering.rs` as a source module for the pinned renderer correction, without bootstrapping the desktop game.

Use the standalone preview for routine animation/asset review. Run the focused game adapter tests and actual-game smoke scenario when game event contracts, integration or relevant shared playback change. A changed sprite does not automatically require the entire game simulation suite.

## Asset contract

`assets/npc/farmer/character.json` is the clip catalog. Frames use `[x, y, width, height]` in a fixed 384×352 registered canvas, foot `[192,340]`, explicit duration and hand socket. All exports retain the same reference canvas: there is no runtime per-frame auto-fit. `registration.json` records source sheet rectangles, sheet-wide scale and placement offsets. Cropped exports must first reconstruct that reference canvas before consumption. Do not reinterpret the existing legacy manifest's `[min_x,min_y,max_x,max_y]` convention as this NPC format.

Full-body atlas: 22 unique frames; expression/gesture timing reuses frames forward/backward. Dialogue uses the neutral body plus separate mouth and eye patches; timed blink can compose with talking. Head turns/gestures use baked head/body poses to avoid applying incompatible straight-ahead eye patches. `hands.png` is a foreground grip layer drawn after interchangeable items. Walking is baked full body. Mirroring also mirrors sockets; the farmer's asymmetric belt/leaves are mirrored as a visual compromise for the MVP, not reinterpreted as different equipment.

Keep source originals, generated source sheets, alpha PNG exports and provenance together. `scripts/pack-npc.py` only splits/registers existing artwork with one fixed scale per source sheet; it does not generate animation pixels. It needs Pillow, NumPy and SciPy. `scripts/validate-assets.py` verifies dimensions, hashes and transparency using the existing asset manifest; Rust tests validate NPC clip geometry, sockets and durations.

## Evidence

See `docs/qa/npc/README.md` for executed checks and limits. Generated art/contact sheets are insufficient evidence of game adapter behavior. Native macOS/Windows GUI acceptance must be distinguished from compilation and Linux software-renderer captures.

Missing character textures leave preview controls available and show an art-unavailable status. Failed facial/item layers are omitted; a valid base pose can still render. NPC illustration loss never blocks game purchases or startup. The standalone capture helper also verifies an empty-assets package.
