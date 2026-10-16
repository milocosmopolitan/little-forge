//! Isolated visual stories; deliberately does not import forge-core or game startup.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
#[allow(dead_code)]
#[path = "../../desktop/src/rendering.rs"]
mod rendering;
use bevy::ui_widgets::{Activate, Button};
use forge_npc::{ACTIONS, Catalog, NpcPlugin, Player};
#[derive(Resource)]
struct Preview {
    action: usize,
    zoom: f32,
    background: usize,
    overlay: bool,
    fixture: bool,
    item_index: usize,
    ticks: u32,
    capture: Option<String>,
    story: String,
    sample: f32,
    left: bool,
    sequence: bool,
}
#[derive(Component)]
struct Actor;
#[derive(Component)]
struct Held;
#[derive(Component)]
struct Label;
#[derive(Component)]
struct Guide(usize);
#[derive(Component)]
struct Story(usize);
#[derive(Resource)]
struct ItemImages(Vec<Handle<Image>>);
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let story = value("--story").unwrap_or_else(|| "idle".into());
    if args.iter().any(|a| a == "--help") {
        println!(
            "NPC Preview (no game/save IO)\n--story idle|talk|blink|surprised|nod|shake_head|walk|give --capture PATH\n1-8 story; Space pause; arrows frame step; R restart; F mirror; I item; B background; G guides; Z size; N test NPC; +/- speed; Esc quit"
        );
        return;
    }
    let bundled = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("assets")));
    let root = bundled
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets"));
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(rendering::plugin(true))
            .set(AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Little Forge | NPC Preview".into(),
                    resolution: (1100, 800).into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(bevy::winit::WinitSettings {
        focused_mode: bevy::winit::UpdateMode::reactive_low_power(
            std::time::Duration::from_secs_f64(1. / 30.),
        ),
        unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(
            std::time::Duration::from_secs_f64(1. / 30.),
        ),
    })
    .add_plugins(NpcPlugin)
    .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.12)))
    .insert_resource(Preview {
        action: ACTIONS.iter().position(|a| *a == story).unwrap_or(0),
        zoom: 1.,
        background: 0,
        overlay: false,
        fixture: false,
        item_index: match value("--item").as_deref() {
            Some("none") => 0,
            Some("dagger") => 2,
            _ => 1,
        },
        ticks: 0,
        capture: value("--capture"),
        sample: value("--at").and_then(|v| v.parse().ok()).unwrap_or(0.24),
        left: args.iter().any(|v| v == "--left"),
        sequence: args.iter().any(|v| v == "--sequence"),
        story,
    })
    .add_observer(select_story)
    .add_systems(Startup, setup)
    .add_systems(Update, controls.before(forge_npc::animate));
    rendering::install_sprite_shader(&mut app);
    app.run();
}
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    c: Res<Catalog>,
    p: Res<Preview>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2d);
    let mut player = Player {
        item: false,
        ..default()
    };
    player.play(&p.story, &c);
    player.facing_left = p.left;
    let owner = commands
        .spawn((
            Sprite {
                image: server.load(&c.image),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.add(c.layout()),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(c.cell[0] as f32 / c.cell[1] as f32 * 420., 420.)),
                ..default()
            },
            Transform::from_xyz(0., -240., 0.),
            c.anchor(),
            player,
            Actor,
        ))
        .id();
    let carrot = server.load("npc/farmer/carrot.png");
    let dagger = images.add(forge_npc::dagger_image());
    commands.insert_resource(ItemImages(vec![carrot.clone(), dagger]));
    commands.spawn((
        Sprite {
            image: carrot,
            custom_size: Some(Vec2::new(45., 70.)),
            ..default()
        },
        Transform::default(),
        Visibility::Hidden,
        Held,
        forge_npc::ItemAttachment {
            owner,
            enabled: true,
        },
    ));
    commands.spawn((
        Text::new("Loading NPC stories…"),
        TextFont {
            font_size: 19.0.into(),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            top: px(16),
            ..default()
        },
        Label,
    ));
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: px(18),
            top: px(160),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        })
        .with_children(|parent| {
            for (i, name) in ACTIONS.iter().enumerate() {
                parent
                    .spawn((
                        Button,
                        Story(i),
                        Node {
                            padding: UiRect::axes(px(18), px(8)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.23, 0.23)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(format!("{}  {}", i + 1, name)),
                            TextFont {
                                font_size: 17.0.into(),
                                ..default()
                            },
                        ));
                    });
            }
        });
    for i in 0..7 {
        commands.spawn((
            Sprite::from_color(
                [
                    Color::srgb(1., 0.3, 0.2),
                    Color::srgb(0.2, 1., 0.3),
                    Color::srgb(0.2, 0.6, 1.),
                ][i.min(2)],
                Vec2::splat(7.),
            ),
            Transform::from_xyz(0., 0., 10.),
            Visibility::Hidden,
            Guide(i),
        ));
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut c: ResMut<Catalog>,
    mut p: ResMut<Preview>,
    mut clear: ResMut<ClearColor>,
    items: Res<ItemImages>,
    mut actor: Query<
        (&mut Player, &mut Sprite, &mut Transform),
        (With<Actor>, Without<Held>, Without<Guide>),
    >,
    mut held: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Sprite,
            &mut forge_npc::ItemAttachment,
        ),
        (With<Held>, Without<Actor>, Without<Guide>),
    >,
    mut labels: Query<&mut Text, With<Label>>,
    mut guides: Query<
        (&Guide, &mut Transform, &mut Visibility, &mut Sprite),
        (With<Guide>, Without<Actor>, Without<Held>),
    >,
    mut exit: MessageWriter<AppExit>,
) {
    p.ticks += 1;
    let Ok((mut a, mut sprite, mut transform)) = actor.single_mut() else {
        return;
    };
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ]
    .iter()
    .enumerate()
    {
        if keys.just_pressed(*key) {
            p.action = i;
            a.play(ACTIONS[i], &c);
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        a.paused = !a.paused;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        a.play(ACTIONS[p.action], &c);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        a.paused = true;
        a.step(true, &c);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        a.paused = true;
        a.step(false, &c);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        a.facing_left = !a.facing_left;
    }
    if keys.just_pressed(KeyCode::KeyI) {
        p.item_index = (p.item_index + 1) % 3;
        a.item = p.item_index > 0;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        a.layers = !a.layers;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        a.blink = !a.blink;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        p.overlay = !p.overlay;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        p.fixture = !p.fixture;
        *c = if p.fixture {
            Catalog::fixture()
        } else {
            Catalog::farmer()
        };
        a.play(ACTIONS[p.action], &c);
    }
    if keys.just_pressed(KeyCode::Equal) {
        a.speed = (a.speed + 0.25).min(3.);
    }
    if keys.just_pressed(KeyCode::Minus) {
        a.speed = (a.speed - 0.25).max(0.);
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        p.zoom = if p.zoom == 1. { 75. / 420. } else { 1. };
    }
    if keys.just_pressed(KeyCode::KeyB) {
        p.background = (p.background + 1) % 3;
        clear.0 = [
            Color::srgb(0.08, 0.10, 0.12),
            Color::srgb(0.85, 0.85, 0.8),
            Color::BLACK,
        ][p.background];
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    transform.scale = Vec3::splat(p.zoom);
    sprite.color = if p.fixture {
        Color::srgb(0.7, 0.85, 1.)
    } else {
        Color::WHITE
    };
    let height = 420. * p.zoom;
    let origin = transform.translation.truncate();
    for (mut t, mut v, mut item, mut attachment) in &mut held {
        attachment.enabled = p.item_index > 0;
        let hand = a.hand_offset(&c, height);
        t.translation = (origin + hand).extend(0.5);
        t.scale = Vec3::splat(p.zoom);
        item.flip_x = a.facing_left;
        item.image = items.0[if p.item_index == 2 { 1 } else { 0 }].clone();
        item.custom_size = Some(if p.item_index == 2 {
            Vec2::new(15., 65.)
        } else {
            Vec2::new(45., 70.)
        });
        *v = if (p.item_index > 0 && (a.item || a.action == "give")) && !a.transferred {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (guide, mut t, mut v, mut s) in &mut guides {
        let scale = height / c.cell[1] as f32;
        let bottom = -(c.cell[1] as f32 - c.foot[1]) * scale;
        let top = c.foot[1] * scale;
        let left = -c.foot[0] * scale;
        let right = (c.cell[0] as f32 - c.foot[0]) * scale;
        let point = match guide.0 {
            0 => Vec2::ZERO,
            1 => Vec2::new(
                (c.head[0] - c.foot[0]) * scale,
                (c.foot[1] - c.head[1]) * scale,
            ),
            2 => a.hand_offset(&c, height),
            3 => Vec2::new(0., bottom),
            4 => Vec2::new(0., top),
            5 => Vec2::new(left, (bottom + top) / 2.),
            _ => Vec2::new(right, (bottom + top) / 2.),
        };
        s.custom_size = Some(match guide.0 {
            3 | 4 => Vec2::new(right - left, 1.),
            5 | 6 => Vec2::new(1., height),
            _ => Vec2::splat(7.),
        });
        t.translation = (origin + point).extend(10.);
        *v = if p.overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut labels {
        **text = format!(
            "NPC Preview / {} / {}   frame {}   {:.2}s   {:.2}x {}\n1-8 actions | Space pause | Left/Right step | R restart | F facing | I item\nB background | G bounds | K blink | L layers | Z size | N fixture NPC | +/- speed\nNo game simulation | No save IO | shared forge-npc player",
            if p.fixture { "fixture B" } else { "farmer" },
            a.action,
            a.frame,
            a.elapsed,
            a.speed,
            if a.paused { "PAUSED" } else { "" }
        );
    }
    if let Some(capture) = p.capture.clone() {
        // reproducible isolated capture independent of real-time clock
        a.paused = true;
        let action = p.story.clone();
        a.play(&action, &c);
        let shot = p.ticks.saturating_sub(12) / 5;
        a.seek(
            if p.sequence {
                0.01 + shot.min(5) as f32 * 0.12
            } else {
                p.sample
            },
            &c,
        );
        if p.ticks == 12
            || (p.sequence && p.ticks > 12 && p.ticks <= 37 && (p.ticks - 12).is_multiple_of(5))
        {
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

fn select_story(
    event: On<Activate>,
    stories: Query<&Story>,
    mut p: ResMut<Preview>,
    c: Res<Catalog>,
    mut actors: Query<&mut Player, With<Actor>>,
) {
    if let Ok(story) = stories.get(event.entity) {
        p.action = story.0;
        for mut a in &mut actors {
            a.play(ACTIONS[p.action], &c);
        }
    }
}
