//! One physical workshop with sales, plus a secondary exterior view. Both read the same
//! `Session` state; nothing here owns a timer, reward or inventory. Motion is a reenactment of
//! `State::activity()`, so switching or hiding a view never changes when work completes.
use crate::{Options, Session, title::TitleScreen, village::Village, window::Desktop};
use bevy::{picking::Pickable, prelude::*, ui_widgets::Activate};
use forge_core::{
    Command,
    showroom::Slot,
    spaces::{Activity, Space, Workstation},
    village::Screen,
};
use forge_npc::{
    player::{Actor, CharacterId, PlayerCatalogs, Walker},
    stations::{self},
};

const EXTERIOR_FOOT: Vec2 = Vec2::new(-72., -85.);

#[derive(Resource)]
pub struct Spaces {
    pub space: Space,
    pub station: Option<Workstation>,
    pub manage: bool,
    pub slot: Option<usize>,
    /// Selected display tab (rack index) in the showroom window.
    pub rack: usize,
    pub redraw: bool,
    /// Integer scene scale and window-pixel origin of the interior scene.
    pub scale: f32,
    pub view_origin: Vec2,
    pub window: Vec2,
}
impl Default for Spaces {
    fn default() -> Self {
        Self {
            space: Space::Workshop,
            station: None,
            manage: false,
            slot: None,
            rack: 0,
            redraw: true,
            scale: 1.,
            view_origin: Vec2::ZERO,
            window: Vec2::new(960., 540.),
        }
    }
}
impl Spaces {
    pub fn enter(&mut self, space: Space) {
        if self.space != space {
            self.station = None;
            self.slot = None;
        }
        self.space = space;
        self.redraw = true;
    }
}
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SceneOf(pub Space);
#[derive(Component)]
pub struct Backdrop;
#[derive(Component)]
pub struct WorkshopActor;
#[derive(Component)]
pub struct ExteriorActor;
#[derive(Component)]
pub struct WindowGlow;
#[derive(Component)]
pub struct HomeRoot;
#[derive(Component)]
pub struct HitRoot;
#[derive(Component)]
pub struct AmbientRoot;
#[derive(Component, Clone, Copy, PartialEq)]
pub enum Action {
    Go(Space),
    Station(Workstation),
    CloseDrawer,
    Manage,
    Slot(usize),
    Rack(usize),
    ForSale(usize, bool),
    Carpenter,
    Enter,
}
/// Registered forge-space art (`assets/forge/spaces/catalog.json`): the two interiors and the
/// nine-slice frames cut from the supplied showroom window and detail panel.
#[derive(Resource, Clone, Deserialize)]
pub struct SpaceArt {
    pub assets: std::collections::BTreeMap<String, SpaceAsset>,
}
#[derive(Clone, Deserialize)]
pub struct SpaceAsset {
    pub path: String,
    #[serde(default)]
    pub insets_ltrb: [f32; 4],
    #[serde(default)]
    pub corner_scale: f32,
}
use serde::Deserialize;
impl SpaceArt {
    pub fn bundled() -> Self {
        serde_json::from_str(include_str!("../../../assets/forge/spaces/catalog.json"))
            .expect("validated space art catalog")
    }
    pub fn frame(&self, id: &str) -> forge_ui::SkinFrame {
        let a = &self.assets[id];
        forge_ui::SkinFrame {
            path: a.path.clone(),
            inset: a.insets_ltrb[0],
            corner_scale: a.corner_scale,
        }
    }
}
pub fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    art: Res<SpaceArt>,
    s: Res<Session>,
    opts: Res<Options>,
    mut spaces: ResMut<Spaces>,
) {
    if opts.qa.is_none() {
        spaces.space = Space::Workshop;
    }
    // QA scenes select their documented view; a new player's selection enters the workshop.
    if opts
        .qa
        .as_deref()
        .is_some_and(|q| q.starts_with("repair-") || q.starts_with("alpha"))
    {
        // Repair fixtures open their drawer; Alpha follows the real title route.
        spaces.space = Space::Workshop;
        spaces.manage = !opts.qa.as_deref().is_some_and(|q| q.starts_with("alpha"));
    }
    if let Some((scene, _)) = opts.forge_scene() {
        spaces.space = match scene {
            "workshop" | "full" | "input" => Space::Workshop,
            "showroom" => Space::Showroom,
            _ => Space::Exterior,
        };
    }
    let character = s.save.state.smith.map_or("male", |c| c.id()).to_string();
    // One continuous room, exported without stretching. The transparent roof margin
    // leaves room for ambient controls; canonical stations remain independent layers.
    commands.spawn((
        Sprite::from_image(server.load("forge/interior/background.png")),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-480., 90., 0.5),
        Visibility::Hidden,
        SceneOf(Space::Workshop),
        Backdrop,
        crate::forge_view::BackdropPart(0.),
        bevy::camera::visibility::RenderLayers::layer(1),
    ));
    commands.spawn((
        Sprite::default(),
        bevy::sprite::Anchor::default(),
        Transform::from_xyz(0., 0., stations::actor_z(1)),
        Visibility::Hidden,
        Actor::default(),
        Walker {
            pos: crate::forge_view::REST,
            target: crate::forge_view::REST,
            ..default()
        },
        CharacterId(character.clone()),
        SceneOf(Space::Workshop),
        WorkshopActor,
        bevy::camera::visibility::RenderLayers::layer(1),
    ));
    // Space::Showroom remains a compatible management route into this same room.

    // Exterior: the chosen smith steps outside when there is nothing to do; a lit window shows work.
    commands.spawn((
        Sprite::default(),
        bevy::sprite::Anchor::default(),
        Transform::from_xyz(EXTERIOR_FOOT.x, EXTERIOR_FOOT.y, 2.5),
        Visibility::Hidden,
        Actor::default(),
        CharacterId(character),
        SceneOf(Space::Exterior),
        ExteriorActor,
    ));
    commands.spawn((
        Sprite {
            image: server.load(art.assets["window-glow"].path.clone()),
            custom_size: Some(Vec2::new(48., 40.)),
            color: Color::srgba(1., 1., 1., 0.),
            ..default()
        },
        Transform::from_xyz(-236., 42., 0.6),
        Visibility::Hidden,
        SceneOf(Space::Exterior),
        WindowGlow,
    ));
    commands.spawn((
        HitRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(11),
        Visibility::Hidden,
    ));
    commands
        .spawn((
            AmbientRoot,
            bevy::input_focus::tab_navigation::TabGroup::new(2),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(9),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                forge_ui::text("···", 14.),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(148),
                    bottom: px(160),
                    width: px(44),
                    height: px(14),
                    ..default()
                },
                BackgroundColor(forge_ui::SURFACE),
                Pickable::IGNORE,
            ));
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(12),
                    bottom: px(152),
                    padding: UiRect::axes(px(8), px(3)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.04, 0.03, 0.75)),
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                row.spawn((
                    forge_ui::text("", 12.),
                    crate::forge_view::AmbientStatus,
                    Pickable::IGNORE,
                ));
            });
            // Transparent hit area over the open front of the building; only the label is drawn.
            p.spawn((
                bevy::ui_widgets::Button,
                crate::forge_view::Action::Open,
                forge_ui::Help("Open management (Tab)".into()),
                bevy::picking::hover::Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(8),
                    bottom: px(152),
                    width: px(100),
                    height: px(28),
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|door| {
                door.spawn((
                    forge_ui::text("Menu", 12.),
                    Node {
                        padding: UiRect::axes(px(6), px(2)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.04, 0.03, 0.75)),
                    Pickable::IGNORE,
                ));
            });
        });
}
/// Which scenes are visible follows window mode, village screen and the chosen space.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn route(
    mut spaces: ResMut<Spaces>,
    layout: Res<crate::forge_view::Layout>,
    village: Res<Village>,
    title: Res<TitleScreen>,
    opts: Res<Options>,
    s: Res<Session>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut scenes: Query<(
        &SceneOf,
        &mut Visibility,
        &mut Transform,
        Option<&Backdrop>,
        Option<&crate::forge_view::BackdropPart>,
    )>,
    mut roots: Query<
        (&mut Visibility, Has<HitRoot>),
        (Without<SceneOf>, Or<(With<HitRoot>, With<AmbientRoot>)>),
    >,
    legacy: Query<Entity, Or<(With<crate::Visitor>, With<crate::Smoke>)>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    spaces.window = Vec2::new(window.width(), window.height());
    let home = village.nav.screen == Screen::Home;
    let shown = !title.active && s.save.state.smith.is_some() && qa_allows(&opts);
    let interior = shown;
    let exterior = shown && home && spaces.space == Space::Exterior;
    let scale = 1.;
    spaces.scale = scale;
    spaces.view_origin = layout.origin;
    let world_origin = Vec2::new(-480., 90.);
    for (scene, mut visibility, mut transform, backdrop, compact) in &mut scenes {
        let visible = match scene.0 {
            Space::Exterior => exterior,
            Space::Workshop => interior && !exterior,
            Space::Showroom => false,
        };
        let next = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
        if let Some(part) = compact {
            transform.translation.x = world_origin.x + part.0;
            transform.translation.y = world_origin.y;
        } else if scene.0.interior() && backdrop.is_some() {
            transform.translation.x = world_origin.x;
            transform.translation.y = world_origin.y;
            transform.scale = Vec3::splat(scale);
        }
    }
    for (mut visibility, hit) in &mut roots {
        let next = if (hit && interior && !exterior) || (!hit && shown) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
    let _ = legacy;
}
fn qa_allows(opts: &Options) -> bool {
    opts.qa
        .as_deref()
        .is_none_or(|q| !q.starts_with("village-") || q == "village-home" || q == "village-input")
}
/// Keep the three actors on the chosen character; re-arm their clip after a swap.
pub fn sync_character(
    s: Res<Session>,
    catalogs: Option<Res<PlayerCatalogs>>,
    mut actors: Query<(&mut CharacterId, &mut Actor)>,
) {
    let (Some(smith), Some(catalogs)) = (s.save.state.smith, catalogs) else {
        return;
    };
    for (mut id, mut actor) in &mut actors {
        if id.0 != smith.id() {
            id.0 = smith.id().into();
            if let Some(c) = catalogs.get(smith.id()) {
                let clip = actor.clip.clone();
                actor.play(&clip, c);
            }
        }
    }
}
/// Drive the workshop smith from the authoritative activity. Walking and clip choice are
/// presentation; completion times live in forge-core.
#[allow(clippy::type_complexity)]
pub fn drive_workshop(
    s: Res<Session>,
    spaces: Res<Spaces>,
    mut q: Query<
        (
            &mut Actor,
            &mut Walker,
            &mut Transform,
            &mut Sprite,
            &CharacterId,
        ),
        With<WorkshopActor>,
    >,
    catalogs: Option<Res<PlayerCatalogs>>,
    interior: Res<crate::interior::Interior>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    for (mut actor, mut walker, mut transform, mut sprite, id) in &mut q {
        let Some(c) = catalogs.get(&id.0) else {
            continue;
        };
        let (spot, clip, hold, tint) = match s.save.state.activity() {
            Activity::Working { station, .. } => (
                stations::Spot {
                    foot: interior.foot(station.id(), c),
                    row: 1,
                },
                station.clip(),
                true,
                Color::WHITE,
            ),
            Activity::Sleeping => (
                stations::Spot {
                    foot: crate::forge_view::REST,
                    row: 1,
                },
                "relaxed",
                true,
                Color::srgb(0.55, 0.65, 0.85),
            ),
            Activity::Eating => (
                stations::Spot {
                    foot: crate::forge_view::REST,
                    row: 1,
                },
                "happy",
                true,
                Color::WHITE,
            ),
            Activity::Drinking => (
                stations::Spot {
                    foot: crate::forge_view::REST,
                    row: 1,
                },
                "nod",
                true,
                Color::WHITE,
            ),
            Activity::Idle | Activity::Waiting(_) => (
                stations::Spot {
                    foot: crate::forge_view::REST,
                    row: 1,
                },
                "idle",
                false,
                Color::WHITE,
            ),
        };
        let wants = walker.target != spot.foot
            || (walker.arrived()
                && walker.arrival.is_none()
                && actor.clip != clip
                && actor.clip != "walk"
                && !(clip == "idle" && actor.clip == "blink"));
        if wants {
            walker.go(spot.foot, clip, hold);
        }
        if !hold && actor.hold {
            actor.release();
        }
        transform.scale = Vec3::splat(spaces.scale);
        let shown = Vec2::new(960., 180.) * spaces.scale;
        transform.translation.x = -shown.x / 2. + walker.pos.x * spaces.scale;
        transform.translation.y = shown.y / 2. - walker.pos.y * spaces.scale;
        transform.translation.z = stations::actor_z(0);
        if sprite.color != tint && sprite.color != Color::NONE {
            sprite.color = tint;
        }
        let _ = c;
    }
}
/// Showroom and exterior smiths: idle presence only, no movement between views.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn drive_presence(
    s: Res<Session>,
    spaces: Res<Spaces>,
    mut exterior: Query<(&mut Actor, &mut Transform, &mut Sprite), With<ExteriorActor>>,
    mut glow: Query<&mut Sprite, (With<WindowGlow>, Without<ExteriorActor>, Without<Actor>)>,
    time: Res<Time>,
    opts: Res<Options>,
    catalogs: Option<Res<PlayerCatalogs>>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    let activity = s.save.state.activity();
    let _ = &spaces;
    let offset = 0.;
    let seconds = if opts.qa.is_some() {
        opts.frames as f32 / 30.
    } else {
        time.elapsed_secs()
    };
    for (mut actor, mut t, mut sprite) in &mut exterior {
        t.translation.x = EXTERIOR_FOOT.x;
        t.translation.y = EXTERIOR_FOOT.y + offset;
        let (clip, outside) = match activity {
            Activity::Idle | Activity::Waiting(_) => ("idle", true),
            Activity::Eating => ("happy", true),
            Activity::Drinking => ("relaxed", true),
            Activity::Working { .. } | Activity::Sleeping => ("idle", false),
        };
        if !outside {
            if sprite.color != Color::NONE {
                sprite.color = Color::NONE;
            }
        } else if sprite.color == Color::NONE {
            sprite.color = Color::WHITE;
        }
        if outside
            && actor.clip != clip
            && actor.clip != "blink"
            && let Some(c) = catalogs.get(&actor_character(&s))
        {
            actor.hold = clip != "idle";
            actor.play(clip, c);
        }
    }
    for mut g in &mut glow {
        let lit = matches!(activity, Activity::Working { .. });
        let alpha = if lit {
            0.55 + 0.25 * (seconds * 5.3).sin().abs()
        } else {
            0.
        };
        g.color = Color::srgba(1., 1., 1., alpha);
    }
}
fn actor_character(s: &Session) -> String {
    s.save.state.smith.map_or("male", |c| c.id()).into()
}
fn status_line(s: &Session) -> String {
    let st = &s.save.state;
    let room = &st.showroom;
    let activity = match st.activity() {
        Activity::Working { station, progress } => {
            format!(
                "{} {} · {}%",
                station.label(),
                st.life
                    .active_job
                    .map_or("work", |job| job.label())
                    .to_lowercase(),
                progress / 10
            )
        }
        Activity::Waiting(reason) => format!("Waiting · {}", reason.label()),
        Activity::Idle => "Idle".into(),
        Activity::Sleeping => "Sleeping".into(),
        Activity::Eating => "Eating".into(),
        Activity::Drinking => "Drinking".into(),
    };
    format!(
        "{activity}   |   Shelves {}/{} used · {} for sale · {} kept{}",
        room.goods() as usize - room.overflow as usize + room.reserved(),
        room.capacity(),
        room.displayed(),
        room.held(),
        if room.overflow > 0 {
            format!(" · {} older items waiting", room.overflow)
        } else {
            String::new()
        }
    )
}
/// Rebuild the interior/ambient UI only when its inputs change.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn draw(
    mut commands: Commands,
    mut spaces: ResMut<Spaces>,
    s: Res<Session>,
    desktop: Res<Desktop>,
    village: Res<Village>,
    server: Res<AssetServer>,
    home: Query<Entity, With<HomeRoot>>,
    art: Res<SpaceArt>,
    mut last: Local<Option<String>>,
) {
    let st = &s.save.state;
    let key = format!(
        "{:?}:{:?}:{}:{:?}:{}:{}:{:?}:{:?}:{}:{}:{:?}:{}:{:?}",
        spaces.space,
        spaces.station,
        spaces.manage,
        spaces.slot,
        spaces.rack,
        desktop.shell.expanded,
        village.nav.screen,
        st.showroom.slots,
        st.showroom.racks.len(),
        st.gold,
        if spaces.manage {
            String::new()
        } else {
            format!("{:?}", st.activity())
        },
        spaces.scale,
        home.single().ok(),
    );
    if !spaces.redraw && last.as_deref() == Some(&key) {
        return;
    }
    *last = Some(key);
    spaces.redraw = false;
    // World-space alpha picking and outlines are owned by interior.rs.
    let Ok(root) = home.single() else {
        return;
    };
    commands.entity(root).despawn_children();
    commands.entity(root).with_children(|p| match spaces.space {
        Space::Exterior => draw_exterior(p, &s),
        Space::Workshop => draw_workshop(p, &s, &spaces, &server),
        Space::Showroom => draw_showroom(p, &s, &spaces, &server, &art),
    });
}
fn draw_exterior(p: &mut ChildSpawnerCommands, s: &Session) {
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            padding: UiRect::all(px(10)),
            column_gap: px(10),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.105, 0.083, 0.068, 0.85)),
    ))
    .with_children(|row| {
        row.spawn(forge_ui::text(status_line(s), 13.));
        forge_ui::button(row, "Workshop", Action::Go(Space::Workshop));
        forge_ui::button(row, "Showroom", Action::Go(Space::Showroom));
    });
}
fn draw_workshop(p: &mut ChildSpawnerCommands, s: &Session, spaces: &Spaces, server: &AssetServer) {
    let st = &s.save.state;
    let open = spaces.station.is_some() || spaces.manage;
    if !open {
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                padding: UiRect::all(px(8)),
                column_gap: px(10),
                flex_wrap: FlexWrap::Wrap,
                right: px(0),
                row_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.105, 0.083, 0.068, 0.85)),
        ))
        .with_children(|row| {
            row.spawn(forge_ui::text(
                format!("{}   ·   Click a station to assign work", status_line(s)),
                13.,
            ));
            forge_ui::button(row, "Manage", Action::Manage);
            forge_ui::button(row, "Hunter expedition", crate::expedition::Action::Prepare);
            forge_ui::button(row, "Chef commission", crate::lifestyle::HomeTab(3));
            forge_ui::button(row, "Research", crate::lifestyle::HomeTab(4));
        });
        return;
    }
    // Keep all three ledger columns readable at the management window's target width.
    let drawer_width = if spaces.manage {
        (spaces.window.x - 48.).min(900.)
    } else {
        330.
    };
    p.spawn((
        forge_ui::SkinPanel,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            bottom: px(0),
            width: px(drawer_width),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(forge_ui::SURFACE),
        BorderColor::all(forge_ui::EDGE),
    ))
    .with_children(|drawer| {
        match spaces.station {
            Some(station) => {
                drawer.spawn(forge_ui::text(station.label(), 21.));
                drawer.spawn(forge_ui::text(status_line(s), 12.));
                if station.simulated() {
                    let good = st.current_good();
                    drawer.spawn(forge_ui::text(
                        format!(
                            "{} · {} ore · {} s\n{} → {} → {} are one job; the smith moves between stations as the work progresses.",
                            good.label(),
                            st.dagger_ore(),
                            st.work_seconds(forge_core::life::WorkKind::IronDagger),
                            Workstation::Forge.label(),
                            Workstation::Anvil.label(),
                            Workstation::QuenchingTub.label()
                        ),
                        13.,
                    ));
                    drawer.spawn(forge_ui::text(
                        format!(
                            "Ore {} · Queue {} · Result slots free {}",
                            st.ore,
                            st.life.queue.len(),
                            st.showroom.free()
                        ),
                        13.,
                    ));
                    if let Some(reason) = st.craft_block_reason() {
                        drawer.spawn(forge_ui::text(reason.label(), 13.));
                    }
                    crate::progression_ui::command_button(
                        drawer,
                        st,
                        "Queue iron dagger [Q]",
                        Command::QueueCraft,
                    );
                    forge_ui::button(
                        drawer,
                        "Buy 2 ore · 4 gold [B]",
                        crate::lifestyle::Action(Command::BuyOre),
                    );
                } else if station == Workstation::ResearchTable {
                    forge_ui::button(drawer, "Open research tree", crate::lifestyle::HomeTab(4));
                } else {
                    drawer.spawn(forge_ui::text(
                        format!(
                            "{} is not available in this alpha. Return to the workshop to continue crafting.",
                            station.label()
                        ),
                        13.,
                    ));
                }
                forge_ui::button(drawer, "Showroom", Action::Go(Space::Showroom));
            }
            None => {
                drawer.spawn((
                    crate::lifestyle::Management,
                    bevy::input_focus::tab_navigation::TabGroup::new(1),
                    Node {
                        flex_grow: 1.,
                        min_height: px(0),
                        ..default()
                    },
                ));
            }
        }
        forge_ui::button(drawer, "Close [Esc]", Action::CloseDrawer);
    });
    let _ = server;
}
fn slot_badge(slot: &Slot) -> (Option<&'static str>, Option<(&'static str, Color)>) {
    match slot {
        Slot::Empty => (None, None),
        Slot::Reserved => (Some("queue"), Some(("RESERVED", forge_ui::BADGE_RESERVED))),
        Slot::Displayed(g) => (Some(g.icon()), Some(("FOR SALE", forge_ui::BADGE_SALE))),
        Slot::Held(g) => (Some(g.icon()), Some(("HOLD", forge_ui::BADGE_HOLD))),
    }
}
/// Showroom per the 2026-10-18 draft: the room sets the mood, the inventory window manages
/// goods with the shared slot component, and the detail panel edits one slot.
fn draw_showroom(
    p: &mut ChildSpawnerCommands,
    s: &Session,
    spaces: &Spaces,
    server: &AssetServer,
    art: &SpaceArt,
) {
    let st = &s.save.state;
    let room = &st.showroom;
    let rack = spaces.rack.min(room.racks.len().saturating_sub(1));
    let range = room.rack_range(rack);
    let used = room.capacity() - room.free();
    // Inventory window, bottom-left.
    p.spawn((
        art.frame("showroom-window"),
        Node {
            position_type: PositionType::Absolute,
            left: px(36),
            bottom: px(42),
            width: px(560),
            height: px(262),
            padding: UiRect::new(px(26), px(26), px(22), px(18)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::clip(),
            ..default()
        },
    ))
    .with_children(|win| {
        win.spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|tabs| {
            for i in 0..room.config.max_racks as usize {
                if i < room.racks.len() {
                    let e = forge_ui::button(
                        tabs,
                        format!(
                            "Display {} · {} slots",
                            (b'A' + i as u8) as char,
                            room.rack_range(i).len()
                        ),
                        Action::Rack(i),
                    );
                    if i == rack {
                        tabs.commands().entity(e).insert(forge_ui::Selected);
                    }
                } else {
                    let e = forge_ui::button(
                        tabs,
                        format!("Display {}", (b'A' + i as u8) as char),
                        Action::Rack(i),
                    );
                    tabs.commands().entity(e).insert((
                        bevy::ui::InteractionDisabled,
                        forge_ui::Unavailable(format!(
                            "Not installed. Buy a display at the carpenter ({} gold, {} timber).",
                            room.config.rack_gold, room.config.rack_timber
                        )),
                    ));
                }
            }
            tabs.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            forge_ui::button(tabs, "Manage displays", Action::Carpenter);
        });
        win.spawn((
            Node {
                height: px(1),
                width: percent(100),
                ..default()
            },
            BackgroundColor(forge_ui::EDGE),
        ));
        win.spawn(Node {
            column_gap: px(14),
            align_items: AlignItems::Baseline,
            ..default()
        })
        .with_children(|row| {
            row.spawn(forge_ui::text("Showroom", 20.));
            row.spawn(forge_ui::text(
                format!("{used} / {} occupied", room.capacity()),
                14.,
            ));
        });
        win.spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        })
        .with_children(|grid| {
            for index in range.clone() {
                let slot = &room.slots[index];
                let (icon, badge) = slot_badge(slot);
                forge_ui::item_slot(
                    grid,
                    server,
                    icon,
                    badge,
                    spaces.slot == Some(index),
                    (
                        Action::Slot(index),
                        forge_ui::FocusKey(format!("showroom-slot:{index}")),
                        forge_ui::Help(slot.label().into()),
                    ),
                );
            }
        });
        win.spawn(forge_ui::text(
            if room.overflow > 0 {
                format!(
                    "1 item = 1 slot  |  {} older items wait beside the displays and sell first.",
                    room.overflow
                )
            } else {
                "1 item = 1 slot  |  Capacity includes reserved items.".into()
            },
            12.,
        ));
    });
    // Detail panel, right.
    let selected = spaces.slot.and_then(|i| room.slots.get(i).map(|s| (i, *s)));
    p.spawn((
        art.frame("detail-panel"),
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(0),
            bottom: px(0),
            width: px(236),
            padding: UiRect::new(px(24), px(24), px(26), px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            overflow: Overflow::clip(),
            ..default()
        },
    ))
    .with_children(|panel| {
        let (title, line, icon) = match selected {
            Some((_, Slot::Displayed(g))) => (g.label(), "Ready for sale", Some(g.icon())),
            Some((_, Slot::Held(g))) => (g.label(), "Kept; visitors cannot buy it", Some(g.icon())),
            Some((_, Slot::Reserved)) => {
                ("Reserved", "Result of the job in progress", Some("queue"))
            }
            Some((_, Slot::Empty)) => ("Empty slot", "The next job can start here", None),
            None => ("Showroom", "Select a slot to inspect it", None),
        };
        panel.spawn(forge_ui::text(title, 22.));
        panel.spawn(forge_ui::text(line, 13.));
        panel
            .spawn((
                Node {
                    width: px(112),
                    height: px(112),
                    align_self: AlignSelf::Center,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(2)),
                    margin: UiRect::vertical(px(6)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.07, 0.06)),
                BorderColor::all(Color::srgb(0.42, 0.42, 0.46)),
            ))
            .with_children(|frame| {
                if let Some(icon) = icon {
                    forge_ui::icon(frame, server, icon, 72.);
                }
            });
        panel.spawn((
            Node {
                height: px(1),
                width: percent(100),
                ..default()
            },
            BackgroundColor(forge_ui::EDGE),
        ));
        match selected {
            Some((i, Slot::Displayed(g) | Slot::Held(g))) => {
                let held = matches!(selected, Some((_, Slot::Held(_))));
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(forge_ui::text("Sale price", 13.));
                        row.spawn(forge_ui::text(format!("{} G", g.price()), 16.));
                    });
                panel
                    .spawn(Node {
                        column_gap: px(8),
                        ..default()
                    })
                    .with_children(|row| {
                        let sale = forge_ui::button(row, "For sale", Action::ForSale(i, true));
                        let hold = forge_ui::button(row, "Hold", Action::ForSale(i, false));
                        row.commands()
                            .entity(if held { hold } else { sale })
                            .insert(forge_ui::Selected);
                    });
            }
            Some((_, Slot::Reserved)) => {
                panel.spawn(forge_ui::text(
                    "Cannot be changed until the job completes.",
                    12.,
                ));
            }
            _ => {}
        }
        panel.spawn(Node {
            flex_grow: 1.,
            ..default()
        });
        panel.spawn((
            Node {
                height: px(1),
                width: percent(100),
                ..default()
            },
            BackgroundColor(forge_ui::EDGE),
        ));
        forge_ui::button(panel, "Manage displays", Action::Carpenter);
        panel.spawn((
            forge_ui::text(
                format!(
                    "Expand at Carpenter · {} / {} displays",
                    room.racks.len(),
                    room.config.max_racks
                ),
                11.,
            ),
            Node {
                align_self: AlignSelf::Center,
                ..default()
            },
        ));
    });
    // Quiet status chip at the bottom centre.
    let reserved = room.reserved();
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(4),
            left: px(300),
            padding: UiRect::axes(px(14), px(6)),
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.04, 0.03, 0.85)),
    ))
    .with_children(|chip| {
        forge_ui::icon(chip, server, "craft", 16.);
        chip.spawn(forge_ui::text(
            match st.activity() {
                Activity::Working { .. } => format!("Next craft: {reserved} slot reserved"),
                Activity::Waiting(reason) => reason.label().to_string(),
                _ => format!("Nothing in progress · {} slots free", room.free()),
            },
            12.,
        ));
    });
}
#[allow(clippy::too_many_arguments)]
pub fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    mut spaces: ResMut<Spaces>,
    mut desktop: ResMut<Desktop>,
    mut village: ResMut<Village>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
    mut host: ResMut<crate::forge_view::Host>,
) {
    if title.active || dialog.is_open() {
        return;
    }
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    if matches!(action, Action::Station(_)) {
        host.legacy = true;
        desktop.shell.expanded = true;
        desktop.shell.recover();
        spaces.manage = false;
    }
    apply(
        *action,
        &mut spaces,
        &mut desktop,
        &mut village,
        &mut s,
        &opts,
    );
}
pub fn apply(
    action: Action,
    spaces: &mut Spaces,
    desktop: &mut Desktop,
    village: &mut Village,
    s: &mut Session,
    opts: &Options,
) {
    let home = village.nav.screen == Screen::Home;
    match action {
        Action::Enter => {
            if !desktop.shell.expanded {
                desktop.shell.recover();
                desktop.shell.expanded = true;
                village.nav = forge_core::village::Navigation::default();
                if !spaces.space.interior() {
                    spaces.enter(Space::Workshop);
                }
            }
        }
        Action::Go(space) if desktop.shell.expanded && home => spaces.enter(space),
        Action::Station(station) if desktop.shell.expanded && home && !spaces.manage => {
            spaces.station = if spaces.station == Some(station) {
                None
            } else {
                Some(station)
            };
            spaces.manage = false;
        }
        Action::Manage if desktop.shell.expanded && home => {
            spaces.manage = true;
            spaces.station = None;
        }
        Action::CloseDrawer => {
            spaces.station = None;
            spaces.manage = false;
        }
        Action::Slot(i) if desktop.shell.expanded && home => {
            spaces.slot = if spaces.slot == Some(i) {
                None
            } else {
                Some(i)
            };
        }
        Action::Rack(i) if desktop.shell.expanded && home => {
            if i < s.save.state.showroom.racks.len() {
                spaces.rack = i;
                spaces.slot = None;
            }
        }
        Action::ForSale(slot, for_sale) if desktop.shell.expanded && home => {
            if s.disabled {
                return;
            }
            let id = s.save.state.command_id + 1;
            s.status = match s
                .save
                .state
                .command(id, Command::SetForSale { slot, for_sale })
            {
                Ok(()) => {
                    if for_sale {
                        "Displayed for sale".into()
                    } else {
                        "Kept; visitors will not buy it".into()
                    }
                }
                Err(e) => format!("{e:?}"),
            };
            crate::persist(s, opts);
        }
        Action::Carpenter if desktop.shell.expanded && home => {
            village.nav.map();
            village.nav.visit(forge_core::village::Location::Carpenter);
            village.redraw = true;
        }
        _ => return,
    }
    spaces.redraw = true;
}
/// Keyboard: 1/2/3 choose a space at home; Escape closes a drawer before collapsing.
#[allow(clippy::too_many_arguments)]
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut spaces: ResMut<Spaces>,
    mut desktop: ResMut<Desktop>,
    mut village: ResMut<Village>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
) {
    if title.active || dialog.is_open() {
        return;
    }
    if !desktop.shell.expanded || village.nav.screen != Screen::Home {
        return;
    }
    for (key, space) in [
        (KeyCode::Digit1, Space::Exterior),
        (KeyCode::Digit2, Space::Workshop),
        (KeyCode::Digit3, Space::Showroom),
    ] {
        if keys.just_pressed(key) {
            apply(
                Action::Go(space),
                &mut spaces,
                &mut desktop,
                &mut village,
                &mut s,
                &opts,
            );
        }
    }
}
/// Escape inside the workshop drawer closes the drawer instead of the whole overlay.
pub fn drawer_open(spaces: &Spaces) -> bool {
    spaces.space == Space::Workshop && (spaces.station.is_some() || spaces.manage)
}
/// At home the overlay shell is see-through so the forge scenes show beneath the navigation.
pub fn shell_transparency(
    village: Res<Village>,
    mut roots: Query<(&mut BackgroundColor, Option<&mut ImageNode>), With<crate::village::Root>>,
) {
    let home = false;
    let _ = &village;
    for (mut bg, image) in &mut roots {
        let next = if home { Color::NONE } else { forge_ui::SURFACE };
        if bg.0 != next {
            bg.0 = next;
        }
        if let Some(mut image) = image {
            let next = if home { Color::NONE } else { Color::WHITE };
            if image.color != next {
                image.color = next;
            }
        }
    }
}
pub fn plugin(app: &mut App) {
    app.init_resource::<Spaces>()
        .insert_resource(SpaceArt::bundled())
        .add_observer(activate)
        .add_systems(Startup, setup.after(crate::setup))
        .add_systems(
            Update,
            (
                route,
                sync_character,
                drive_workshop,
                drive_presence,
                input,
                draw,
                shell_transparency,
            )
                .chain()
                .before(forge_npc::player::walk_actors),
        );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_hundred_window_round_trips_preserve_work_and_actor_phase() {
        let dir = std::env::temp_dir().join(format!("d01-continuity-{}", std::process::id()));
        let mut state = forge_core::State::new(42);
        state.progression.legacy_dagger = true;
        state
            .command(1, Command::ChooseSmith(forge_core::Smith::Female))
            .unwrap();
        state.command(2, Command::Craft).unwrap();
        let before = state.clone();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Session {
                save: forge_core::save::Save::new(state, 100),
                dir: dir.clone(),
                _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
                last: std::time::Instant::now(),
                fraction: 0.,
                status: String::new(),
                disabled: false,
                trace: vec![],
            })
            .insert_resource(Spaces::default())
            .insert_resource(Village::default())
            .insert_resource(TitleScreen {
                active: false,
                ..default()
            })
            .insert_resource(Options::default())
            .insert_resource(crate::forge_view::Layout::default())
            .add_systems(Update, route);
        let window = app
            .world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow))
            .id();
        let actor = app
            .world_mut()
            .spawn((
                SceneOf(Space::Workshop),
                WorkshopActor,
                Transform::default(),
                Visibility::Hidden,
                Actor {
                    clip: "hammering".into(),
                    elapsed: 0.375,
                    ..default()
                },
                Walker {
                    pos: Vec2::new(576., 156.),
                    target: Vec2::new(576., 156.),
                    ..default()
                },
            ))
            .id();
        for _ in 0..100 {
            for height in [540., 180.] {
                app.world_mut()
                    .get_mut::<Window>(window)
                    .unwrap()
                    .resolution
                    .set(960., height);
                app.insert_resource(crate::forge_view::Layout::calculate(
                    Vec2::new(960., height),
                    UVec2::new(960, height as u32),
                    1.,
                ));
                app.update();
                let a = app.world().get::<Actor>(actor).unwrap();
                assert_eq!(a.clip, "hammering");
                assert_eq!(a.elapsed, 0.375);
                assert_eq!(
                    app.world().get::<Visibility>(actor),
                    Some(&Visibility::Visible)
                );
                assert_eq!(app.world().resource::<Session>().save.state, before);
                assert_eq!(app.world().resource::<Spaces>().scale, 1.);
                assert_eq!(
                    app.world().resource::<Spaces>().view_origin.y,
                    height - 180.
                );
            }
        }
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<WorkshopActor>>()
                .iter(app.world())
                .count(),
            1
        );
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn space_actions_need_an_expanded_home_and_toggle_cleanly() {
        let dir = std::env::temp_dir().join(format!("forge-spaces-{}", std::process::id()));
        let mut state = forge_core::State::new(9);
        state.progression.legacy_dagger = true;
        state
            .command(1, Command::ChooseSmith(forge_core::Smith::Female))
            .unwrap();
        state.command(2, Command::Craft).unwrap();
        state.advance(6).unwrap();
        let mut s = Session {
            save: forge_core::save::Save::new(state, 100),
            dir: dir.clone(),
            _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
            last: std::time::Instant::now(),
            fraction: 0.,
            status: String::new(),
            disabled: false,
            trace: vec![],
        };
        let opts = Options {
            qa: Some("forge-workshop".into()),
            ..default()
        };
        let mut spaces = Spaces::default();
        let mut desktop = Desktop::new(false, false);
        let mut village = Village::default();
        // Collapsed: nothing but Enter does anything.
        apply(
            Action::Go(Space::Showroom),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(spaces.space, Space::Workshop);
        apply(
            Action::Enter,
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert!(desktop.shell.expanded);
        assert_eq!(spaces.space, Space::Workshop);
        apply(
            Action::Station(Workstation::Anvil),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(spaces.station, Some(Workstation::Anvil));
        assert!(drawer_open(&spaces));
        apply(
            Action::Station(Workstation::Anvil),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(spaces.station, None);
        spaces.manage = true;
        let before = s.save.state.clone();
        apply(
            Action::Station(Workstation::SalvageTable),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert!(spaces.manage);
        assert_eq!(spaces.station, None);
        assert_eq!(s.save.state, before);
        apply(
            Action::Go(Space::Showroom),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        apply(
            Action::Slot(0),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(spaces.slot, Some(0));
        // Keeping the finished dagger removes it from sale without touching gold or count.
        let gold = s.save.state.gold;
        apply(
            Action::ForSale(0, false),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(s.save.state.showroom.held(), 1);
        assert_eq!(s.save.state.sellable(), 0);
        assert_eq!(s.save.state.gold, gold);
        assert_eq!(s.save.state.command_id, 3);
        // A reserved slot cannot be changed; the state is untouched.
        s.save.state.command(4, Command::Craft).unwrap();
        let before = s.save.state.clone();
        apply(
            Action::ForSale(1, false),
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(s.save.state, before);
        apply(
            Action::Carpenter,
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut s,
            &opts,
        );
        assert_eq!(
            village.nav.screen,
            Screen::Destination(forge_core::village::Location::Carpenter)
        );
        drop(s);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
