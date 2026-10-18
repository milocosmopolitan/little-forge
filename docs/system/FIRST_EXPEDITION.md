# First Expedition — Hunter's Return

The first playable target is **forge → equip → watch → understand retreat → improve → retry**.
The old timer-only result concept is superseded by [automatic combat](EXPEDITION.md).
The canonical `assets/npc/hunter` character remains unchanged. Expedition derivatives live
under `assets/expedition/hunter` and preserve the 128×160 canvas and (64,140) foot anchor.

## Play sequence

1. Sell ordinary daggers or complete the existing farmer repair to fund materials.
2. Open Outside / Expeditions (X). Buy timber and food if needed.
3. Queue a shortbow (1 ore, 2 timber), knife (2 ore), and arrows (1 ore, 1 timber per 12).
   Every recipe takes 8 effective work seconds and respects smith fatigue, sleep and the queue.
4. Cycle primary to the bow, leave secondary empty, pack five food and 24 arrows, launch.
5. Bow-only clears two encounters, then retreats against the boar. Read the precise advice.
6. Craft/replenish arrows, select the knife as secondary, pack five food and 24 arrows, retry.
7. Combined equipment clears all five encounters, reopens the path and awards 8 timber once.
   Spend it toward carpenter furniture; field records remain on the individual weapons.

The Hunter speaks in the forge management view after returning, with a shortcut back to
loadout preparation. There is no death or lost weapon. No hire fee in this first playable
slice; food and arrows supply the repeatable cost. A full Hunter order/research chain remains
tracked by #80 and progression #72. This slice does not claim those prior requirements complete.

## Evidence boundaries

Deterministic scenario coverage includes bow-only, knife-only, combined, empty food and empty
quiver. Journal/offline tests compare the entire state. Actual Bevy captures show event
boundaries. Asset structural validation, visual approval, automated UI activation, Windows
CI and native Shadow PC acceptance are separate checks. Epic #73 stays open until the whole
accepted player experience is verified.


## Crafted-weapon gameplay follow-up

The implemented knife material/technique/approach choices, research gates, permanent instance history, migration and Wave 3 retry are documented in [Crafting × Combat](../qa/CRAFTING-COMBAT.md). The broader systems above remain design targets where that report identifies gaps.
