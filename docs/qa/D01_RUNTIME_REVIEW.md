# D01 persistent indoor forge: implementation review

Status: implementation review, **not native acceptance or visual approval**. No implementation issue is closed. D02–D07 and later redesigned panels remain outside this change.

## Source and decisions

Base main: `6c6e8339f070229f8ea67925aeede550fee54a9d`. PR #120 remained open at `a25c8db10b96d764f253ed15cd585744ba15b904`; its D01 decisions were read as references, not merged. Branch: `feat/107-live-indoor-forge`. References: #107, #118, #63, #89; selective passthrough is separately tracked in #121.

Approved implementation constraints: native 1× existing smith art, interior default, persistent bottom 180 logical pixels, 960×540 expanded client with upper 360 management host, normal input by default, explicit F8 whole-window passthrough with 10-second recovery. Final station positions, background composition and visual acceptance remain unapproved.

## Rendering and navigation

`forge_view.rs` owns logical/physical layout. A layer-1 Camera2d renders the live workshop into the bottom physical viewport; the layer-0 camera renders existing exterior content and the shared UI, with no second clear. The existing smith Actor/Walker and 20 workstation layers persist across management transitions. Neither navigation nor playback completes core work. The core simulation remains authoritative.

```text
960×540 logical client             960×180 logical client
┌────────────────────────────┐
│ Management host: y=0..360   │
├────────────────────────────┤     ┌────────────────────────────┐
│ Same live forge: y=360..540 │  ↔  │ Same live forge: y=0..180  │
└────────────────────────────┘     └────────────────────────────┘
Desktop bottom edge is retained when the existing OS work-area clamp permits.
```

The title routes New/Continue to ambient, not expanded management. Tab or Manage opens the explicitly labeled D01 QA host. It contains working close, legacy-control and modal checks, not fake future screen tabs. Exterior remains reachable through the existing navigation. Showroom navigation currently retains the bottom workshop instead of rendering the old showroom backdrop. This is an explicit presentation compatibility change to review, not a completed showroom redesign. Legacy panels retain their existing gameplay; their container scrolls within the upper region rather than clipping unreachable controls. This compatibility scroll still needs native pointer/keyboard acceptance after the final change.

A frame-start modal guard prevents the same Escape from dismissing the dialog and collapsing management. Focus returns to the triggering modal button, or to the visible ambient control after collapse. The ambient controls have their own tab group. Dragging uses the labeled handle. Passthrough expiry uses wall time rather than a clamped frame delta, so long suspension expires the lease; backward wall-clock changes fail toward recovery. Sleep/wake hardware acceptance is still pending.

## Assets and composition

No asset files were generated, modified or exported. Reused `assets/forge/spaces/catalog.json` workshop backdrop, existing `forge-npc` male/female player manifests and atlases, registered workstation layers and animation offsets, Farmer runtime sprites, and `forge-ui` controls/fonts. The existing backdrop is rendered as six native-scale source crops; this is runtime composition, not a new background PNG. Station translations preserve authored station-relative offsets.

Candidate anchors, in native logical pixels: research112, rest352, forge416, anvil576, quench720, salvage864; feet y156. These are review candidates, not approved final art coordinates. Salvage is scenery only; no command was invented. Repeated wall/lantern motifs and the missing dedicated rest furnishing need art-direction review. Existing canonical characters remain unchanged.

## Reproduction

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p little-forge -- --qa d01 --save-dir /tmp/lf-d01-male --screenshot /tmp/d01-male.png
cargo run --locked -p little-forge -- --qa d01-female --save-dir /tmp/lf-d01-female --screenshot /tmp/d01-female.png
cargo run --locked -p little-forge -- --save-dir /tmp/lf-d01-native
```

Use fresh isolated QA directories. `d01_qa.rs` invokes production title/host Activate handlers and real AcceptChef/QueueChef commands, paying the normal two ore. No fixture dagger/research unlock is granted. Core and animation clocks advance 1/30 second per rendered frame. This is actual Metal/Bevy rendering, **not OS-input automation and not a real-time performance benchmark**.

Each smith run performs 100 open/close cycles, asserts one stable Actor entity and 20 station layers, captures live heating/hammering/quenching/rest, and reads back saved state during work and at completion. JSON records 1,290 samples, window geometry, core work, entity identity and animation phase. The separate unit regression preserves actor elapsed time and exact core state across 100 route cycles. Geometry tests cover scale factors1,1.25,1.5,2; only scale2 was observed natively.

## Evidence and acceptance boundaries

Screenshots and compact results are in [d01 evidence](evidence/d01/). They are actual Bevy captures. Full JSON traces and logs were retained locally at `/private/tmp/little-forge-d01-evidence`.

- Automated: fmt, strict workspace Clippy and 98 workspace tests passed. Five asset/scene/UI/village/expedition validators passed. Existing Alpha production-control scenario passed before the final scroll/tab-group compatibility adjustment; final D01 runs cover the delivered geometry and scene.
- Native macOS Apple M4/Metal, 200% DPI: mouse Continue restored the selected female smith into ambient; Tab/Esc expanded and collapsed; modal Esc dismissed only the modal and restored its button focus; the next Escape collapsed and focused Manage. Window bounds were960×180 /960×540 logical, with constant bottom edge in observed transitions.
- F8 native diagnostics observed `Idle`, lease9.934181; later automatic recovery observed `Interaction`, lease0. Global shortcut registration succeeded. Actual cross-application click receipt, focus-independent global recovery and sleep/wake recovery are **not qualified** by these diagnostics.
- Native tool failure: after subsequent shortcut testing, CUA returned `cgWindowNotFound`, including after app/session restart. `screencapture -v -l` exited1 without a recording; ffmpeg AVFoundation received no frames. No recording is attached, and no screenshot sequence is presented as a recording. Cause is unresolved; do not classify it as either an application regression or a permission failure without evidence.
- Save readback equality passed during work and after completion; a normal idle save/quit/Continue restored selection. **Actual process restart during ongoing work, offline continuation and duplicate-reward acceptance remain pending.** Existing core save tests are not a substitute.
- Native drag, final legacy scrolling, safe-window passthrough block, transparent composition over another app,100/125/150% native DPI, Windows/Shadow PC and localization acceptance remain pending. FPS diagnostic around30 on this host is a sample, not frame-pacing or memory qualification.

## Baseline failures and regressions

Main already has Farmer runtime-atlas regeneration mismatch and UI release gate `runtime_ready=false`; basic validators pass. Neither exporter nor assets nor release rules changed. Linux golden ratio0.11321180555555556 exceeds0.005 on the earlier baseline audit; Windows Alpha has240/360-second timeout evidence. PR #120 run39185966470 also showed a macOS expedition-movement120-second timeout; that observation is not proof of identical baseline cause. Golden baselines and thresholds remain untouched. This PR deliberately changes the default scene, so additional screenshot differences must be reviewed as D01 changes, never waved away as baseline failures.

Native review found and corrected old HUD text overlapping ambient controls, hidden-host focus retention, expanded New/Continue routing, and legacy panel clipping. No unresolved new core failure is known; the native control/recording failure and final compatibility acceptance remain open.

## Required before acceptance and next stage

1. Restore native observation/recording and verify the final build: drag, scrolling, native click receipt, global recovery from another app, safe mode and actual ongoing-work restart.
2. Supply a native expand/collapse recording and desktop-composition capture. Recheck both smiths’ tool contacts in motion, especially quenching and rest; still frames do not establish playback approval.
3. Product owner approves station spacing, character scale, repeated backdrop composition and rest treatment from the actual game. Do not infer approval from passing pixel or entity assertions.
4. Review fresh PR CI failures individually and complete Windows/Shadow PC qualification separately. Do not weaken tests or update goldens without explicit review.

Only after these gates should the next screen stage proceed. This PR does not approve D05 storage semantics or D02/D03/D04 balancing and does not complete Epic #106.
