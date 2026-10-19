# Research principles v1 — initial 16-node balance contract

Status: **approved design direction; numerical baseline proposed for implementation/playtest, not yet shipped**. Date: 2026-10-18. Research is about reusable physical/engineering principles, not a linear kitchen-knife → sword → axe → bow dependency chain. See [Research](RESEARCH.md), [Progression](PROGRESSION.md), [Crafting](CRAFTING.md).

## Canonical separation

- **Basic products are never research nodes:** club, wooden shield, kitchen knife, rake, frying pan, nails are craftable at the start. Crafting them earns technique XP. A basic product neither automatically unlocks its weapon analogue nor counts as research completion.
- **Technique proficiency** = demonstrated fabrication practice; use the existing eight IDs: `blade_forging`, `edge_geometry`, `handle_fitting`, `hafting`, `point_forging`, `impact_geometry`, `serration`, `bow_assembly`. A technique is *not* an item recipe or a research node. Existing mappings are augmented for six new basic items in the table below; add new technique IDs only if future actual gameplay needs one.
- **Research node** = a generally applicable insight in metallurgy, geometry, assembly, elasticity, defense, or precision. Completion may unlock a recipe, a fabrication option, a new material eligibility, or quality control.
- **Recipe** = a separate combination of completed research and technique qualification. No implicit universal weapon chain. Equipment art tier, item quality and research tier remain independent.
- **Clue** reveals a previously hidden node. It never counts as XP or completes the node.
- **Research XP** is accumulated only on the **selected active research node** by executing a research work action; routine crafting grants General XP and Technique XP, not unbounded passive node XP. Discovery from crafting can enable or discount research in a future version, but does not fill every node.

## Tuning constants (v1 proposal)

| Metric | Tier I | Tier II | Tier III |
|---|---:|---:|---:|
| Node Research XP requirement | 24 | 48 | 80 |
| Gold cost charged on research start | 6 | 12 | 24 |
| Active research time | 30 simulated seconds | 60 s | 100 s |
| XP earned by research work | 8 XP per 10 simulated seconds | same | same |
| Required clues | 0 | 1 | 2 |
| Default proficiency prerequisite | one related technique Lv.1 | one related technique Lv.2 | one related technique Lv.3 |

Research time and XP must not be double-paid: one active research action advances time and node XP at 8 per 10 s, capped at the node goal. The table's nominal durations are 30/60/100 s, including 10 units of work for Tier III. Charge gold once on research activation, never on each tick/reload; pausing saves remaining work. Tier I nodes can be revealed at start and researched with Lv.1 (initial technique Lv.1 is allowed, meaning no mandatory XP grinding before first discovery). Only one active node. Research completes once when its progress reaches the goal and requirements remain valid. A full bar is not a second purchase. The node event owns the reward settlement ID.

**Technique level thresholds (lifetime XP):** Lv.1 = 0, Lv.2 = 20, Lv.3 = 55, Lv.4 = 105, Lv.5 = 170. XP never disappears at a soft level cap; no research node requires a level that only that node itself can unlock. First two levels remain reachable by baseline crafting without any research. Technique XP from existing NPC tools is preserved; numbers below extend the first six starting items. No diminishing returns in the tutorial; production balancing may introduce repetitive-craft throttling later with evidence.

**Discovery:** Tier I is visible by default (clues 0). Tier II becomes visible once its listed parent principle is completed, and each parent completion grants a deterministic one-time clue for each newly eligible direct child. Tier III is revealed with completion of its last missing direct parent and receives two deterministic one-time clues from its two prerequisites (where there is only one scientific parent, a qualifying material experiment supplies the second clue; see MET-03). A clue for a known node is idempotent. No mandatory progression is gated by random chance. A node may be visible but locked if a skill level is missing.

**Cost sanity:** First meaningful Tier I research costs 6G/30s, not a long wait. The baseline starter orders must provide a reliable way to earn gold before an early research purchase. A failed expedition must still offer a sustainable route to research and re-equipping. Confirm this through playtesting before tuning gold/time.

## Basic crafting XP mappings — proposed additional recipes

Technique XP is granted exactly once on successful craft, not on selecting an order. Existing chef/farmer/carpenter/leatherworker items in PROGRESSION.md continue to work. General Crafting XP is distinct.

| Initially craftable recipe | General XP | Technique XP | Physical rationale |
|---|---:|---|---|
| Club | 8 | `impact_geometry` +5; `hafting` +3 | balance of a hand-held striking tool |
| Wooden Shield | 10 | `hafting` +4; `handle_fitting` +4 | grasp, joined boards and bracing |
| Kitchen Knife | 10 | `blade_forging` +6; `handle_fitting` +3 | forge an edge and attach handle |
| Rake | 10 | `hafting` +6; `point_forging` +3 | long shaft and pointed teeth |
| Frying Pan | 12 | `blade_forging` +3; `impact_geometry` +4 | hot metal forming and mechanical distribution; **not** evidence of metal-shield geometry by itself |
| Nails (bundle) | 8 | `point_forging` +6; `hafting` +1 | small-scale point forming; fastening practice |

Frying-pan sheet-forming capability is evidenced by a **recipe-specific discovery flag** (`sheet_forming_practice`, granted on first successful frying pan), not mislabeled as blade-edge skill. Wooden shield similarly contributes `shield_structure_practice`; club `striking_head_practice`; basic wooden bow experimentation is a research action on ELA-01, not a requirement to forge a knife. These are one-time practice discoveries, not extra XP currencies.

## Node catalog

All prereqs are **AND** unless the condition explicitly says **OR**. Listed skill requirements supplement the universal Tier policy. Tier I has no research prerequisite, Tier II requires its indicated parents, Tier III has multiple; DAG edges point forward.

| ID | Tier | Principle | Research prerequisites | Technique / practice gate | Clues | Research XP | Gold | Seconds | Effect when completed |
|---|---|---|---|---|---:|---:|---:|---:|---|
| MET-01 | I | Controlled forging temperature | — | `blade_forging` Lv.1 | 0 | 24 | 6 | 30 | heat-control fabrication option; reduces quality variance |
| GEO-01 | I | Cutting angle and edge profile | — | `edge_geometry` Lv.1 **OR** `blade_forging` Lv.1 | 0 | 24 | 6 | 30 | dagger design; edge-focused option |
| GEO-03 | I | Impact transfer and mass distribution | — | `impact_geometry` Lv.1 | 0 | 24 | 6 | 30 | striking-head design knowledge |
| ASM-01 | I | Haft sockets and fastening | — | `hafting` Lv.1 | 0 | 24 | 6 | 30 | hafted weapon construction knowledge |
| ELA-01 | I | Wood bending and elasticity | — | `bow_assembly` Lv.1 (beginning practice allowed) | 0 | 24 | 6 | 30 | bow-limb experiment; not shortbow recipe yet |
| DEF-01 | I | Load spreading and impact dissipation | — | `handle_fitting` Lv.1; basic wooden shield available without it | 0 | 24 | 6 | 30 | defensive reinforcement option |
| PRE-01 | I | Measurement and tolerances | — | `handle_fitting` Lv.1 | 0 | 24 | 6 | 30 | precision quality-control option |
| MET-02 | II | Quenching and tempering | MET-01 | `blade_forging` Lv.2 | 1 | 48 | 12 | 60 | tempered steel option |
| GEO-02 | II | Point and penetration profiles | GEO-01 **OR** ASM-01 | `point_forging` Lv.2 | 1 | 48 | 12 | 60 | arrowheads and penetration option; knife-edge knowledge not mandatory |
| ASM-02 | II | Structural bracing and rivets | ASM-01 | `hafting` Lv.2 | 1 | 48 | 12 | 60 | reinforced wooden shield option |
| ELA-02 | II | String tension and energy transfer | ELA-01 | `bow_assembly` Lv.2 | 1 | 48 | 12 | 60 | shortbow recipe and draw-tension option |
| DEF-03 | II | Sheet metal forming | MET-01 | `blade_forging` Lv.2 **or** `sheet_forming_practice` + `impact_geometry` Lv.2 | 1 | 48 | 12 | 60 | stamped plate components; standalone physical prerequisite of metal shields |
| PRE-02 | II | Center of mass and balance tuning | PRE-01 + GEO-03 | `impact_geometry` Lv.2 | 1 | 48 | 12 | 60 | balanced fabrication option |
| DEF-02 | III | Metal shield architecture | DEF-01 + DEF-03 + ASM-02 | `handle_fitting` Lv.3 | 2 | 80 | 24 | 100 | iron shield recipe |
| MET-03 | III | Material microstructure and selection | MET-02 | `blade_forging` Lv.3 + one deterministic material test | 2 | 80 | 24 | 100 | advanced material compatibility |
| PRE-03 | III | Controlled precision forging | PRE-02 + MET-02 | `handle_fitting` Lv.3 | 2 | 80 | 24 | 100 | higher Masterwork quality ceiling |

**Sixteen nodes:** 7 Tier I, 6 Tier II, 3 Tier III = 16. DEF-03 fills the missing sheet-forming scientific link. Although GEO-02 has an OR parent, completing either grants its Tier II disclosure clue. DEF-02 has three parents but requires only two clues: award one at the first relevant parent completion and the second at the last; all three parents still must be completed. MET-03 requires one clue from MET-02 and one from a **deterministic material test** available after MET-02; the test cannot require MET-03 itself.

## Recipe gates — distinct from knowledge nodes

| Weapon/item | Basic starting point | Recipe unlock contract |
|---|---|---|
| Club | basic craft, always available | its own starter recipe |
| Wooden shield | basic craft, always available | its own starter recipe |
| Kitchen knife | basic craft, always available | its own starter recipe |
| Rake | basic craft, always available | its own starter recipe |
| Frying pan | basic craft, always available | its own starter recipe |
| Nails (bundle) | basic craft, always available | its own starter recipe |
| Dagger / hunting knife | kitchen knife produces related practice, **not a prerequisite item** | GEO-01 completed + `blade_forging` Lv.1 + `handle_fitting` Lv.1 |
| Mace | club is experiential precedent, **not an item gate** | GEO-03 + ASM-01 completed + `impact_geometry` Lv.1 |
| Spear | rake is experiential precedent, **not an item gate** | ASM-01 + GEO-02 completed + `hafting` Lv.1 |
| Arrowheads | nails are experiential precedent, **not an item gate** | GEO-02 completed + `point_forging` Lv.2 |
| Shortbow | no dependency on knife or dagger | ELA-02 completed + `bow_assembly` Lv.2 |
| Reinforced wooden shield | wooden shield remains craftable without research | DEF-01 + ASM-02 completed |
| Iron shield | frying pan supplies practice of sheet-forming, not a shield unlock | DEF-02 completed + actual metal materials |

**No hidden weapon-family edges.** Axe is a separate future recipe using impact/hafting research; it never requires dagger. Higher-tier swords, crossbows and elemental weapons are outside v1. A recipe becoming known does not grant materials. No durability physics.

## DAG / feasibility proof

Topological layers:

- Layer 0: MET-01, GEO-01, GEO-03, ASM-01, ELA-01, DEF-01, PRE-01.
- Layer 1: MET-02, GEO-02 (either GEO-01 or ASM-01), ASM-02, ELA-02, DEF-03.
- Layer 2: PRE-02 (PRE-01, GEO-03), DEF-02 (DEF-01, DEF-03, ASM-02), MET-03 (MET-02 plus material test).
- Layer 3: PRE-03 (PRE-02, MET-02).

For every edge, source layer is strictly lower than target layer. Each Lv.2 threshold (20 XP) is reachable with at most four successful baseline eligible crafts at +5 or +6, except `bow_assembly` which initially has no basic-bow recipe. **Required exception:** ELA-01 adds a guaranteed accessible practice action `bow_limb_experiment` (no advanced bow recipe) providing `bow_assembly` +8 XP per use, up to at least 24 XP, at minimal wood/time cost. This action is accessible before ELA-02, so the bow route is not circular. For Lv.3 threshold (55), continue baseline craft XP without requiring a research unlock; verify economy/resource supply before implementation.

**Graph check:** enumerate every node, check all parent IDs exist, evaluate OR alternatives, detect cycles, confirm all basic recipes exist at the start, ensure reachable qualification for each gate, guarantee mandatory clues, verify no unlock relies on its own product, and check no RPG party/other future content is required for research.

## Player-facing presentation and acceptance

Horizontal Civilization-style tree, not a linear weapon ladder. Left optional starter-product/technique journal, central scrollable engineering DAG, right selected principle with: reason, precise prerequisite names + AND/OR, current technique XP/level, clues, node XP, one-time gold, remaining simulated time, concrete affected recipes and fabrication properties. Grey nodes cannot be used until research completes. Clearly mark three independent statuses: discovered, eligible, completed. Completed research produces a substantial visible unlock notification, Codex entry and shortcut to the Craft screen.

QA acceptance fixtures: first Kitchen Knife earns skill without being a research node; GEO-01 unlocks Dagger; Club → GEO-03 + ASM-01 → Mace; Rake → ASM-01 + GEO-02 → Spear; Nails → GEO-02 → Arrowheads; frying pan + shield practice → DEF-03/DEF-01/ASM-02 → DEF-02 → Iron Shield; Bow experiments → ELA-01/ELA-02 → Shortbow with **no knife or dagger dependency**. Research stays deterministic across save/load/offline; node XP is node-local and gold isn't charged twice.

## Implementation boundary

This document defines the **next balance contract**, not shipped core behavior. Existing expedition gear can temporarily bypass the research gates for legacy saves, QA fixtures and existing inventory; migration must **grandfather already owned weapons**, not delete them or block an active expedition. Production gameplay should migrate to researched crafting only after a full end-to-end tutorial and non-softlock test.
