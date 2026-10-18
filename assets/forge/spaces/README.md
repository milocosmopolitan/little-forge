# Forge space art

Registered runtime art for the exterior/workshop/showroom views (#90, #93, #94). Everything in
this folder is produced by `scripts/build-space-assets.py` from the supplied references in
`source/` and verified by `scripts/validate-spaces.py`; `catalog.json` carries sizes, insets,
SHA-256 and provenance.

| Export | Source | Use |
|---|---|---|
| `workshop-backdrop.png` 480×270 | `source/smithy-interior.png` (supplied 2026-10-18) | workshop scene, drawn at integer 2× in the 960×540 window |
| `showroom-backdrop.png` 480×270 | `source/showroom-interior.png` (supplied 2026-10-18) | showroom scene |
| `showroom-window.png` 240×240, insets 80 | `source/showroom-window.png` frame-only crops | nine-slice frame of the inventory window |
| `detail-panel.png` 330×330, insets 110 | `source/detail-panel.png` frame-only crops | nine-slice frame of the detail panel |
| `window-glow.png` 24×20 | procedural | exterior window light while a job runs |

`source/showroom-draft.png` is the owner's layout draft the showroom view follows: the room
sets the mood, goods are managed through the shared slot UI with badges, racks are tabs, and
upgrades add slots instead of new furniture art. Interiors are box-reduced once (about 3.48
source px per art px) and never resampled at runtime; frames are alpha hard-cut at 96 like the
UI exports. This is registration, not visual approval.
