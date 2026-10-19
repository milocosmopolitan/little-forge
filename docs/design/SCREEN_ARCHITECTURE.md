# Screen architecture audit

Audit baseline: `6c6e8339f070229f8ea67925aeede550fee54a9d` (fresh `origin/main`). This is a pre-implementation contract, not a shipped-screen claim. Decision IDs refer to [implementation sequence](IMPLEMENTATION_SEQUENCE.md). Evidence and qualification are in [acceptance matrix](../qa/SCREEN_ACCEPTANCE_MATRIX.md).

[D01 owner-review packet](D01_WORKSHOP_REVIEW.md) refines policy candidates and separates approval to verify a layout from visual acceptance. It does not declare the compact scene implemented.

## Approved decision

Indoor workshop is the default ambient; exterior is secondary. Logical client sizes are 960×180 collapsed and 960×540 expanded. Management occupies only the upper 360 logical pixels. The same live workshop occupies the lower 180, with the same smith, work, animation and simulation continuity. All twelve screens reuse the existing Rust domain and Bevy application. No new assets or implementation are supplied by this audit.

## Existing implementation

- `apps/desktop/src/main.rs:435` creates a transparent undecorated 960×180 window, AutoVsync, nearest images and a transparent clear; `:560` spawns one `Camera2d`. `:465` starts expanded for title/QA. Update ordering at `:519` chains simulation, native geometry and UI fit.
- `crates/forge-core/src/shell.rs:11` preserves the bottom with `y + old_height - new_height`, clamped to work area. `apps/desktop/src/window.rs:95` requests 540/180 times monitor DPI. `window.json` stores placement/topmost/decorations, not gameplay routes.
- `apps/desktop/src/spaces.rs:34` and `crates/forge-core/src/spaces.rs:10` default to Exterior. `spaces.rs:295` requires expanded Home for interior and forces exterior when collapsed. `:299` centers a 480×270 scene at integer scale 1–2. This is not the approved lower-strip layout.
- Actors/station sprites are spawned in `spaces.rs:121`; `route` changes visibility, not lifetime. `drive_workshop` at `:371` reads `State::activity`, but transforms depend on current window scale. Existing full-scene geometry cannot simply be reused as the strip.
- UI has village Root (`village.rs:135`), HomeRoot, HitRoot, AmbientRoot (`spaces.rs:223`), title root, lifestyle children, expedition Root (`expedition.rs:258`, `:704`), ReturnRoot (`:494`) and global dialog/tooltip roots (`forge-ui/src/lib.rs:768`, `:856`). Expedition's opaque 960×540 panel covers the workshop. Village bounds reach bottom 16 (`village.rs:493`). Independent redraw/visibility keys are not one routing authority.
- Global modal guards exist in `main.rs:719`, village and spaces handlers; expedition `activate` at `:96` lacks a DialogState guard. This is a static input-ownership gap, not proof of a manually reproduced click leak.
- Scene routing is ephemeral; smith, work, stock and expedition are in schema 7. Loading must restore activity, not replay completion animation for rewards. `simulate` stays independent of management view; title clock handling must remain explicit.

## Proposed resolution: viewport and ownership contract

Use two cameras with explicit render layers/targets: a persistent WorldCamera clipped to the bottom strip and a full-window UiCamera. The UI camera clears neither world color nor transparent exterior, and all UI roots target it explicitly. Keep WorldRoot and smith entity alive across routes and resize. Camera viewport uses physical pixels; widget layout uses logical pixels. One `ViewportLayout` resource is the only conversion authority.

Alternative A, one full-window camera with translated world root, is simpler but does not clip tall scene layers and invites UI/world hit-test drift. Alternative B, render-to-texture at fixed resolution, isolates pixel art but adds image memory and coordinate/picking indirection. Recommend direct viewport plus UI camera, with an integration test for alpha clearing and UI targeting before broader screens. Bevy supports explicit camera viewport, order and clear policy ([Camera documentation](https://docs.rs/bevy/0.20.0/bevy/camera/struct.Camera.html)); this choice is an architectural recommendation, not native qualification.

```text
Expanded, logical client coordinates, x right and y down
(0,0)       +-----------------------------------------+ (960,0)
            | header/navigation                 40 px |
            | screen content: list / detail / graph   |
            | content y=40..320                280 px |
            | status / contextual actions        40 px|
(0,360)     +-----------------------------------------+ (960,360)
            | SAME LIVE WORLD, local y=0..180         |
            | smith, stations, work, ambient actions  |
(0,540)     +-----------------------------------------+ (960,540)
Collapsed: world local rect (0,0)-(960,180); same bottom desktop edge.
```

At normal size `world_rect=(0,H-180,960,180)`. In world local pixels with center origin, `world_x=x-480`, `world_y=90-y`; window input subtracts `(0,H-180)` first. DPI conversion rounds viewport edges together: physical edges = round(logical edge × scale factor). Do not multiply input by DPI twice. Window expansion changes top by −360×DPI, except work-area clamping; bottom error must be at most one physical pixel where the work area permits.

Earlier constrained-work-area candidate (not approved; see the review packet for the recommended first qualification target): uniform fit `s=min(W/960,H/540,1)` keeps the partition 360s/180s; show a readable scale option / scrollable detail and qualify legibility separately. At 640×480 this gives a 640×360 canvas, 240px management and 120px world. This is a fallback, not pixel-perfect 1× support. Integer sprite scaling and fractional desktop DPI are different guarantees; 125/150% must be tested, not described as perfectly uniform pixels. The initial minimum is now approved as960×540 logical usable pixels; smaller-area typography/layout remains future support.

Proposed scene adaptation: native 128×160 smith fits at 1× in 180px (foot around y=156); it cannot fit at 2×. Existing forge station is 128×144; anvil 64×64; tables 128×96. Recompose existing station layers across a 960×180 native scene, preserving each clip's contact offsets and head/foot margins. Existing 480×270 backdrop needs a reviewed crop/repeatable wall/floor export, not vertical squeezing. D01 owns this composition; no crop is approved here. A 480×90 strip at 2× would cut canonical characters and is rejected as the default approach.

One persistent `GameUiRoot` owns `ManagementHost`, `AmbientHitHost`, `NotificationHost`, `ModalHost`. World sprites stay outside that tree. Route changes replace only screen children, not work entities. Patch live text/progress values; do not rebuild whole roots on tick, gold or animation frame. Stable semantic focus keys include route and entity ID, never vector index.

| Current state / input | Next state | Domain effect | Input ownership |
|---|---|---|---|
| Title / new or continue | character choice if missing; then indoor Ambient | only explicit ChooseSmith creates choice/save | shell until entry |
| Ambient / Manage | Dashboard expanded | none; work continues | management + strip hotspots |
| Management / screen navigation | selected screen, expanded | none | exactly one panel |
| Management / Close or top-level Esc | Ambient indoor | none; preserve selection in session | strip only |
| Detail/modal / Esc | close topmost, restore prior focus | cancel uncommitted draft only | modal consumes key |
| Destination / Next | PartyPrep(destination, mode) | no hire/resource debit | party panel |
| PartyPrep / Back | same destination selection | discard no committed item | destination panel |
| Explicit exterior / Back | indoor Ambient or previous management | no scene recreation | one view |
| F8 with no modal | collapsed timed click-through | no command | OS background receives clicks |
| Ctrl+Alt+F8 or 10s lease expiry | interactive Ambient | none | native recovery always works |

`AppRoute`/`PanelRoute` is session UI state; `Space` is presentation location, not progression. Reopen to indoor Ambient after continue; do not save modal state or pointer selection into domain. Preserve active trip and work on load and expose a quiet summary. Recommended observation scene stays in the upper 360 as well; a full-window expedition exception needs explicit approval D01, never an implicit opaque root.

Input pipeline: collect intent → route/modal ownership check → command validation → atomic state commit → journal persist → read-only projections → visual effects. All keyboard, pointer and programmatic Activate paths use the same guard. Modal makes lower strip noninteractive while it continues rendering; block F8 while modal is open, retain native emergency recovery. Global Tab opens from Ambient and traverses controls once expanded; Esc/Close collapses. Notifications do not focus or switch screens.

Transparency does not imply per-pixel hit testing. Existing `CursorOptions.hit_test` switches the whole window. Keep the explicit 10-second click-through lease and safe opaque mode. Stage1 retains whole-window input plus explicit10-second F8 passthrough. Final ambient UX targets selective noninteractive-background passthrough in follow-up #121; it is not implemented by Bevy transparency and does not block Stage1. Management retains normal input/modal ownership.

## Open decision

D01 limited implementation is authorized; final 1× composition and native visual approval remain pending. The initial management minimum is960×540 logical pixels; smaller-area fallback is deferred. Register the eleven approved concept images and their crop/viewport intent under #118. Geometry above is a precise proposed contract, not visual approval. #63 owns routing consolidation and modal guards; #89/#107 own viewport adaptation.

## Explicitly excluded scope

No Bevy upgrade, general scene engine, pathfinding, new renderer, screenshot substitution for motion, native window implementation or scene art generation during this audit. Existing DX12 compositor/shader correction remains untouched.
