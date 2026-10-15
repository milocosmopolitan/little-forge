# Temporary DX12 transform investigation

Source: crates.io bevy_sprite_render 0.20.0, MIT OR Apache-2.0. Source and licenses retained.

The first-instance buffer-slicing hypothesis was disproved: the Windows output was unchanged (12.0712% differing pixels). That modification has been reverted. This draft now compares original, explicit affine dot products, and explicit view multiplication shaders, and records CPU instance data. These diagnostics must be removed before accepting a production fix.
