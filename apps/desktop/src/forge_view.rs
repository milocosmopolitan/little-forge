//! Persistent D01 viewport and deliberately small management host. No gameplay state lives here.
use crate::{Session, spaces, title::TitleScreen, village::Village, window::Desktop};
use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    prelude::*,
    ui_widgets::Activate,
};
use forge_core::{spaces::Space, village::Screen};

pub const WIDTH: f32 = 960.;
pub const HEIGHT: f32 = 180.;
pub const FOOT_Y: f32 = 156.;
pub const REST: Vec2 = Vec2::new(352., FOOT_Y);

#[derive(Resource, Default, Debug)]
pub struct Layout {
    pub origin: Vec2,
    pub physical_origin: UVec2,
    pub physical_size: UVec2,
}
impl Layout {
    pub fn calculate(logical: Vec2, physical: UVec2, dpi: f32) -> Self {
        let h = (HEIGHT * dpi).round().max(1.) as u32;
        let height = h.min(physical.y.max(1));
        Self {
            origin: Vec2::new((logical.x - WIDTH) / 2., (logical.y - HEIGHT).max(0.)),
            physical_origin: UVec2::new(0, physical.y.saturating_sub(height)),
            physical_size: UVec2::new(physical.x.max(1), height),
        }
    }
}
#[derive(Component)]
pub struct WorldCamera;
#[derive(Component)]
pub struct BackdropPart(pub f32);
#[derive(Resource, Default)]
pub struct ModalFrame(pub bool);
fn remember_modal(dialog: Res<forge_ui::DialogState>, mut guard: ResMut<ModalFrame>) {
    guard.0 = dialog.is_open();
}
pub fn no_modal_at_frame_start(guard: Res<ModalFrame>) -> bool {
    !guard.0
}
#[derive(Resource, Default)]
pub struct Host {
    pub legacy: bool,
}
#[derive(Component)]
struct HostRoot;
#[derive(Component)]
struct HostStatus;
#[derive(Component)]
pub struct AmbientStatus;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Open,
    Close,
    Legacy,
    Modal,
}

pub fn spot(id: &str) -> Vec2 {
    Vec2::new(
        match id {
            "research-table" => 112.,
            "forge" => 416.,
            "anvil" => 576.,
            "quenching-tub" => 720.,
            "salvage-table" => 864.,
            _ => REST.x,
        },
        FOOT_Y,
    )
}
pub fn station_origin(placement: &forge_npc::player::StationPlacement) -> Vec2 {
    spot(&placement.id) + placement.offset.as_vec2()
}
pub fn shown(title: &TitleScreen, village: &Village, desktop: &Desktop, host: &Host) -> bool {
    !title.active && desktop.shell.expanded && village.nav.screen == Screen::Home && !host.legacy
}
pub fn setup(mut c: Commands) {
    c.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(1),
        WorldCamera,
    ));
    c.spawn((HostRoot, GlobalZIndex(30), Node { display:Display::None, position_type:PositionType::Absolute,
        left:px(0),top:px(0),width:percent(100),height:px(360), padding:UiRect::all(px(20)),
        flex_direction:FlexDirection::Column,row_gap:px(16), overflow:Overflow::clip(),..default() },
        BackgroundColor(forge_ui::SURFACE),bevy::input_focus::tab_navigation::TabGroup::new(0)))
        .with_children(|p| {
            p.spawn(forge_ui::text("Workshop · D01 layout validation", 24.));
            p.spawn(forge_ui::text("The workshop below stays live. This is the management host, not the finished Dashboard.",16.));
            p.spawn((HostStatus, forge_ui::text("",16.)));
            p.spawn(Node {column_gap:px(12),..default()}).with_children(|p| {
                forge_ui::button(p,"Existing workshop controls",Action::Legacy);
                forge_ui::button(p,"Check modal focus",Action::Modal);
                forge_ui::button(p,"Close [Esc]",Action::Close);
            });
            p.spawn(forge_ui::text("F8: 10-second click-through · Ctrl+Alt+F8: recovery · Drag the handle to move",14.));
        });
}
pub fn layout(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut layout: ResMut<Layout>,
    mut cameras: Query<&mut Camera, With<WorldCamera>>,
) {
    let Ok(w) = windows.single() else { return };
    *layout = Layout::calculate(
        Vec2::new(w.width(), w.height()),
        UVec2::new(
            w.resolution.physical_width(),
            w.resolution.physical_height(),
        ),
        w.resolution.scale_factor(),
    );
    for mut c in &mut cameras {
        c.viewport = Some(Viewport {
            physical_position: layout.physical_origin,
            physical_size: layout.physical_size,
            ..default()
        });
    }
}
fn activity_label(s: &forge_core::State) -> String {
    use forge_core::spaces::Activity;
    match s.activity() {
        Activity::Working { station, progress } => {
            format!("{} · {}%", station.label(), progress / 10)
        }
        Activity::Idle => "Taking a breather".into(),
        Activity::Waiting(reason) => reason.label().into(),
        Activity::Sleeping => "Resting".into(),
        Activity::Eating => "Eating".into(),
        Activity::Drinking => "Drinking".into(),
    }
}
#[allow(clippy::too_many_arguments)]
fn update(
    title: Res<TitleScreen>,
    v: Res<Village>,
    d: Res<Desktop>,
    host: Res<Host>,
    s: Res<Session>,
    mut roots: Query<&mut Node, With<HostRoot>>,
    mut texts: Query<&mut Text, With<HostStatus>>,
    mut ambient: Query<&mut Text, (With<AmbientStatus>, Without<HostStatus>)>,
) {
    for mut n in &mut roots {
        n.display = if shown(&title, &v, &d, &host) {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut t in &mut ambient {
        let next = activity_label(&s.save.state);
        if t.0 != next {
            t.0 = next;
        }
    }
    for mut t in &mut texts {
        let next = format!(
            "{} · {}\nQueued jobs: {}\n{}",
            s.save.state.smith.map_or("Smith", |x| x.id()),
            activity_label(&s.save.state),
            s.save.state.life.queue.len(),
            s.status
        );
        if t.0 != next {
            t.0 = next;
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn activate(
    e: On<Activate>,
    actions: Query<&Action>,
    mut host: ResMut<Host>,
    mut d: ResMut<Desktop>,
    mut v: ResMut<Village>,
    mut spaces: ResMut<spaces::Spaces>,
    mut dialog: ResMut<forge_ui::DialogState>,
    title: Res<TitleScreen>,
) {
    if title.active || dialog.is_open() {
        return;
    }
    let Ok(a) = actions.get(e.entity) else { return };
    match a {
        Action::Open => {
            host.legacy = false;
            d.shell.recover();
            d.shell.expanded = true;
            v.nav = Default::default();
            spaces.enter(Space::Workshop);
        }
        Action::Close => {
            d.shell.expanded = false;
            d.shell.recover();
        }
        Action::Legacy if d.shell.expanded => {
            host.legacy = true;
            spaces.manage = true;
            spaces.redraw = true;
        }
        Action::Modal if d.shell.expanded => {
            dialog.open(
                "d01-focus",
                "Input check",
                "The forge stays live while this modal owns input.",
                Some(e.entity),
            );
        }
        _ => {}
    }
}
pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
    mut d: ResMut<Desktop>,
    host: Res<Host>,
    v: Res<Village>,
) {
    if title.active || dialog.is_open() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && shown(&title, &v, &d, &host) {
        d.shell.expanded = false;
    }
}
#[allow(clippy::too_many_arguments)]
fn restore_host_focus(
    title: Res<TitleScreen>,
    v: Res<Village>,
    d: Res<Desktop>,
    host: Res<Host>,
    dialog: Res<forge_ui::DialogState>,
    mut previous: Local<bool>,
    actions: Query<(Entity, &Action)>,
    mut focus: ResMut<bevy::input_focus::InputFocus>,
) {
    let visible = shown(&title, &v, &d, &host);
    if visible != *previous && !dialog.is_open() {
        let target = if visible {
            Action::Legacy
        } else {
            Action::Open
        };
        if let Some((entity, _)) = actions.iter().find(|(_, a)| **a == target) {
            focus.set(entity, bevy::input_focus::FocusCause::Auto);
        }
    }
    *previous = visible;
}
fn drag(
    mouse: Res<ButtonInput<MouseButton>>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
    guard: Res<ModalFrame>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if title.active || dialog.is_open() || guard.0 || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(mut w) = windows.single_mut() else {
        return;
    };
    if let Some(p) = w.cursor_position()
        && p.x >= 120.
        && p.x < 220.
        && p.y >= w.height() - 176.
        && p.y < w.height() - 152.
    {
        w.start_drag_move();
    }
}
fn scroll_legacy(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
    dialog: Res<forge_ui::DialogState>,
    mut roots: Query<(&ComputedNode, &mut ScrollPosition, &Node), With<crate::village::Root>>,
) {
    let delta: f32 = wheel
        .read()
        .map(|e| {
            e.y * if e.unit == bevy::input::mouse::MouseScrollUnit::Line {
                24.
            } else {
                1.
            }
        })
        .sum();
    let Ok(w) = windows.single() else { return };
    if dialog.is_open() || !w.cursor_position().is_some_and(|p| p.y < 360.) {
        return;
    }
    for (computed, mut scroll, node) in &mut roots {
        if node.display == Display::None {
            continue;
        }
        let max = ((computed.content_size().y - computed.size().y)
            * computed.inverse_scale_factor())
        .max(0.);
        scroll.0.y = (scroll.0.y - delta).clamp(0., max);
    }
}
pub fn plugin(app: &mut App) {
    app.init_resource::<ModalFrame>()
        .add_systems(First, remember_modal)
        .init_resource::<Layout>()
        .init_resource::<Host>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                layout,
                update,
                keyboard.run_if(no_modal_at_frame_start),
                restore_host_focus,
                crate::npc::render_layers,
                drag,
                scroll_legacy,
            )
                .chain()
                .after(crate::window::update)
                .before(spaces::route),
        );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dpi_edges_keep_the_same_lower_strip() {
        for dpi in [1., 1.25, 1.5, 2.] {
            let compact = Layout::calculate(
                Vec2::new(960., 180.),
                UVec2::new((960. * dpi) as u32, (180. * dpi) as u32),
                dpi,
            );
            let expanded = Layout::calculate(
                Vec2::new(960., 540.),
                UVec2::new((960. * dpi) as u32, (540. * dpi) as u32),
                dpi,
            );
            assert_eq!(compact.physical_size, expanded.physical_size);
            assert_eq!(
                expanded.physical_origin.y + expanded.physical_size.y,
                (540. * dpi) as u32
            );
            assert_eq!(expanded.origin.y - compact.origin.y, 360.);
        }
    }
}
