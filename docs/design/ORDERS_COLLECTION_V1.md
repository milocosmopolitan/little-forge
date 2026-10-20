# Orders and Weapon Collection v1 (#109, #115)

The first two kit-v2 screens running in the real app. They sit in the upper 960×360 management host,
above the live 960×180 workshop. Open them from the host's **의뢰 / Orders** and **무기도감 / Weapon
Collection** buttons.
- **Back** or **Esc** returns to the host.
- A second **Esc** collapses the window.

Status: implemented on real state; owner visual approval pending. Issues stay open (see Limits).

Other ways in:
- **Clicking the service counter** (fixture `display`) opens Orders, and the counter shows its selection outline.
  The wall shelf (`display-shelf`) still opens the showroom.
- **O** opens Orders.
- **Esc** opens the menu from the collapsed workshop. Inside a screen it steps back, and on the host it collapses.

## Redraw rule (no blinking)

- **Whole screen.** It is rebuilt only when a rail filter or family, the language, or the game data changes.
- **Selecting a row or card.** Only the detail panel is redrawn, and the gold rim moves with the `Selected` component.
- **New text.** It is created with the theme font (`forge_ui::theme_font`). Before, rebuilt Korean text showed one frame in Bevy's default font, which has no Hangul.
- **Proof.** QA `screen-collection-clicks` clicks a card, a family and another card, and captures the frames right after each click.
  `clicks-101/131/161.png` in the evidence folder show complete panels.

## Code

| File | Role |
|---|---|
| `crates/forge-ui/src/kit2.rs` | Kit-v2 widgets: nine-slices, state art for buttons, tabs and cards, the gold focus ring, slots, tags, stars. A test keeps its slice table equal to `assets/ui/kit-v2/manifest.json`. |
| `apps/desktop/src/screens.rs` | Both screens. View models are pure functions of `forge_core::State` and drawing only reads them. |
| `apps/desktop/src/text.rs` | Korean / English strings. Korean is the default; `LITTLE_FORGE_LANG=en` switches to English. |
| `apps/desktop/src/forge_view.rs` | `Host.screen` and the selections, entry buttons, and Esc stepping back. |
| `scripts/build-silhouettes.py` | Silhouettes of undiscovered weapons, made from the approved icons. |

## Data sources (nothing invented)

**Orders**
- The list is the two orders that exist in the domain today:
  - The Chef's first commission, from `progression.chef_phase`. Its reward and recipe ore and seconds come from the content rules.
  - The Farmer's sickle repair, from `farmer_order`. Its reward, favor, ore, work time and 2-hour deadline are the `commission` constants and the live deadline.
- The rail filters by phase: All, In progress, Completed. The reference's daily / special split has no domain model yet.
- Buttons send the same commands as the older panels, through `lifestyle::Action`: accept, queue, buy ore, deliver.
  - Each button is dry-run against a clone of the state first.
  - If the command would be refused, the button is disabled and shows the reason (e.g. not enough ore).
- The deadline countdown is patched in place, so the screen is not rebuilt every second.

**Weapon Collection**
- The catalog is 10 weapons from the finished-goods catalog, grouped into 6 families.
- **Hunting knife and shortbow** come from `expedition.weapons`.
  - Shown: count owned, best quality, the ore it used, expedition uses, defeats and best wave.
  - Stars are the real appearance tier: Common 1, Refined 2 (quality 60+), Masterwork 3 (quality 85+).
- **Iron dagger** comes from the `crafted`, `sold` and `daggers` counters.
- **Other weapons** show as silhouettes with "its recipe is not known yet".
- The shortbow has no item icon yet. It uses the bow layer of the hunter's equipment.

## Limits (kept visible, not faked)

- A sold hunting knife is removed from `expedition.weapons`, so its record disappears. The screen says so.
  A permanent record needs the provenance / history decision (D05 in #106).
- There are only two order types, so there is no paging or scroll yet. Repeatable orders wait for D02.
- "Required skill" and "estimated time" from the reference are not shown. The research and timing data for them does not exist yet (#111, D03).
- The rail has no icons yet; it shows text only.
- Only these two screens use the string table. The rest of the app is still English.

## QA

```sh
cargo test -p little-forge screens   # view models, filters, records, navigation + command activation
cargo test -p forge-ui               # kit slice table matches the manifest; every state has art
cargo run -p little-forge -- --qa screen-orders --screenshot out.png
#   also: screen-orders-en, screen-collection, screen-collection-en,
#         screen-collection-clicks (post-click frames), all on an isolated QA save
```

The QA fixtures use real commands:
- **Orders:** the farmer repair is accepted, so the deadline is live.
- **Collection:** `weapon::qa_smith()` plus one Careful-approach hunting knife.

Captures (macOS, 200%) are in `docs/qa/evidence/screens-v1/`:
- `orders`, `orders-en`, `collection`, `collection-en`;
- `expanded`, the host with the two entry buttons.

At the time of this change, `cargo fmt`, strict Clippy and all workspace tests passed.

Not yet verified:
- Windows, and DPI other than 200%;
- keyboard-only traversal across every control;
- the owner's visual approval.
