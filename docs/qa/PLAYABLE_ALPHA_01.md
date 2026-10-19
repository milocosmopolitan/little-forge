# Playable Alpha 01

2026-10-18. Original captures were made against `ad09f282e30876dc01f61ca65da9f9264c95dad1`; the implementation is now integrated on combat merge `c4a35bf`.
The Chef → Dagger → Codex slice is playable. This is local implementation and macOS
verification, not completion of the four parent epics or cross-platform release approval.

## Play

```sh
cargo run --locked -p little-forge -- --safe-window --save-dir /tmp/little-forge-alpha01-play
```

1. Choose New Game, Start forging, then Eira or Bram. Selection enters the workshop.
2. Choose Chef commission or press O. Inspect materials, duration and rewards, accept,
   then queue the Kitchen Knife. Close the drawer to watch the smith work.
3. Return to Chef when ready and deliver. Follow the discovery into Research.
4. Inspect the revealed Dagger node, technique requirements and 6/10 research XP.
   Queue a study, wait for 10/10, then explicitly complete research.
5. Open Crafting and queue the first Dagger. Open Codex and select Iron Dagger.
6. Journal retains technique levels, Chef delivery, discovery, research and first creations.
   Close the drawer to continue ordinary crafting, sales and needs simulation.

Esc closes a drawer before collapsing the workshop. Tab expands/collapses the forge;
1/2/3 choose exterior/workshop/showroom. M opens the village. Farmer repair remains
available under Manage → Farmer; A/S/L apply only in that tab. F9 returns to title.
Use Quit → Save and Quit to exit. Existing saves retain their progress; use a fresh
save directory to test the locked first-play progression.

## Authoritative behavior and provisional balance

`forge-content::progression_rules()` validates `assets/progression.json` and supplies
rules to `forge-core`. Bevy submits commands and displays results; animations never
award XP, research or items. Chef jobs and research share the existing typed work
queue, fatigue, sleep and offline clock. Chef's entrusted knife never occupies a
sale slot, including when the showroom is full.

- Kitchen Knife: 2 ore, 20 effective work seconds. Creation awards 10 general XP,
  6 blade-forging XP, 3 handle-fitting XP and 6 research XP. Both techniques reach
  level 1. Delivery awards 24 gold once and the `chef_balance` clue. No deadline/RNG.
- The clue reveals `dagger_structure`; it does not unlock the recipe. The root has
  no prior research node and requires both techniques at level 1. Study costs
  4 gold, takes 10 effective seconds and adds 4 research XP. Completion at 10/10
  requires a separate command. Content validation rejects unreachable/circular gates.
- Iron Dagger: 2 ore, 5 effective seconds, 12 general XP and technique XP [4, 2].
  General XP, technique XP/level, node XP, clues and Codex are separate saved fields.
  Technique level is capped at 2 for this slice; excess XP is retained. Advanced
  level gates, attributes and perks are outside this milestone.
- Codex keeps the first simulation tick, first creation ID and cumulative count
  after delivery/sale. Quality is honestly shown as not measured. Stable milestone
  IDs prevent duplicate journal entries. Repeated crafting uses quiet feedback.

New saves use schema 6. Schemas 1–5 migrate without losing stock, active work or
pre-existing dagger authorization. Migration does not invent historical XP or Codex
records. Missing current-schema progression is rejected. Older binaries cannot read
schema 6; retain an older save before downgrading. The existing two-generation save
journal is reused; this does not implement three independent player save slots.

## Actual implementation audit

The baseline already supplied deterministic commands, typed queues, farmer repair,
needs/offline progression, showroom reservations, smith selection, three forge spaces,
village merchants, shared widgets and asset previews. Those paths were reused.
Chef settlement, separate progression fields, research actions, Codex and achievement
history were missing from the playable route and are now integrated.

### #58: Player-facing UI Overlays, still partial

- #59: inventory/export contract exists and validates. #60–61: 47 runtime exports
  and fonts exist; final art acceptance remains pending (`runtime_ready: false`).
  File existence does not satisfy owner approval, every art-state or focus-ring requirement.
- #62: shared components and standalone UI Preview work. The progress track now has
  a nonshrinking minimum height so dense panels cannot collapse it.
- #63: existing shell/modal guards reused; manage drawers now block underlying
  station hit areas and commands. Closing/navigation does not run simulation commands.
  This is targeted input evidence, not exhaustive keyboard-accessibility qualification.
- #64: crafting, queue, materials and provisions reused. Timers now use the actual
  job duration. Drawers no longer rebuild every tick, avoiding flicker and lost input.
- #65–66: differentiated merchant panels and village routes exist; actual-app captures
  cover map, carpenter, leatherworker, farm, tavern, chief, guild and outside.
  Unimplemented destinations remain explicitly unavailable.
- #67: existing confirmation paths remain; milestone feedback and paged persistent
  history now cover the primary loop. No forced dialog for repeated crafts.
- #68: local 960×540 management, 960×180 ambient, shared preview and native input
  checked. Final art review and Windows/Linux visual/input qualification remain open.

### #72: Smith's Codex & Crafting Progression, Alpha subset implemented

- #74: stable content definitions for two recipes, two techniques, one Chef order and
  one research node. The full item/skill/research registry remains partial.
- #75: authoritative general XP and technique XP/levels, saved independently; advanced
  gates and the complete skill set remain deferred.
- #76: hidden → revealed → ready → completed root node works. Unknown dependencies
  are rejected; a general multi-branch graph engine is not claimed.
- #77: Chef Knife and existing farmer sickle repair work. Rake, Leather Punch and
  Hatchet commissions remain missing; they do not block Alpha 01.
- #78: interactive first research branch, prerequisites/progress and Skill Journal
  are connected. Large-tree pan/zoom/search is deferred until there is a larger tree.
- #79: first creation and persistent counts work for Knife/Dagger. Quality scoring,
  expedition/field-use records and the full First Legacy proof remain missing.

### #73: First Expedition & World Progression, deferred

#80 Hunter/Bow/Arrows content, #81 hiring/lending/result core, #82 return dialogue,
#83 region milestone changes and #84 expedition end-to-end proof are not implemented
by this slice. Hunter artwork and future design documents are not completion evidence.

### #86: Living Blacksmith Spaces, implementation present, acceptance partial

- #87: domain/UX contract present. #88: both choices and migration work; fixed an
  actual Eira rendering defect where changing characters retained Bram's atlas layout.
- #89: three-space routing works; modal click-through fixed and regression tested.
- #90–91: existing space art and actor/station animation adapters are connected.
  Research now uses the research table. No replacement character art was generated.
- #92–93: finite result reservation, overflow migration, holding/display/sale and
  showroom UI reused. #94: compact exterior and workshop entry work.
- #95: existing rack purchase/upgrade and carpenter UI remain connected.
- #96: core/save/offline tests and both-character native captures pass locally;
  multi-platform and final art/occlusion acceptance remain separate requirements.

All referenced issues remain open. No duplicate implementations or issue closures.
Full combat/expeditions, all archetypes/perks/attributes, multiplayer, marketplaces,
procedural weapons and physical durability remain outside Alpha 01.

## Verification and evidence

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python -m pip install -r scripts/requirements-qa.txt
python scripts/validate-assets.py
python scripts/validate-ui.py
python scripts/validate-spaces.py
python scripts/validate-village.py
python scripts/export-farmer-runtime.py --check
cargo build --locked -p little-forge -p ui-preview -p npc-preview
python scripts/capture-alpha.py target/debug/little-forge
python scripts/capture-ui.py target/debug/ui-preview
python scripts/capture-forge.py target/debug/little-forge
python scripts/capture-village.py target/debug/little-forge
python scripts/capture-player.py target/debug/npc-preview
python scripts/capture-lifestyle.py target/debug/little-forge
cargo run --locked --release -p forge-qa
cargo run --locked --release -p forge-qa -- life-loop
```

Workspace tests: 80 passing across 12 suites. Core coverage includes both smiths ×
32 seeds, duplicate/overflow rollback, every-phase persistence, split/batched time,
offline replay, full showroom, sleep, missing-schema fields and unreachable content.
Render/input regressions cover atlas replacement and modal station guards.

`capture-alpha.py` runs real desktop controls through Bevy `Activate` events with
0.1 simulation second per rendered frame. It does not assign rewards or recipe state.
Both characters assert delivered Chef, completed research, 22 general XP, 6 ore,
one Knife, one Dagger, and an exact save/reload match. Twelve GPU captures per
character cover title through workshop return. CI now runs this scenario on native
macOS/Windows and Linux Xvfb; those remote jobs have not been run for this working tree.

Native macOS pointer/keyboard testing separately completed Eira's entire chain on
the production clock, closed drawers during work, saved/quit, relaunched, confirmed
the original Codex record and crafted again. Count increased to 2 while the first
creation remained tick 127, work #2. The initial input pass exposed the atlas mismatch
and station click leak; both were fixed and the affected interactions rechecked.

Environment: Apple M4, Metal, native 2× display. Management images are 1920×1080
physical pixels for a 960×540 logical window. Pixel textures use nearest sampling.
Safe-window rendering does not qualify transparent-window behavior, global click
recovery, Linux X11, Windows native input or a 30-minute production-clock soak.
The macOS debug linker emits an existing compact-unwind-size warning; clippy passes.

Shared UI Preview captured 22 fixtures. Forge captures cover nine scenes including
both characters, space transitions and finite storage. Village captures also exercise
missing optional map/NPC artwork. The missing-art fixture now preserves required
character/UI resources; omitting those previously prevented its capture clock starting.
The standalone player preview produced 110 captures covering both characters' 17 clips, work sequences,
base variants, mirrored/light variants and station-free compositions. These are
rendering regressions, separate from the production gameplay-state assertions.
Both actual-app lifecycle scenarios passed sleep → eat → drink → work → two sales,
including the compact ambient view. The preview's developer diagnostic header is
still dense; it is not used as a production gameplay overlay.
The 259,200-second lifecycle simulation is a scripted purchase/queue policy, not
evidence of production auto-spending or wall-clock performance on other platforms.

Retained evidence in [alpha-01/](alpha-01/):

- [Active workshop](alpha-01/male-active-crafting.png),
  [Eira workshop](alpha-01/female-active-crafting.png),
  [Chef commission](alpha-01/male-chef-commission.png).
- [Discovered research](alpha-01/male-research-discovered.png),
  [Completed research](alpha-01/male-research-completed.png),
  [First Dagger Codex](alpha-01/male-codex.png),
  [Journal](alpha-01/male-journal.png).
- [Native reload Codex](alpha-01/native-reloaded-codex.png),
  [Native repeated-craft diagnostics](alpha-01/native-repeat-diagnostics.json).
- [Checks and source hashes](alpha-01/verification.json),
  [Capture hashes](alpha-01/summary.json), [male state](alpha-01/male.png.json),
  [female state](alpha-01/female.png.json), [lifecycle report](alpha-01/life-multiday.json).

Full local captures/logs are under `qa-output/alpha/` (gitignored). Retained screenshots
are actual application output, not mockups or edited composites.

## Integration check after combat merge

On 2026-10-18, workspace tests (86), strict workspace/all-target clippy and formatting pass on the combined progression and PR #102 implementation. The earlier screenshots qualify the original progression run only. The expedition implementation is now present from #102; the deferred expedition statements above describe Alpha 01 scope, not current repository absence. No visual baselines were replaced.
