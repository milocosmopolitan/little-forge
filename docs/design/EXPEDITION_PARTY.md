# Expedition party and challenge architecture

## Approved decision

Two independent preparation steps: Destination then Party Hiring & Equipment. MVP plays Hunter plus one Guard/frontline companion. Data/UI support up to four, but no third/fourth member is playable yet. Preserve canonical Hunter. Additional challenge must involve roles and simultaneous threats, not enemy-HP multiplication.

## Existing implementation

`crates/forge-core/src/expedition.rs:34` Loadout holds one primary/secondary pair and food/arrows. Battle (`:128`) has one health, distance, opponent and bow/knife. WAVES (`:59`) lists five arrays but `step` at `:211` resolves their enemies sequentially. Trace (`:119`) has weapon and enemy kind but no attacker/target instance. `launch_expedition` at `:501` reserves supplies and snapshots weapons, not recruited members. There is no hire fee or roster. `settle_expedition` at `:610` returns arrows, converts unused food to gold, awards 2G per wave (minimum 2 except voluntary), and first full clear gives 8 timber once. Armory ownership stays player; active loans block SellWeapon.

`apps/desktop/src/expedition.rs:223` combines preparation/observation; `:704` adds equipment crafting and field ledger as a second full-screen root path. `:496` returning Hunter text uses actual Battle. Records retain 16 previous plus active; trace bound 1024 and battle seconds ≤600. Active versions 1/2 retain old rules. Existing tests at `:723`, `:760`, `:797` prove deterministic gear differences, restart/offline and invalid-loadout rollback. They do not exercise a two-person party.

## Proposed resolution

Use compact vectors of member/enemy state with validated IDs, not a general RPG ECS or parallel battle per member. Parallel single-Hunter battles cannot model protection or shared threats. Preserve v1/v2 resolver for existing active saves; add one explicit party rules version for newly launched trips.

```text
MemberId, Role = Ranged | Frontline
RecruitDefinition { id,npc,role,availability,hire_terms,legal_equipment }
PartyDraft { region, mode:Solo|Party, members:[DraftMember] } // session only
Contract { id,content_version,region,mode,terms_snapshot,members,supplies,settlement }
MemberState { id,role,formation,hp,max_hp,equipment:[ItemId],cooldown,status }
EnemyState { id,archetype,lane,hp,intent,windup,target }
CombatEventId = (ExpeditionId,sequence)
CombatEvent { id,tick,actor,target,action,item_id?,raw_amount,applied_amount,reason }
```

Capacity is data `max_members=4`, enabled MVP count 2. Reject duplicate members/items, invalid role slots, absent region/mode, insufficient supplies/fees and ownership conflict before any debit. RegionId starts with real OldForestPath only. Unknown destinations are hidden unless authored locked content with a real unlock predicate exists; do not imply placeholder rewards. Select destination has no economic side effect; PartyPrep preserves draft when navigating to crafting. A quoted hire is not yet a paid contract. Recommend charge all selected members atomically on Deploy; alternative paying immediately on Hire needs cancellations/refunds and locks with little gameplay value. UI wording must say cost due at deployment. D04 chooses fee values, unlock conditions and minimum equipment. Initial solo remains viable.

Recommendation: Hunter rear, Guard front; no drag-and-drop grid or pathfinding. One weapon plus optional shield for Guard, existing bow/knife for Hunter. Player owns all gear. Deployed items retain reserved return capacity and cannot be sold, reassigned or salvaged. Party food pool is charged upfront from the same pantry as smith food; arrows belong to Hunter's quiver. Validate totals before transfer; each member's consumption is explicit. D04 must approve per-member ration schedule and unused-food reimbursement versus return-to-pantry; preserve old contracts' current refund semantics.

Deterministic action order proposal: advance cooldown/windup → choose intents from current living roster → intercept declared charge once if eligible → resolve actions by stable actor ID → apply damage → remove defeated enemies → evaluate retreat/wave → settle if terminal. Record actual actor and target, not guessed by adjacency of trace rows. Applied damage is capped by remaining HP so Field Legacy does not treat overkill as contribution. Tie-break rules and seed/version are snapshotted; camera/frame rate has no effect.

A ready Guard can intercept one telegraphed charge per cooldown, taking its damage; without interception, the charge targets declared rear member. Frontline presence keeps ordinary melee pressure off Hunter; flanking enemy may target rear directly. Hunter has ranged opportunity while front holds; a shield helps survival but does not generate arrows/penetration. Keep force, penetration, recovery and handling orthogonal to appearance/quality. D04 owns interception cooldown, defense reduction, health and damage table; no numbers are fabricated here.

Propose whole-party retreat on any member reaching 0HP, no permanent death, no lost equipment, immediate safe return after settlement. Alternative continue with remaining member increases downed-state and targeting complexity; defer. No persistent injuries/leveling/salaries. Voluntary recall consumes already-used resources and settles partial result once. Return dialogue is per actual member contribution/cause, with one summary entry; no invented praise or dialogue claims unsupported by trace.

### Solo and Party challenge modes

Solo retains existing five-wave difficulty and regression results for old rules. Party is an explicit authored encounter set with at most two simultaneous enemies for MVP. Select mode before deployment; mode appears in every result and best record. Never compare solo best to party best as one difficulty ladder. Reuse Wolf/Boar stats as a starting test fixture, tune composition/timing only through recorded balance review. Party reward table/first-clear policy must be approved D04; retain one region-wide first timber milestone unless a distinct mode reward is explicitly accepted, preventing double first-clear farming.

| Deterministic encounter test | Controlled comparison | Required evidence |
|---|---|---|
| telegraphed boar charge | same Hunter bow/knife/supplies with and without Guard shield/interception | attributed intercept target changes; less Hunter damage; Guard actually pays HP/cooldown |
| wolf + delayed boar | sequential solo versus simultaneous party fixture, fixed weapons and timings | no HP scaling; frontline cannot intercept both simultaneous intents for free |
| ranged protection | Guard alone versus Hunter+Guard with same frontline gear | ranged attack during protected interval changes time/damage, not just summed starting HP |
| equipment complement | high-quality low-penetration blade versus suitable point + shield | quality alone cannot bypass boar resistance; protection cannot create ammo |
| exhaustion / retreat | zero arrows, insufficient food, Guard reaches 0, voluntary recall | exact resource usage, cause, safe item return and one settlement |
| save at each action | one large advance, partitioned advance, restart, 8h catch-up | identical events, member HP, gear IDs, rewards; no extra hires |

These tests define falsifiable tactical relationships. Exact timings/stats and pass thresholds remain a D04-approved fixture table before implementation. A test that merely shows higher total HP is insufficient.

### Animation minimum

Existing Hunter derivative manifest has 20 frames and combat-idle/walk/bow/dagger/hit/retreat/victory; visual approval remains pending. Preserve `assets/expedition/canonical-hashes.json` and `assets/npc/hunter` bytes. Guard has no asset directory. Required future Guard clips: idle/blink, walk (also retreat), ready/brace, intercept/block impact, attack anticipation/contact/recovery, hit, victory/return talk. Require coherent 128×160 canvas, (64,140) foot anchor unless approved character contract says otherwise, equipment sockets, front/back shield layers, attack marker timing and mirrored pivot tests. Gameplay resolves before presentation markers. Reuse canonical NPC renderer where compatible; do not feed authoring atlas metadata into farmer Catalog accidentally. No Guard art generated here.

## Open decision

D04: hire availability/fees, equipment legality, Guard stats/health/interception, consumption, retreat and mode rewards/encounters. #81 owns recruitment/loan contract, #99 combat, #82 dialogue, #83 milestones, #101 motion, #114 presentation. #113 depends on real region/mode projections but does not wait for completed Guard artwork to define its layout.

## Explicitly excluded scope

No classes, skill trees, permanent roster payroll, death, injuries, four playable members, tactical movement, full RPG, enemy HP multiplication or new Guard art during audit.
