# Screen acceptance and qualification matrix

## Approved decision

A screen requires real Bevy rendering, authoritative commands/state, input/save/offline verification and screenshot review. Fixtures, merged PRs and closed issues alone cannot qualify it. No failed test or golden image is altered in this audit.

## Existing pipeline and freshly executed evidence

Audited source: `6c6e8339f070229f8ea67925aeede550fee54a9d`. Local macOS arm64, Rust workspace locked. `cargo test --locked --workspace`: **96 tests passed** (forge-content5, forge-core58, forge-npc12, forge-ui2, desktop17, each preview1). `cargo fmt --all -- --check` and strict workspace/all-target Clippy pass. Five validators (`validate-assets`, `validate-village`, `validate-ui`, `validate-spaces`, `validate-expedition`) pass under isolated Python with repository-pinned Pillow11.3.0/fonttools4.60.1/psutil7.0.0. Initial system-Python run lacked PIL; it is an environment setup failure, superseded by the isolated rerun, not a project defect.

Two fresh failures retained: `validate-ui.py --release` rejects source-manifest.runtime_ready=false; `export-farmer-runtime.py --check` reports stale `assets/npc/farmer/runtime-atlas.png`. Prior CRAFTING-COMBAT report found equal decoded RGBA with differing encoded bytes; this audit reproduced the check failure, not that pixel comparison. Do not infer visual corruption or waive byte reproducibility.

CI at [main run39181315993](https://github.com/milocosmopolitan/little-forge/actions/runs/39181315993): latest observed Ubuntu/macOS verify success, visual failure, Windows verify failed; the run is completed with failure. Current failed logs confirm the same ambient ratio0.11321180555555556 and Windows240/360s Alpha timeouts ([log](screen-audit/ci-final-failed.txt)). Previous same-code gameplay head3fe3476 [run39174229808](https://github.com/milocosmopolitan/little-forge/actions/runs/39174229808) has Ubuntu/macOS success; Linux ambient changed_ratio0.11321180555555556 versus allowed0.005 at960×180; Windows Alpha timed out240s and crafting Alpha360s. Do not describe that Windows failure as a golden mismatch. Compiler success is separate from those runtime failures. Native Shadow PC remains unverified.

Fresh macOS safe-window crafting capture **passed**, including Chef → research → common knife → Wave3 retreat → tailored knife → five-wave completion → Codex, with equal save/reload. It used production Activate controls,0.1s/frame and initial ore40/gold80/timber8 grants. [Fresh native screenshot](screen-audit/native-common-crafting.png) confirms the existing opaque full-height management screen, with no visible lower workshop. Full resulting state is retained in [native evidence](screen-audit/native-crafting.json). This verifies that path, not redesigned screens, native transparency, normal economy or manual input. Existing screenshot files were visually inspected as historical composition evidence, not assumed fresh.

### What each harness proves

| Harness | Real command path? | Scope / limit |
|---|---|---|
| forge-core tests / forge-qa | yes, with explicit fixture resources in some cases | deterministic transactions/save/gear effects; no UI/native proof |
| ui-preview / capture-ui | no game simulation | shared widget fixture layout only |
| npc-preview / capture-player | no gameplay | motion/atlas/contacts only |
| capture-alpha + alpha_qa.rs | production Activate controls | actual Bevy + accelerated0.1 sim-sec/frame, seeded resources in crafting case; not OS input |
| capture-expedition / capture-forge | mixed fixtures and state assertions | actual rendering; inspect each scenario's preloaded state before claiming played loop |
| test-village-input / test-forge-input / test-title-input | real XTest input | Linux X11 only, not macOS/Windows input or transparency |
| validate-* | asset/data structure | no aesthetic/native qualification; UI --release is separate gate |
| capture-windows.ps1 | Windows builds/captures | distinguish WARP/software from owner GPU; does not replace Shadow checklist |
| profile-process / soak-lifestyle | process sampling / production clock | CPU/RSS and lifecycle; GPU power/frame time requires native measurement |

`.github/workflows/ci.yml:17` verifies three OS; later visual job stops at first golden failure, so skipped downstream tests are not passes. Soak is workflow_dispatch opt-in, not routine coverage. `scripts/capture-alpha.py:27` and `:50` have240/360s subprocess budgets; diagnose completion clock/renderer before changing any timeout. Bevy startup uses30Hz focused and15Hz unfocused reactive redraw (`main.rs:468`), not a proven60FPS target.

## Proposed minimum evidence envelope

Every screen PR must attach build SHA/dirty status, content+save versions, OS/arch/GPU/driver/renderer, logical/physical dimensions+DPI, scenario seed/initial save, actual command sequence, before/after state assertions, screenshots and input method, final save/reload hash, logs and unresolved failures. Separate plain read-only fixtures from played state. Use disposable saves; never the player's normal save.

Common checks for all twelve: empty/default, active work, completion/reward, locked/unavailable with reason, save/restart, online versus partition/offline including8h cap/backwards clock, Korean/English/long labels, mouse/keyboard/focus/Enter/Space/Esc, opaque panels+transparent exterior,100/125/150/200%DPI,640×480 work-area fallback, title/continue and both smiths. Non-mutating screens must prove they do not change command/event IDs or rewards. Capture before/after states, not just static pretty screenshots.

| Issue | Automated domain/integration witness | Actual desktop/native witness |
|---|---|---|
| #107 | unchanged entity/clip/work IDs over100 toggles, geometry edges, modal guards | both smiths at all work stages; two temporal frames/video; actual background clicks and recovery |
| #108 | resolver priority ties, blocked/complete/idle and missing legacy facts | CTA reaches exact target, return summary, no focus steal |
| #109 |4 NPCs, repeat template IDs, deadlines−1/equal/+1, ignored/late/fail, entrusted isolation | accept→source→queue→deliver via mouse and keyboard; restart while each phase active |
| #110 | exact paid inputs/result identity/quote version, capacity, duplicate/overflow rollback | plans produce visibly inspectable real property differences; queue full and pending research |
| #111 |16-node DAG/AND/OR/clues/skills/work proofs, no RNG/self-lock, option unlock consumers | horizontal/vertical scroll+focus; actual node completion enables recipe, preserved live research motion |
| #112 | hold/browse/sale race, chosen item snapshot, stable slots on upgrade, one gold credit | browse/reject/purchase/same-item carry-out, full/held/overflow and capacity resume |
| #113 | real region/mode gates, no debit on selection, honest locked rows | Back/Next carries exact destination and selected-smith narrative |
| #114 | two actors/targets/intercepts, duplicate gear, deployment costs, zeroHP/recall/offline parity | two-member composition changes tactics, Guard/Hunter clips attributed to real events |
| #115 | knowledge separate from inventory, best quality/unknowns, sold retention | recipe/research/instance links and family/filter/locked states |
| #116 | first/best/count dedup, repair distinction, retained/archived/legacy provenance | exact crafted→sold/loaned item still inspectable; no invented dates |
| #117 | mode-specific best/first, party snapshots, event attribution, trace eviction aggregates | failure→improvement→retry comparison, historical weapon links after sale |
| #118 | source/manifest/font/slice bounds, all route/modal guards, no hidden activation | visual owner approval, normal/safe windows, input, native hardware/performance report |

### Native platform qualification

macOS: Metal normal transparent launch and safe launch separately;100 toggles, Dock/work area, Spaces, monitor scale/disconnect, focus changes, background app receives click-through, keyboard recovery with game unfocused, sleep/wake and crash-safe reload. Windows: native build/test plus in-guest Shadow PC capture with OS/GPU/driver/Shadow stream scale; DX12 alpha/shader regression, taskbar/work-area,100–200%DPI,Win+D,virtual desktop and recovery. Follow existing [platform checklist](../system/PLATFORMS.md) and [handoff](HANDOFF.md). Streaming compression/latency is not game-rendering evidence. Linux CI is regression evidence, not an unsupported platform promise.

### Performance and storage plan (proposed budgets, approval required)

Profile fresh and maximum-retention save, empty and4-member-data/2-active-party scene, all16 graph nodes, full racks and repeated navigation. Warm30s, sample60s per ambient/expanded/unfocused/minimized/restore;30-minute production-clock soak. Record p50/p95/p99 CPU frame, GPU frame when available, RSS peak/steady, loaded texture bytes, allocations, save-write time and redraw rate. At intended30Hz active/15Hz unfocused, proposed p95 application work budgets33.3/66.7ms; do not treat sleep/presentation cadence as processing time. Compare same machine/build settings; no >10% RSS growth after100 route toggles beyond warmed cache without explanation. Initial proposals are not measured guarantees; owner/hardware qualification must set final budgets. UI registered base texture estimate13.3002MiB is static only. State cloning plus unbounded current history needs scale testing before retention acceptance.

Golden maintenance: keep current baselines/tolerance8/changed ratio0.005 untouched. A failing comparison must produce diff, source SHA and reviewed diagnosis. Later deliberate visual changes need a separately reviewed baseline update with old/new images, not an automatic regenerate step. No tests skipped to make audit green.

## Open decision

D01 minimum readable work area/pixel/DPI behavior; D05 history/storage bounds; platform performance budget after measurement. #68/#96 own existing QA defect triage; #118 coordinates screen acceptance; #84 full loop. No claim of readiness while their material gates fail.

## Explicitly excluded scope

No QA result fabrication, screenshot-only closure, baseline change, test weakening, new native platform guarantee, or implementation of redesigned screens in this audit.
