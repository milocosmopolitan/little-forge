# Ashbrook Village Map — Design & Implementation

> Status: design contract (2026-10-15). Engine: **Bevy / Pure Rust**, not Godot. This is a planned implementation, not a claim that the map or assets are integrated.
>
> Related: [Architecture](../../ARCHITECTURE.md), [Platforms & QA](PLATFORMS.md), [MVP](../mvp/README.md), [Economy](ECONOMY.md), [Expedition](EXPEDITION.md).

## 1. Product and art direction

Ashbrook is a believable, modest rural village threatened by monsters from outside. The map is a **navigational illustration**, not a tactical walking simulation. The player observes a tiny always-running forge on the desktop, opens the village map on demand, highlights a destination by hovering or keyboard focus, and selects it to open its relevant management/shop interface.

**Approved art direction:** antique hand-drawn cartography on light parchment, restrained ink linework, generous negative space, believable winding dirt roads and small residences, fields, central public well, trees, a river/bridge, modest workshops and civic buildings. Avoid crowded high-fantasy capital-city architecture, lavish decoration, brightly painted icon-like buildings, and a forge at the geographic center. The name **ASHBROOK** and compass are part of the parchment presentation; gameplay labels and navigation controls should be UI elements, not baked into the map.

The world-map art style intentionally differs from the colorful pixel-art 2D forge and character scenes. Highlighting uses **subtle transparent watercolor tint** over the selected location while retaining the original ink outlines and geography. No arbitrary roof color: the forge keeps its established teal roofing where tinted; earth/grass use natural muted washes. No glow, halo, opaque sticker, changed geometry, red-tinted forge, or blanket per-location brand color.

## 2. Geographic locations / interaction model

**Exactly eight selectable map destinations for the initial map:**

| Stable ID | Korean / label | Visual location on approved Ashbrook reference | Initial action |
|---|---|---|---|
| `blacksmith` | 대장간 / Home | West of central well; chimney and forge yard | Return to home/forge scene |
| `carpenter` | 목공소 | East of well, timber and sawhorses | Wood, handles, related supplies |
| `leatherworker` | 가죽세공소 | Southeast of well, tanning racks | Leather, straps, bindings |
| `farm` | 농장 | Northwest fields and farmhouse | Food, fibers, basic provisions |
| `tavern` | 주점 | Lower-right village road, inn/barrels | Food, rumors, expedition preparation |
| `village_chief` | 촌장의 집 | Lower-left civic residence | Dialogue and village progression (MVP may be read-only) |
| `merchant_guild` | 상회 길드 | Lower center/right substantial hall | Trade/commission interface (MVP may be read-only) |
| `outside` | 마을 밖 | Southern exit road/gates at bottom | Open expedition destinations panel |

Locations above are **semantic assignments to the chosen artwork**, not pixel-perfect bounding boxes. Confirm and record coordinates against the canonical final base map. Other cottages, church, graveyard, well and ordinary facilities add worldbuilding; they are **not** clickable destinations in the initial eight. The well remains the village center, and must never be shifted to favor the player's forge. No extra selectable icon is implied by illustrative buildings.

The tavern opens a panel, **not** an explorable interior; outside opens expedition selection rather than direct player-controlled travel. All selected interactions are data-driven; disabled/unavailable actions display their requirement rather than silently failing.

## 3. Canonical assets and layer contract

Use one **immutable canonical map raster** as the coordinate reference for every overlay and every hit region.

```text
assets/map/ashbrook/
  base.png                   # canonical ink-on-parchment map
  overlays/
    blacksmith.png           # EXACT base dimensions, alpha elsewhere
    carpenter.png
    leatherworker.png
    farm.png
    tavern.png
    village_chief.png
    merchant_guild.png
    outside.png
  preview/
    all-highlighted.png      # review only, not runtime input
  locations.ron              # IDs, clickable polygons, labels, actions
  attribution.md             # source/provenance/license if needed
```

**Critical constraint:** Generative redraws of the full map with individual color washes often alter the ink, buildings, and placement. Such images are *concept candidates*, **not valid production overlays**. Derive production overlays as aligned **transparent-only color layers** using the original base, or produce masks and tint in the engine. Preserve the base map completely: no regeneration, edits, perspective correction or resizing per hover. Every overlay must have identical width/height and origin to `base.png`, transparent outside its own selected region; do not include a second parchment background or duplicated ink. Keep line art legible on top of the wash via overlay alpha/blend order. For overlays shown previously as entire recolored maps, extract/repaint only the tint area and validate registration; do not simply stack the generated full map images.

The composite preview is exactly the same base + eight registered watercolor layers for art review, not a ninth interactive state. The grayscale base and interactive hover graphics must share a single source of truth. Colors are muted watercolor washes constrained to the actual roof/wood/ground shapes; grass olive-green, dirt ochre, wood brown, slate/teal forge as appropriate. Do not enforce a unique hue by destination.

**Import gate:** Before checking art into the game, review PNG dimensions, transparency and alpha bounds, base/overlay alignment, absence of altered linework, and the shape of the eight distinct hit regions. Assets currently generated in chat should be treated as references until imported and validated; GitHub documentation alone does not imply image files have been committed.

## 4. UX / states

- **Closed:** ambient forge remains visible and simulation continues.
- **Open map:** a restrained management overlay shows the parchment, no heavy ornate wooden frame. Opening must be explicit; no passive NPC event steals desktop focus.
- **Hover or keyboard focus:** only the active destination acquires its subtle watercolor overlay. The other seven remain uncolored. Tooltip shows localized name and available action; no baked-in map labels.
- **Click / Enter:** trigger destination action once. `blacksmith` returns to forge; shops open their respective existing/common panel; `outside` opens expedition selection.
- **Disabled:** show accessible reason/requirements; do not treat undiscovered story content as ready.
- **Leave/close:** clear hover and hide the map; keep simulation state unchanged.

Mouse hit targets may be larger than the painted shapes (small roofs are hard to click) but must not overlap in ambiguous ways. Choose the nearest/priority hotspot explicitly for overlaps. Keyboard focus order follows a stable list independent of visual road order. Provide focus and selected appearance distinguishable without relying only on color, and a pointer-free fallback. Support screen resizing and high DPI by transforming pointer coordinates through the same camera/map rectangle used to display the image; letterboxing must never create clickable hotspots.

**Map is not the desktop click-through surface:** interactive panel/map accepts inputs while open; ambient desktop window click-through policy is owned by the outer window controller (see [Platforms](PLATFORMS.md)). Always preserve a safe recovery/close path.

## 5. Bevy implementation boundaries (proposed)

Bevy owns scene composition, input, UI overlays, texture loading, and camera scaling. The pure Rust domain core owns purchase validation, inventory, gold, unlocks and travel/expedition state; **hover and visual transitions never mutate economic state**. Do not add Godot nodes, GDScript or GDExtension.

Suggested decomposition (names are proposals, not current directories):

```text
game/src/map/
  mod.rs             # MapPlugin, map open/close
  assets.rs          # handle loading and readiness
  locations.rs       # LocationId, RON registry and validation
  picking.rs         # scaled pointer/keyboard hit testing
  visuals.rs         # base + active overlay, tooltips
  navigation.rs      # typed destination intent -> panel commands
```

Proposed ECS/UI state:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum LocationId {
    Blacksmith, Carpenter, Leatherworker, Farm,
    Tavern, VillageChief, MerchantGuild, Outside,
}

#[derive(Default)]
struct VillageMapState {
    open: bool,
    hovered: Option<LocationId>,
    focused: Option<LocationId>,
    selected: Option<LocationId>,
}

// Request is UI intent; the domain validates any purchase/expedition command.
struct NavigateToLocation(LocationId);
```

A registry record uses stable ID, localized label key, polygon in canonical image coordinates, overlay path, enabled/unlock rules, and typed destination action. Do not use English labels or image filenames as save keys. Map layout data is static content and need not be included in gameplay saves; selected screen may be ephemeral.

**Hit test algorithm:** pointer position -> map presentation rectangle -> canonical image coordinates -> point-in-polygon across enabled locations; hover updates only when target changes. Never do full PNG alpha sampling per frame; polygons or precomputed masks suffice. For a nonzero top-left image origin, subtract the displayed image's top-left and apply inverse scale. Test aspect ratio preservation / letterboxing explicitly.

**Rendering:** draw canonical base once; draw at most one hovering/focused overlay using identical transform and nearest/appropriate filtered material (the cartographic map is illustration, so validate linear scaling at target UI size rather than blindly forcing pixel-art nearest filtering). Keep watercolor alpha subtle. A non-image shader/mask overlay is acceptable and may be simpler for reliable alignment. Do not recreate GPU textures on pointer movement.

## 6. Asset validation / tests

Automated:
1. Validate exactly eight unique stable IDs, overlay paths and actions.
2. Confirm canonical raster and all overlay images have identical dimensions; transparent pixels exist outside associated region; no opaque map backgrounds.
3. Validate polygon ranges and minimum usable hit area; no polygons entirely off-map.
4. Snapshot image-space hit tests (inside, outside, border, overlapping hotspots), transform roundtrips at two UI sizes, high DPI, aspect-ratio letterboxing.
5. UI test: open -> hover one -> switch hover -> click -> close; exactly one navigation event, map overlay cleaned up.
6. Validate that hover/focus does not change simulation state, pause forge production, gold, or NPC schedule.
7. Stable screenshot fixture with base + each one of eight hover states + composite, reviewed against canonical map.

Manual visual QA:
- Same houses, ink lines, pathways and margins in every state; tint is a watercolor **layer**, not a new painting.
- Do not recolor unrelated roofs, grass, or whole village; forge teal roof stays semantically consistent with established building asset.
- Correct association of building and clickable region; no accidental activation of adjacent cottages.
- Keyboard visibility, map readability on small management panel, low-contrast and color-vision accessibility.
- On Windows Shadow PC, test opening/closing management map and desktop click-through recovery under platform checklist.

## 7. Delivery plan and acceptance

**Slice 1:** commit canonical grayscale Ashbrook map and eight polygon definitions; render in Bevy management panel without interaction.

**Slice 2:** add hover/focus hit testing and eight low-opacity aligned overlays; allow base+one active overlay only. Create one combined review image outside runtime.

**Slice 3:** wire eight destination actions to shop/home/expedition/placeholder information panels. Do not build explorable 3D town or pathfinding.

**Slice 4:** add tests for transforms, hit regions, overlay alpha, UI state, and platform/manual acceptance. Verify no desktop taskbar interference.

Acceptance gates: map geography remains identical under hover; eight (and only eight) explicit locations; correct hit targets at multiple window sizes; no visual flashing/reflow when moving pointer; all inputs accessible; closing restores ambient gameplay; purchases and expedition decisions pass through existing domain commands; the rustic cartographic style and white space survive. Keep content limited to navigation—not a town simulation.
