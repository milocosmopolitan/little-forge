# Title scene — static hammer / living heat

Owner-approved pivot, 2026-10-16: keep the arm and hammer absolutely still in the supplied **Little Forge: Sparks of Tomorrow (1)** composition. The old moving cutout assets are retired.

## Production assets

- `forge-static.png`: 1672×941 opaque plate, generated with the built-in image editor from the owner's supplied reference. Same pose, lighting and framing; embedded UI and detached sparks removed. No extra scene generations or frame sheets.
- `heat.wesl`: one translucent UI material registered to the blade. Source-color gating prevents brightening the surrounding anvil. Slow ~3-second heat breath; no camera, arm, hammer or background displacement.
- Sparks: 26 tiny Bevy UI quads, six discrete poses at 8 poses/second (750 ms cycle), deterministic stagger and fade. No bitmap atlas or physics dependency. All positions use the plate's normalized coordinates; right-hand menu area stays clear.
- `DejaVuSerif.ttf`: live menu typography; redistributable license in `FONT-LICENSE.txt`.

The static plate remains visible if the effect shader is unavailable; the opaque black root and live controls remain usable if art is missing. Settings → Toggle animation provides still mode (no sparks; constant heat). No focus is requested programmatically.

## Image editing prompt

> Edit this exact image into a production game title background. Preserve the exact composition, arm, hand, hammer resting against glowing sword, anvil, forge, pixel art and orange lighting. Remove ALL text, logo, ornaments, buttons, cursor and version number from the right side and bottom left, replacing only those areas with matching dark forge background. Remove the flying detached sparks (keep sword hot and bright). No change to pose or framing. Full bleed 16:9. This is a clean static plate to overlay real interactive menu controls and animated sparks.

## Navigation

Up/Down select, Enter/Space activate, Escape returns from a submenu and never quits directly. F9 returns from gameplay/management to the title. Simulation is paused while the title is active; returning resets the monotonic timestamp so menu time is not simulated on reentry. Save-and-quit reports failures and stays on screen.

This implements #36 routing and the replacement title art for #38/#42. Fresh New Game creates the current journey only after confirmation. Continue/Load are disabled until a valid journey exists. Three independent user save slots, their migration and full settings remain the separately tracked #37/#39/#40; the new/load routes explicitly describe the current single-journey limitation instead of overwriting progress.
