use crate::Session;
use bevy::{
    prelude::*,
    window::{CursorOptions, Monitor, PrimaryWindow, WindowLevel, WindowPosition},
};
use forge_core::shell::{Mode, Rect, Shell};
#[derive(Resource)]
pub struct Desktop {
    pub shell: Shell,
    pub decorations: bool,
    initialized: bool,
    previous_height: f32,
    last_write: f32,
    pub diagnostics: String,
}
impl Desktop {
    pub fn new(safe: bool, expanded: bool) -> Self {
        Self {
            shell: Shell {
                safe,
                expanded,
                ..default()
            },
            decorations: safe,
            initialized: false,
            previous_height: 180.,
            last_write: -5.,
            diagnostics: String::new(),
        }
    }
}
pub fn update(
    time: Res<Time<Real>>,
    native: NonSend<crate::native::NativeRuntime>,
    mut d: ResMut<Desktop>,
    mut q: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    session: Res<Session>,
) {
    let Ok((mut w, mut cursor)) = q.single_mut() else {
        return;
    };
    d.shell.tick(time.delta_secs());
    if native.recovery_requested() {
        d.shell.recover();
    }
    cursor.hit_test = d.shell.mode == Mode::Interaction;
    w.window_level = if d.shell.always_on_top {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    };
    w.decorations = d.decorations;
    let saved = if !d.initialized {
        std::fs::read(session.dir.join("window.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Placement>(&b).ok())
            .filter(|s| {
                [s.rect.x, s.rect.y, s.rect.w, s.rect.h]
                    .iter()
                    .all(|v| v.is_finite())
                    && s.rect.w > 0.
                    && s.rect.h > 0.
            })
    } else {
        None
    };
    let position = saved
        .as_ref()
        .map(|s| IVec2::new(s.rect.x as i32, s.rect.y as i32))
        .unwrap_or_else(|| match w.position {
            WindowPosition::At(p) => p,
            _ => IVec2::ZERO,
        });
    let monitor = saved
        .as_ref()
        .and_then(|s| {
            monitors.iter().find(|m| {
                m.name == s.monitor
                    && m.physical_position.x == s.monitor_x
                    && m.physical_position.y == s.monitor_y
            })
        })
        .or_else(|| {
            monitors
                .iter()
                .find(|m| {
                    position.x >= m.physical_position.x
                        && position.y >= m.physical_position.y
                        && position.x < m.physical_position.x + m.physical_width as i32
                        && position.y < m.physical_position.y + m.physical_height as i32
                })
                .or_else(|| monitors.iter().next())
        });
    let Some(m) = monitor else {
        return;
    };
    let scale = m.scale_factor as f32;
    let native_area = crate::native::work_area(m);
    let work = native_area.unwrap_or(Rect {
        x: m.physical_position.x as f32,
        y: m.physical_position.y as f32 + 32. * scale,
        w: m.physical_width as f32,
        h: (m.physical_height as f32 - 112. * scale).max(1.),
    });
    let height = if d.shell.expanded { 540. } else { 180. };
    let current = Rect {
        x: position.x as f32,
        y: position.y as f32,
        w: w.resolution.physical_width() as f32,
        h: w.resolution.physical_height() as f32,
    };
    let mut rect = current.fit(work, height * scale);
    if !d.initialized {
        rect = saved
            .map(|s| s.rect)
            .unwrap_or(Rect {
                x: work.x + (work.w - 960. * scale).max(0.) / 2.,
                y: work.y + work.h - 180. * scale,
                w: 960. * scale,
                h: 180. * scale,
            })
            .fit(work, height * scale);
        d.initialized = true;
    }
    if current != rect || d.previous_height != height {
        w.position = WindowPosition::At(IVec2::new(rect.x as i32, rect.y as i32));
        w.resolution.set(rect.w / scale, rect.h / scale);
        d.previous_height = height;
    }
    if time.elapsed_secs() - d.last_write >= 5. {
        d.last_write = time.elapsed_secs();
        let _ = std::fs::write(
            session.dir.join("window.json"),
            serde_json::to_vec(&Placement {
                rect,
                monitor: m.name.clone(),
                monitor_x: m.physical_position.x,
                monitor_y: m.physical_position.y,
            })
            .unwrap(),
        );
    }
    d.diagnostics = format!(
        "monitor={:?} physical={}x{} scale={} position={:?}; native_work_area={}; {}",
        m.name,
        m.physical_width,
        m.physical_height,
        m.scale_factor,
        w.position,
        native_area.is_some(),
        native.status
    );
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Placement {
    rect: Rect,
    monitor: Option<String>,
    monitor_x: i32,
    monitor_y: i32,
}
