//! Player smith stories: the same `forge_npc::player` component and station floor plan the game
//! uses, with no simulation, save or economy. `npc-preview --character female|male`.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui_widgets::{Activate, Button},
};
use forge_npc::{
    player::{Actor, CharacterId, PLAYER_CLIPS, PlayerCatalogs, PlayerPlugin, Walker},
    stations::{self, StationCatalog, StationLayer},
};

#[derive(Resource)]
pub struct PlayerPreview {
    pub character: String,
    pub story: String,
    pub capture: Option<String>,
    pub sample: f32,
    pub sequence: bool,
    pub left: bool,
    pub base: bool,
    pub ticks: u32,
    pub background: usize,
    pub stations: bool,
    pub guides: bool,
    pub walk: bool,
}
#[derive(Component)]
struct Smith;
#[derive(Component)]
struct Label;
#[derive(Component)]
struct Story(usize);
#[derive(Component)]
struct Backdrop;
#[derive(Component)]
struct Guide;
const SCALE: f32 = 2.;

pub fn run(root: std::path::PathBuf, args: &[String]) {
    let value = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let character = value("--character").unwrap_or_else(|| "male".into());
    assert!(
        forge_npc::player::CHARACTERS.contains(&character.as_str()),
        "--character female|male"
    );
    let story = value("--story").unwrap_or_else(|| "idle".into());
    assert!(
        PLAYER_CLIPS.contains(&story.as_str()) || story == "walk-to-anvil",
        "unknown player story {story}"
    );
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(crate::rendering::plugin(true))
            .set(AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Little Forge | Player Preview".into(),
                    resolution: (1100, 800).into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(PlayerPlugin)
    .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.12)))
    .insert_resource(PlayerPreview {
        character,
        capture: value("--capture"),
        sample: value("--at").and_then(|v| v.parse().ok()).unwrap_or(0.3),
        sequence: args.iter().any(|a| a == "--sequence"),
        left: args.iter().any(|a| a == "--left"),
        base: args.iter().any(|a| a == "--base"),
        ticks: 0,
        background: if args.iter().any(|a| a == "--light") {
            1
        } else {
            0
        },
        stations: !args.iter().any(|a| a == "--no-stations"),
        guides: false,
        walk: story == "walk-to-anvil",
        story,
    })
    .add_observer(select_story)
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (controls, drive)
            .chain()
            .before(forge_npc::player::walk_actors),
    );
    crate::rendering::install_sprite_shader(&mut app);
    app.run();
}
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    catalogs: Res<PlayerCatalogs>,
    p: Res<PlayerPreview>,
) {
    commands.spawn(Camera2d);
    let origin = Vec2::new(-stations::SCENE_SIZE.x, stations::SCENE_SIZE.y) * SCALE / 2.;
    commands.spawn((
        Sprite {
            image: server.load("forge/spaces/workshop-backdrop.png"),
            custom_size: Some(stations::SCENE_SIZE),
            ..default()
        },
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(origin.x, origin.y, 0.5).with_scale(Vec3::splat(SCALE)),
        Backdrop,
    ));
    let stations_art = StationCatalog::bundled().expect("validated station catalog");
    if let Some(c) = catalogs.get(&p.character) {
        for placement in stations::placements(c) {
            stations::spawn_station(
                &mut commands,
                &server,
                &stations_art,
                placement,
                SCALE,
                origin,
            );
        }
    }
    let spot = stations::REST_SPOT.foot;
    commands.spawn((
        Sprite::default(),
        bevy::sprite::Anchor::default(),
        Transform::from_xyz(0., 0., stations::actor_z(1)).with_scale(Vec3::splat(SCALE)),
        Actor {
            facing_left: p.left,
            prefix: if p.base {
                "base/".into()
            } else {
                String::new()
            },
            ..default()
        },
        Walker {
            pos: spot,
            target: spot,
            ..default()
        },
        CharacterId(p.character.clone()),
        Smith,
    ));
    for _ in 0..3 {
        commands.spawn((
            Sprite::from_color(Color::srgb(1., 0.3, 0.2), Vec2::splat(6.)),
            Transform::from_xyz(0., 0., 20.),
            Visibility::Hidden,
            Guide,
        ));
    }
    commands.spawn((
        Text::new("Loading player stories…"),
        TextFont {
            font_size: 17.0.into(),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(12),
            ..default()
        },
        Label,
    ));
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: px(12),
            top: px(12),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        })
        .with_children(|parent| {
            for (i, name) in PLAYER_CLIPS
                .iter()
                .chain(["walk-to-anvil"].iter())
                .enumerate()
            {
                parent
                    .spawn((
                        Button,
                        Story(i),
                        Node {
                            padding: UiRect::axes(px(10), px(3)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.23, 0.23, 0.9)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(*name),
                            TextFont {
                                font_size: 13.0.into(),
                                ..default()
                            },
                        ));
                    });
            }
        });
}
fn story_name(i: usize) -> &'static str {
    PLAYER_CLIPS.get(i).copied().unwrap_or("walk-to-anvil")
}
fn select_story(event: On<Activate>, stories: Query<&Story>, mut p: ResMut<PlayerPreview>) {
    if let Ok(story) = stories.get(event.entity) {
        p.story = story_name(story.0).into();
        p.walk = p.story == "walk-to-anvil";
        p.ticks = 0;
    }
}
/// Place the smith for the current story: at the clip's station (held) or resting.
fn drive(
    catalogs: Option<Res<PlayerCatalogs>>,
    mut p: ResMut<PlayerPreview>,
    mut q: Query<(&mut Actor, &mut Walker, &mut Transform, &CharacterId), With<Smith>>,
    mut last: Local<Option<(String, bool, bool)>>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    let Ok((mut actor, mut walker, mut t, id)) = q.single_mut() else {
        return;
    };
    let Some(c) = catalogs.get(&id.0) else {
        return;
    };
    let key = (p.story.clone(), p.base, p.left);
    if *last != Some(key.clone()) {
        *last = Some(key);
        actor.prefix = if p.base {
            "base/".into()
        } else {
            String::new()
        };
        actor.facing_left = p.left;
        if p.walk {
            walker.pos = stations::REST_SPOT.foot;
            walker.go(stations::spot("anvil").unwrap().foot, "hammering", true);
        } else {
            let clip = c.clips.get(&p.story);
            let spot = clip
                .and_then(|k| k.station.as_deref())
                .and_then(stations::spot)
                .unwrap_or(stations::REST_SPOT);
            walker.pos = spot.foot;
            walker.target = spot.foot;
            walker.arrival = None;
            if clip.is_some_and(|k| k.station.is_some()) {
                actor.work(&p.story, c);
            } else {
                actor.hold = false;
                actor.play(&p.story, c);
            }
        }
        p.ticks = 0;
    }
    let origin = Vec2::new(-stations::SCENE_SIZE.x, stations::SCENE_SIZE.y) * SCALE / 2.;
    t.translation.x = origin.x + walker.pos.x * SCALE;
    t.translation.y = origin.y - walker.pos.y * SCALE;
    t.translation.z = stations::actor_z(if walker.pos.y < stations::FRONT_FLOOR - 20. {
        0
    } else {
        1
    });
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut p: ResMut<PlayerPreview>,
    mut clear: ResMut<ClearColor>,
    mut clock: ResMut<forge_npc::PlaybackClock>,
    catalogs: Option<Res<PlayerCatalogs>>,
    server: Res<AssetServer>,
    mut smith: Query<(&mut Actor, &Walker, &Sprite, &Transform, &CharacterId), With<Smith>>,
    mut backdrops: Query<&mut Visibility, (With<Backdrop>, Without<StationLayer>, Without<Guide>)>,
    mut layers: Query<&mut Visibility, (With<StationLayer>, Without<Backdrop>, Without<Guide>)>,
    mut guides: Query<
        (&mut Transform, &mut Visibility),
        (
            With<Guide>,
            Without<Smith>,
            Without<Backdrop>,
            Without<StationLayer>,
        ),
    >,
    mut labels: Query<&mut Text, With<Label>>,
    mut exit: MessageWriter<AppExit>,
) {
    p.ticks += 1;
    let Some(catalogs) = catalogs else {
        return;
    };
    let Ok((mut a, walker, sprite, transform, id)) = smith.single_mut() else {
        return;
    };
    let Some(c) = catalogs.get(&id.0) else {
        return;
    };
    clock.fixed_delta = p.capture.as_ref().map(|_| 1. / 30.);
    if keys.just_pressed(KeyCode::Space) {
        a.paused = !a.paused;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let story = p.story.clone();
        if c.clips.get(&story).is_some_and(|k| k.station.is_some()) {
            a.hold = false;
            a.work(&story, c);
        } else {
            a.play(&story, c);
        }
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        a.paused = true;
        let t = a.elapsed + 0.05;
        a.seek(t, c);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        a.paused = true;
        let t = (a.elapsed - 0.05).max(0.);
        a.seek(t, c);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        p.left = !p.left;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        p.base = !p.base;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        a.blink = !a.blink;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        p.guides = !p.guides;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        p.stations = !p.stations;
    }
    if keys.just_pressed(KeyCode::KeyH) {
        a.release();
    }
    if keys.just_pressed(KeyCode::Equal) {
        a.speed = (a.speed + 0.25).min(3.);
    }
    if keys.just_pressed(KeyCode::Minus) {
        a.speed = (a.speed - 0.25).max(0.);
    }
    if keys.just_pressed(KeyCode::KeyB) {
        p.background = (p.background + 1) % 3;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    clear.0 = [
        Color::srgb(0.08, 0.10, 0.12),
        Color::srgb(0.85, 0.85, 0.8),
        Color::BLACK,
    ][p.background];
    for mut v in &mut backdrops {
        *v = if p.background == 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut v in &mut layers {
        *v = if p.stations {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let frame = a.sprite(c);
    let foot = transform.translation.truncate();
    let top = foot + Vec2::new(0., frame.pivot[1] as f32 * SCALE);
    let flip = if a.facing_left { -1. } else { 1. };
    let head = foot
        + Vec2::new(
            flip * (frame.source_size[0] as f32 / 2. - frame.pivot[0] as f32) * SCALE,
            (frame.pivot[1] as f32 - 24.) * SCALE,
        );
    for (i, (mut t, mut v)) in guides.iter_mut().enumerate() {
        t.translation = [foot, top, head][i].extend(20.);
        *v = if p.guides {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let art = if matches!(
        server.get_load_state(sprite.image.id()),
        Some(bevy::asset::LoadState::Failed(_))
    ) {
        "art unavailable"
    } else {
        id.0.as_str()
    };
    for mut text in &mut labels {
        **text = format!(
            "Player Preview / {art} / {} / {}{}   frame {:?}   {:.2}s   {:.2}x {}{}\nfoot ({:.0},{:.0})  facing {}  events {:?}\nStory buttons | Space pause | Left/Right ±50 ms | R restart | H release hold | F facing | E equipment | S stations | G guides | B background | K blink | +/- speed\nNo game simulation | No save IO | shared forge-npc player + station floor plan",
            if p.base { "base" } else { "equipped" },
            a.clip,
            if a.hold { " (held)" } else { "" },
            a.clip_frame,
            a.elapsed,
            a.speed,
            if a.paused { "PAUSED" } else { "" },
            if a.finished { " done" } else { "" },
            walker.pos.x,
            walker.pos.y,
            if a.facing_left { "left" } else { "right" },
            a.events,
        );
    }
    if let Some(capture) = p.capture.clone() {
        // Reproducible isolated capture: fixed clock, seek from the clip start, no walking.
        if !p.walk {
            a.paused = true;
            let shot = p.ticks.saturating_sub(12) / 5;
            let t = if p.sequence {
                0.01 + shot.min(5) as f32 * 0.12
            } else {
                p.sample
            };
            a.seek(t, c);
        }
        if p.ticks == 12
            || (p.sequence && p.ticks > 12 && p.ticks <= 37 && (p.ticks - 12).is_multiple_of(5))
        {
            let shot = p.ticks.saturating_sub(12) / 5;
            let path = if p.sequence {
                format!("{}-{}.png", capture.trim_end_matches(".png"), shot)
            } else {
                capture
            };
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
        if p.ticks > if p.sequence { 47 } else { 24 } {
            exit.write(AppExit::Success);
        }
    }
}
