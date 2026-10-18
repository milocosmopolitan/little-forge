//! Visual fixture runner. No forge-core dependency, simulation, save or transaction IO.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui_widgets::Activate,
};
use forge_ui as ui;
#[allow(dead_code)]
#[path = "../../desktop/src/rendering.rs"]
mod rendering;
const STORIES: [&str; 18] = [
    "farm",
    "chef",
    "carpenter",
    "leatherworker",
    "empty",
    "full",
    "insufficient",
    "sold-out",
    "long-label",
    "controls",
    "confirmation",
    "showroom-empty",
    "showroom-partial",
    "showroom-full",
    "showroom-reserved",
    "showroom-held",
    "showroom-upgraded",
    "showroom-long",
];
/// Showroom fixtures (#93): one slot per finished item, shared badges, frames from
/// assets/forge/spaces. Values are sample data, never game state.
#[derive(Clone, Copy, PartialEq)]
enum FixtureSlot {
    Empty,
    Sale,
    Hold,
    Reserved,
}
fn showroom_fixture(name: &str) -> (Vec<FixtureSlot>, usize, Option<usize>) {
    use FixtureSlot::*;
    match name {
        "showroom-empty" => (vec![Empty; 4], 1, None),
        "showroom-partial" => (vec![Sale, Sale, Empty, Empty], 1, Some(0)),
        "showroom-full" => (vec![Sale, Hold, Sale, Sale], 1, Some(1)),
        "showroom-reserved" => (vec![Sale, Reserved, Empty, Empty], 1, Some(1)),
        "showroom-held" => (vec![Hold, Hold, Sale, Empty], 1, Some(0)),
        "showroom-upgraded" => (
            vec![Sale, Sale, Hold, Reserved, Empty, Empty, Empty, Empty],
            2,
            Some(3),
        ),
        _ => (vec![Sale, Hold, Reserved, Empty], 1, Some(0)),
    }
}
fn draw_showroom(p: &mut ChildSpawnerCommands, server: &AssetServer, name: &str) {
    let (slots, racks, selected) = showroom_fixture(name);
    let long = name == "showroom-long";
    let used = slots.iter().filter(|s| **s != FixtureSlot::Empty).count();
    p.spawn(Node {
        flex_grow: 1.,
        min_height: px(0),
        column_gap: px(16),
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            ui::SkinFrame {
                path: "forge/spaces/showroom-window.png".into(),
                inset: 80.,
                corner_scale: 0.5,
            },
            Node {
                width: px(580),
                padding: UiRect::new(px(26), px(26), px(22), px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|win| {
            win.spawn(Node {
                column_gap: px(8),
                ..default()
            })
            .with_children(|tabs| {
                for i in 0..3 {
                    let label = if long {
                        format!("진열대 {} · 아주 긴 한글 라벨 {} slots", i + 1, slots.len())
                    } else {
                        format!("Display {}", (b'A' + i as u8) as char)
                    };
                    if i == 0 {
                        ui::button(tabs, label, ui::Selected);
                    } else if i < racks {
                        ui::button(tabs, label, ());
                    } else {
                        ui::button(
                            tabs,
                            label,
                            (
                                ui::Unavailable("Fixture: not installed".into()),
                                bevy::ui::InteractionDisabled,
                            ),
                        );
                    }
                }
                tabs.spawn(Node {
                    flex_grow: 1.,
                    ..default()
                });
                ui::button(
                    tabs,
                    if long {
                        "진열대 관리 (목공소)"
                    } else {
                        "Manage displays"
                    },
                    (),
                );
            });
            win.spawn((
                Node {
                    height: px(1),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(ui::EDGE),
            ));
            win.spawn(ui::text(
                format!(
                    "{} {used} / {} occupied",
                    if long { "진열장" } else { "Showroom" },
                    slots.len()
                ),
                18.,
            ));
            win.spawn(Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(8),
                ..default()
            })
            .with_children(|grid| {
                for (i, slot) in slots.iter().enumerate() {
                    let (icon, badge) = match slot {
                        FixtureSlot::Empty => (None, None),
                        FixtureSlot::Sale => (
                            Some("dagger"),
                            Some((if long { "판매 중" } else { "FOR SALE" }, ui::BADGE_SALE)),
                        ),
                        FixtureSlot::Hold => (
                            Some("dagger"),
                            Some((if long { "보관" } else { "HOLD" }, ui::BADGE_HOLD)),
                        ),
                        FixtureSlot::Reserved => (
                            Some("queue"),
                            Some((
                                if long { "제작 예약" } else { "RESERVED" },
                                ui::BADGE_RESERVED,
                            )),
                        ),
                    };
                    ui::item_slot(
                        grid,
                        server,
                        icon,
                        badge,
                        selected == Some(i),
                        ui::Help(format!("fixture slot {i}")),
                    );
                }
            });
            win.spawn(ui::text(
                "1 item = 1 slot  |  Capacity includes reserved items.  |  fixture only",
                12.,
            ));
        });
        row.spawn((
            ui::SkinFrame {
                path: "forge/spaces/detail-panel.png".into(),
                inset: 110.,
                corner_scale: 0.4,
            },
            Node {
                width: px(240),
                padding: UiRect::new(px(24), px(24), px(26), px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
        ))
        .with_children(|panel| {
            let state = selected.map(|i| slots[i]);
            panel.spawn(ui::text(
                if long {
                    "철제 단검 · 매우 긴 아이템 이름 표시"
                } else {
                    "Iron dagger"
                },
                22.,
            ));
            panel.spawn(ui::text(
                match state {
                    Some(FixtureSlot::Sale) => "Ready for sale",
                    Some(FixtureSlot::Hold) => "Kept; visitors cannot buy it",
                    Some(FixtureSlot::Reserved) => "Result of the job in progress",
                    _ => "Select a slot to inspect it",
                },
                13.,
            ));
            panel
                .spawn((
                    Node {
                        width: px(112),
                        height: px(112),
                        align_self: AlignSelf::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(2)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.07, 0.06)),
                    BorderColor::all(Color::srgb(0.42, 0.42, 0.46)),
                ))
                .with_children(|f| {
                    if matches!(state, Some(FixtureSlot::Sale | FixtureSlot::Hold)) {
                        ui::icon(f, server, "dagger", 72.);
                    }
                });
            panel.spawn(ui::text("Sale price  12 G", 14.));
            panel
                .spawn(Node {
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|r| {
                    if state == Some(FixtureSlot::Hold) {
                        ui::button(r, "For sale", ());
                        ui::button(r, "Hold", ui::Selected);
                    } else {
                        ui::button(r, "For sale", ui::Selected);
                        ui::button(r, "Hold", ());
                    }
                });
            panel.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            ui::button(panel, "Manage displays", ());
            panel.spawn(ui::text("Expand at Carpenter · fixture", 11.));
        });
    });
}
#[derive(Resource)]
struct Story {
    name: String,
    capture: Option<String>,
    ticks: u32,
    dirty: bool,
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Choose(usize);
#[derive(Component)]
struct OpenDialog;
fn open_dialog(
    e: On<Activate>,
    q: Query<(), With<OpenDialog>>,
    mut state: ResMut<ui::DialogState>,
    focus: Res<bevy::input_focus::InputFocus>,
) {
    if q.contains(e.entity) {
        state.open(
            "preview-only",
            "Confirm action?",
            "This is an isolated dialog fixture. No save or game state will change.",
            focus.get(),
        );
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let value = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let width = value("--width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1100u32);
    let height = value("--height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(720u32);
    let name = value("--story").unwrap_or_else(|| "farm".into());
    assert!(STORIES.contains(&name.as_str()), "unknown story");
    let root = value("--assets")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("assets")))
                .filter(|p| p.is_dir())
        })
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets"));
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(rendering::plugin(true))
            .set(AssetPlugin {
                file_path: root.to_string_lossy().into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Little Forge | UI Preview (fixtures)".into(),
                    resolution: (width, height).into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(ui::ForgeUiPlugin)
    .insert_resource(ClearColor(Color::srgb(0.035, 0.03, 0.025)))
    .insert_resource(Story {
        name,
        capture: value("--capture"),
        ticks: 0,
        dirty: true,
    })
    .add_systems(Startup, setup)
    .add_systems(Update, (draw, fit, capture).chain())
    .add_observer(choose)
    .add_observer(open_dialog);
    rendering::install_sprite_shader(&mut app);
    app.run();
}
fn setup(mut c: Commands) {
    c.spawn(Camera2d);
    c.spawn((
        Root,
        bevy::input_focus::tab_navigation::TabGroup::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            right: px(16),
            top: px(16),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
    ));
}
fn choose(e: On<Activate>, q: Query<&Choose>, mut s: ResMut<Story>) {
    if let Ok(c) = q.get(e.entity) {
        s.name = STORIES[c.0].into();
        s.dirty = true;
    }
}
fn fit(w: Query<&Window>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = w.single() {
        scale.0 = (w.width() / 1100.).min(w.height() / 720.).min(1.);
    }
}
fn draw(
    mut c: Commands,
    q: Query<Entity, With<Root>>,
    server: Res<AssetServer>,
    mut s: ResMut<Story>,
) {
    if !s.dirty {
        return;
    }
    s.dirty = false;
    let root = q.single().unwrap();
    c.entity(root).despawn_children();
    c.entity(root).with_children(|p| {
        p.spawn(ui::text(
            "UI WORKSHOP · Isolated fixtures · 실제 게임 데이터 아님",
            20.,
        ));
        p.spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(5),
            row_gap: px(4),
            ..default()
        })
        .with_children(|p| {
            for (i, n) in STORIES.iter().enumerate() {
                ui::button(p, *n, Choose(i));
            }
        });
        if s.name.starts_with("showroom-") {
            draw_showroom(p, &server, &s.name);
            return;
        }
        let id = match s.name.as_str() {
            "chef" => "chef",
            "carpenter" => "carpenter",
            "leatherworker" => "leatherworker",
            _ => "farmer",
        };
        ui::merchant_header(
            p,
            &server,
            id,
            if s.name == "long-label" {
                "농장 상점 · 오늘도 좋은 하루를 준비하세요"
            } else {
                id
            },
            "Welcome back. What can I help you with today?",
        );
        p.spawn(Node {
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(12),
            ..default()
        })
        .with_children(|row| {
            row.spawn(ui::panel(percent(20))).with_children(|p| {
                ui::button(p, "Provisions", ui::Selected);
                ui::button(p, "Materials", ());
                ui::button(
                    p,
                    "Locked",
                    (
                        ui::Unavailable("Requires a commission".into()),
                        bevy::ui::InteractionDisabled,
                    ),
                );
            });
            row.spawn(ui::panel(percent(42))).with_children(|p| {
                p.spawn(ui::text("Available goods", 20.));
                if s.name == "empty" {
                    p.spawn(ui::text("Nothing in storage yet.", 16.));
                } else {
                    p.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(6),
                        row_gap: px(6),
                        ..default()
                    })
                    .with_children(|p| {
                        for i in 0..if s.name == "full" { 9 } else { 3 } {
                            ui::item_card(
                                p,
                                &server,
                                ["staples", "meal", "water"][i % 3],
                                ["Staples", "Warm meal", "Water"][i % 3],
                                if s.name == "sold-out" { 0 } else { 20 },
                            );
                        }
                    });
                }
            });
            row.spawn(ui::panel(percent(38))).with_children(|p| {
                ui::icon(p, &server, "staples", 64.);
                p.spawn(ui::text("Staple provisions", 21.));
                p.spawn(ui::text("Everyday food for a busy smith.", 15.));
                ui::progress(p, &server, 0.6, "Capacity · 60%");
                let disabled = s.name == "insufficient" || s.name == "sold-out";
                if disabled {
                    p.spawn(ui::text(
                        if s.name == "sold-out" {
                            "Sold out"
                        } else {
                            "Need 10 gold · You have 2"
                        },
                        15.,
                    ));
                    ui::button(
                        p,
                        "Purchase unavailable",
                        (
                            ui::Unavailable("Fixture".into()),
                            bevy::ui::InteractionDisabled,
                        ),
                    );
                } else {
                    ui::button(p, "Purchase · fixture only", ());
                }
                if s.name == "controls" {
                    ui::button(p, "Selected", ui::Selected);
                    ui::button(p, "Open confirmation", OpenDialog);
                    ui::progress(p, &server, 0., "Empty");
                    ui::progress(p, &server, 1., "Full");
                    p.spawn(ui::text(
                        "Hover or press a control to inspect its state.",
                        14.,
                    ));
                }
            });
        });
    });
}
fn capture(
    mut c: Commands,
    mut s: ResMut<Story>,
    server: Res<AssetServer>,
    mut exit: MessageWriter<AppExit>,
    mut dialog: ResMut<ui::DialogState>,
) {
    if s.name == "confirmation" && s.ticks == 120 {
        dialog.open(
            "fixture",
            "Leave the workshop?",
            "Preview only: no save is written and no game is closed.",
            None,
        );
    }
    s.ticks += 1;
    if s.capture.is_some()
        && s.ticks == 180
        && server.is_loaded_with_dependencies(server.load::<Font>("ui/fonts/NotoSansKR.ttf").id())
    {
        c.spawn(Screenshot::primary_window())
            .observe(save_to_disk(s.capture.clone().unwrap()));
    }
    if s.capture.is_some() && s.ticks >= 220 {
        exit.write(AppExit::Success);
    }
}
