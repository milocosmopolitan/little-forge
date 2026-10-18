# Expedition asset provenance

All derivatives are visual-review candidates. No canonical NPC files are modified.

- Hunter identity: `../npc/hunter/source/reference.png`, `reference-native.png`, neutral,
  pose recipes and layered equipment. Reuse exact head, hat, cloak, outfit, boots, expressions
  and six walking frames. Bow/knife action overlays are localized programmatic reconstruction;
  this is explicitly allowed by the task when whole-sheet generation drifts. The supplied
  six-frame combat sheet is movement reference only, never imported as the Hunter base.
- Wolf: supplied `ChatGPT Image Oct 18, 2026, 04_16_23 PM.png`, copied to `source/wolf.png`.
- Boar: supplied `ChatGPT Image Oct 18, 2026, 04_16_27 PM.png`, copied to `source/boar.png`.
- Forest: supplied `ChatGPT Image Oct 18, 2026, 04_16_33 PM.png`, copied to `source/forest.png`.
- Props: supplied `ChatGPT Image Oct 18, 2026, 04_16_41 PM.png`, copied to `source/props.png`.
- Supplied impact/arrow sheet is reference only: it has contaminated alpha edges and excessive
  effect scale. The new 32×16 arrow and 32×32 impact use small opaque native shapes.
- Supplied Oct 15 UI plate is style reference only. Existing shared forge-ui buttons are reused.

Monster candidates use one fixed source pose each at a shared 1/2 source scale. Local leg
translations preserve head and torso identity across idle/run/attack/hit/defeat; they are
limited first-pass motions and require art approval. Do not describe them as final animation.
Wolf/boar cells are 128×96, foot (64,88), maximum 64 shared colors per archetype. Hunter cells
are 128×160, foot (64,140), at most 64 shared colors. Frames use binary alpha, nearest filtering
and integer 2× UI display. The 480×270 forest plate has a documented 96-color exception;
continuous 64×32 ground joins at identical edge columns. No floating platform tiles are used.

Rebuild: `uv run --with pillow python scripts/build-expedition-assets.py`.
Validate: `uv run --with pillow python scripts/validate-expedition.py`.
The runtime loads individual native frames; atlases are review/export artifacts with roundtrip
checks, not an assumed compatible replacement for forge-npc's older Catalog format.
`canonical-hashes.json` records unchanged approved inputs. `hunter/comparison-2x.png` and
per-character `preview-4x.png` are visual checks, not approval certificates.

Remaining art work: refined bow grip/near-arm anatomy and recovery, larger dagger arc,
convincing monster attack/defeat posing, polished independently authored distant/midground forest art.
Distant, midground and foreground are disjoint static masks of the normalized plate;
they recompose exactly and do not claim reconstructed hidden scenery or parallax support. All pending work remains in #101.

## Bow revision after motion review

The original flat polygon arms were rejected. `source/bow-motion-reference.png` preserves
the supplied six-frame motion reference. `scripts/expedition-bow.py` masks only its sleeves,
forearms and gloves at a common 4:1 source/native scale, registers them to canonical shoulders,
and maps them to the approved Hunter palette. Canonical head, outfit, cloak, legs and boots
remain the base. Small sleeve bridges reuse canonical fold texture. These derivatives are
not independently interchangeable canonical NPC clothing layers.

Six keys now cover ready, lift/draw, pull, aim, release follow-through and recovery.
`hunter/bow-sockets.json` records the registration and weapon contacts. The source body/head
is not adopted. The renderer advances the draw once and holds aim instead of alternating
draw/aim from global modulo time.

`hunter/bow-preview.html`, normal/slow GIFs and native/2x light/dark contact sheets expose
every key for review. Palette/alpha/atlas/head tests do not constitute visual approval.
