# Windows DX12 sprite placement regression

PR #15 resolved transparent-window composition using DX12/DirectComposition. The owner then reported the workbench over the roof and a missing smith on NVIDIA RTX A4500. The exact package reproduces the same placement on Windows WARP/DXC: ambient differs from the reviewed Linux reference in 12.0712% of pixels (channel tolerance 8). This is independent of Shadow streaming or native DPI.

## Actual rendering path

Bevy 0.20 renders ordinary `Sprite` entities through `SpriteMeshPlugin`, `Mesh2d` and `sprite_mesh/sprite_material.wesl`. The older `render/sprite.wesl` path does not render these entities. Initial instance-buffer and old-shader experiments therefore left the output unchanged; those changes and the temporary crate fork were removed.

The sprite-mesh vertex shader binds `vertex.instance_index` with `let`, then uses it for transform, mesh metadata and material lookups. Naga's HLSL backend has a documented base-instance propagation defect for this pattern: the base offset can be added both when the local is defined and when it is used. See [wgpu #4564](https://github.com/gfx-rs/wgpu/issues/4564). This selects another sprite's data, rather than merely moving the camera or changing DPI.

## Narrow correction

The app embeds the pinned Bevy 0.20 sprite-mesh shader with that local changed to `var`. `rendering::install_sprite_shader` replaces the embedded asset after DefaultPlugins registration and before RenderStartup loads it. The shader's geometry, UVs, materials, blending and scene coordinates otherwise match upstream. The MIT notice is included in portable packages. Bevy remains pinned to 0.20.0; there is no forked Rust dependency.

`LITTLE_FORGE_QA_UNPATCHED_SHADER` bypasses the override only when `--qa` is active, providing an A/B control from the same executable. Normal launches always install the corrected shader.

## Regression gate and limits

`scripts/capture-windows.ps1` runs on Windows CI in both debug and release. It first captures the original DXC path, which must reproduce the mismatch, then requires ambient/sale/expanded captures from both FXC (the default shipped compiler) and DXC to match the reviewed Linux references. It rejects missing captures, renderer errors and panics, including render-thread failures that leave process exit status zero. The existing tolerance is 8 per channel and 0.5% changed pixels; baselines are not regenerated automatically.

CI uses Microsoft WARP, not the owner's NVIDIA GPU. After the automated gate passes, native transparent-mode placement, Tab expansion and NPC hand attachment still require the owner's Windows/Shadow confirmation. This document does not mark the wider input/DPI/monitor/sleep checklist complete. Evidence and accepted build links are tracked in PR #16 and issue #10.
