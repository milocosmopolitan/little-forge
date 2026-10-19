# D01 persistent indoor forge: implementation and native review

Status: **current screenshots recovered; visual approval and full native acceptance still pending**. PR #122 is open for review, not approved for merge. No implementation issue is closed. No later management screen or D02–D07 gameplay contract is implemented here.

## Source and decisions

Base main: `6c6e8339f070229f8ea67925aeede550fee54a9d`. Audit PR #120 is a design reference, not a merged prerequisite. Implementation branch: `feat/107-live-indoor-forge`; references #107, #118, #63, #89. Selective background passthrough remains #121.

Approved first-slice constraints: existing native1× smiths and stations, interior default, persistent bottom180 logical pixels,960×540 expanded client with upper360 management host, default mouse interaction and explicit F8 whole-window passthrough with10-second recovery. Station positions, backdrop composition, rest treatment and visual acceptance remain review decisions.

## Rendering and routing

`apps/desktop/src/forge_view.rs` owns logical/physical viewport geometry. A layer-1 Camera2d clips the live forge to the bottom180 logical pixels. The layer-0 camera owns the shared UI/exterior with no second clear. Smith Actor/Walker and20 station layers persist while management opens/closes. Core work completion is independent of animation playback.

```text
960×540 logical client             960×180 logical client
┌────────────────────────────┐
│ Management host: y=0..360   │
├────────────────────────────┤     ┌────────────────────────────┐
│ Same live forge: y=360..540 │  ↔  │ Same live forge: y=0..180  │
└────────────────────────────┘     └────────────────────────────┘
Desktop bottom edge is retained when the existing work-area clamp permits.
```

New/Continue enters ambient. Manage/Tab opens the explicitly labeled D01 QA host. Its real controls open legacy controls, check a modal, and close; it is not a Dashboard prototype. A frame-start modal guard prevents one Escape from dismissing both modal and host. Closing restores focus to a visible ambient control. Legacy panels scroll within the upper360 region; no panel gameplay was redesigned.

The10-second lease uses wall time rather than clamped frame delta. Backward wall-clock changes recover input. The local F8 handler now recognizes Ctrl+Alt modifiers so a recovery chord delivered locally does not unconditionally rearm passthrough. This defensive handling is distinct from proof that native global-hotkey delivery works; see the outstanding input evidence below.

## Assets and visual findings

No runtime assets were generated, changed or exported. Reused: `assets/forge/spaces/catalog.json` backdrop, canonical male/female manifests and atlases, registered station layers/animation offsets, Farmer runtime sprites, existing forge-ui controls/fonts. Six runtime source crops recompose the old backdrop without scaling the character artwork or baking a new background image.

Candidate native anchors: research112, rest352, forge416, anvil576, quench720, salvage864; feet y156. In current captures both smiths remain unclipped, raised hammers fit inside the strip, and forge/anvil/quench contacts retain station-relative placement. Native resumed female quenching was also observed. This is inspection evidence, not approval of every animation frame. The wall/lantern repetition, forge-to-anvil overlap, station spacing and absence of dedicated rest furnishing require owner review. Salvage remains scenery only.

### Showroom compatibility is an open decision

`spaces.rs::route` explicitly hides `Space::Showroom` scene artwork while retaining workshop ambience. The showroom backdrop/catalog/data were not deleted. Nevertheless the old independent room experience is **not preserved by this routing**, and must not be described as preserved just because assets remain.

Recommendation for the later #112 screen: an independent room presentation in the upper management region, with the persistent forge below, using the existing fixed-slot art. That is a proposed future solution, not implemented or approved here. Do not merge the always-hidden showroom route as an implicit permanent product decision; resolve the compatibility disposition in #89/#112 review without implementing the new showroom in this follow-up.

## Capture failure investigation

| Boundary | Current evidence | What it establishes |
|---|---|---|
| Window identity | A fresh CUA app binding finds the current window and captures it; stale prior-process IDs are not reused. | Current `cgWindowNotFound` blocker is cleared. Historical wrong-ID causality is not proven. |
| Capture permission | A Swift `CGPreflightScreenCaptureAccess()` probe returned false for that shell-launched process. CUA capture works. | Capture paths have different access. It does not prove all CLI tools share the same permission or explain Bevy GPU readback. |
| OS session/display | Current session reports on-console/login-complete; the current online display is active and not asleep. | Current session is renderable. No equivalent state was retained at the old failure instant. |
| Actual rendering | Unchanged9444c5c Rust build rendered through both CUA and in-game F10 before further Rust changes. Follow-up executable captures pass for both smiths/all work stages. | The old black output is not a consistently reproduced renderer failure. Historical root cause remains unclassified. |

The earlier black image is retained as `blocked-final-black.png`, a failed artifact, not accepted output. `scripts/capture-d01.py` now rejects blank/degenerate bottom strips after checking the real ECS scenario, so successful entity assertions cannot alone publish a successful visual capture result. No golden threshold was reduced.

## Current evidence and reproduction

[Current screenshots, executable/source hashes and capture results](evidence/d01/README.md). `source-hashes.json` identifies the delivered Rust files; `capture-validation.json` identifies the executable and image bytes. Captures were made on9742708 plus the explicit follow-up changes, before their commit; documentation-only commits do not alter the executable. Raw logs/traces remain at `/private/tmp/little-forge-d01-followup`.

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked -p little-forge
python scripts/capture-d01.py target/debug/little-forge --output /tmp/lf-d01-review
python scripts/capture-alpha.py target/debug/little-forge --output /tmp/lf-alpha-review
cargo run --locked -p little-forge -- --save-dir /tmp/lf-d01-native
```

Use fresh isolated output/save directories. D01 QA invokes real title/host Activate handlers and paid AcceptChef/QueueChef commands; no dagger/research fixture unlock. Core and animation advance1/30 second per rendered frame. Both smiths pass100 transitions, one stable smith entity,20 station layers, saved-state readback, and nonblank heating/hammering/quenching/rest captures. These GPU runs are not OS-input automation or frame-pacing benchmarks.

### Native macOS observations

- Apple M4/Metal at200% DPI: actual mouse Continue restored the selected female smith to960×180. Tab, mouse Manage and Escape exercised960×540 management with the same bottom scene. Actual window/capture dimensions agree.
- Modal click → Escape returned focus to the modal trigger without collapse; the following Escape collapsed and focused Manage.
- Actual wheel scrolling exposed the previously clipped Chef acceptance button. Mouse acceptance and Queue Kitchen Knife worked; no direct state mutation was used in this native run.
- Saved/quit during working state and restarted a new process. Diagnostics before quit showed19,000 work units, Working,8 ore,0 gold; after Continue showed3,000, Working,8 ore,0 gold and actual quenching. Later Ready remained8 ore/0 gold. Thus selection/work resumed without a second material charge or premature reward. This is not a full offline-duration qualification.
- F8 produced Idle with a9.967881-second lease; after automatic expiry a real Manage click expanded the window. Safe-mode F8 retained Interaction, lease0 and safe=true.
- Injected Ctrl+Alt+F8 did **not** establish immediate recovery, including after local modifier handling was added. Registration success is insufficient; native modifier/global-event delivery remains unresolved. No focus-independent global shortcut claim is made.
- CUA drag attempts on the custom handle and native titlebar did not establish movement. Drag qualification remains open, not passed.
- Native recording and cross-app desktop-composition capture are still absent. Earlier `screencapture` exited1 and AVFoundation received no frames; no synthetic screenshot sequence is presented as a native recording. Cross-app click receipt, sleep/wake,100/125/150% native DPI, Windows/Shadow PC and localization remain unqualified.

## CI: baseline versus D01 regression

Main run39181315993 is pinned to6c6e8339f070229f8ea67925aeede550fee54a9d. Its Linux golden changed ratio0.11321180555555556 exceeded0.005. Windows Alpha/Alpha-crafting timed out at240/360 seconds. Farmer atlas regeneration and UI release readiness are separate known local failures; those source assets/exporters/release rules are unchanged.

PR #122 initial run39193055909: Ubuntu passed. macOS and Visual failed earlier, at `capture-alpha.py:54`, because the validator required960×540 for every image while the newly approved startup workshop is960×180. This was a **new D01 QA contract regression**, not the old golden failure. The fix asserts960×180 only for `workshop`, retaining960×540 for every other Alpha scene and preserving brightness checks. Local male/female Alpha capture validation now passes. Windows again timed out at240/360 seconds, matching the baseline symptom but not proving identical root cause.

Fresh run39197434914 tests the geometry-validator fix; later follow-up commits require their own CI disposition. Do not report overall CI success while checks are pending or failing. fmt, strict Clippy and98 workspace tests passed after the local F8 handling change. No screenshot baselines or CI timeouts were changed.

## Remaining gates

1. Resolve native global recovery/drag and obtain the native recording/desktop-composition evidence. Automatic recovery is verified but does not qualify every native input path.
2. Review fresh CI by exact commit; keep baseline Windows/golden problems separate from new D01 failures.
3. Explicitly settle the showroom routing compatibility decision; do not erase the independent showroom experience by default.
4. Product owner reviews current character scale, contacts, spacing, backdrop and rest treatment. Passing entity/blank-image assertions is not art approval.

No merge, issue closure, next management panel or overall Epic #106 completion is authorized by these results.
