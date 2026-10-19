# Atlas v2 evidence

Design: [WORKSHOP_ATLAS_V2](../../../design/WORKSHOP_ATLAS_V2.md).

- `runtime/` holds real Bevy captures (macOS Metal, 1920×360 = 2× of 960×180). They cover
  forging, hammering, quenching, researching, salvaging, a D01 paid crafting run and a
  scheduled sale.
- `composites/` holds offline composites from `scripts/preview-workshop-atlas.py`. Runtime feet
  equal these by test.

Local results for this change: fmt, clippy `-D warnings` and workspace tests pass. `d01`,
`d01-female` and `d01-sales` complete, including the 100-cycle runs, save/load equality,
exactly one sale and gold = 12.

Known capture issue: the `rest` capture is sometimes an all-black 1920×1080 frame. It happens
in the expanded window at frame ~851, with both old and new layouts. The UI is black too, so it
is a readback/resize timing artifact, not prop art. It was not investigated further.

```sh
python3 scripts/export-workshop-atlas.py --check
python3 scripts/preview-workshop-atlas.py
LITTLE_FORGE_INTERIOR_POSE=forging target/debug/little-forge --qa forge-interior-female --screenshot /tmp/v2.png --save-dir /tmp/v2-save
target/debug/little-forge --qa d01-sales --screenshot /tmp/v2-sale.png --save-dir /tmp/v2-sale-save
```
