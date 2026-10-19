# Navigation and meaningful dashboard

## Approved decision

Dashboard (#108) expresses one meaningful goal, active work and the next consequential decision. Orders, Crafting, Research, Showroom, Destination, Party Preparation and three Codex screens are separately navigable. The lower workshop remains live. No UI-only progression tracker.

## Existing implementation

`crates/forge-core/src/village.rs:49` has Home/Map/Destination only. `apps/desktop/src/lifestyle.rs` uses numeric tabs; `progression_ui.rs:17` embeds Chef, research, journal and Codex. `expedition.rs:10` owns a separate preparation/loadout selection. `State::activity` and current order phases, research eligibility, `Battle.cause`, `path_reopened`, capacity and gold can drive useful objectives. There is no generic order resolver, region registry, party roster or 16-node selector. Do not derive them from display strings.

## Proposed resolution

Use typed `Route` variants: Ambient, Dashboard, Orders(OrderId?), Crafting(RecipeId?, OrderId?), Research(NodeId?), Showroom(ItemId?), Destinations(RegionId?), PartyPrep(RegionId,ChallengeMode), WeaponCollection(ArchetypeId?), Chronicle(ItemId?), FieldLegacy(ExpeditionId?), Village(Location?). Shell routes remain owned by #35. A short route stack preserves selection/scroll and origin; Back returns to the caller, Home goes to Dashboard, Close goes to Ambient. Invalid deep links resolve to list + reason, never another item with the same index.

Pure domain selector interface (proposal):

```text
resolve_dashboard(&State, &Content) -> DashboardModel {
  objective: Objective { key, availability, reason_key, progress, target, action },
  active_work: Option<WorkSummary>, next_decision: Option<DecisionSummary>
}
availability = Available | Blocked(reason) | InProgress | Complete
```

No RNG, wall clock, new XP field or mutated state in selectors. Stable ordering tuple is `(priority, deadline_if_applicable, stable_id)`. Action availability reuses core command/quote validation; the command revalidates when activated. An unavailable CTA retains the reason and links to the required resource/research, not a silent disabled button.

| Priority | Existing authoritative evidence | Objective / action | Completion recognition |
|---|---|---|---|
| 0 | current save error / disabled session | explain persistence issue, recovery route | valid recovered session, no fabricated repair |
| 1 | completed trip with unread result; ready accepted order | inspect return / deliver oldest ready order | persisted acknowledgement only for read state; settlement remains core-owned |
| 2 | accepted order at deadline risk | queue its work / source missing materials | its completed_at or terminal contract state |
| 3 | blocked active/queued work | free a held slot / capacity / materials | actual blocker disappears |
| 4 | incomplete scripted starter objective | accept starter, craft, research, equip, retry in state-derived order | corresponding command result/event, not clicking CTA |
| 5 | failed trip with attributed cause | inspect suggested property/research and retry | a newer result; never imply quality alone fixes it |
| 6 | available player-pinned research/recipe goal | study or prepare known requirements | unlocked recipe/node; pin is preference only |
| 7 | active work/trip | watch progress; optional next order | work/trip completion |
| 8 | no active commitment | choose an available routine order, then craftable recipe, then field record | no invented deadline or urgent task |

Priority 1 ties: return first, then ready orders by completion tick/id. Priority 2 recommendation: risk iff estimated queue work exceeds remaining accepted deadline, not arbitrary flashing timers. Estimate reflects fatigue/rest and states uncertainty. Completed goals briefly appear as event feedback, then recompute; unavailable children cannot conceal the next achievable parent. The owner should approve this emphasis order (D06), while all data remains derived.

Returning player: run existing capped catch-up first, commit once, display one summary of completed work/sales/trips and pending delivery. Do not re-run historical toasts or auto-open a result. Persist only last acknowledged notification/expedition IDs if needed, separate from reward settlement IDs. A delivered starter is never suggested again. Existing saves without records show “Not recorded”, not a guessed welcome-back achievement.

Cross-screen edges: Orders → linked recipe/entrusted repair; Crafting → missing materials vendor / prerequisite research / queue job / result instance; Research → unlocked recipe; Showroom → instance Chronicle / Carpenter capacity; Destination → PartyPrep carrying region+mode; PartyPrep → Crafting with return context; return report → FieldLegacy → contributing item Chronicle → recipe/research. WeaponCollection describes knowledge, not currently owned inventory. Purchases return to the original context after the player acts; navigating never buys, hires or equips.

## Open decision

D06: approve objective priority and notification acknowledgement behavior. #108 is blocked on the shared route contract, generic domain selectors and D02–D04 for goals referring to new mechanics. A dashboard over existing starter data can be a partial vertical slice, never a claim of full #108 acceptance.

## Explicitly excluded scope

No daily quests, independent dashboard counters, procedural objectives, forced popup tutorial, auto-buy, auto-accept or focus stealing. Navigation drafts do not reserve inventory; deployment/queue commands do.
