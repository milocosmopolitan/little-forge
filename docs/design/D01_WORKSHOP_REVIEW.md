# D01 workshop layout, window and input decision packet

Status: **D01 limited implementation authorized; visual acceptance pending**. This refines PR #120 after owner review. Main was refetched and remains `6c6e8339f070229f8ea67925aeede550fee54a9d`. No runtime or asset change accompanies this packet.

## Approved decision

Previously approved: indoor default; logical960×180 collapsed /960×540 expanded; persistent lower180px workshop; upper360px management; exterior secondary. Owner agrees with reuse/recomposition and bottom viewport direction, conditional on actual960×180 visual review. This is not approval of final station coordinates.

Owner clarified two stages. **Stage1 approved:** reuse existing characters/stations at native1×, indoor default, same live lower180px, upper360px management host; retain default whole-window input plus manual F8/10-second recovery. This is the first implementation fallback, not final product UX. **Stage2 product goal:** ambient noninteractive background passes clicks to desktop applications while semantic NPC/station/button regions receive input. Investigate native macOS/Windows behavior separately in [#121](https://github.com/milocosmopolitan/little-forge/issues/121), after the live scene is visually stable. Management uses normal input with modal focus ownership.

Official initial management minimum is960×540 logical usable pixels. Smaller-display accessible layouts remain future support, not a reason to squash characters or text. Final station coordinates, backdrop composition, small-display readability and platform selective-input implementation are not approved. D02–D07 are not approved as complete contracts. The owner now explicitly authorizes a separate bounded implementation PR for #107/#118/#63/#89; do not implement Epic #106 wholesale or merge PR #120 automatically.

## Existing implementation and evidence

- `crates/forge-core/src/shell.rs:11,38–58`: bottom-preserving rectangle fitting,10-second lease, safe-mode denial and automatic recovery. `apps/desktop/src/window.rs:34–95` applies whole-window hit testing and native work-area/DPI geometry. This is reusable policy, not proof of new layout.
- `apps/desktop/src/native.rs:15–40`: Ctrl+Alt+F8 registration can fail; status reports failure and the timed recovery remains available. Do not promise a global shortcut always registers.
- `apps/desktop/src/main.rs:719–747`: existing modal guard, F8, Tab and T/D shortcuts. Current Esc recovers interaction rather than implementing the proposed route stack. Bare-letter commands must not leak into focused text inputs or management screens.
- `crates/forge-npc/src/stations.rs:60–95`: current480×270 scene, back floor160/front floor245, fixed station and idle spots. `:111` transforms layers at scale; `:134` preserves authored local occlusion. `apps/desktop/src/spaces.rs:295–306,371` centers/scales the scene and drives the actor. A bottom viewport alone will clip the old layout.
- Both `assets/characters/{male,female}/character.json` specify128×160 canvas/pivot(64,140). Their animation manifests' `workshop` registration defines character origin(32,16), station origins, contacts and occlusion. These are constraints to preserve, not permission to redraw canonical actors.

## Proposed resolution A: one horizontal work lane

Use native1× sprite scale at logical960px width. Keep the actor foot at y=156; the full frame then occupies y=16..176, leaving16px above and4px below. Not every inactive station must be fully visible at once: emphasize active work, allow inactive equipment to blend into the scene, and preserve usable navigation. All animation-frame alpha, held tools and station layers still need verification; frame arithmetic is not playback proof. Do not enlarge the actor when management opens.

Candidate work positions, **for validation, not approved artwork**:

| Location | Foot x,y | Purpose |
|---|---|---|
| Research |112,156 | left work desk |
| Rest |272,156 | idle/needs station |
| Forge |416,156 | heat |
| Anvil |576,156 | hammer |
| Quench |720,156 | cool |
| Salvage bench |864,156 | reuse existing scene object; no new salvage command |

```text
logical960×180, same composition in both window modes
x0     research      rest        forge      anvil      quench      bench     960
       foot112       272         416        576        720         864
          <---------------- single walking lane, foot y156 ---------------->
y180   ------------------------------------------------------------------
Expanded: this exact rectangle moves to client y360..540; it does not rescale.
```

Recompute station origins from `candidate_foot + authored_station_origin - (character_origin + pivot)`, separately validating both smith manifests. Translate station/contact/occlusion together; do not move hands, tools or character pivots to repair a scene-placement error. Preserve local layer order for each station. Replace the old two-row scene policy only in the compact scene layout; keep authored animation previews and their contact fixtures intact. Walking is presentation, not a new work-duration or economy rule.

Alternative B: retain the full480×270 room with pan/zoom inside the strip. It preserves more existing composition but hides destinations, complicates picking and breaks a stable ambient view. Reject as the first-slice default. A240%-wide stretch or vertical squash of the backdrop is also rejected. Reuse/crop/tile only an explicitly reviewed wall/floor region; if source art cannot produce a coherent960px backdrop, record the asset gap rather than generate new art in this review.

## Proposed window and interaction policy

Use the exact camera/coordinate contract in [SCREEN_ARCHITECTURE](SCREEN_ARCHITECTURE.md). Below are policy choices to approve independently of visual composition.

| Situation | Proposed behavior | Domain/simulation |
|---|---|---|
| Manage button or Tab from ambient | expand upward to960×540; open dashboard host; focus first meaningful control | no command; same actor/work continues |
| Close or Esc from top-level management | collapse to960×180; preserve last selection only in session | no work reset |
| Esc in modal/detail | close only topmost layer, restore prior focus; modal blocks all underlying actions | world still renders and simulation advances |
| Click station in either mode | open relevant management view; no automatic purchase/queue/claim | explicit control issues commands |
| F8 / visible click-through action, no modal | collapse, enter approved10-second whole-window passthrough | no purchases/commands; existing work continues |
| Ctrl+Alt+F8 or lease expiry | regain input in collapsed view; do not steal foreground focus or reopen management | no command; preserve simulation |
| Safe opaque window | passthrough disabled; normal UI input available | same gameplay |
| Unfocused/minimized | no automatic view changes; existing capped catch-up policy | no new pause semantics |

The chosen input policy is approved; details such as focus restoration and mouse access to the passthrough action are proposed implementation requirements. Show a small countdown/recovery hint during passthrough without a clickable overlay that intercepts the background. Ctrl+Alt+F8 registration failure must remain visible before enabling the lease.10 seconds is elapsed wall time, not simulation time; after suspend/restore, expire a lease whose deadline has passed. This strengthens the current tick-based behavior and requires native testing.

Recommend one explicit drag handle, shared across collapsed/expanded modes; never interpret clicking smith/workstation as window dragging. Preserve position and always-on-top preference; do not enable topmost automatically. No automatic expansion on rewards or NPC arrival. Modals stay in the upper360px and block lower hit regions while retaining lower rendering. Title/character choice may remain the separate existing shell before gameplay entry; expedition observation is not a full-window exception.

For nominal geometry, keep bottom edge within1 physical pixel through resize; clamp to the OS work area when necessary. On clamp, show the full client and accept the documented bottom shift instead of offscreen controls. Multi-monitor coordinates may be negative. One layout resource owns logical/physical conversion; no independent DPI multiplication in hit regions.

## Open decision: small displays and visual approval

Use the approved960×540 **logical usable work area** as the first qualification target, excluding Dock/taskbar. Keep a smaller-area recovery fallback but do not claim full readability at640×480 from uniform scaling alone. The earlier uniform-fit proposal remains an option for later qualification; neither shrinking to640×360 nor scrolling a full management canvas is approved yet. At125/150% DPI, physical pixel edges will vary even with nearest sampling; evaluate readability and sprite consistency on actual displays. Do not silently reinterpret960×540 as physical pixels to make a high-DPI screen fit.

D01 review gates, in order:

1. **Policy:** owner approves geometry/recomposition direction and chooses input behavior. Input choice is now recorded above; exact coordinates and drag behavior remain proposed; the initial management minimum is approved.
2. **Permission for a bounded real-Bevy validation slice:** owner authorization now covers only persistent indoor strip, top management host and input behavior. Do not require completed final Bevy visual approval before this validation slice can exist. Record baseline CI failures separately; do not hide them or require unrelated baseline fixes before starting this authorized slice. Do not run the old full implementation prompt.
3. **Visual/native acceptance:** owner reviews actual960×180 and960×540 playback for both smiths. Only then freeze scene placement and claim D01 visual approval. Broader screens remain blocked by their own domain decisions.

Required evidence for gate3: idle, walking, forge, hammer, quench and research contacts; needs/rest motion as currently authored;100 expand/collapse transitions without actor/work restart; both temporal frames and video; no clipped heads/tools/flame/feet; backdrop seams inspected; lower scene unchanged while upper controls update; modal input isolation; actual mouse+keyboard tests; passthrough sends a click to a background app, then automatically recovers after10s; unavailable global shortcut and sleep/wake recovery; macOS and Shadow Windows, normal/safe modes and100/125/150/200%DPI. Report any smaller-screen result separately. Use disposable saves and real command state. No decorative mockup substitutes for this evidence.

## Explicitly excluded scope

This documentation change implements no runtime or assets. The separately authorized slice excludes new character generation, unrelated QA fixes, blanket D01–D07 approval, capacity merger, research/economy/party balance adoption or automatic merge. Follow-up order is D01 policy/validation planning → D05 identity/ownership review → D02/D03/D04 balance; D01 limited implementation may now proceed; visual acceptance remains a separate owner gate.
