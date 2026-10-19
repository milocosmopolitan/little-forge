# Individual screen audit, #107–#118

Baseline: `6c6e8339f070229f8ea67925aeede550fee54a9d`. Approved screen direction comes from #106; **all technical resolutions below are proposals until the referenced decision is accepted**. Existing implementation is distinguished from required work. No screen is declared implemented or ready by this audit. [Sequence/decisions](IMPLEMENTATION_SEQUENCE.md) and [shared QA](../qa/SCREEN_ACCEPTANCE_MATRIX.md) apply to every definition of done. No gameplay/screens/assets are implemented here; full RPG, new art during audit and unrelated refactors are excluded.

## #107 — Ambient Forge

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

`apps/desktop/src/spaces.rs:34,268,371` defaults Exterior, centers 480×270 interior and drives existing smith/stations from activity. `window.rs:95` / `shell.rs:11` already preserve bottom edge on resize; `main.rs:560` has one Camera2d. Both smiths have 17 clips; compact indoor composition is absent.

### Missing entities, command and event contract

No new gameplay entity. Add authoritative viewport/layout resource and one route owner; preserve persistent world entities, animation phase, work clock, smith selection and exterior secondary route.

### Missing gameplay rules / open decisions

D01: approve native1× compact station composition, reference files, small work-area policy and interactive hit-testing. Never squeeze a160px character into a90px native strip.

### Required UI components

WorldCamera viewport + UiCamera targeting; GameUiRoot/ManagementHost/AmbientHitHost; canonical station hit regions, quiet activity chip, Manage/Exterior controls.

### Assets / transformations

Register crop/layout from assets/forge/spaces/workshop-backdrop.png and existing station layers; preserve contacts, occlusion, trim and pivot; no new smith.

### Navigation / transitions

Ambient→Dashboard expands upward; all management routes preserve world; Close→indoor Ambient; exterior only by explicit transition. See exact coordinate/state table.

### Save / migration

No domain schema change for route. Preserve window.json preferences and schema7 smith/work. UI route/selection is ephemeral; continue enters indoor ambient.

### Implementation tasks (proposed)

- [ ] Implement #63/#89 unified route/input and proposed viewport conversion after D01 approval.
- [ ] Adapt compact station layout preserving same Actor/Walker entities across 100 toggles.
- [ ] Bind station actions to existing state commands and retain safe-window/native recovery.

### Automated/native QA and exact definition of done

- [ ] At960×180 and960×540 compare bottom position within1 physical pixel; work/animation progress must continue over100 toggles.
- [ ] Both smiths perform heat/hammer/quench/research, idle/sleep/eat/drink; restart at mid-work and compare result once.
- [ ] Native macOS + Shadow Windows prove alpha, actual background click receipt, recovery, DPI100/125/150/200%; screenshot alone fails.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#118 D01 asset/layout gate; #63/#89 routing; #91 station integration; #96 acceptance.

Existing centered full-height layout, multiple roots, 2× clipping and modal leaks. Final native acceptance still required.

Detailed contract: [SCREEN_ARCHITECTURE.md](SCREEN_ARCHITECTURE.md).

## #108 — Workshop Dashboard

**Status: Blocked — Domain Dependency**

### Existing implementation / reusable sources

`State::activity` in core spaces.rs:136, order phases in commission.rs:45/progression.rs:108, Battle cause/path in expedition.rs:82,347 supply initial selectors. Desktop lifestyle tabs/progression_ui.rs:17 have no goal resolver.

### Missing entities, command and event contract

Pure DashboardModel/Objective and typed deep links; no persisted duplicate quest counter. Read generic order/work/research/trip projections when their owner issues land.

### Missing gameplay rules / open decisions

D06 priority ordering and acknowledgement; D02–D04 new mechanic objectives cannot be invented by UI.

### Required UI components

One goal, active work, next-action card; blocked reason/link; quiet return summary; screen navigation and semantic focus.

### Assets / transformations

Existing panel/button/icon exports and selected-smith portrait; missing reviewed dashboard reference; no new illustrated admin tiles.

### Navigation / transitions

Use typed routes for all panels; CTA preserves return context to selected order/recipe/item; no navigation command debits resources.

### Save / migration

Only optional notification acknowledgement/preference persistence. Milestones/rewards remain in domain; old unknown progress shown honestly.

### Implementation tasks (proposed)

- [ ] Add pure resolver using documented priority/deadline/stable-ID ordering and command eligibility.
- [ ] Integrate shared navigation and each real domain selector as available, marking partial coverage.
- [ ] Add return summary after catch-up without focus steal or duplicate settlement.

### Automated/native QA and exact definition of done

- [ ] Fixture tests cover each priority collision, blocked→available→complete, no work, all starter goals finished and legacy missing records.
- [ ] Actual desktop CTA selects exact target; domain result changes goal; repeated renders never grant progress.
- [ ] Offline return offers actual delivered work/trip facts once; keyboard focus and bottom live scene preserved.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#118/#107/#63; #77/#76/#81/#119 for generalized selectors; D06.

UI-derived fake objectives, stale CTA authorization, popup spam and contradictory progression tracker.

Detailed contract: [UI_NAVIGATION.md](UI_NAVIGATION.md).

## #109 — Orders

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

`commission.rs:6–105` implements one Farmer repair with7200s deadline/24G on-time/12G late and +5/+0 favor; `progression.rs:179` implements one Chef knife with24G/no deadline. `life.rs:294` shared runnable-job queue; tests cover ready-time boundary/restart.

### Missing entities, command and event contract

OrderTemplate/OrderInstance/NpcProgress, immutable terms, entrusted ItemId, delivery settlement and eligible feedback IDs; generic accept/queue/deliver/decline commands.

### Missing gameplay rules / open decisions

D02 deadline/offline semantics, recurring channel/caps, late/cancel/failure deltas and recovery income. NPC-local0–100 relationships; repeatable routine versus one-time special are distinct.

### Required UI components

Routine/special/done filters, NPC portrait list and scroll detail, terms/materials/deadline/reward, exact state CTA and explanation.

### Assets / transformations

Reuse assets/npc/{chef,farmer,leatherworker,carpenter}; crop registered portraits, order badges from common UI; no invented NPC variants.

### Navigation / transitions

Order detail→Crafting(order,recipe)→ready order→manual delivery; missing materials→vendor→same order. Closing panel never accepts or delivers.

### Save / migration

Migrate both special phases into exact instances retaining paid work/completion times/favor and unknown entrusted provenance; no repeated starter rewards.

### Implementation tasks (proposed)

- [ ] Approve D02, author real templates under #77 and deterministic scheduler under existing economy scope.
- [ ] Use #119 identity/ownership for entrusted/new-tool delivery and exactly-once settlement.
- [ ] Bind all four NPCs to real rules; reject unavailable templates rather than creating mock cards.

### Automated/native QA and exact definition of done

- [ ] Commands exercise offer/accept/queue/work/ready/deliver; insufficient inputs, queue full, duplicate delivery, deadline−1/equal/+1.
- [ ] Entrusted item never sells/lends/salvages; full player storage does not block its repair.
- [ ] Repeat orders across save/offline have unique IDs; penalties0–100 NPC-local and once; ignored offers do not penalize; all4 NPCs playable.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#77/#74 and #119; #118/#63; D02, D05.

Current shared offline tick conflicts with proposed paused deadlines; Chef has no generic recipe/repeat API; no shop restock or zero-resource recovery.

Detailed contract: [GAMEPLAY_CONTRACTS.md](GAMEPLAY_CONTRACTS.md).

## #110 — Crafting

**Status: Blocked — Domain Dependency**

### Existing implementation / reusable sources

`craft.rs:10` only Good::IronDagger; `life.rs:7` six WorkKind variants; `weapon.rs:116,193,231` real knife plans and properties; `expedition.rs:437` bow/arrows; `assets/progression.json` only2 recipes. Current quality/plan loop tested, general catalog absent.

### Missing entities, command and event contract

Canonical RecipeId/ItemId/WorkId, material terms, output destination and reservation; QuoteCraft/QueueWork; shared item ownership and immutable provenance.

### Missing gameplay rules / open decisions

D03 starter recipe costs and research-option effects; D05 capacity/return slots. Propose enqueue snapshot of proficiency/cost/plan, no unspecifed cancellation/refund.

### Required UI components

Category/recipe grid, selected requirements and qualities/properties, plan controls, queue list with work/block reason; exact command validation.

### Assets / transformations

Reuse registered dagger/bow icons; inventory required for six baseline products, missing icons explicitly tracked under #61/#118, no generated catalog.

### Navigation / transitions

Orders pass order ID; Research selects unlocked recipe; PartyPrep asks for gear then returns; finished ItemId links to Showroom/Chronicle.

### Save / migration

Preserve paid queue and active legacy plan computation; schema migration with #119. No retroactive quality recomputation or duplicated output slots.

### Implementation tasks (proposed)

- [ ] Land #119 and #74 recipe registry, then connect approved research gates/options from #76/#103.
- [ ] Implement quote and queue destination contract using one work queue and source snapshots.
- [ ] Replace numeric/type-only UI shortcuts with recipe/instance links, keeping unavailable options honest.

### Automated/native QA and exact definition of done

- [ ] Each supported recipe consumes exact inputs once and creates one correct instance; same item can display/hold/lend/return/sell without duplication.
- [ ] Full capacity delays general output but not entrusted repairs; failed queue/overflow leaves state identical.
- [ ] Plan properties predict actual result under saved version; quality never substitutes for penetration/handling; restart/offline equivalence.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#119/#74/#75/#76/#103/#92; #118; D03/D05.

Separate Good/armory systems, quote/result proficiency drift, false research gating, storage deadlocks and implying salvage exists.

Detailed contract: [GAMEPLAY_CONTRACTS.md](GAMEPLAY_CONTRACTS.md).

## #111 — Research Tree

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

PR #105 merged at audited main. `progression.rs:40,116,179,253` remains2-technique/1-node Alpha. `assets/progression.json` has10XP/4G study; apps/desktop/src/progression_ui.rs:17 renders one node. Document DAG analysis is attached, not engine proof.

### Missing entities, command and event contract

16-node registry,8-technique map, All/Any qualification, deterministic clue-source IDs, node-local progress, shared research job, recipe/fabrication consumers.

### Missing gameplay rules / open decisions

D03 experiment/basic-recipe costs, option magnitudes, deterministic variance alternative, automatic completion and sustainable first unlock. Keep6/12/24G and30/60/100s proposal unchanged.

### Required UI components

Horizontal six-field graph with bounded vertical/horizontal scrolling, AND/OR connectors, selected detail/requirements/start-pause-resume and unlock link.

### Assets / transformations

Reuse UI frame/buttons; six-field/state icons and node geometry need registered common art or Bevy shapes; no painted graph.

### Navigation / transitions

Research→Crafting unlocked recipe; Crafting→blocking node; selection/focus survives panel return; lower world research motion remains live.

### Save / migration

Old2-technique XP retained by ID, legacy dagger permission separate from fabricated node completions; paid study and old battles finish under original rules.

### Implementation tasks (proposed)

- [ ] Resolve D03 then implement #74/#75/#76 graph/skill/work validation.
- [ ] Test document witness paths with real commands and zero-resource/retry economy; preserve legacy entitlements.
- [ ] Build graph projection after real16-node state exists and test keyboard/scroll/detail geometry.

### Automated/native QA and exact definition of done

- [ ] All16 nodes exist, DAG/OR valid; bow/spear independent of dagger; basic knife has no research; metal shield requires sheet+structure; axe is future, no invented gate.
- [ ] Three bow experiments can reach24XP before ELA-02; no mandatory RNG/self-level lock.
- [ ] No double payment, node-local XP/time equivalence, pause/reload/offline, duplicate clue/completion; actual Bevy unlock enables actual recipe.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#74/#75/#76/#78; #119 migration coordination; #118; D03/D05.

Existing research differs materially; missing practice actions and option effects make full feasibility unproven; old XP prose can reintroduce circularity.

Detailed contract: [RESEARCH_IMPLEMENTATION.md](RESEARCH_IMPLEMENTATION.md).

## #112 — Showroom

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

`showroom.rs:8,154,178,220` slots/hold/sale/rack upgrade exist; `spaces.rs:845` actual UI; `presentation.rs:17` generic post-sale8s animation discards event payload; `lib.rs:360` immediate generic dagger sale every18–30s.

### Missing entities, command and event contract

VisitId, explicit browse/decision state and immutable purchased-item art snapshot; same canonical ItemId and stable SlotId as craft/party.

### Missing gameplay rules / open decisions

D07 recommendation price/rejection/visit timing and optional player price scope; D02 restock/economy; D05 storage. Do not equate12G generic dagger with4G crafted knife.

### Required UI components

Rack tabs, fixed64px slots, item detail, hold/display action, visitor lane, capacity/carpenter shortcut, unavailable reasons.

### Assets / transformations

Reuse showroom backdrop/window/detail-panel and item sprites; per-item grip anchors and browse/reject motion, not weapon-specific rooms.

### Navigation / transitions

From Crafting result/Chronicle to exact item; Carpenter upgrade returns to selected RackId; sold selection shows terminal fact rather than another vector item.

### Save / migration

Migrate anonymous goods and slot IDs via #119; persist visit outcome and sale event; offline visits settle without visual replay rewards.

### Implementation tasks (proposed)

- [ ] Approve D07 economy/visit contract; adapt #92/#93 to canonical items.
- [ ] Preserve selection and ownership through upgrade, hold, loan and concurrent browse decision.
- [ ] Render snapshot-based visitor carry-out and quiet offline sale summary.

### Automated/native QA and exact definition of done

- [ ] Empty/partial/full/reserved/held/upgraded fixtures plus real craft→display→hold→buy/reject→depart sequence.
- [ ] Hold during browse prevents purchase; repeated presentation never changes gold; carried sprite matches sold ItemId after removal.
- [ ] Economy reports30/60/120 minutes, full stock, zero resources, research spender and two failed trips; no baseline/timing changes during audit.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#119/#92/#93/#95; #118/#107; D02/D05/D07.

Current events lack instance identity; index shifts; no rejection model; visitor profit can dominate intended routine repairs.

Detailed contract: [SHOWROOM_UX.md](SHOWROOM_UX.md).

## #113 — Destination Selection

**Status: Blocked — Domain Dependency**

### Existing implementation / reusable sources

`expedition.rs:59,347` one hard-coded forest and path_reopened; `apps/desktop/src/expedition.rs:223` one full-screen prepare/watch flow. Existing forest art and current best results reusable; no region registry.

### Missing entities, command and event contract

RegionId, mode-specific challenge definition, unlocked/reason selector, threats/rewards/best projection and typed PartyDraft context.

### Missing gameplay rules / open decisions

D04 Solo/Party mode eligibility and milestone reward policy. One actual region is sufficient for MVP; extra regions need authored rules, not fictional cards.

### Required UI components

Region list, scenic detail, threat/reward preview, locked reason, mode selector, Back/Next. Narrative is selected blacksmith.

### Assets / transformations

Reuse Old Forest distant/midground/foreground and selected smith portrait; no new region backgrounds required.

### Navigation / transitions

Dashboard→Destination→PartyPrep(region,mode); Back restores selection; selecting/Next never pays hire or consumes gear.

### Save / migration

Region/mode added to future expedition facts; migrate known existing trips as legacy OldForestPath/Solo, never guess unseen areas.

### Implementation tasks (proposed)

- [ ] Define minimal real region/mode registry under #83/#99.
- [ ] Project unlocked/best/reward data by mode and connect #114 route draft.
- [ ] Fit scenic art and labels within upper360 with live forge below.

### Automated/native QA and exact definition of done

- [ ] Empty/locked/unavailable/mode states use actual predicates; Next transfers exact RegionId/mode.
- [ ] Back/save/restart does not launch/debit; legacy results map correctly; no unsupported region rewards.
- [ ] Keyboard navigation/localized threats readable; real destination→party→return flow at normal and constrained size.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#83/#99/#81 projections; #118/#107/#63; D04.

Pretending multiple destinations exist, mixing solo/party bests and full-window art hiding forge.

Detailed contract: [EXPEDITION_PARTY.md](EXPEDITION_PARTY.md).

## #114 — Party Hiring & Equipment

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

`expedition.rs:34,119,128,501,610` one Hunter loadout/health/target, no hire contract or actor attribution. Existing loan validation and snapshots reusable. Hunter20-frame derivative exists; Guard absent.

### Missing entities, command and event contract

MemberId/Role, draft roster max4 enabled2, deployment contract, per-member equipment/health, simultaneous enemy IDs/intents, attributed trace and one settlement.

### Missing gameplay rules / open decisions

D04 hire fees/eligibility, Guard combat table, equipment legality, supplies, retreat and explicit mode rewards. No arbitrary HP multiplier.

### Required UI components

Two active roster cards plus future capacity, selected-member slots, owned-item picker, cost/supply quote, formation preview and Deploy confirmation.

### Assets / transformations

Preserve Hunter canonical hashes; Guard idle/walk/brace/intercept/attack/hit/return clips and shield sockets required later; no art generated by audit.

### Navigation / transitions

Destination back/next preserves draft; Crafting shortage link returns; Deploy commits atomically; observation hidden still simulates; return report links Field Legacy.

### Save / migration

Preserve active battle versions1/2 to finish unchanged; new version stores member/target IDs and launch snapshots. D05 return-slot reservation and history mapping.

### Implementation tasks (proposed)

- [ ] Approve D04 complete encounter/fee table before combat implementation.
- [ ] Implement #81 contracts and #99 minimal multi-actor resolver, #82/#83 result/milestones; keep solo regression.
- [ ] Commission #101 Guard motion after action contract, then bind real #114 UI to deploy/settlement commands.

### Automated/native QA and exact definition of done

- [ ] Reject duplicate member/equipment, unavailable role, missing gear/fees/supplies without debit.
- [ ] Deterministic intercept/protected ranged/multiple-threat tests show composition advantage beyond total HP; poor-quality match can outperform mismatched masterwork.
- [ ] Save each action boundary, recall, zero HP, no arrows/food and8h catch-up return items/pay once; native two-person battle has attributed animations.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#119/#81/#99/#100/#101/#82/#83/#103 and #113; #118; D04/D05.

Unapproved game rules, missing Guard art, sequential battle assumptions, lost return capacity and four-slot UI implying four playable members.

Detailed contract: [EXPEDITION_PARTY.md](EXPEDITION_PARTY.md).

## #115 — Weapon Collection

**Status: Blocked — Domain Dependency**

### Existing implementation / reusable sources

`progression.rs:91,108` real first/count/optional quality; `weapon.rs:231` knife quality history survives sale. Current apps progression_ui.rs shows small Codex only; no complete archetype knowledge registry.

### Missing entities, command and event contract

Archetype knowledge projection from canonical recipe/research/events, separate current ownership count and best comparable quality.

### Missing gameplay rules / open decisions

D05 legacy unknown history and comparable rulesets; D03 actual recipe knowledge. No60-item catalog expansion required.

### Required UI components

Family tabs, known/unknown silhouettes, count/best badge, detail with research/craft links; independent screen.

### Assets / transformations

Reuse real registered weapon art/silhouettes; missing archetype icons tracked with actual supported recipe list, not decorative completion.

### Navigation / transitions

Collection→Research node or Crafting recipe; current item links Chronicle; unknown deep link shows truthful unavailable state.

### Save / migration

Aggregate known old counts/first dates preserved; unknown quality remains None; sale/loan never erases knowledge.

### Implementation tasks (proposed)

- [ ] Land #74/#79/#119 knowledge/event projection and align unlock gates.
- [ ] Build standalone family/detail screen with same IDs as crafting.
- [ ] Test legacy unknowns and records surviving ownership changes.

### Automated/native QA and exact definition of done

- [ ] Actual craft increments count once, research changes recipe availability, sale preserves entry and best.
- [ ] Unknown silhouette does not leak locked recipe detail; no fabricated old entries.
- [ ] Save/offline/replay and keyboard/localized list/detail state pass through desktop.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#74/#76/#79/#119; #118/#63; D03/D05.

Archetype conflated with instance, knowledge inferred from inventory, fake best quality and inconsistent unlocks.

Detailed contract: [CODEX_EVENTS.md](CODEX_EVENTS.md).

## #116 — Crafting Chronicle

**Status: Blocked — Design Decision**

### Existing implementation / reusable sources

`progression.rs:101,147` milestone text IDs; `weapon.rs:231` stored craftsmanship and quality record; no general chronological item facts or retention bound for progression.weapons.

### Missing entities, command and event contract

Durable CraftFact/disposition event, canonical ItemId and historical provenance snapshot, permanent first/best summaries plus bounded chronology.

### Missing gameplay rules / open decisions

D05 retention/pinning and legacy migration; D06 optional player-authored notes scope. Proposed256 recent craft events/16 pinned works must be approved, not silently enforced.

### Required UI components

Scroll chronology, filters, selected individual provenance/disposition, first/best milestone and related trip links.

### Assets / transformations

Existing item sprites and quiet event badges; real maker portrait optional; no fake written diary or new decorative background.

### Navigation / transitions

From crafted/sold/lent item to same history; Chronicle→recipe→Crafting; expedition references→Field Legacy.

### Save / migration

Keep recorded creator/material/quality/version; unrecorded old history marked unknown; archived detail labelled, aggregates never reset.

### Implementation tasks (proposed)

- [ ] Approve D05 retention semantics and note scope; implement #79 facts on #119 identity.
- [ ] Process creation/disposition once in same transaction; avoid rendering-time event writes.
- [ ] Implement standalone chronology with bounded pagination and explicit unknown/archived states.

### Automated/native QA and exact definition of done

- [ ] Craft→sell and craft→loan→return retain exact provenance; repairs do not falsely increment crafted count.
- [ ] Duplicate events/reloads/retention overflow preserve counts/first/best and retained trip references.
- [ ] No UI fabricated dates/creators/quality; save/restart and long Korean labels tested.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#119/#79/#103; #118/#63; D05/D06.

Unbounded cloned State, identity collisions, invented legacy entries and quiet history deletion.

Detailed contract: [CODEX_EVENTS.md](CODEX_EVENTS.md).

## #117 — Field Legacy

**Status: Blocked — Domain Dependency**

### Existing implementation / reusable sources

`expedition.rs:347`16 prior + active battles and path_reopened; `:610` weapon counters, one-time8 timber and story causes. `weapon.rs:407` retry/history test; current field Codex is inside expedition UI.

### Missing entities, command and event contract

Region/mode-expedition summaries, member/equipment launch snapshots, attributed raw/applied contributions, stable event/settlement IDs.

### Missing gameplay rules / open decisions

D04 mode/milestone rules; D05 history bounds. Old raw overkill damage must not be relabelled applied contribution.

### Required UI components

Region/mode selector, result chronology, party/loadout summary, failure/first-clear/best badges and item links; independent screen.

### Assets / transformations

Existing forest art, actual NPC portraits and equipment; Guard portrait only after approved production; graph/bar values from recorded facts.

### Navigation / transitions

Return dialogue→exact trip→item Chronicle→recipe/research→retry same destination/mode; no automatic relaunch.

### Save / migration

Keep retained old Solo/Hunter facts without invented missing snapshots; new party records versioned. Aggregates survive full trace eviction.

### Implementation tasks (proposed)

- [ ] Implement #82/#83/#79 projections using #119 identity and #81/#99 party facts.
- [ ] Separate historical mode/rules compatibility and honest unknown/archived details.
- [ ] Build standalone report and cross-screen retry/item links.

### Automated/native QA and exact definition of done

- [ ] Actual fail→new suitable weapon→retry succeeds; IDs and contributions match trace and canonical item history.
- [ ] Settlement and first-clear reward once under replay/offline; sold weapon still inspectable.
- [ ] 16-trace eviction preserves best/first/region counts; party and solo records never merged into misleading best.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#119/#79/#81/#82/#83/#84/#99/#103/#114; #118; D04/D05.

Adjacency-based kill attribution fails with multiple actors, bounded trace eviction loses story, UI overstates causality.

Detailed contract: [CODEX_EVENTS.md](CODEX_EVENTS.md).

## #118 — Shared assets and integrated QA

**Status: Blocked — Existing Defect**

### Existing implementation / reusable sources

`forge-ui/src/lib.rs` shared widgets/focus/dialogs; ui-preview explicitly fixture-only.47 runtime exports validate but runtime_ready=false. Current96 workspace tests and5 basic validators pass. Farmer exporter fails; UI release gate fails; Linux golden/Windows timeout evidence recorded.

### Missing entities, command and event contract

No new gameplay store. One shared UI contract, reference/asset manifest, layout metrics, semantic focus, reproducible evidence manifest and source-pinned test scenarios.

### Missing gameplay rules / open decisions

D01 concept registration/native1× strip/size acceptance; D04 Guard actions; D07 visitor visuals. QA budgets are proposed until measured and approved.

### Required UI components

Nine-slice frames/buttons/slots/progress, scrollbars, tooltips/modals, localization keys; actual route and command guards shared by mouse/keyboard/Activate.

### Assets / transformations

Register11 concept source files/hashes; reuse47 controls/icons/headers; Guard and compact room exports pending. No baseline/image edits during audit.

### Navigation / transitions

Validate all route pairs plus modal/Back/Close/click-through recovery; one active management root, lower world uninterrupted.

### Save / migration

Schema1–7 fixtures plus pending migrated version; interrupted journal write, active work/trip, 8h cap and clock rollback; no production save used.

### Implementation tasks (proposed)

- [ ] Investigate unchanged Linux golden mismatch, Windows240/360s alpha timeouts and Farmer export mismatch under #68/#96; preserve thresholds.
- [ ] Resolve reference/art gate and common360/180 geometry; add integrated acceptance scenarios to existing pipeline in implementation PRs.
- [ ] Capture real command state+screenshots+native input evidence per screen, and measure frame/RSS at bounded history scale.

### Automated/native QA and exact definition of done

- [ ] Every #107–#117 acceptance row has source SHA, content/save version, command/state assertions, actual UI capture and native evidence or explicit unsupported status.
- [ ] Empty/active/completed/locked, restart/offline, Korean/English, mouse/keyboard,100–200%DPI and small work-area all covered.
- [ ] No fixture image qualifies transactions; no safe-window screenshot qualifies transparency; no failed test weakened or golden auto-replaced.

All applicable shared acceptance gates must pass; owner approval and unblocked domain contracts are required before final readiness.

### Dependencies / architecture risks

#58/#59–#68/#96 existing ownership; all screen vertical slices; D01 and approval gates.

Current red CI, absent reference package, unapproved art, stale QA documentation and overstating build/screenshot evidence.

Detailed contract: [ASSET_PRODUCTION.md](ASSET_PRODUCTION.md).
