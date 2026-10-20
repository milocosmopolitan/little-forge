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
/// Free floor left of the anvil, in front of the counter floor (y=160) so the resting smith is
/// never drawn behind a fixture. Kept close to the furnace: work starts there.
pub const REST: Vec2 = Vec2::new(360., 166.);

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
    /// Kit-v2 screen shown in the upper 360px (Home keeps the D01 host content).
    pub screen: HostScreen,
    pub order_filter: usize,
    pub order: Option<crate::screens::OrderId>,
    pub family: usize,
    pub weapon: Option<usize>,
    /// Crafting (#110): selected tab and recipe.
    pub craft_tab: usize,
    pub recipe: Option<forge_core::items::Id>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HostScreen {
    #[default]
    Home,
    Orders,
    Collection,
    Crafting,
}
#[derive(Component)]
pub(crate) struct HostRoot;
#[derive(Component)]
pub(crate) struct HostStatus;
#[derive(Component)]
pub struct AmbientStatus;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Open,
    Close,
    Legacy,
    Modal,
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
    c.spawn((HostRoot, Visibility::Hidden, GlobalZIndex(30), Node { display:Display::None, position_type:PositionType::Absolute,
        left:px(0),top:px(0),width:percent(100),height:px(360),
        flex_direction:FlexDirection::Column, overflow:Overflow::clip(),..default() },
        BackgroundColor(forge_ui::SURFACE),bevy::input_focus::tab_navigation::TabGroup::new(0)))
        .with_children(|root| {
            root.spawn(crate::screens::page());
            root.spawn((crate::screens::HomePage, Node { width:percent(100),height:percent(100), padding:UiRect::all(px(20)),
                flex_direction:FlexDirection::Column,row_gap:px(16),..default() })).with_children(|p| {
            p.spawn(forge_ui::text("Workshop · D01 layout validation", 24.));
            p.spawn(forge_ui::text("The workshop below stays live. This is the management host, not the finished Dashboard.",16.));
            p.spawn((HostStatus, forge_ui::text("",16.)));
            p.spawn(Node {column_gap:px(12),..default()}).with_children(|p| {
                let lang = crate::text::Lang::from_env();
                forge_ui::button(p,crate::text::ORDERS.get(lang),crate::screens::Nav::Screen(HostScreen::Orders));
                forge_ui::button(p,crate::text::COLLECTION.get(lang),crate::screens::Nav::Screen(HostScreen::Collection));
                forge_ui::button(p,crate::text::CRAFTING.get(lang),crate::screens::Nav::Screen(HostScreen::Crafting));
                forge_ui::button(p,"Existing workshop controls",Action::Legacy);
                forge_ui::button(p,"Check modal focus",Action::Modal);
                forge_ui::button(p,"Close [Esc]",Action::Close);
            });
            p.spawn(forge_ui::text("F8: 10-second click-through · Ctrl+Alt+F8: recovery · Drag the handle to move",14.));
            });
        });
}
pub fn layout(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut layout: ResMut<Layout>,
    mut cameras: Query<(&mut Camera, &mut Projection, Has<WorldCamera>)>,
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
    for (mut c, mut projection, world) in &mut cameras {
        // Winit publishes WindowResized in Last, after camera/UI layout. Invalidate
        // projection now so this frame cannot combine a new surface with old coordinates.
        if c.physical_target_size() != Some(w.resolution.physical_size()) {
            projection.set_changed();
        }
        if !world {
            continue;
        }
        let viewport = Some(Viewport {
            physical_position: layout.physical_origin,
            physical_size: layout.physical_size,
            ..default()
        });
        c.viewport = viewport;
    }
}
// Keep layout active while the native surface catches up with the requested size.
// Visibility is published only after UI layout, before visibility propagation.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn publish_host(
    mut commands: Commands,
    mut capture: Local<(bool, u8)>,
    _main_thread: bevy::ecs::system::NonSendMarker,
    windows: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, Has<WorldCamera>)>,
    mut roots: Query<(&ComputedNode, &UiGlobalTransform, &mut Visibility), With<HostRoot>>,
    title: Res<TitleScreen>,
    village: Res<Village>,
    desktop: Res<Desktop>,
    host: Res<Host>,
    opts: Res<crate::Options>,
) {
    let Ok((entity, window)) = windows.single() else {
        return;
    };
    if let Some(path) = std::env::var_os("LITTLE_FORGE_RESIZE_CAPTURE_DIR") {
        if capture.0 != desktop.shell.expanded {
            capture.0 = desktop.shell.expanded;
            capture.1 = 4;
        }
        if capture.1 > 0 {
            capture.1 -= 1;
            let path = std::path::PathBuf::from(path);
            if std::fs::create_dir_all(&path).is_ok() {
                commands
                    .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                    .observe(crate::save_capture(
                        path.join(format!("frame-{:04}.png", opts.frames)),
                    ));
            }
        }
    }

    let physical = window.resolution.physical_size();
    let native = bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        windows.get_window(entity).map(|w| {
            let size = w.inner_size();
            UVec2::new(size.width, size.height)
        })
    });
    let dpi = window.resolution.scale_factor();
    let expected = Vec2::new(WIDTH, 540.) * dpi;
    let geometry_ready = native == Some(physical)
        && physical.as_vec2().abs_diff_eq(expected, 1.)
        && cameras.iter().all(|(camera, world)| {
            camera.physical_target_size() == Some(physical)
                && (!world
                    || camera.viewport.as_ref().is_some_and(|v| {
                        v.physical_position == UVec2::new(0, (360. * dpi).round() as u32)
                            && v.physical_size
                                == UVec2::new(physical.x, (HEIGHT * dpi).round() as u32)
                    }))
        });
    for (computed, transform, mut visibility) in &mut roots {
        let size = computed.size();
        let center = transform.to_scale_angle_translation().2;
        let bounds_ready =
            size.abs_diff_eq(Vec2::new(WIDTH, 360.) * dpi, 1.) && center.abs_diff_eq(size / 2., 1.);
        let visible = shown(&title, &village, &desktop, &host) && geometry_ready && bounds_ready;
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some(path) = std::env::var_os("LITTLE_FORGE_TRACE_DIR") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::PathBuf::from(path).join("host-layout.jsonl"))
            {
                let cameras: Vec<_> = cameras
                    .iter()
                    .map(|(c, world)| {
                        serde_json::json!({
                            "world":world,"target":format!("{:?}",c.physical_target_size()),
                            "viewport":format!("{:?}",c.viewport)
                        })
                    })
                    .collect();
                let _ = writeln!(
                    file,
                    "{}",
                    serde_json::json!({"frame":opts.frames,
                    "expanded":desktop.shell.expanded,"native":native.map(|v|v.to_array()),
                    "window":physical.to_array(),"host_size":size.to_array(),
                    "host_center":center.to_array(),"visible":visible,"cameras":cameras})
                );
            }
        }
    }
}
fn activity_label(s: &forge_core::State) -> String {
    use forge_core::spaces::Activity;
    match s.activity() {
        Activity::Working { station, progress } => {
            format!("{} · {}%", station.label(), progress / 10)
        }
        Activity::Idle => String::new(),
        Activity::Waiting(reason) => reason.label().into(),
        Activity::Sleeping => "Resting".into(),
        Activity::Eating => "Eating".into(),
        Activity::Drinking => "Drinking".into(),
    }
}
#[allow(clippy::too_many_arguments)]
pub fn update_host(
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
#[allow(clippy::too_many_arguments)]
pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
    mut d: ResMut<Desktop>,
    mut host: ResMut<Host>,
    mut v: ResMut<Village>,
    mut spaces: ResMut<spaces::Spaces>,
    mut was_expanded: Local<bool>,
) {
    // Expanded state as it was when this system last ran, so an Escape that another system used
    // to collapse the window this frame does not reopen it here.
    let previous = std::mem::replace(&mut *was_expanded, d.shell.expanded);
    if title.active || dialog.is_open() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && !d.shell.expanded && !previous {
        // Escape opens the menu from the collapsed workshop (same as Tab / Manage).
        host.legacy = false;
        d.shell.recover();
        d.shell.expanded = true;
        v.nav = Default::default();
        spaces.enter(Space::Workshop);
        *was_expanded = true;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && shown(&title, &v, &d, &host) {
        // A kit screen steps back to the host first; Esc on the host collapses the window.
        if host.screen != HostScreen::Home {
            host.screen = HostScreen::Home;
        } else {
            d.shell.expanded = false;
        }
        *was_expanded = d.shell.expanded;
    }
}
#[allow(clippy::too_many_arguments)]
fn restore_host_focus(
    roots: Query<&Visibility, With<HostRoot>>,
    dialog: Res<forge_ui::DialogState>,
    mut previous: Local<bool>,
    actions: Query<(Entity, &Action)>,
    mut focus: ResMut<bevy::input_focus::InputFocus>,
) {
    let visible = roots.iter().any(|v| *v != Visibility::Hidden);
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
            PostUpdate,
            (publish_host, restore_host_focus)
                .chain()
                .after(bevy::ui::UiSystems::Layout)
                .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
        )
        .add_systems(
            Update,
            (
                layout,
                update_host,
                keyboard.run_if(no_modal_at_frame_start),
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
    fn resize_updates_both_camera_targets_before_winit_resize_message() {
        use bevy::{
            render::{camera::camera_system, texture::ManualTextureViews},
            window::{PrimaryWindow, WindowCreated, WindowResized, WindowScaleFactorChanged},
        };
        let mut app = App::new();
        app.init_resource::<Layout>()
            .init_resource::<Assets<Image>>()
            .init_resource::<ManualTextureViews>()
            .add_message::<WindowCreated>()
            .add_message::<WindowResized>()
            .add_message::<WindowScaleFactorChanged>()
            .add_message::<AssetEvent<Image>>()
            .add_systems(Update, (layout, camera_system).chain());
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (960, 180).into(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let world = app.world_mut().spawn((Camera2d, WorldCamera)).id();
        let ui = app.world_mut().spawn(Camera2d).id();
        app.update();
        for height in [540, 180, 540, 180] {
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .resolution
                .set(960., height as f32);
            // Native acknowledgement deliberately has not arrived yet.
            app.update();
            for entity in [world, ui] {
                assert_eq!(
                    app.world()
                        .get::<Camera>(entity)
                        .unwrap()
                        .physical_target_size(),
                    Some(UVec2::new(960, height))
                );
            }
            let viewport = app
                .world()
                .get::<Camera>(world)
                .unwrap()
                .viewport
                .as_ref()
                .unwrap();
            assert_eq!(viewport.physical_position.y, height - 180);
            assert_eq!(viewport.physical_size, UVec2::new(960, 180));
        }
    }
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
    #[test]
    fn escape_opens_steps_back_then_collapses() {
        let mut app = App::new();
        app.insert_resource(TitleScreen::default())
            .init_resource::<forge_ui::DialogState>()
            .insert_resource(Desktop::new(false, false))
            .init_resource::<Host>()
            .init_resource::<Village>()
            .init_resource::<spaces::Spaces>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, keyboard);
        let esc = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::Escape);
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.clear();
            app.update();
        };
        app.update();
        esc(&mut app);
        assert!(
            app.world().resource::<Desktop>().shell.expanded,
            "Escape opens the menu"
        );
        app.world_mut().resource_mut::<Host>().screen = HostScreen::Orders;
        esc(&mut app);
        assert!(app.world().resource::<Desktop>().shell.expanded);
        assert_eq!(
            app.world().resource::<Host>().screen,
            HostScreen::Home,
            "Orders steps back"
        );
        esc(&mut app);
        assert!(
            !app.world().resource::<Desktop>().shell.expanded,
            "host collapses"
        );
        esc(&mut app);
        assert!(
            app.world().resource::<Desktop>().shell.expanded,
            "and opens again"
        );
    }
}
