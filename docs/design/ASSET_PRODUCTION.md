# Asset production feasibility

## Approved decision

Reuse canonical smiths, NPCs, Hunter and environments. Text, buttons and state are real Bevy UI. Source concepts are references, never flattened interactive screens. No assets generated or transformed during this audit.

## Existing inventory and evidence

| Category | Exact source / consumer | Finding |
|---|---|---|
| reusable runtime sprites | `assets/characters/{male,female}/{animation.json,atlas-manifest.json}`, `crates/forge-npc/src/player.rs:15` | both 128×160, pivot64/140, 17 required clips incl. five workstation actions; runtime adapter reads trim/source-size/pivot |
| workstation layers | `assets/characters/stations/catalog.json`, `forge-npc/src/stations.rs:66` | 480×270 current floor plan, separate front/back/water/fire; catalog runtime_integration=false is stale metadata, desktop consumes it |
| NPC production consumers | `assets/npc/farmer/character.json`, `forge-npc/src/lib.rs:43`; other NPC animation/atlases under `assets/npc` | fixed-cell farmer Catalog differs from authoring atlas; do not substitute manifests; portrait/source presence does not prove motion integration |
| canonical Hunter and derivative | `assets/npc/hunter`, `assets/expedition/hunter/animation.json`, `canonical-hashes.json` | canonical hashes pass; derivative20 frames, 8 clips, visual_approval pending; no Guard directory |
| combat environment | `assets/expedition/forest/{distant,midground,foreground,ground}.png`, wolf/boar animation and atlases, effects | structural checks pass, Wolf8/Boar8 frames; art approval separate |
| room source/export | `assets/forge/spaces/catalog.json`, source smithy/showroom interiors, workshop/showroom backdrops480×270, window240²/detail330² | source hashes and exports exist; 960×180 composition still needs crop/registration review; supplied draft is decorative reference |
| shared UI runtime | `assets/ui/runtime/manifest.json`, `forge-ui/src/lib.rs:506` | **47** images:19 controls,24 icons,4 headers; runtime_ready=false, not production-approved art |
| font | `assets/ui/fonts/{NotoSansKR.ttf,manifest.json,OFL.txt}` | ASCII +11,172 Hangul syllables validated; localized copy/layout not thereby implemented |
| source candidates | `assets/ui/source/{controls-v1,icons-v1,merchant-headers-v1}.png` | generator provenance/hash registered; not suitable direct runtime sheets |
| redesigned concepts | #106–#118 bodies and tracked tree | direction approved, but no eleven-screen concept manifest or attached images in fetched issue bodies; exact reference mapping/hashes unavailable in repository |

Production-ready for *reuse* means valid file/consumer/identity, not final screen approval. Reviewed actual existing images `docs/qa/alpha-01/male-workshop.png`, `docs/qa/crafting-combat/crafting-common-crafting.png` and workshop-backdrop: full-height scenes/panels visibly use all 540 logical pixels. They do not prove a 360px management layout. No visual comparison to missing approved concepts was possible; register them before fidelity acceptance.

### Required transformations and missing work

#59/#118 should register each concept's supplied origin, exact file/hash, approved region, screen ID and approval date. Keep source and derivative paths distinct. Existing room export script is `scripts/build-space-assets.py`; UI export/review/validation is `export-ui-assets.py`, `review-ui-assets.py`, `validate-ui.py`; NPC packing/export/preview is `pack-npc.py`, `export-farmer-runtime.py`, `capture-npc.py`, `capture-player.py`; expedition generator/validator is `build-expedition-assets.py`, `validate-expedition.py`. Do not run generators as an audit fix.

Missing reusable semantics: six research-field symbols and node-state/connectors, order status badges, split mode/party-role badges, codex provenance/breakthrough badges. First use existing registered icons or Bevy shapes plus text; create art only for genuinely missing reviewed requirements. Focus rings/separators are deliberately Bevy nodes per manifest; a bespoke sprite is not required. Scrollbar/modal art actually exists, despite old docs calling it absent. Missing motion: Guard and generalized customer item grips/browse/reject for the selected character; Hunter derivative visual approval remains outstanding. Rich backgrounds for every screen are decorative, not a blocker if approved composition permits existing panels.

## Proposed UI production contract

Design units are logical pixels. Upper layout: 40 header +280 content +40 contextual footer; outer margins16, gap16. Standard list/detail: width928 = list256 +gap16 +detail656. Optional icon rail consumes width only on screens that need it. Font body16, secondary14 minimum at normal size, title20–24, line height at least1.35; Korean gets two-line labels and no reduced font to conceal overflow. Dates/numbers never baked into images. Content uses localization keys, interpolation, plurals and explicit missing-record text. Test Korean/English and 1.5× pseudo-localized labels; Noto glyph coverage is not translation coverage.

Buttons minimum32px high, focus outline visible, horizontal tabs scroll or overflow into a menu. Icon source grid is 4px blocks within256px files (native64); use64 or32 logical pixels only when art-grid mapping remains uniform, never automatic arbitrary fit. New status icons target16/24/32 native as approved manifest entries, not resizing one detailed icon into unreadability. Item slots64×64; detail96×96; four future party cards at216px each plus3×16 gaps fit912px but equipment/detail is selected-member pane, not four full inventories at once. MVP shows2 active plus2 unavailable slots honestly.

Nine-slice insets come from manifest: window source1012×376 inset52/88/56/40 (native253×94 at grid4); modal468×296 inset40/60/40/40; button288×84 inset20 each. Match source-space inset and corner_scale consistently; do not copy raw insets into logical padding. Shared content padding16; minimum dimensions must exceed scaled border sums. Showroom window240² inset80 at0.5; detail330² inset110 at0.4. Their thick borders consume valuable height, so use standard thin frame for management if those leave insufficient content. Approve native-size screenshots before calling them final.

Lists scroll only their content, keep header/footer fixed. Research has two-axis graph scroll, detail separately scrolls. Scroll wheel over a panel never navigates the world. Tooltips within management rectangle with max width280, positioned beside focused item then clamped; in Ambient use a compact tip entirely within180 or an explicit expansion action. Modal max640×288 centered in upper360 with scrolling body, focus trap and backdrop blocking lower inputs while world continues. NPC art max128×160 at1× in details or a registered portrait crop; never shrink canon faces arbitrarily. Body/eye/mouth/equipment layers and handedness/pivots are preserved.

### Complexity by screen (planning estimate, not measured production time)

| Issue | Complexity | Concrete work |
|---|---|---|
| #107 | High | compact station/background registration, contact/occlusion, both smiths, DPI |
| #108 | Low–medium | shared goal/action cards and restrained icons; no new character |
| #109 | Medium | canonical NPC portraits, badges, scroll list/detail |
| #110 | Medium | real recipe/item icon coverage, plan/property cards, queue |
| #111 | High | readable16-node DAG, connectors/states and graph input |
| #112 | Medium–high | fixed racks, per-item grip/scale, browse/exit motion |
| #113 | Low–medium | reuse Old Forest layers; no fictional new regions |
| #114 | High | Guard motion/equipment, selected-member layout, role/mode labels |
| #115 | Medium | archetype silhouettes/icons, real knowledge badges |
| #116 | Low–medium | provenance timeline and selected record, existing item art |
| #117 | Medium | region/mode record and contribution views, Hunter/Guard portraits |
| #118 | High shared | reference registration, native slicing/typography/DPI and acceptance evidence |

### Static cost and runtime qualification

All47 registered UI images total **13.3002MiB** base RGBA8 (`sum(width*height*4)`); not measured GPU residency, excludes fonts, copies, mipmaps, source sheets and other sprites. Two480×270 room backdrops are0.9888MiB combined. Existing 1024×480 Hunter atlas is1.875MiB; load only required runtime derivatives, not source/QA history. Confirm actual texture dimensions from runtime and capture resident memory before making an optimization claim. More shared atlas pages can improve batching but do not prove draw-call reduction; z/material/clip/camera boundaries matter.

Validate hashes, binary alpha, palette/grid, frame duration, trim/pivot roundtrip, sockets, seams at1×/2×, light/dark checkerboards, missing-art fallback and no baked UI text. `validate-expedition.py` confirms canonical preservation, not visual taste. Farmer exporter --check currently fails; do not rebuild committed images to hide it. Profile identical scenes, warm-up30s then60s samples, record CPU/GPU/RSS and frame percentiles, then30-minute soak. Do not propose compression/refactoring without measurement.

## Open decision

D01: compact scene/reference approval and small-work-area readability. D04: Guard motion tied to approved combat action table. D07: customer art/decision needs. #118 remains blocked by missing reference registration, runtime art approval and existing QA defects; valid exports alone do not clear it.

## Explicitly excluded scope

No replacement canonical NPCs, generated concepts, decorative prototypes, new image formats/GPU compression, full-screen painted UI or editing any asset during audit.
