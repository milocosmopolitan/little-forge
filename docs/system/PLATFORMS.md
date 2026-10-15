# Platforms & QA — Little Forge

> Status: Foundation policy (2026-10-15). Engine: Bevy / Rust. Primary development and automated desktop testing: macOS. Windows manual acceptance: Shadow PC. Linux: future support, no launch guarantee.

## 1. Platform policy

| Platform | Scope | Verification |
|---|---|---|
| macOS (Apple Silicon) | Primary development and first playable desktop target | CI + local native UI/graphics and performance |
| Windows x64 | Steam target after basic desktop milestone | Cross-platform CI + **manual Shadow PC acceptance** |
| Linux (X11 / Wayland) | Later evaluation | Build checks where practical; desktop integration is not claimed supported until tested |

Use one Rust workspace. Bevy handles windowing, 2D rendering, UI and input; a pure Rust forge-core owns deterministic economy, time, crafting, orders, and expeditions. No Godot or GDExtension is required. Do not introduce platform-specific code until a verified gap exists.

## 2. Window interaction contract

- Ambient: bottom-anchored floating transparent window; candidate 960x180.
- Management: opens upward while bottom remains anchored; candidate 960x540.
- Idle: click-through mode so work underneath remains accessible; explicit recovery via shortcut, tray/menu or alternative accessible control. Never trap the user in a permanently non-interactive window.
- Interaction: click-through disabled; visible controls accept input.
- Always-on-top must be user-controlled, default OFF. Window can be dragged and settings remember display and position.
- Safe window mode: opaque, decorated, interactive window accessible by launch flag/config reset even if transparency/compositing fails.
- Window APIs and compositor behavior differ across OS and GPU; capabilities are implementation targets, not verified results.
- Focus must not be stolen on passive events. On a small display, clamp expanded bounds to usable area.
- Capture window transparency (including alpha/compositing), pointer hit tests, display scaling, virtual desktops/Spaces, sleep/resume, external monitors, and normal/full-screen apps in platform checks.

## 3. QA layers and CI

1. Every PR: cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; content/asset schema and reference validation.
2. Deterministic simulation: fixed seed and fake clock; time-step partition invariance; save/load replay parity; duplicate delivery and reward prevention; no negative stock/gold.
3. Bevy ECS integration (where windowless): transitions for crafting, visitor arrival, sale, holding purchased item's sprite, exit, panel expansion, click-through command transitions.
4. Visual regression: fixed frame/state screenshot, stable GPU/OS baseline, diff artifact; deliberate baseline updates require review. Do not use Linux screenshots as exact pixel baselines for macOS/Windows.
5. Native platform acceptance: transparency, input passthrough, window anchoring, DPI/scaling, focus, performance, packaging. Headless or screenshot tests cannot prove system-level click-through.

CI must cross-compile/build Windows (and ideally execute Rust tests on a Windows runner) even when graphical acceptance is manual.

## 4. Windows Shadow PC — manual acceptance checklist

Fill this for every release candidate and after changes to windowing, rendering, input, packaging or DPI. Record OS build, Shadow client version, display resolution/scaling, GPU, Bevy version, build SHA, timestamp, and evidence paths.

### Install and recovery
- [ ] Windows x64 build downloads, launches and exits cleanly; no missing runtime libraries.
- [ ] Fresh install, update, reinstall and save-data retention behave as documented.
- [ ] `--safe-window` or documented equivalent restores an opaque interactive window.
- [ ] If transparent mode fails, player can recover without editing game source.
- [ ] Crash logs and runtime diagnostics are saved with no sensitive user paths or account details exposed publicly.

### Transparency and focus
- [ ] Transparent areas show desktop/other apps; no black/opaque rectangle, flickering or alpha artifacts.
- [ ] Idle click-through passes pointer input to a controlled background test app (log actual received clicks).
- [ ] Idle recovery shortcut/control works; Interaction mode captures clicks; returning to Idle restores passthrough.
- [ ] UI interactive hotspots are correct; inactive regions do not block the desktop.
- [ ] Passive NPC sales, notifications and timer events do not steal keyboard focus.
- [ ] Always-on-top OFF/ON works and setting persists.
- [ ] Minimize/restore and Win+D/task switching behave without trapping input.

### Placement and scaling
- [ ] Ambient 960x180 and Expanded 960x540 expand upward with stable bottom edge.
- [ ] Bounds remain within Windows work area/taskbar and do not jump on repeated 100x toggle.
- [ ] Test scaling at 100%, 125%, 150% and, if available, 200%; no misplaced controls.
- [ ] Test multi-monitor placement, monitor disconnect, resolution change and position restoration.
- [ ] Fullscreen application interaction and virtual desktop switching do not cause unwanted overlay behavior.

### Rendering and gameplay
- [ ] Pixel-art edges are readable with nearest-neighbor/integer scale policy; no unexpected blur.
- [ ] Character/forge visual states and animations render with no transparency artifacts.
- [ ] NPC enters, inspects display, purchases exactly once, holds the purchased sprite and exits.
- [ ] Management UI opens/closes without changing the simulation state or blocking the taskbar.

### Stability and resources
- [ ] 30+ minute idle soak: record CPU, GPU, memory, FPS/frame time, process growth and anomalies.
- [ ] Test active animation, expanded panel and minimized/occluded state separately.
- [ ] Sleep/wake, lock/unlock, app restart and offline catch-up preserve save consistency.
- [ ] Confirm Windows Defender/SmartScreen and Steam packaging behavior; record failures.

### Shadow-specific caveats
- [ ] Separate streamed image compression/input latency from actual rendered frame defects.
- [ ] Use in-guest screenshots and in-guest CPU/GPU measurements rather than streamed frames alone.
- [ ] Treat Shadow configuration as **one** Windows hardware profile, not universal Windows compatibility.

## 5. Evidence template

| Field | Value |
|---|---|
| Build SHA / version | |
| Platform + OS version | |
| GPU + driver | |
| Resolution + scale | |
| Shadow client/version | |
| Test scenario | |
| Expected / actual | |
| Pass / fail / blocked | |
| Screenshot/video/log path | |
| Issue link + severity | |
| Tester + date | |

Attach failures to GitHub issues with reproduction steps and supporting files. Do not merge a release candidate with a failing critical input-recovery or background-click-through test unless safe-window fallback is explicitly accepted as the supported mode.

## 6. Automation and ownership

- Agents own Rust unit/property/integration tests, CI, deterministic fixtures, build artifacts, visual captures and QA scenario commands.
- Developer performs Shadow PC acceptance and records evidence; agents interpret failures and make fixes.
- macOS is the primary native integration target; test at least real Apple Silicon, with Retina and sleep/resume.
- Platform claims in README/Steam must reflect tested builds, not just compilation.
- No separate throwaway window POC required: implement window shell as product code and preserve its regression tests.

## 7. Release gates

**Foundation gate:** macOS window + recovery, deterministic tests, headless ECS, screenshot capture, Windows CI build.

**Windows release gate:** complete Shadow PC checklist, evidence recorded, no unresolved critical window/input/transparent rendering defects, recovery path confirmed.

**Steam gate:** signed/notarized macOS package as applicable, Windows distributable, save migration, logs, crash handling, platform-specific smoke tests.

References: Bevy 0.20 window examples/API, winit platform behavior, Steamworks documentation. Verify against pinned version during implementation; avoid assuming platform parity.
