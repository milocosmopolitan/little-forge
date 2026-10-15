# Foundation implementation / acceptance map

Code availability, automated validation and native acceptance are separate. The epic is not done until native gates are recorded.

| Issue | Implementation | Automated evidence | Native/manual gate |
|---|---|---|---|
| #2 workspace | four crates/apps, pinned versions, lockfile, documentation migration | fmt, clippy, workspace tests on CI | Apple Silicon launch |
| #3 window | ambient/expanded, native work bounds, position/preferences, global recovery, timed lease, opaque fallback | pure geometry/recovery tests; desktop build | actual focus, pass-through, mixed DPI, 100 toggles |
| #4 macOS | PostMultiplied alpha, main-thread AppKit visible frame, global shortcut | macOS compile/tests | Apple Silicon Retina, Spaces, Dock, sleep/wake |
| #5 scene | original forge/character/smoke/workbench PNGs, manifest/hash/alpha checks, nearest atlas sampling | asset checks; fixed-frame screenshots | scale and sprite alignment approval |
| #6 visitor | ordered committed sale events, ENTER/WALK/INSPECT/PURCHASE/EXIT_WITH_ITEM, shared dagger image | scheduled Bevy sprite/visibility/hand-position test and dedup FSM test | native visual observation |
| #7 simulation | integer clock, seeded RNG, atomic commands, stock/gold invariants | 99 seeded partition replays, overflow/duplicate/failure tests | none |
| #8 saves | two-slot journal, OS writer lock, persisted IDs/RNG, offline cap and high-water clock | corruption, abandoned temp, future schema, restart, lock, duplicate reward tests | suspend/OS shutdown behavior |
| #9 QA | three OS matrix, isolated scenarios, screenshots, comparison tool, retained artifacts | CI and reviewed baseline comparison | golden approval by renderer; no cross-OS pixel equality claim |
| #10 Windows | portable package, F12 diagnostics with adapter/FPS, safe recovery and handoff | Windows x64 build/tests | **owner executes Shadow PC checklist** |
| #11 performance/package | portable Windows/Linux and unsigned macOS .app, process sampler, simulation baseline | CI packaging and headless benchmark | native CPU/GPU/RSS/FPS profile, sleep/wake, private Steam smoke |

The Foundation timings are intentionally accelerated. No village economy/card/expedition completion is implied.
