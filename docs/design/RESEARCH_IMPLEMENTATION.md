# Research implementation audit

## Approved decision and source of truth

PR #105 is **merged**, merge commit `6c6e8339f070229f8ea67925aeede550fee54a9d`, verified through GitHub and fetched main. [RESEARCH_NODES_V1](../system/RESEARCH_NODES_V1.md) owns all 16 node names, gates and numerical baseline. Its status explicitly calls the values proposed for implementation/playtest. This audit does not silently promote or change them.

Six starting products (Club, Wooden Shield, Kitchen Knife, Rake, Frying Pan, Nails) are not research nodes. There are eight technique IDs, separate node-local Research XP, deterministic clues, and recipe/option unlocks. No mandatory path may require RNG or its own unlock to earn prerequisite technique levels.

## Existing implementation

`crates/forge-core/src/progression.rs:40` fixes Rules to two techniques, two recipes and one dagger node. `:116` uses XP/divisor levels capped at 2, starting at 0; `:124` discovery requires chef_balance. `:179` requires study plus explicit completion; `:282` routine knife crafting adds research XP. `assets/progression.json` has goal 10, craft contribution 6, study 4 XP / 4G / 10s. This is a working Alpha slice, not the new graph.

`apps/desktop/src/progression_ui.rs:17` renders that slice; it has no horizontal DAG. `expedition.rs:437` still allows shortbow crafting without ELA-02. `weapon.rs:193` gates hunting knives through dagger_unlocked and two old techniques. Steel/Careful are currently available through those old gates; new MET/PRE effects cannot be assumed to work by displaying nodes.

## Proposed model and algorithm

Keep JSON content registry in forge-content and deterministic progress in forge-core. Avoid hard-coded enum variants per node; stable NodeId/TechniqueId/RecipeId newtypes plus validated BTreeMap are sufficient. Content compiler validates referenced IDs, nonempty Any/All groups, cycles, reachable skill sources, unlock consumers, finite costs and bounds. A UI-only graph layered over current Rules is rejected.

```text
Prerequisite = All([Requirement]) | Any([Requirement])
Requirement = NodeComplete(NodeId) | TechniqueAtLeast(TechniqueId,level)
            | PracticeFlag(id) | MaterialTest(id)
ResearchNode { id,field,tier,parents,qualification,clue_policy,cost,xp_goal,effects }
NodeProgress { revealed,clue_sources:Set<ClueSourceId>,work_milli,xp,completed_event? }
ResearchState { nodes, active: Option<NodeId>, paused: bool }
```

Discovery, eligibility, queued/working/paused and completion are separate dimensions. Parent prerequisites are AND unless explicitly OR. `GEO-02 = Any(GEO-01,ASM-01)`; `DEF-03 qualification = Any(blade Lv2, All(sheet_forming_practice,impact Lv2))`. Recipe qualification is separate from discovery and materials. Never hide a material shortage as missing knowledge.

Commands proposed: StartResearch(node), PauseResearch, ResumeResearch; one selected active node, one smith shared work queue. Start validates and charges once, queues one work job. Pause at integer work boundary retains paid cost/progress and frees smith; resuming charges nothing. Switching requires explicit pause and retains separate saved progress; only one node accrues work at once. Alternative background research would contradict one-smith work and existing station animation; do not add it. Cancellation/refund excluded until approved.

Use completed effective work, not render time: `xp=min(goal, floor(work_milli*8/10000))`. Tier I completes at 30000 work units/24XP, II 60000/48XP, III 100000/80XP. Fatigue reduces work rate; sleeping adds no work. Time and XP describe the same action, never double-charge. Proposed completion occurs automatically and atomically at goal after validation, records unlocks once and queues quiet feedback. An explicit “claim” can acknowledge the result but must not own reward. Old full-bar/CompleteDaggerResearch remains for old job versions during migration only.

Clues: Tier I visible with zero clues. Tier II reveals only after its parent expression is true, then grants one deterministic clue; GEO-02 accepts either parent. PRE-02 requires both parents although only one clue is needed. DEF-02 receives clue 1 on first relevant parent, clue 2 on last, but stays locked until all three complete. MET-03 clue 2 comes from a deterministic material experiment available after MET-02. Stable `(node,source)` IDs deduplicate reload/offline delivery.

## Feasibility proof and its limits

[Machine-readable document graph check](research-proof.json) extracts all 16 catalog rows. It confirms existing parent IDs and a topological order. It is analysis of the specification, not a runtime test of an unimplemented engine. The document's layers are valid nonminimal ranks; the audit computes minimal layers without changing edges.

| Required property | Witness | Result / remaining condition |
|---|---|---|
| Shortbow independent of dagger | ELA-01 → bow_limb_experiment ×3 (+8 each) → ELA-02; bow Lv2 at 20 XP | graph yes; experiment must actually exist with approved affordable wood/time |
| Spear independent of dagger | ASM-01 → GEO-02 via OR alternative; Nails ×4 yields point XP24 | graph yes; starter recipe must be repeatable |
| Axe independent of dagger | proposed future impact/hafting recipe, explicitly no dagger edge | no axe recipe in v1 catalog; cannot claim executable feasibility or invent exact gate |
| Kitchen Knife needs no research | starting recipe declaration | design yes; current only QueueChef once, not repeatable free crafting |
| Metal Shield has scientific gates | DEF-02 requires DEF-01 + DEF-03 + ASM-02; transitive MET-01 + ASM-01 | sheet forming and structural bracing are both mandatory |
| No RNG mandatory gate | direct-parent clues, practice flags, deterministic material test | design yes; old discovery/XP prose must not override it |
| No skill self-lock | blade Lv2 via 4 knives (24), Lv3 via 10 (60); haft Lv2 via 4 rakes (24); point Lv2 via 4 nail bundles (24); impact Lv2 via 4 clubs (20); handle Lv3 via 14 shields (56); bow Lv2 via 3 experiments (24) | algebraically reachable if repeated baseline recipes/material supply exist; edge/serration have no >Lv1 gate in these 16 nodes |

Technique thresholds remain lifetime XP `[0,20,55,105,170]` for Lv1–5. No soft cap at Lv2 may block Lv3. Existing stored XP values survive migration, but old numeric levels cannot be copied as new levels. D05 needs explicit legacy entitlement mapping; recommend grandfather owned items and old known dagger recipe as legacy authorization, not fabricate completed GEO/MET nodes or XP. Old in-flight battles and jobs use their original version to completion. New recipes follow v1 once the tutorial and economy tests pass.

### Economy and first unlock

All nodes sum to **186G and 870 nominal effective seconds** (7×6 + 6×12 + 3×24; 7×30 + 6×60 + 3×100). This excludes practice, resource acquisition, fatigue and other smith work. First Tier I costs 6G/30s. With existing guaranteed Chef reward unchanged: healthy empty-queue starter knife takes 20s and grants 24G, then first Tier I finishes after 30s; lower-bound **50 simulated seconds plus player interaction**, 18G remaining, 8 ore remaining. This is a conditional calculation, not a measured redesigned tutorial.

Shortest shortbow research costs 18G/90s plus three experiments. The current 24G Chef reward would leave 6G after research, while a current bow's two timber alone costs 6G (and 1 ore); experiment wood/food/ammo require additional income. Farmer gives another 24G once, but two one-off rewards cannot prove an indefinitely sustainable retry economy. A maximal-root spender uses 42G before Tier II; content must preserve a path back to earning. Do not optimize only the intended tutorial path.

Critical missing numeric contracts: five new basic recipe inputs/durations (Kitchen Knife already has 2 ore/20s); bow experiment wood/time and repeat cap (must permit at least three); deterministic MET-03 experiment inputs/time; every node's fabrication effect magnitude and recipe consumption of it; repeat order economy, merchant restocking, and recovery from zero gold/materials. MET-01 says reduced quality variance, but current quality has no variance; recommend a deterministic precision option rather than adding RNG, subject to D03 approval. Steel/PRE-03 quality ceilings must explicitly reconcile existing Steel/Careful behavior and legacy items. Do not silently invent new material inventory or shrink old quality.

## Graph layout contract

Six field lanes ordered MET/GEO/ASM/ELA/DEF/PRE; topology runs left to right. Proposed upper 360: header 40, bottom actions 40, graph/detail body 280. Optional field filter uses header, not permanent left journal. Graph x=16..624 (608px); detail x=640..944 (304px). Within graph use 160×64 node cards, 40px inter-column gaps and 24px row gaps; content extends horizontally and vertically, with clipping and explicit scrollbars. The 16 nodes cannot all be legible simultaneously in this viewport. Arrow keys select connected nodes, Enter selects detail, wheel vertical/Shift-wheel horizontal, Home returns origin. Focus auto-scrolls selected node into view; pan bounds derived from layout. AND joins and OR branches have distinct labels, not color-only meaning. Tooltip is clamped to management area. Hidden nodes do not reveal future recipes through tooltip text.

## Open decision

D03 owns missing costs/effects and first-unlock/recovery playtest targets; D05 owns grandfather policy. #74/#75/#76 must implement registry, qualification and deterministic work/migration before #111 can be considered domain-ready. Proposed automatic completion also needs approval because old UI required explicit completion.

## Explicitly excluded scope

No new balance numbers adopted, 60-node tree, axe/sword/crossbow catalog, random mandatory insights, passive XP to all nodes, background scientist, or UI implementation in this audit.
