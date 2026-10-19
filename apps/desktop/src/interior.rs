//! Station geometry is independent of the camera and of canonical character artwork.
use crate::{
    spaces::{Spaces, WorkshopActor},
    title::TitleScreen,
};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use forge_core::{
    spaces::{Space, Workstation},
    village::Screen,
};
use forge_npc::player::{Actor, CharacterId, PlayerCatalog, PlayerCatalogs};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const CHARACTER_SCALE: f32 = 0.75;
pub fn depth(foot_y: f32) -> f32 {
    1. + foot_y * 0.001
}

#[derive(Clone, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub origin: [f32; 2],
    pub size: [u32; 2],
    pub contact: [f32; 2],
    pub depth: f32,
    pub mode_contacts: BTreeMap<String, [f32; 2]>,
    pub hit_mask: Vec<String>,
}
impl Fixture {
    pub fn hit(&self, point: Vec2) -> bool {
        let local = point - Vec2::from_array(self.origin);
        if !local.is_finite()
            || local.x < 0.
            || local.y < 0.
            || local.x >= self.size[0] as f32
            || local.y >= self.size[1] as f32
        {
            return false;
        }
        self.hit_mask
            .get(local.y.floor() as usize)
            .and_then(|row| row.as_bytes().get(local.x.floor() as usize))
            == Some(&b'1')
    }
}
#[derive(Resource)]
pub struct Interior(pub Vec<Fixture>, forge_npc::stations::StationCatalog);
impl Default for Interior {
    fn default() -> Self {
        Self(
            serde_json::from_str(include_str!("../../../assets/forge/atlas-v1/catalog.json"))
                .expect("validated interior exports"),
            forge_npc::stations::StationCatalog::bundled().expect("station contacts"),
        )
    }
}
impl Interior {
    pub fn fixture(&self, station: &str) -> Option<&Fixture> {
        let id = match station {
            "research-table" | "salvage-table" => "workbench",
            _ => station,
        };
        self.0.iter().find(|f| f.id == id)
    }
    pub fn foot(&self, station: &str, catalog: &PlayerCatalog) -> Vec2 {
        let Some(f) = self.fixture(station) else {
            return crate::forge_view::REST;
        };
        let Some(reg) = catalog.stations.get(station) else {
            return crate::forge_view::REST;
        };
        let contact = self.1.stations[station].contact;
        let mode = if station == "research-table" {
            "researching"
        } else {
            "salvaging"
        };
        let target = f.mode_contacts.get(mode).unwrap_or(&f.contact);
        (Vec2::from_array(f.origin) + Vec2::from_array(*target)
            - (reg.offset.as_vec2() + UVec2::from_array(contact).as_vec2()) * CHARACTER_SCALE)
            .round()
    }
}
#[derive(Component)]
pub(crate) struct FixturePart {
    index: usize,
    front: bool,
    outline: bool,
}
#[derive(Component)]
pub(crate) struct ResearchArms;
#[derive(Resource, Default)]
struct Hover(Option<usize>);

#[derive(Deserialize)]
struct SmithFrames {
    columns: u32,
    rows: u32,
    ids: Vec<String>,
}
struct SmithPage {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    ids: BTreeMap<String, usize>,
}
#[derive(Resource, Default)]
struct ScaledSmiths(BTreeMap<String, SmithPage>);

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    art: Res<Interior>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut pages = BTreeMap::new();
    for (sex, raw) in [
        (
            "female",
            include_str!("../../../assets/forge/atlas-v1/female-75.json"),
        ),
        (
            "male",
            include_str!("../../../assets/forge/atlas-v1/male-75.json"),
        ),
    ] {
        let spec: SmithFrames = serde_json::from_str(raw).expect("validated scaled atlas");
        pages.insert(
            sex.into(),
            SmithPage {
                image: server.load(format!("forge/atlas-v1/{sex}-75.png")),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::new(96, 120),
                    spec.columns,
                    spec.rows,
                    None,
                    None,
                )),
                ids: spec
                    .ids
                    .into_iter()
                    .enumerate()
                    .map(|(i, id)| (id, i))
                    .collect(),
            },
        );
    }
    commands.insert_resource(ScaledSmiths(pages));
    for (index, f) in art.0.iter().enumerate() {
        for (kind, front) in [("back", false), ("front", true)] {
            for outline in [false, true] {
                let suffix = if outline { "-outline" } else { "" };
                commands.spawn((
                    FixturePart {
                        index,
                        front,
                        outline,
                    },
                    Sprite::from_image(
                        server.load(format!("forge/atlas-v1/{}-{kind}{suffix}.png", f.id)),
                    ),
                    bevy::sprite::Anchor::TOP_LEFT,
                    Transform::from_xyz(f.origin[0] - 480., 90. - f.origin[1], 1.2),
                    Visibility::Hidden,
                    RenderLayers::layer(1),
                ));
            }
        }
    }
    commands.spawn((
        ResearchArms,
        Sprite::default(),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::default(),
        Visibility::Hidden,
        RenderLayers::layer(1),
    ));
}

#[allow(clippy::too_many_arguments)]
fn pointer(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    art: Res<Interior>,
    mut hover: ResMut<Hover>,
    mut spaces: ResMut<Spaces>,
    layout: Res<crate::forge_view::Layout>,
    title: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
    modal_frame: Res<crate::forge_view::ModalFrame>,
    mut village: ResMut<crate::village::Village>,
    mut host: ResMut<crate::forge_view::Host>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut session: ResMut<crate::Session>,
    opts: Res<crate::Options>,
) {
    hover.0 = None;
    if title.active
        || modal_frame.0
        || dialog.is_open()
        || spaces.space == Space::Exterior
        || village.nav.screen != Screen::Home
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let point = (cursor - layout.origin) / spaces.scale;
    if !(0. ..180.).contains(&point.y) {
        return;
    }
    hover.0 = art.0.iter().rposition(|f| f.hit(point));
    if buttons.just_pressed(MouseButton::Left)
        && let Some(i) = hover.0
    {
        let id = &art.0[i].id;
        desktop.shell.expanded = true;
        desktop.shell.recover();
        host.legacy = true;
        spaces.manage = false;
        if id == "display" {
            spaces.enter(Space::Showroom);
        } else {
            spaces.enter(Space::Workshop);
            if let Some(station) = Workstation::from_id(if id == "workbench" {
                "research-table"
            } else {
                id
            }) {
                crate::spaces::apply(
                    crate::spaces::Action::Station(station),
                    &mut spaces,
                    &mut desktop,
                    &mut village,
                    &mut session,
                    &opts,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn project(
    art: Res<Interior>,
    spaces: Res<Spaces>,
    title: Res<TitleScreen>,
    hover: Res<Hover>,
    server: Res<AssetServer>,
    catalogs: Res<PlayerCatalogs>,
    actor: Query<
        (&Actor, &CharacterId, &Transform),
        (
            With<WorkshopActor>,
            Without<FixturePart>,
            Without<ResearchArms>,
        ),
    >,
    mut parts: Query<
        (&FixturePart, &mut Sprite, &mut Transform, &mut Visibility),
        Without<ResearchArms>,
    >,
    mut arms: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (
            With<ResearchArms>,
            Without<FixturePart>,
            Without<WorkshopActor>,
        ),
    >,
) {
    let visible = !title.active && spaces.space != Space::Exterior;
    let Ok((actor, id, actor_transform)) = actor.single() else {
        return;
    };
    let clip = actor.clip.strip_prefix("base/").unwrap_or(&actor.clip);
    let working = match clip {
        "forging" => "forge",
        "hammering" => "anvil",
        "quenching" => "quenching-tub",
        "researching" => "workbench",
        _ => "",
    };
    for (part, mut sprite, mut t, mut v) in &mut parts {
        let f = &art.0[part.index];
        // Travel compares the foot with the fixture floor. Authored work poses use
        // their layer contract; outlines stay in the same depth group as the prop.
        let working_here = working == f.id;
        let foreground = working_here && (part.front || f.id == "workbench");
        let z = if foreground {
            actor_transform.translation.z + 0.02
        } else if working_here {
            actor_transform.translation.z - 0.02
        } else {
            depth(f.depth) - 0.0005
        };
        t.translation.z = z + if part.outline { 0.0001 } else { 0. };
        let selected = spaces
            .station
            .is_some_and(|s| art.fixture(s.id()).is_some_and(|a| a.id == f.id))
            || (f.id == "display" && spaces.space == Space::Showroom);
        let hovered = hover.0 == Some(part.index);
        *v = if visible && (!part.outline || selected || hovered) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if part.outline {
            sprite.color = if selected {
                Color::srgb(0.88, 0.61, 0.24)
            } else {
                Color::srgb(0.56, 0.43, 0.25)
            };
        }
    }
    for (mut sprite, mut t, mut v) in &mut arms {
        *v = Visibility::Hidden;
        if visible
            && clip == "researching"
            && let Some(c) = catalogs.get(&id.0)
        {
            let frame = actor
                .sprite(c)
                .id
                .rsplit('/')
                .next()
                .unwrap_or("researching-0");
            let variant = if actor.prefix.is_empty() {
                "equipped"
            } else {
                "base"
            };
            sprite.image = server.load(format!(
                "forge/atlas-v1/arms/{}-{variant}-{frame}.png",
                id.0
            ));
            t.translation = actor_transform.translation + Vec3::new(-48., 105., 0.04);
            *v = Visibility::Visible;
        }
    }
}

// Playback remains canonical. Only its presentation uses whole-canvas nearest exports,
// avoiding a different sampling phase when the source atlas trim changes between frames.
#[allow(clippy::type_complexity)]
fn scaled_smith(
    pages: Res<ScaledSmiths>,
    catalogs: Res<PlayerCatalogs>,
    mut q: Query<
        (
            &Actor,
            &CharacterId,
            &forge_npc::player::Walker,
            &mut Sprite,
            &mut bevy::sprite::Anchor,
            &mut Transform,
        ),
        With<WorkshopActor>,
    >,
) {
    for (actor, id, walker, mut sprite, mut anchor, mut transform) in &mut q {
        let Some(catalog) = catalogs.get(&id.0) else {
            continue;
        };
        let Some(page) = pages.0.get(&id.0) else {
            continue;
        };
        let frame = actor.sprite(catalog);
        let Some(index) = page.ids.get(&frame.id) else {
            continue;
        };
        sprite.image = page.image.clone();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: page.layout.clone(),
            index: *index,
        });
        sprite.custom_size = Some(Vec2::new(96., 120.));
        anchor.0 = Vec2::new(0., -0.375);
        transform.scale = Vec3::ONE;
        transform.translation = Vec3::new(
            walker.pos.x.round() - 480.,
            90. - walker.pos.y.round(),
            depth(walker.pos.y),
        );
    }
}

pub fn install(app: &mut App) {
    app.init_resource::<Interior>()
        .init_resource::<Hover>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            pointer
                .after(crate::spaces::input)
                .before(crate::spaces::draw),
        )
        .add_systems(
            Update,
            pose_qa
                .after(crate::spaces::drive_workshop)
                .before(forge_npc::player::walk_actors),
        )
        .add_systems(
            Update,
            project
                .after(scaled_smith)
                .after(crate::spaces::drive_workshop),
        )
        .add_systems(
            Update,
            scaled_smith
                .after(forge_npc::player::animate_actors)
                .after(crate::spaces::drive_workshop),
        );
}

// Isolated pose evidence only. This never claims to execute research or salvage commands.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn pose_qa(
    mut commands: Commands,
    opts: Res<crate::Options>,
    art: Res<Interior>,
    catalogs: Res<PlayerCatalogs>,
    mut spaces: ResMut<Spaces>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut q: Query<
        (
            &mut Actor,
            &CharacterId,
            &mut forge_npc::player::Walker,
            &mut Transform,
            &mut Visibility,
        ),
        With<WorkshopActor>,
    >,
) {
    if !opts
        .forge_scene()
        .is_some_and(|(scene, _)| scene == "interior")
    {
        return;
    }
    let Ok(pose) = std::env::var("LITTLE_FORGE_INTERIOR_POSE") else {
        return;
    };
    let station = match pose.as_str() {
        "forging" => "forge",
        "hammering" => "anvil",
        "quenching" => "quenching-tub",
        "researching" => "research-table",
        "salvaging" | "overlap" => "salvage-table",
        _ => "",
    };
    spaces.space = Space::Workshop;
    spaces.station = Workstation::from_id(station);
    desktop.shell.expanded = std::env::var_os("LITTLE_FORGE_INTERIOR_EXPANDED").is_some();
    for (mut actor, id, mut walker, mut t, mut visibility) in &mut q {
        let Some(c) = catalogs.get(&id.0) else {
            continue;
        };
        let walking = pose.starts_with("walk-");
        let clip_name = if walking {
            "walk"
        } else if pose == "overlap" || pose == "empty" {
            "idle"
        } else {
            &pose
        };
        actor.play(clip_name, c);
        if pose == "empty" {
            *visibility = Visibility::Hidden;
        }
        actor.paused = true;
        let frame = ((opts.frames.saturating_sub(30)) / 20).min(5) as usize;
        if let Some(clip) = c.clips.get(clip_name) {
            let time = clip
                .frames
                .iter()
                .take(frame)
                .map(|f| f.duration)
                .sum::<f32>()
                + 0.001;
            actor.seek(time, c);
        }
        walker.pos = art.foot(station, c);
        if walking {
            let x = 520. + (opts.frames.saturating_sub(30) as f32 * 1.4).min(210.);
            walker.pos = Vec2::new(x, if pose.ends_with("behind") { 148. } else { 168. });
            spaces.station = Some(Workstation::Anvil);
        }
        walker.target = walker.pos;
        walker.arrival = None;
        t.translation = Vec3::new(walker.pos.x - 480., 90. - walker.pos.y, 1.5);
    }
    if [35, 55, 75, 95, 115, 135].contains(&opts.frames)
        && let Some(path) = &opts.capture
    {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(crate::save_capture(format!(
                "{}-pose-{}.png",
                path.trim_end_matches(".png"),
                opts.frames
            )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scaled_pages_cover_every_canonical_frame_and_travel_crosses_prop_depth() {
        for (sex, raw) in [
            (
                "female",
                include_str!("../../../assets/forge/atlas-v1/female-75.json"),
            ),
            (
                "male",
                include_str!("../../../assets/forge/atlas-v1/male-75.json"),
            ),
        ] {
            let page: SmithFrames = serde_json::from_str(raw).unwrap();
            let catalog = PlayerCatalog::bundled(sex).unwrap();
            for frame in catalog.sprites {
                assert!(page.ids.contains(&frame.id), "missing {} {}", sex, frame.id);
            }
        }
        let art = Interior::default();
        for id in ["anvil", "quenching-tub"] {
            let floor = art.fixture(id).unwrap().depth;
            assert!(depth(148.) < depth(floor));
            assert!(depth(168.) > depth(floor));
        }
    }
    #[test]
    fn legacy_animation_cannot_move_or_hide_fixture_parts() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(crate::Options::default())
            .insert_resource(crate::Art {
                images: vec![],
                visitor_sizes: vec![],
                visitor_hands: vec![],
                building_active: Handle::default(),
                building_idle: Handle::default(),
            })
            .insert_resource(crate::Presentation::default())
            .add_systems(Update, crate::animate);
        let transform = Transform::from_xyz(-96., 83., 1.2);
        let e = app
            .world_mut()
            .spawn((
                FixturePart {
                    index: 0,
                    front: false,
                    outline: false,
                },
                Sprite::default(),
                transform,
                Visibility::Visible,
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Transform>(e), Some(&transform));
        assert_eq!(app.world().get::<Visibility>(e), Some(&Visibility::Visible));
    }
    #[test]
    fn contacts_reuse_both_canonical_registrations_and_one_desk() {
        let art = Interior::default();
        assert!(std::ptr::eq(
            art.fixture("research-table").unwrap(),
            art.fixture("salvage-table").unwrap()
        ));
        for sex in ["female", "male"] {
            let c = PlayerCatalog::bundled(sex).unwrap();
            let old = forge_npc::stations::StationCatalog::bundled().unwrap();
            for (id, reg) in &c.stations {
                let f = art.fixture(id).unwrap();
                let mode = if id == "research-table" {
                    "researching"
                } else {
                    "salvaging"
                };
                let target = f.mode_contacts.get(mode).unwrap_or(&f.contact);
                let error = art.foot(id, &c)
                    + (reg.offset.as_vec2()
                        + UVec2::from_array(old.stations[id].contact).as_vec2())
                        * CHARACTER_SCALE
                    - Vec2::from_array(f.origin)
                    - Vec2::from_array(*target);
                assert!(error.abs().max_element() <= 0.5);
            }
        }
    }
    #[test]
    fn alpha_hit_rejects_padding_and_negative_coordinates() {
        let f = Fixture {
            id: "test".into(),
            origin: [-5., -3.],
            size: [3, 2],
            contact: [1., 1.],
            depth: 0.,
            mode_contacts: BTreeMap::new(),
            hit_mask: vec!["000".into(), "010".into()],
        };
        assert!(!f.hit(Vec2::new(-5., -3.)));
        assert!(f.hit(Vec2::new(-4., -2.)));
        assert!(!f.hit(Vec2::new(-5.1, -2.)));
        assert!(!f.hit(Vec2::new(f32::NAN, 0.)));
    }
}
