# Reviewed Linux visual baselines

Source: commit `d70318e9335fd9f4a1d28bc3400798a29ccd7fa3`, [Actions run 38799159689](https://github.com/milocosmopolitan/little-forge/actions/runs/38799159689), artifact 12003667032.

Reviewed ambient, sale and expanded captures: building base remains visible and aligned with the floor; character feet and source frames are stable; the purchased dagger is attached to the visitor hand; UI text is legible. These safe-window captures do not validate native transparency or input pass-through.

Renderer: Ubuntu 24.04 Xvfb, Vulkan llvmpipe (LLVM 20.1.2), Mesa 25.2.8. Fixed frame 180 after asset readiness. Ambient/sale 960x180; expanded 960x540. CI requires all three baselines and compares with channel tolerance 8 and changed-pixel ratio <=0.005. Review renderer upgrades explicitly; never silently regenerate goldens on failures.
