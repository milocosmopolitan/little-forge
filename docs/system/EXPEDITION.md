# Expedition combat

The approved milestone replaces timer-only dispatch with observable automatic combat.
One Hunter, Old Forest Path, wolves and wild boars, five sequential encounters and three
loadout slots. Direct attack controls, parties, procedural maps and permanent loss are excluded.

Open **Outside / Expeditions** from the map, or press **X**. Queue a shortbow, hunting knife
and arrows using the existing smith work queue. Click primary and secondary to cycle owned
instances, choose food (0–5) and arrows (0/12/24/36), then launch. Back to forge hides the
view without pausing either simulation. The returning Hunter explains the actual result.

## Rules, version 1

The Rust core advances one integer second at a time. Wolves have 8 condition and close
faster. Boars have 18 condition, telegraph their approach and charge the final gap.
A ranged bow hit deals 8 to a wolf and 2 to a boar; at close range it deals 4 and 1.
A knife deals 4 to a wolf and 6 to a boar. Hunter attacks alternate seconds; an enemy in
range deals 12 every third second. No hit/reward random rolls occur. The recorded seed is
reserved provenance; version 1 does not consume it. View time does not determine outcomes.

Encounters: wolf; two wolves; boar; wolf then boar; two boars. Threats resolve sequentially,
without physics or navigation. The Hunter starts with 60 condition when fed (48 without).
One ration is consumed on departure, then one between each wave, restoring 20 condition.
An empty ration allocation loses 12 condition between waves. Zero condition means safe retreat.

These are playtest values, recorded in `assets/expedition/encounters.json` and versioned
against the core's encounter contract. They are not a final balance qualification.

## Ownership, persistence and recovery

Crafted expedition weapons have stable IDs and stay in a bounded 32-instance owned armory;
they are separate from sale stock and cannot be sold or salvaged while lent. One concurrent
contract locks its equipment snapshot. Arrows and food are debited atomically on launch.
Unused arrows return once. Unused food is reimbursed at the current saved staple price,
so a full pantry cannot block return. Each cleared wave pays 2 gold. A natural retreat pays
at least 2 gold in trail salvage, allowing recovery even with no ammunition or provisions;
voluntary recall only pays for cleared waves. First full completion grants 8 timber once.

Schema 5 migrates journal schemas 1–4, retaining their prior migrations and adding an empty
expedition ledger. Current saves require expedition state. Battle version, seed, loadout,
condition, distance, progress, trace and settlement flag are persisted. Unsupported future
versions are rejected. The existing eight-hour offline cap applies. No automatic relaunch.
The latest 16 previous battles plus the active battle retain complete bounded event traces;
per-weapon field-use/defeat counters survive history eviction. Viewing never settles rewards.

## Presentation and remaining acceptance

Bevy renders authoritative event snapshots with independent cosmetic animation frames.
The forest backdrop and continuous ground are static; two actors and light effects keep
render cost small. New action/monster art is a visual candidate, not approved production art.
See [combat QA](../qa/EXPEDITION-COMBAT.md) for actual checks and evidence boundaries.


## Crafted-weapon gameplay follow-up

The implemented knife material/technique/approach choices, research gates, permanent instance history, migration and Wave 3 retry are documented in [Crafting × Combat](../qa/CRAFTING-COMBAT.md). The broader systems above remain design targets where that report identifies gaps.
