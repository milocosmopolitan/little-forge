# Local sprite batch addressing investigation

Source: crates.io bevy_sprite_render 0.20.0, MIT OR Apache-2.0. All source and licenses retained. The sole behavior change is in DrawSpriteBatch::render: bind the exact instance-buffer byte range and draw from instance zero. Geometry, anchors, UVs, animation and platform coordinates are unchanged.

This targets a reproduced DX12 draw-addressing regression. Windows WARP with DXC reproduces the same misplaced workbench and missing smith as the native NVIDIA screenshot. FXC crashes on WARP; DXC alone does not fix placement. Fixed-batch capture validation is pending. Remove this patch when an upstream equivalent is verified on both Windows DX12 and Linux Vulkan.
