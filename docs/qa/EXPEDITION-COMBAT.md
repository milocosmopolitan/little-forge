# Expedition combat acceptance

Implementation branch: `feat/expedition-combat`, based on `origin/main` `ad09f28`.
The original checkout had unrelated uncommitted alpha/progression work; it was preserved.
Implementation lives in the sibling `little-forge-expedition` worktree and this PR.
Epic #73 and #80–#84 were revised; #99 simulation, #100 renderer and #101 assets cover new work.
None are automatically closed by this slice.

## What is playable

In the actual desktop application, press X or choose Outside / Expeditions. Queue bow, knife
and arrows through the existing smith work queue, select actual owned weapon IDs and food/
ammunition allocation, launch, watch automatic fighting or return to the forge. Smith work
continues in either view. Hunter returns safely, explains the actual failed threat and opens
loadout preparation for retry. Field-use/defeat counts persist per crafted weapon. All five
waves reopen the path and grant eight timber once, usable for existing carpenter furniture.
No free equipment is granted in normal gameplay. QA grants initial resources only in fixtures.

## Deterministic outcomes (version 1, QA seed 42)

| Loadout | Cleared | Result |
|---|---:|---|
| Bow, 5 food, 24 arrows | 2/5 | Boar hide, missing knife |
| Knife, 5 food | 4/5 | Earlier wolf damage, missing bow |
| Bow + knife, 5 food, 24 arrows | 5/5 | Path reopened |
| Bow + knife, no food, 24 arrows | 3/5 | No provisions |

Empty quiver reaches 4/5 and reports ammunition shortage. The QA harness exports full traces,
including the exact wave, threat, action, damage and weapon ID. Stored seed is provenance;
version 1 deliberately has no random combat rolls. Balancing remains a playtest task.

## Automated checks

- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace`: 78 tests passed locally, including actual Bevy Activate
  observer crafting, failed launch, changing gear, retry, hidden-button guard and view switching.
- Existing asset, village, UI and forge-space validators.
- Canonical Hunter animation verifier: 22 frames, 10 clips, 286 native layers, 2,816 equipment
  combinations, no underlayer loss or expression drift, deterministic rebuild.
- Expedition validator: 18 Hunter frames / 63 colors; 8 wolf frames / 43 colors;
  8 boar frames / 45 colors, binary alpha, head preservation, atlas roundtrip, ground seam.
- Schema-4 migration imports an empty expedition ledger without changing existing work.
  Schema-5 missing expedition state is rejected. Journal restart every combat second matches
  offline catch-up; repeated timestamps, recall and settled rewards cannot duplicate ownership.

Bevy event-boundary captures are actual app screenshots, not generated mockups. They freeze
simulation at a real authoritative event while cosmetic animation continues. The separate
observer test exercises the complete UI command path. A separate native macOS mouse/keyboard playthrough also queued the bow, knife and arrows,
launched bow-only, observed 2/5 retreat, read the forge return dialogue, selected the knife,
bought two food, crafted another arrow bundle, and completed 5/5 on the production clock.
Its journal and screenshots are retained; no real-time video is claimed.

## Evidence and reproduction

Run `cargo run --locked -p forge-qa -- expedition` for five scenario traces.
Run `python scripts/capture-expedition.py target/debug/little-forge` for entry, movement,
bow, dagger, wolf, boar, retreat and victory. Each uses an isolated QA save directory.
Committed review evidence is in `docs/qa/expedition/`; raw runtime logs remain in `qa-output/`.
Run `python scripts/validate-expedition.py` independently from game captures.
New sprites are candidates; see `assets/expedition/PROVENANCE.md` and native previews.

## Remaining acceptance and deliberately open work

- Visual approval and polish of reconstructed Hunter arms/bow/knife and limited monster motions.
- Broader player playtesting for enjoyment and balance beyond the verified macOS input loop.
- Full Hunter order, technique/research progression and general Codex panels from #80/#72;
  this slice records field performance but does not pretend those broader systems are shipped.
- Windows CI must pass on the PR. Native Shadow PC transparency/DPI/focus/input acceptance
  remains manual and separate from CI builds or macOS screenshots.
- No independent parallax reconstruction is claimed. Depth masks exactly recompose the static
  forest candidate. Native Linux screenshots and any retained golden comparisons run in CI.

macOS debug linking emits the existing large `__eh_frame` compact-unwind warning; compilation
succeeds. It is recorded rather than hidden. No tests or golden baselines were weakened.

## Native input findings

The actual input pass found two presentation defects: fast frame changes released asset
handles, causing intermittent invisible walk/run frames, and the old destination placeholder
remained in keyboard/accessibility navigation under the combat overlay. The final change
retains all combat handles, gates QA captures on their load completion and hides the old
village root while the expedition view is active. A dark status plate improves readability.

Existing local rendered regressions: all nine forge scenes (both smiths and work stages),
life-loop and life-ambient completed. All eight normal village scenes rendered on the initial
pass. Its missing-art test exposed an outdated fixture that omitted required smith atlases;
the fixture now copies characters/UI while still withholding village/NPC art. A final local
rerun produced entirely black village and combat captures, and native UI inspection reported
`cgWindowNotFound`. That rerun is failed, not accepted; no golden baselines were changed.
The earlier retained combat evidence is visibly nonempty. Windows/native Linux coverage and
the repaired missing-art fixture remain tied to PR #102 workflow results.

The expedition asset pipeline was rebuilt and all output bytes matched. Capture scripts
reject empty screenshots. [PR #102](https://github.com/milocosmopolitan/little-forge/pull/102)
remains a draft pending CI and art acceptance.
