# Actual desktop evidence

These are native macOS Metal captures from the real Bevy application, using production Activate controls and the authoritative simulation. No artwork was generated or composited for these screenshots.

Run `python scripts/capture-alpha.py <little-forge executable> --crafting --output qa-output/crafting-event-verified` from the repository root. The scenario begins at the title, completes Chef and research, forges a weak knife, fails Wave 3, crafts a suitable replacement, retries and inspects the Codex. The accelerated clock is 0.1 simulation second per rendered frame. Initial resource grants only are documented in `crafting.png.json`; weapon quality, research, victories and rewards are earned through commands.

- `crafting-common-crafting.png`: initial knife properties and investment.
- `crafting-first-wolves.png`: first expedition combat.
- `crafting-wave-three-retreat.png`: boar retreat.
- `crafting-failure-feedback.png`: actual cause, previous best and equipment history.
- `crafting-masterwork-crafting.png`: changed material/approach and property preview.
- `crafting-retry-boar.png`: actual Wave 3 boar defeat, captured from the authoritative event rather than a fixed frame.
- `crafting-victory.png`: five-wave completion.
- `crafting-breakthrough-codex.png`: permanent weapon achievement records.
- `crafting.png.json`: full authoritative state and save/reload equality.
- `balance.json`: 24 controlled builds, including every material/technique/approach with and without a bow.

See [the gameplay report](../CRAFTING-COMBAT.md) for results and limitations. These captures do not qualify native Windows input, Shadow PC acceptance, or final asset approval.
