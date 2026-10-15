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
    fn windows_transparency_requires_dx12() {
        assert_eq!(
            settings(false).backends,
            Some(bevy::render::settings::Backends::DX12)
        );
    }
}
