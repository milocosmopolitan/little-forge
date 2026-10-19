# One historical model, three Codex views

## Approved decision

Weapon Collection records knowledge by archetype; Crafting Chronicle records actual workmanship and individual history; Field Legacy records real expeditions, party/equipment contribution and milestones. Records survive sale/delivery/salvage. Never invent historic entries or quality in old saves.

## Existing implementation

`progression.rs:91` Record tracks first_tick, first_item_id, count and optional quality string; `:101` Milestone is id/tick/text, deduplicated by id. `:108` stores BTreeMap codex, journal and crafted-weapon copies. `weapon.rs:231` records knife workmanship and increments a different next_item_id from expedition IDs. `expedition.rs:347` retains 16 Battle records plus active; `:610` refreshes weapon counters and path_reopened. Transient `Event {id,tick,kind}` (`lib.rs:136`) is not a durable provenance ledger. `Sold` identifies a string archetype, not the sold instance. Existing first-record IDs must not be relabeled as weapon IDs. `State::command` and `advance` clone State, so unbounded journal/history also increases per-tick cost.

## Proposed resolution

Small durable facts + bounded recent event log + permanent aggregates, all updated in the same core transaction. Full event sourcing/replaying every historic action is unnecessary and worsens clone/save cost; three screen-owned stores cause disagreement and are rejected.

```text
DomainEventId(u64) // one monotonic persisted domain sequence
Event { id, tick, content_version, origin: WorkId|OrderId|ExpeditionId|VisitId,
        kind: Crafted|Repaired|Delivered|ResearchCompleted|Sold|Loaned|Returned|
              ExpeditionSettled|WorldMilestone }
CraftFact { item_id,archetype,recipe,created_at?,creator?,plan?,quality?,properties?,origin }
ExpeditionFact { id,region,mode,rules_version,party_snapshot,equipment_snapshots,
                 outcome,cause,rewards,contributions,first_clear }
KnowledgeRecord { archetype,discovered,recipe_known,first_craft?,count,best_quality? }
```

Crafted fact comes from work completion, not preview/queue. Repairs are Repair facts, not new crafted objects. ResearchCompleted references node and unlocked recipes; discovery never sets recipe_known. Expedition settlements refer to launch snapshots, not the weapon's present owner or latest balance rules. Combat event IDs `(trip,sequence)` are distinct from domain event IDs. Settlement key `(origin_id,operation_kind)` prevents duplicate application even when commands use new IDs. Persistence stores event and resulting aggregate atomically; UI acknowledgement never gates rewards. If a projection rebuild is later needed, version it and use processed event high-water plus unique origin keys, not locale strings.

Numeric quality comparisons use stored numeric score and rules version. Best record is per archetype and comparable ruleset; on equal score keep earliest event/id. Do not compare quality string lexicographically, or interpret legacy None as zero. Permanent knowledge survives item disposal. A sold item keeps a provenance/tombstone snapshot reachable from Field Legacy. Route selection uses canonical ItemId, not vector order.

### Proposed retention (approval D05)

Keep all current owned/entrusted/lent instances, first and best per archetype, named world milestones and the item snapshots required by retained expedition facts. Retain last 256 craft/disposition events and 32 settled expedition summaries per region/mode; keep full traces for last 16 trips plus active (current trace ceiling 1024 and duration 600s require requalification for party). Older counts/best/first remain permanent; UI labels evicted detail “Detailed history not retained”. A bookmarked representative work may be pinned, proposed maximum 16, explicit replace prompt. These are proposed bounds, not silent loss of existing records. Alternative permanent individual history meets literal unlimited-history desire but requires bounded storage strategy and measured clone/save costs. Owner must approve archival semantics before #116/#117 can claim readiness.

### Migration

Map old expedition IDs to canonical ItemIds using explicit namespace aliases. Preserve existing Craftsmanship as recorded; do not recompute with new proficiency. General scalar daggers receive ImportedLegacy identities and unknown recipe/date/quality where absent. Existing codex count/first_tick retain their known values but unjoinable first_item_id remains a legacy reference, never fabricated provenance. Old active v1 battle with no snapshot continues using its preserved equipment registry. Historical battle rows record known Hunter-only attribution where model guarantees it; unknown source damage remains unknown, not assigned by heuristic. Preserve original 16 records, armory and progression copies; never assume a missing old history means no prior achievement.

Screen projections:

- #115 family/known/locked grid; counts and best are history, ownership count is separate; recipe/research links derived from content.
- #116 chronological craft milestones with material/approach/proficiency/source and current disposition; filters use actual facts; smith notes are optional authored player annotations with their own length/save policy, not fabricated quotes.
- #117 region+mode summaries, failure/cause, parties, exact equipped IDs and applied contribution; retry comparison uses comparable region/mode/rules. Old trace damage may include overkill, label it as recorded raw damage rather than normalized applied damage.

## Open decision

D05: retention/pinning, unified ID/ownership migration without pooling capacity, incomparable-quality presentation; D06: whether personal editable smith notes are MVP and their bounds. Shared identity prerequisite must land before three parallel projections. #79 owns records, #103 properties, #82/#83 field narrative/milestones.

## Explicitly excluded scope

No fabricated historic creators, repaired-item creation counts, online identity, public market ownership, analytics service, three independent databases or UI awarding achievements.
