# Workshop atlas integration at 3/4 character scale

> Superseded by [WORKSHOP_ATLAS_V2](WORKSHOP_ATLAS_V2.md): characters returned to 1:1 and props were redrawn at that density.

## Authority and source

The owner fixes the character ratio at exactly 3/4: source 128×160, displayed
96×120, foot pivot (64,140) → (48,105). The placement grid is 32 logical pixels.
This supersedes the earlier D01 1× character policy, not the persistent viewport.
The same 960×180 scene remains under the upper 360px host in a 960×540 window.

Input: `little-forge-atlas-v1.zip`, SHA-256 recorded in the evidence input audit.
Its five props, transparent 384×256 atlas, back/front atlases, contact JSON,
12 reference poses and two six-frame GIFs were inspected. All 12 supplied
character references exactly match current repository frames reduced by nearest
at 3/4. This verifies provenance, not contact quality. No canonical character
source, animation timings, save schema or core economic rule is modified.

## SpriteFusion capability boundary

Configured server: `https://www.spritefusion.com/mcp`, not a local tile editor.
Initial initialize returned HTTP 401. Owner completed OAuth and authenticated
`tools/list` succeeded. Python's default certificate store then failed for an
upload call (`CERTIFICATE_VERIFY_FAILED: unable to get local issuer certificate`);
using the system CA bundle with certificate validation enabled resolved it.

The actual MCP offers generation, targeted image edits, animation, normal maps,
asset library queries and private generation-input uploads. It exposes **no
tilemap placement, layer editor, tileset assembly or project export API**.
`create_upload` and the exact atlas-byte PUT succeeded (HTTP 200). Upload URLs,
IDs and credentials are deliberately excluded from evidence. No generation
credits were spent and no characters were regenerated.

Consequently `layout.json` is an editable **Little Forge format**, not a fabricated
SpriteFusion project. The supplied atlas/layers remain portable PNG sources.
There is no SpriteFusion placement screenshot because no placement operation
was available. See the saved tool schemas and upload receipt in the QA directory.

## Fixture layout and source mapping

All coordinates below are top-left logical pixels, before the one-pixel outline
padding. Grid origins are multiples of 32; sprite silhouette/contact coordinates
need not be multiples of 32. Props are already native size and are not scaled 0.75.

| Prop | Prior exported size / origin | New atlas cell / grid origin | Change reason |
|---|---|---|---|
| Furnace | 97×150 / (384,7) | 96×128 / (384,32) | Compact fire opening matches scaled tongs |
| Anvil | 70×66 / (514,99) | 64×64 / (544,96) | Smaller stump and reachable striking surface |
| Tub | 75×60 / (634,102) | 64×64 / (672,96) | Low rim matches scaled quench hand/tool |
| Shared desk | 163×80 / (769,73) | 128×96 / (800,64) | Solid front hides research legs; one prop for both modes |
| Counter | 175×76 / (153,83) | 96×64 / (160,96) | Empty stock surface; runtime goods remain independent |

The final catalog records source filename, atlas rectangle, grid/visual origins,
native dimensions, contact points, mode-specific contacts, floor/depth and cached
binary hit mask. Back/front files exactly reconstruct each source prop, byte for
byte. Outline padding shifts image origin by (-1,-1) and contact by (+1,+1),
without moving the actual artwork. Background remains the existing separate PNG.

## Contact and pixel registration

For each sex, existing `animation.json` and `StationCatalog` determine the contact:

```
foot_exact = prop_origin + mode_contact
             - 0.75 * (canonical_station_offset + canonical_station_contact)
foot_render = round(foot_exact)
```

Maximum registration rounding is 0.5 logical pixels per axis. Sex-specific furnace,
anvil and quench differences remain; no global body deformation or motion
regeneration hides them. `composites/contacts.json` records exact and snapped
coordinates for every sampled pose. The Rust test checks both registrations.

Whole canonical 128×160 frames are reconstructed from atlas trim metadata and
reduced once to 96×120 with nearest. The derivative atlas uses fixed full-canvas
cells, including the existing submerged-tool quench derivatives. Canonical Actor
playback chooses the same frame IDs; the desktop adapter projects those IDs onto
the derivative. This avoids trim-dependent resampling phase changes. Source pixel
widths are **not** uniformly preserved at 0.75: nearest necessarily drops pixels.
No new interpolation colors or fractional motion translations are introduced.
Render foot is snapped; continuous simulation/Walker position remains unchanged.

Farmer's old `95 * cell_height / 128` is removed. Its native 128×160 canvas now
renders at 96×120, including transparent padding; hand attachment uses the same
display ratio. The legacy sale visitor is a separate high-resolution sheet with
219–285×314–352 cropped frames, not native 128×160 artwork. Applying 0.75 directly
would produce up to 264px height and overflow the ambient viewport. A recorded
source-density normalization of 1/3 first places these crops in 128×160 foot-
registered canvases, then applies the common 0.75 display ratio. Its resulting
96×120 canvas and variable visible bounds require visual approval. This is a
legacy-input normalization decision, not a different smith/Farmer display scale.

## Depth, shared desk and selection

The existing Walker travels straight between station feet in two dimensions;
there is no general collision/pathfinding system. All fixture floors are y=160.
Travel depth compares the character foot to that floor, rather than adding a
Y-sort label to an unchanged constant-Y arrangement. QA explicitly crosses anvil
and tub at rear y=148 and front y=168. Actual crafting uses the registered feet
and existing Walker; neither camera nor work completion timing changes.

Ordinary prop depth is `1 + floor_y * 0.001 - 0.0005`, actor depth is
`1 + foot_y * 0.001`. At a corresponding work pose, back parts use actor depth
minus 0.02; front parts use actor depth plus 0.02. Research places the same whole
desk ahead of the body and canonical front arms at actor depth plus 0.04. Salvage
uses the back-facing canonical motion in front of that same desk. No salvage
gameplay command is introduced. The solid desk front covers the research legs
without hiding the face, torso above the desk or front hands.

A one-logical-pixel 8-neighbor alpha dilation minus the complete silhouette creates
the selection ring. It is partitioned by back/front alpha ownership, so no
internal split border appears. Each ring is only 0.0001 above its own prop part,
never a global topmost UI overlay. Cached binary alpha also drives picking;
transparent tile padding does not respond. Existing modal gating remains.

## Showroom and exclusions

The counter is in this same workshop. Existing displayed slot indices, capacities,
reserved/held state, timed sales and settlement remain core-owned. Four stable
preview items sit on the new counter. The compatible Showroom management route
does not create another physical room. This is not the complete #112 panel.

No Dashboard, new balance, animation reauthoring, general navigation mesh,
dynamic lighting or live artwork reload is included. Export then restart remains
the supported asset update path. The native branch still requires owner approval
of apparent scale, contacts, legacy visitor normalization and overall spacing.

See [runtime and composite evidence](../qa/evidence/atlas-v1/README.md).
