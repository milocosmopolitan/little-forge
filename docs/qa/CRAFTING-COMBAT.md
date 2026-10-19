# Crafted weapons and the Wave 3 breakthrough

## Scope and foundation

Gameplay branch `feat/crafting-combat` starts at merged combat PR #102 (`c4a35bf`). The previously uncommitted Chef/research/Codex implementation is preserved in foundation commit `47c6fea`, then extended. No Hunter, enemy, terrain or animation files changed. #103 tracks the new gameplay; #72/#73/#79/#84 retain their broader requirements.

## Play in the desktop application

Start a fresh game, choose either smith, and complete Chef's Kitchen Knife commission. Delivery reveals Dagger research; study and explicitly complete it. The existing technique levels and research prerequisites gate new Hunting Knives. Discovery alone never unlocks them.

Open Hunter from the workshop or press X. Craft a bow and three arrow bundles. In Forge equipment, choose Iron / Reinforced point / Quick Craft and forge the knife. Supply five food, lend bow as primary and the knife as secondary, pack 24 arrows, and launch. The Hunter clears two waves and retreats from the Wave 3 boar because the blade cannot penetrate its hide.

Open Field Codex to read the cause. Forge Steel / Reinforced point / Careful Craft, select the new knife as secondary, replenish five food and ammunition, and retry. With the same encounter and bow, the Hunter completes all five waves. The new knife records four defeats, highest cleared wave five, one completion and the first breakthrough. Inspect the same permanent record from the workshop Codex link. Selling the knife returns four gold and preserves history; equipment currently lent to an active battle cannot be sold.

A normal-start economy test also reaches the same breakthrough without QA resource grants: one ordinary Iron Dagger sale funds the additional ore and five retry rations. No extended grind is required.

The scene remains optional, automatic, and driven by the Rust simulation. Work continues while it is hidden. Return dialogue and the five-wave timber milestone are reused.

## Compact, inspectable balance

`assets/weapon-crafting.json` supplies two material profiles and three approaches. Steel means a denser forged billet using more existing ore; no new merchant currency or resource inventory was added. Blade forging and handle fitting XP are reused. A knife grants 15 general XP and 5/4 technique XP when finished. Neither a character combat level nor RNG decides the result.

Quality is a workmanship score: Quick 40, Standard 65, Careful 90, plus two per earned blade-forging level (capped at two). Appearance is a separate field and cannot change combat properties. Approach precision improves handling for Light edge, or penetration for Reinforced point. A reinforced point takes one additional second to recover. Steel has greater force/penetration but slower recovery and higher ore investment. Quality never multiplies force. Range is unchanged because this slice does not need another statistic.

Boar hide subtracts five force, offset by penetration, with a minimum of one damage. Handling reduces wolf melee punishment. Existing bow/ammunition behavior remains the baseline; improved arrowheads, enchantment sockets and bow-quality crafting are not implemented.

At fixed seed 42, technique level 1, identical bow, five food and 24 arrows:

| Steel reinforced-point knife | Waves cleared | Duration | Hunter damage | Knife investment |
|---|---:|---:|---:|---|
| Common / Quick | 4 | 78s | 108 | 3 ore / 8s |
| Refined / Standard | 5 | 79s | 96 | 3 ore / 16s |
| Masterwork / Careful | 5 | 67s | 60 | 4 ore / 28s |

The accepted failure/retry uses Common Iron reinforced point (2 cleared, 33s, penetration 1) then Masterwork Steel reinforced point (5 cleared, 67s, penetration 6). Both were actually crafted in the desktop scenario. Preparation grants initial materials only, not weapons, research, XP or rewards.

A poorly matched Masterwork Iron Light edge still fails (three waves cleared with a bow). Without a bow, Common Iron Light edge clears two waves while Masterwork Steel reinforced point clears only one. Better quality therefore cannot substitute for the equipment mix. A complete, well-supplied build can win this introductory region; it is not evidence of universally optimal future balance.

[Full 24-build matrix](crafting-combat/balance.json) reports duration, hunter damage, arrows and provisions consumed, knife contribution, investment and waves per ore. These are deterministic single-run durations, also the mean for identical repeats. Resource efficiency excludes the fixed bow/provisions investment, as stated in the JSON. Damage records attack force before overkill clipping.

## Persistence and ownership

Schema 7 extends schema 6 progression and schema 5 combat. Migration preserves an in-flight old battle under its original rules, existing equipment IDs and rewards. Old equipment gets no invented craftsmanship rating. New battles snapshot their weapons and combat rules. The physical armory remains separate from permanent instance records under Progression/Codex; selling does not erase snapshots or historical results. Existing journal, eight-hour offline cap, transactional costs and one-time settlement are reused.

## Verification

Run the existing tools:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p forge-qa -- crafting-combat
python scripts/capture-alpha.py target/debug/little-forge --crafting --output qa-output/crafting-combat
```

The new ten core tests cover controlled quality comparisons, the Wave 3 breakthrough, a mismatched Masterwork, cosmetic independence, research/cost rollback, journal/offline equivalence, legacy in-flight migration, sale history, lent-equipment protection and shared armory reservations. Existing expedition/view-switch/no-double-reward tests remain intact.

The actual Bevy playthrough follows production Activate controls from title through Chef, research, crafting, failure, retry and Codex. Its clock is accelerated to 0.1 simulation second per rendered frame. This is real application rendering and domain commands, not static previews or a core-only test. Initial resource grants are explicitly listed in its state report. It does not qualify manual Windows input or Shadow PC acceptance.

## Remaining boundaries

The full research graph, Farmer/Rake/Spear and other NPC chains remain outside this slice. Legacy bow crafting and ammunition retain type-level behavior. No salvage UI or post-craft enhancement system was added; implemented selling proves ownership/history separation. The existing Hunter/monster visual acceptance remains separate and unchanged. Balance is a first-region candidate, not a broad-content balance claim.

The baseline already has Linux/Windows visual-golden failures. No baselines or thresholds were changed; new gameplay evidence is collected before those gates. Remote CI and native Shadow PC acceptance must be reported separately from local passes.

## Local check results

- Formatting, strict workspace/all-target clippy and all 96 workspace tests pass.
- Asset, UI, spaces, village and expedition validators pass; canonical source hashes remain unchanged.
- Additional Farmer exporter byte check fails on both the unchanged combat branch and this branch with pinned Pillow 11.3.0. Decoded RGBA comparison is identical (512×960, zero changed pixels). This is an existing PNG encoding reproducibility issue, not silently repaired or waived. The ordinary manifest/hash validator passes against committed files.
- macOS debug build succeeds with the existing compact-unwind-size linker warning.
- Native Windows/Shadow PC interaction acceptance remains pending; CI is separate from this local evidence.

Existing foundation/lifecycle/expedition simulation reports also pass. Both smiths pass the actual Bevy Chef → research → Dagger → Codex regression, including journal reload equality.
All nine existing real-game expedition captures (entry, movement, bow, release, dagger, wolf, boar, retreat and victory) pass on the integrated build.
