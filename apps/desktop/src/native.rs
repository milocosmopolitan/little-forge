//! Small platform boundary. Main-thread ownership is enforced through a NonSend resource.
use bevy::window::Monitor;
use forge_core::shell::Rect;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
pub struct NativeRuntime {
    manager: Option<GlobalHotKeyManager>,
    key: HotKey,
    pub status: String,
}
impl NativeRuntime {
    pub fn new() -> Self {
        let key = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::F8);
        match GlobalHotKeyManager::new().and_then(|manager| {
            manager.register(key)?;
            Ok(manager)
        }) {
            Ok(manager) => Self {
                manager: Some(manager),
                key,
                status: "Ctrl+Alt+F8 global recovery registered".into(),
            },
            Err(e) => Self {
                manager: None,
                key,
                status: format!(
                    "Global recovery unavailable ({e}); 10-second automatic recovery remains enabled"
                ),
            },
        }
    }
    pub fn recovery_requested(&self) -> bool {
        let mut requested = false;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            requested |= event.id == self.key.id() && event.state == HotKeyState::Pressed;
        }
        requested
    }
}
impl Drop for NativeRuntime {
    fn drop(&mut self) {
        if let Some(manager) = &self.manager {
            let _ = manager.unregister(self.key);
        }
    }
}
#[cfg(target_os = "windows")]
pub fn work_area(m: &Monitor) -> Option<Rect> {
    use windows_sys::Win32::{
        Foundation::POINT,
        Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint},
    };
    let point = POINT {
        x: m.physical_position.x + 1,
        y: m.physical_position.y + 1,
    };
    // SAFETY: POINT is a value; the handle is checked; info is correctly sized writable storage.
    unsafe {
        let handle = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        if handle.is_null() {
            return None;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        if GetMonitorInfoW(handle, &mut info) == 0 {
            return None;
        }
        let r = info.rcWork;
        Some(Rect {
            x: r.left as f32,
            y: r.top as f32,
            w: (r.right - r.left) as f32,
            h: (r.bottom - r.top) as f32,
        })
    }
}
#[cfg(target_os = "macos")]
pub fn work_area(m: &Monitor) -> Option<Rect> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    let screens = NSScreen::screens(MainThreadMarker::new()?);
    let primary = screens.firstObject()?;
    let main_height = primary.frame().size.height;
    let scale = m.scale_factor;
    for screen in screens.iter() {
        let frame = screen.frame();
        // Match winit's CGDisplay top-left coordinates, scaled per monitor.
        let x = frame.origin.x * scale;
        let y = (main_height - frame.origin.y - frame.size.height) * scale;
        if (x - m.physical_position.x as f64).abs() > 2.
            || (y - m.physical_position.y as f64).abs() > 2.
        {
            continue;
        }
        let visible = screen.visibleFrame();
        return Some(Rect {
            x: (x + (visible.origin.x - frame.origin.x) * scale) as f32,
            y: (y
                + (frame.origin.y + frame.size.height - visible.origin.y - visible.size.height)
                    * scale) as f32,
            w: (visible.size.width * scale) as f32,
            h: (visible.size.height * scale) as f32,
        });
    }
    None
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn work_area(_m: &Monitor) -> Option<Rect> {
    None
}
