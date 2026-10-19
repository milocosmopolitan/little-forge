# Little Forge pre-implementation audit

Audit source: **`6c6e8339f070229f8ea67925aeede550fee54a9d`**, fetched from origin/main. Branch `docs/screen-architecture-audit` is an isolated documentation worktree; the user's original checkout/branch was not changed. Audit timestamps use UTC (GitHub merges on2026-10-19, local New York date2026-10-18). Do not substitute a previously reported SHA.

[Immutable source path/line index](SOURCE_EVIDENCE.md) · [Fresh reproducible QA evidence](../qa/screen-audit-evidence.json)

## Readiness verdict

**No full redesigned screen is ready for unconditional implementation.** Existing gameplay is useful and real, but common ownership,16-node research, generic orders,2-person party, compact viewport and provenance contracts must precede the dependent screens. Proposed resolutions are executable once explicit decision gates are approved; unresolved mechanics are not delegated as hidden implementation choices.

## Approved decision

Indoor960×180 default,960×540 expanded with the same live lower180; independent destination/preparation and three Codex views;16 scientific principles; Hunter+Guard2 playable, up to4 in data/UI; fixed reusable showroom slots; actual Bevy/domain/input/save/native evidence. These requirements come from #106–#118 and this mission. No implementation/assets/refactor, merge or closure is part of the audit.

## Existing implementation and evidence classification

| Classification | Findings | Evidence |
|---|---|---|
| Implemented and tested, bounded scope | atomic commands, paid typed queue, needs, Farmer/Chef initial orders, one research node, versioned solo combat and tailored knife retry, journal migration | fresh96 workspace tests; relevant source/test paths below; fresh native capture separately qualified in QA evidence |
| Implemented but incomplete | space routing/actors, showroom capacity/hold, UI components, return dialogue and permanent crafted-knife history | desktop/core wiring exists; default exterior/full-height panels and anonymous goods contradict new requirements |
| Documented, not implemented |16-node engine/8-technique registry, six freely repeatable starters, generic4-NPC repeat orders, party hiring/frontline combat | RESEARCH_NODES_V1 + core Rules/Loadout/Order model comparison |
| Mock/fixture only | ui-preview showroom data; QA-only resources; salvage motion without command | apps/ui-preview/src/main.rs:56; weapon.rs:311; work-station availability |
| Approved design without complete specification | revised screen direction and2-person role intent | concepts not registered in tree; D01–D07 missing costs/rules/ownership details |
| Contradictory/obsolete | exterior default, no-party exclusions, old schema numbers, UI exports claimed absent, old research XP and dagger chain | supersession table below and annotations in canonical documents |

Freshly fetched PRs #97/#98/#102/#104/#105 are merged; no open PR was returned in the all-state recent35 listing. Their status does not qualify final gameplay or art. Main CI is red on its visual job; Windows current status is recorded in the evidence snapshot, not inferred from merged state. Prior completed Windows run timed out in both Alpha modes. Current issue states remain open.

## Critical architecture findings

1. **Window geometry is not scene continuity.** `shell.rs:11` anchors the window, while desktop `spaces.rs:295` hides interior outside expanded Home and centers480×270. Expedition roots cover960×540. Reuse persistent actors under a bottom-only viewport, central route and explicit input owner.
2. **There is no shared instance lifecycle.** `craft.rs:10`/`showroom.rs:8` anonymous goods, `expedition.rs:22` weapon IDs and `progression.rs:91` first_item_id cannot be joined by equal u64 values. #119 coordinates one identity/ownership/reservation/migration contract.
3. **Research specification exceeds both code and economic proof.** Current two-technique one-node Rules is incompatible with16-node qualification and XP. DAG witnesses pass on paper, but experiment costs, repeat recipes, effect magnitude and resource recovery remain unresolved.
4. **A second sprite is not a party.** Battle has one health/opponent/loadout and sequential threats; trace has no actor/target IDs. Explicit Solo/Party versioned scenarios, frontline protection and attributed outcomes are needed before Guard production.
5. **Historical views cannot be projected from present inventory alone.** Three Codex screens need one event/fact model, known/unknown legacy fields and bounded retention. Current unbounded progression weapon copies are cloned each command/tick.
6. **Economy is a tutorial slice.** Two one-off orders, no merchant restock,12G fast visitor dagger versus4G crafted-knife disposal and30–60minute intended repairs do not establish sustainable research/party progression.
7. **Acceptance infrastructure has real gaps.**47 valid UI exports remain unapproved; Farmer byte regeneration fails; Linux golden fails and prior Windows Alpha times out. Real input/transparent compositor/hardware performance remain separate from fixtures and safe captures.

## Recommended decisions and unresolved approvals

[Decision register D01–D07](IMPLEMENTATION_SEQUENCE.md#open-decision-register) compares feasible alternatives and names owners. Recommended smallest solution: persistent viewport/root, pure goal selector, one typed item registry with exact reservations, generic snapshotted order terms, deterministic single-smith research work, compact versioned front/rear party resolver and shared bounded historical projections. No event-sourcing platform, full RPG or per-screen databases.

Owner approval is still required for compact scene/reference/minimum-display policy; deadline/late/relationship/recovery/restock rules; starter/experiment costs and research effects; hiring/Guard stats/encounters/mode rewards; unified capacity/legacy entitlements/history retention; dashboard priorities/notes; showroom price/rejection/pacing. Proposed numbers are clearly identified in their contracts; none changes shipped code or the16-node numerical baseline.

## Contradictions and disposition

| Source at audited main | Conflict / stale assertion | Audit disposition |
|---|---|---|
| docs/system/FORGE_SPACES.md:13–17, #86/#89/#94 | exterior is default; management covers full height | historical implementation only; #107 indoor/lower180 governs redesign |
| ARCHITECTURE.md:37, FORGE_SPACES.md:63 | schema4/5 called current | current code schema7; retain historical migration descriptions with warning |
| docs/system/EXPEDITION.md:3–4, FIRST_LEGACY.md exclusions, #73 | no parties / no expedition implemented | solo slice historical; #102/#104 merged, new2-person design governs |
| docs/system/FIRST_EXPEDITION.md:12 | every recipe8s/2-ore knife | legacy simple gear only; crafted knife uses8/16/28s and plan-specific inputs |
| docs/system/PROGRESSION.md older cap/claim paragraphs; RESEARCH.md/EXPRIENCE.md RNG prose | skill self-lock risk, passive node XP, old discovery/claim | RESEARCH_NODES_V1 wins for new16-node content; proposed migration/complete behavior explicit |
| NPC_RELATIONSHIP.md deadline-clock proposal vs save.rs:44 | paused-offline suggestion versus current shared tick | D02 unresolved; preserve existing contracts, do not quietly change clock |
| ECONOMY.md offline sales undecided | actual advance sells during capped offline catch-up | label current behavior, preserve unless approved replacement |
| CRAFTING.md reserve-at-enqueue/consume-at-start proposal | actual debit at enqueue | preserve current paid job semantics; proposed snapshot contract distinguishes them |
| docs/ui/PREVIEW.md and ACCEPTANCE older sections | exports still absent/source-sheet sampling;48 count typo |47 runtime files actually registered; approval false; preview still fixture-only |
| docs/qa/PLAYABLE_ALPHA_01.md old expedition-deferred section; VILLAGE.md outside placeholder | stale scope snapshot | do not use as latest absence claim; reference current audit/CRAFTING-COMBAT |
| core spaces.rs:89 and station catalog runtime_integration=false | research still preview-only / not integrated | code consumes station layers and DaggerStudy; stale comment/metadata noted, not edited during audit |
| #106 PR105 draft caveat, #103 PR104 draft/CI running | merged and CI state drift | live status annotation added; no issue closed |

Annotations lead the relevant canonical documents; old balance/evidence is retained as historical, not rewritten into an invented success claim. System docs retain their ownership; docs/design adds cross-system decisions and integration contracts, not a duplicate authoritative balance table.

## Deliverables and issue ownership

- [SCREEN_ARCHITECTURE](SCREEN_ARCHITECTURE.md): coordinates, cameras, routing, modal/native boundaries and transitions.
- [UI_NAVIGATION](UI_NAVIGATION.md): deterministic objectives and deep links.
- [GAMEPLAY_CONTRACTS](GAMEPLAY_CONTRACTS.md): identity, lifecycle, commands, orders and migrations.
- [RESEARCH_IMPLEMENTATION](RESEARCH_IMPLEMENTATION.md) + [document graph proof](research-proof.json).
- [EXPEDITION_PARTY](EXPEDITION_PARTY.md), [SHOWROOM_UX](SHOWROOM_UX.md), [CODEX_EVENTS](CODEX_EVENTS.md).
- [ASSET_PRODUCTION](ASSET_PRODUCTION.md): actual inventory, fit/scaling/export rules and complexity.
- [SCREEN_AUDIT](SCREEN_AUDIT.md): all12 issues against all12 requested audit axes.
- [IMPLEMENTATION_SEQUENCE](IMPLEMENTATION_SEQUENCE.md): readiness matrix, decision register, dependency graph and gated first implementation PR.
- [SCREEN_ACCEPTANCE_MATRIX](../qa/SCREEN_ACCEPTANCE_MATRIX.md) + [fresh evidence](../qa/screen-audit-evidence.json).

#106–#118 receive Current State, Design Contract, Implementation Tasks, Acceptance Criteria, Dependencies, Risks and Status. #58/#72/#73/#86 and #103 receive supersession/current-state annotations. #119 is the sole new prerequisite; existing #74/#75/#76/#77/#79/#81/#82/#83/#92/#93/#95/#99/#101 retain domain/art ownership. #68/#96 receive existing QA defect ownership notes; no duplicate defect epic.

## Proposed execution and final gate

Audit review/decisions → relevant QA defect resolution + #118 reference/art gate → #63/#89/#107 live strip → #119 identity/migration → content/orders/research → real screen slices → showroom and party → three historical views → native integrated acceptance. See dependency graph for safe parallel tracks and exact exit evidence. Do not start the first screen implementation PR until D01/reference/QA prerequisites are satisfied. This audit is delivered as a documentation-only PR for review, not merged automatically.

## Explicitly excluded scope

No runtime Rust, scripts, assets, lockfile or tests changed. No gameplay numbers adopted, screenshots/goldens replaced, issue closed or PR merged. Unknown QA/history stays unknown. The mission delivers an actionable plan with explicit blocked decisions rather than pretending all mechanics are resolved.
