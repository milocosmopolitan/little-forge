# UI kit v2: timber, parchment and gold (#118)

A hand-drawn nine-slice kit for the Orders, Crafting, Research and Weapon Collection screens.
It follows the two owner references registered in `assets/ui/references/`.

Status: **candidate, pending owner approval.** It is not wired into Bevy yet, and the v1 kit
(`assets/ui/runtime/`, #58) is unchanged.

## Why a new kit

The v1 kit uses charcoal panels with a muted blue selection. The references ask for a different look:
- dark timber frames with iron corner brackets;
- ivory parchment pages with thin brown rules;
- a gold rim and glow on whatever is selected;
- green and red tags.

That is a different palette and set of shapes, so v2 is drawn new rather than recoloured from v1.

## References

| File | Screen | Use |
|---|---|---|
| `assets/ui/references/orders-reference.jpg` | Orders | Layout: filter rail, NPC order list, request detail. Palette sampled from it. |
| `assets/ui/references/weapon-collection-reference.jpg` | Weapon Collection | Layout: family rail, card grid, selected detail. |

Both were supplied by the owner on 2026-10-20 and are registered in `assets/ui/source-manifest.json` with their hashes.
They are visual targets only. Nothing is cut or traced from them, and their rights are not independently assessed.

## Contract

- **Density.** One art pixel is one logical pixel, the same as the workshop scene and the 32px item icons.
  Item icons are shown at 1× in slots, so they are never resampled.
- **Nine-slice.** Every frame is a nine-slice with integer insets (`slice_ltrb` in
  `assets/ui/kit-v2/manifest.json`). The centre and edges carry no texture that would smear when stretched.
- **Pixels.** Binary alpha, nearest sampling, ≤10 colours per component, shared outline `#0b0806`.
- **Text.** No text is baked into the art. Bevy draws labels in Noto Sans KR, using the colours under `text_colors` in the manifest.
- **Size.** The management area is 960×360 above the live 960×180 workshop, inside the 960×540 window.
  The references are close to full screen, so their layout is re-fitted to that area, not scaled down.

## Components

| Group | Files | Notes |
|---|---|---|
| Frames | `window-frame`, `panel-parchment` | Outer timber frame; parchment page with a double rule |
| Lists | `card`, `card-selected` | Order rows and collection cards; selected has a gold rim, glow and corner studs |
| Rail | `tab-normal`, `tab-hover`, `tab-selected`, `sparkle` | Category planks; selected is a lit plank with a gold rim, and the glint sits beside it |
| Buttons | `button-{normal,hover,pressed,disabled}`, `button-primary-{normal,hover,pressed}` | Primary (e.g. "의뢰 수락") has a warm plank and a 2px gold rim |
| Slots | `slot-{normal,selected,locked}` | Recessed square for 32px icons; content inset 4 |
| Tags | `tag-{daily,special,rank,gold}` | Pill behind a label |
| Bars | `progress-track`, `progress-fill`, `scrollbar-track`, `scrollbar-thumb`, `divider` | The divider stretches horizontally only |
| Glyphs | `glyph-coin`, `glyph-hourglass`, `glyph-star`, `glyph-star-empty` | 8–10 px inline symbols, because the font has none |

## Layout at 960×360 (mockups)

`scripts/mockup-ui-kit-v2.py` renders both screens at real size on top of the workshop strip.

**Orders**
- Left rail of 4 tabs, 172 wide.
- Order list, 392 wide: 3 rows of 80, then scroll.
- Request detail, 352 wide: NPC portrait and quote, requested item, deadline, reward and reputation, the
  catalog recipe's parts, and three actions.

**Weapon Collection**
- Left rail of 7 family tabs, 42 high.
- 4×2 card grid with undiscovered silhouettes.
- Detail panel: a 2× hero icon, records and recipe parts.

Only real repository art is used:
- the kit;
- finished, part and material icons;
- NPC heads cropped from each NPC's `neutral.png`;
- recipes from the finished-goods catalog.

Rewards, counts, deadlines, quotes and stars are sample values, and each mockup is marked "목업 · 예시 수치".

Fit findings:
- The order list shows 3 rows at a time, so it needs a scrollbar. The reference shows 6.
- A row's quote is one line with an ellipsis. The full quote lives in the detail panel.
- The collection shows 8 cards per page. The reference shows 12.
- The detail panel has no room for "required skill" and "estimated time" beside the materials. Those wait for
  real research and timing data (#111, D02/D03) and are left out rather than faked.

## Evidence

`docs/qa/evidence/ui-kit-v2/`:
- `orders-960x540.png` and `collection-960x540.png`: management over the live workshop strip, 1×.
- `orders-960x360-2x.png` and `collection-960x360-2x.png`: the management area at 2× for review.
- `kit-sheet.png`: every component at 3× on dark and light, with each nine-slice stretched to three sizes.

```sh
python3 scripts/draw-ui-kit-v2.py      # kit PNGs + manifest
python3 scripts/mockup-ui-kit-v2.py    # mockups + kit sheet
```

## Open for owner review

- The overall look against the references: frame weight, parchment tone and gold strength.
- The 2× hero icon in the collection detail. It is the only place an icon is scaled up; the alternative is 1× in a smaller slot.
- Rail icons (the references show one per tab) are not drawn yet. The tabs are text only for now.
- Bevy integration: loading the kit as nine-slices, and the real Orders and Collection data. It stays gated by
  #109 and #115 and the D02/D05 decisions in #106.
