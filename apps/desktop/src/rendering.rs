//! Desktop compositor policy. Window transparency alone cannot select a compatible swapchain.
use bevy::{
    render::{
        RenderPlugin,
        settings::{RenderCreation, WgpuSettings},
    },
    window::CompositeAlphaMode,
};

/// Called only as the first operation in main, before any threads are started.
pub fn prepare_process() {
    #[cfg(target_os = "windows")]
    if !std::env::args_os().any(|arg| arg == "--safe-window") {
        // Bevy 0.20 exposes this wgpu option only through the environment.
        // The default HWND swapchain does not support per-pixel transparency.
        // SAFETY: main calls this before creating any threads or initializing libraries.
        unsafe { std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "DxgiFromVisual") };
    }
}

pub fn transparent_alpha() -> CompositeAlphaMode {
    if cfg!(target_os = "macos") {
        CompositeAlphaMode::PostMultiplied
    } else {
        CompositeAlphaMode::PreMultiplied
    }
}

fn settings(safe: bool) -> WgpuSettings {
    #[cfg(target_os = "windows")]
    let settings = WgpuSettings {
        // Embed DXC so release rendering uses the same compiler as native QA.
        // Legacy FXC loses the WARP device in release builds even after the index fix.
        dx12_shader_compiler: bevy::render::settings::Dx12Compiler::StaticDxc,
        ..WgpuSettings::default()
    };
    #[cfg(not(target_os = "windows"))]
    let settings = WgpuSettings::default();
    #[cfg(target_os = "windows")]
    if !safe {
        // Vulkan/Auto may produce an opaque surface on Windows despite transparent=true.
        return WgpuSettings {
            backends: Some(bevy::render::settings::Backends::DX12),
            ..settings
        };
    }
    let _ = safe;
    settings
}

pub fn plugin(safe: bool) -> RenderPlugin {
    RenderPlugin {
        render_creation: RenderCreation::Automatic(Box::new(settings(safe))),
        ..Default::default()
    }
}

/// Replace the pinned Bevy 0.20 sprite-mesh shader before RenderStartup loads it.
/// Only the instance-index binding differs from upstream; see docs/qa/DX12-SPRITES.md.
pub fn install_sprite_shader(app: &mut bevy::prelude::App) {
    use bevy::asset::io::embedded::EmbeddedAssetRegistry;
    use std::path::{Path, PathBuf};
    app.world()
        .resource::<EmbeddedAssetRegistry>()
        .insert_asset(
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/shaders/sprite_material.wesl"
            )),
            Path::new("bevy_sprite_render/sprite_mesh/sprite_material.wesl"),
            include_bytes!("shaders/sprite_material.wesl").as_slice(),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_surface_never_uses_unspecified_alpha() {
        assert!(matches!(
            transparent_alpha(),
            CompositeAlphaMode::PreMultiplied | CompositeAlphaMode::PostMultiplied
        ));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_shader_compiler_is_self_contained() {
        for safe in [false, true] {
            assert!(matches!(
                settings(safe).dx12_shader_compiler,
                bevy::render::settings::Dx12Compiler::StaticDxc
            ));
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_transparency_requires_dx12() {
        assert_eq!(
            settings(false).backends,
            Some(bevy::render::settings::Backends::DX12)
        );
    }
}
