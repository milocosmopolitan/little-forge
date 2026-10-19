# Supplied interior integration

## Approval and dependency

PR #122 dependency: `70ea31509abe1cdb427fc548bbf4946d86264348`.
The supplied modest-shop direction is approved. Final placement, exported pixel
quality and character contacts still require runtime visual approval.

The owner chose a separate full-room 960×180 background rather than cropping or
compressing the supplied 1672×941 image (room artwork approximately y=244–694).
The owner then requested a source-based edit. The resulting room is uniformly
exported to 943×180 inside a 960×180 canvas, with 8px/9px transparent side margins.
No room content is cropped or compressed. Its final visual acceptance is pending.
Earlier captures with the previous local backdrop are diagnostic history only.
The supplied background is preserved byte-for-byte under `assets/forge/interior/source`.

## Source audit before integration

Every supplied image was viewed. All five fixtures are RGBA; the background is RGB
with white margins, not transparent. Very faint alpha pixels extend far beyond the
visible fixtures. Raw alpha bounds are preserved in `catalog.json`; the table below
uses alpha >=224 bounds (exclusive right/bottom). Export is uniform nearest sampling
after alpha cleanup, followed by one transparent pixel of outline padding. No
character image, animation manifest or authored frame was modified.

| Source filename | Dimensions | Opaque bounds | Export | Top-left / contact | Role |
|---|---|---|---|---|---|
| ChatGPT Image Oct 19, 2026, 12_19_56 PM.png | 1672×941 | Entire RGB canvas | Source-based edited derivative | 0,0 | Background, see BACKGROUND_EDIT.md |
| ChatGPT Image Oct 19, 2026, 12_20_01 PM.png | 1254×1254 | 278,72–977,1163 | 97×150 | 384,7 / 46,104 | Furnace behind smith |
| ChatGPT Image Oct 19, 2026, 12_20_05 PM.png | 1254×1254 | 224,313–1023,1062 | 70×66 | 514,99 / 27,5 | Anvil back/front |
| ChatGPT Image Oct 19, 2026, 12_20_22 PM.png | 1254×1254 | 112,317–1154,1141 | 75×60 | 634,102 / 37,19 | Tub back/front |
| Pixel-Art Craftsman’s Workbench.png | 1448×1086 | 68,234–1384,873 | 163×80 | 769,73 / 81,29 | One research/salvage desk |
| ChatGPT Image Oct 19, 2026, 12_20_29 PM.png | 1448×1086 | 65,319–1383,883 | 175×76 | 153,83 / 86,26 | Customer display |

Coordinates use x right/y down in the persistent 960×180 scene. Exported sprites
render at 1×, never fractional runtime scale. These high-resolution source exports
are not automatically qualified as hand-authored native pixel art.

## Canonical animation contracts

Read both `assets/characters/{female,male}/animation.json`, their atlas manifests,
`assets/characters/ANIMATION.md`, `crates/forge-npc/src/player.rs`, and
`crates/forge-npc/src/stations.rs` before replacing station rendering.
Both characters retain 128×160 canvases and foot pivot (64,140).
`PlayerCatalog.stations` preserves sex-specific station-to-foot offsets.

For each activity:

```
fixed station world contact = fixture origin + fixture local contact
smith foot = fixed world contact - canonical station offset - canonical local contact
```

This adjusts the actor's standing location from its existing registration rather
than changing any body or tool pixels. Male/female hammering differ by (1,-2),
quenching by (8,-6), and heating by (0,-3) in their resulting standing positions.
The tests check contact equality for every registration for both characters.
Coordinate equality alone does not establish convincing visual contact.

Previous local candidate foot positions were research(208,156), furnace(360,156),
anvil(508,156), tub(620,156), salvage(752,156). New feet are derived from the fixed
fixture contacts above and each character's manifest, not scattered constants.

PR #122 at 70ea315 used research(112,156), furnace(416,156), anvil(576,156),
tub(720,156), salvage(864,156), rest(352,156). Final female feet are furnace(374,161),
anvil(495,150), tub(630,146), research(850,159), salvage(850,164). Male feet are
furnace(374,158), anvil(496,148), tub(638,140), with the same desk positions.

Research and salvage resolve to the same fixture. Research uses the existing front
pose behind the desk; salvage uses the existing back pose in front. Their foot
depth differs by 5px under the original contact contract. Research reuses the
authored `arms-front`/`arms-front-base` and equipped glove layers above the table.
The layer export is lossless at canonical coordinates. No salvage command exists
or is introduced; its captures are explicitly isolated pose QA.

## Selection and layering

`scripts/export-interior.py` generates a binary alpha mask and a one-pixel
8-neighbour dilation minus the original mask. White outline sprites are tinted
restrained gold for selection and muted brown-gold for hover. There is no shader,
blur, rectangular visible border or per-frame CPU image processing.

Back/front parts exactly reconstruct the exported object. The full silhouette
ring is partitioned between these parts; the internal split cannot create a new
outline seam. Research's desk and outline render at z=1.8/1.81, with front arms at
1.95. Other stationary props render at 1.2/1.21 behind the smith at 1.5. Anvil/tub
foreground moves ahead only during its corresponding authored work pose. Walking
and salvage remain in front of the object and its outline.

Pointer picking uses the cached alpha mask after conversion from window coordinates
through the authoritative D01 layout origin and scene scale. Transparent padding
and negative local coordinates do not hit. Modals, title and non-home routes block
world picking. No generous rectangular hit region is currently used.

Export/restart is required after artwork changes: compiled catalog masks and outline
PNGs must be regenerated together. Live PNG-only reload is not yet qualified and
must not be used to validate changed hit masks. This remains a delivery limitation.
The current Bevy feature set does not enable file watching. Restart is the supported
asset-change path; this milestone does not claim live asset reload acceptance.

## Showroom

Existing showroom state, capacities, sales and saves are unchanged. The display
fixture is in the same workshop; the compatible Showroom route opens existing
management controls. Four stable preview indices read existing displayed slots;
held, reserved and empty slots do not show the runtime dagger icon. Storage capacity
is not reduced to four. Existing paid-sale presentation uses the existing visitor.

The supplied counter contains baked decorative kitchen knives. They are not actual
inventory instances. Runtime sale icons are separate overlays. An empty decorative
counter variant is still required if every pictured knife must represent stock.
No new economy or item identity model is introduced here.

## Acceptance boundaries

Preserve D01's persistent camera, upward resize handshake, F8 recovery and simulation.
The exact frame-counter/pose QA and real-command lifecycle QA are separate evidence.
Pose screenshots cannot establish research/salvage gameplay or sale settlement.
No golden images are updated. No merge or final visual approval is authorized.

See [runtime evidence](../qa/evidence/interior/README.md) for the tested build,
native observations, contact captures, reproducible commands and remaining gates.
Background editing prompts and source provenance are retained in
`assets/forge/interior/source/BACKGROUND_EDIT.md`.
