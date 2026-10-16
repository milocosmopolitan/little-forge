//! Bevy presentation for the village. All money/stock changes go through State::command.
use crate::{Options, Session, title::TitleScreen, window::Desktop};
use bevy::{
    prelude::*,
    ui::RelativeCursorPosition,
    ui_widgets::{Activate, Button},
};
use forge_content::village::Registry;
use forge_core::{
    Command,
    village::{Item, Location, Navigation, Screen},
};

#[derive(Resource)]
pub struct AssetRoot(pub std::path::PathBuf);
#[derive(Resource)]
pub struct Village {
    pub nav: Navigation,
    registry: Registry,
    hover: Option<usize>,
    quantity: u32,
    category: usize,
    redraw: bool,
    art_available: bool,
}
impl Default for Village {
    fn default() -> Self {
        Self {
            nav: Navigation::default(),
            registry: Registry::bundled().expect("validated map registry"),
            hover: None,
            quantity: 1,
            category: 0,
            redraw: true,
            art_available: true,
        }
    }
}
#[derive(Component)]
pub(crate) struct Root;
#[derive(Component)]
pub(crate) struct MapSurface;
#[derive(Component)]
pub(crate) struct MapLayer(Option<usize>);
#[derive(Component)]
pub(crate) struct Caption;
#[derive(Component)]
pub(crate) struct Wallet;
#[derive(Component)]
pub(crate) struct Status;
#[derive(Component, Clone, Copy)]
pub(crate) enum Action {
    Map,
    Home,
    Back,
    Close,
    Visit(Location),
    Category(usize),
    Quantity(i32),
    Buy(Item, u64),
}
fn text(s: impl Into<String>, size: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(s),
        TextFont {
            font_size: size.into(),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.87, 0.77)),
    )
}
fn button(parent: &mut ChildSpawnerCommands, label: impl Into<String>, action: Action) {
    parent
        .spawn((
            Button,
            action,
            Node {
                padding: UiRect::axes(px(10), px(3)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.4, 0.32, 0.23)),
            BackgroundColor(Color::srgb(0.14, 0.18, 0.17)),
        ))
        .with_children(|b| {
            b.spawn(text(label, 13.));
        });
}
/// Keep management controls inside smaller logical work areas and high-DPI windows.
/// Native DPI remains owned by Bevy; this is only an additional fit for the 960x540 UI.
pub fn fit_ui(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    desktop: Res<Desktop>,
    mut scale: ResMut<UiScale>,
) {
    if let Ok(window) = windows.single() {
        let reference_height = if desktop.shell.expanded { 540. } else { 180. };
        scale.0 = (window.width() / 960.)
            .min(window.height() / reference_height)
            .clamp(0.1, 1.);
    }
}
pub fn setup(
    mut commands: Commands,
    root: Res<AssetRoot>,
    opts: Res<Options>,
    mut v: ResMut<Village>,
) {
    if let Some(q) = opts.qa.as_deref().and_then(|q| q.strip_prefix("village-")) {
        v.nav.map();
        if let Some(l) = match q {
            "carpenter" => Some(Location::Carpenter),
            "leatherworker" => Some(Location::Leatherworker),
            "farm" => Some(Location::Farm),
            "tavern" => Some(Location::Tavern),
            "chief" => Some(Location::VillageChief),
            "guild" => Some(Location::MerchantGuild),
            "outside" => Some(Location::Outside),
            _ => None,
        } {
            v.nav.visit(l);
        } else {
            v.nav.focus = Some(1);
        }
    }
    // Optional assets fail to readable navigation; existing forge manifest remains strict.
    let root = &root.0;
    v.art_available = root.join("map/ashbrook/base.png").is_file()
        && v.registry
            .locations
            .iter()
            .all(|r| root.join(&r.overlay).is_file());
    if !v.art_available {
        warn!("Village map unavailable: destination list remains usable");
    }
    commands.spawn((
        Root,
        Node {
            display: Display::None,
            ..default()
        },
        GlobalZIndex(10),
    ));
}
pub fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    mut v: ResMut<Village>,
    mut d: ResMut<Desktop>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
) {
    if title.active || !d.shell.expanded {
        return;
    }
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    apply(*action, &mut v, &mut d, &mut s, &opts);
}
fn purchase_error(error: &forge_core::Error) -> &'static str {
    match error {
        forge_core::Error::OutOfStock => "This item is out of stock.",
        forge_core::Error::InsufficientResources => {
            "You do not have enough gold for this purchase."
        }
        forge_core::Error::PantryFull => "Your pantry is full. Use provisions before buying more.",
        forge_core::Error::Overflow => "Your storage capacity has been reached.",
        forge_core::Error::InvalidPurchase => {
            "Choose an available item and a quantity from 1 to 99."
        }
        forge_core::Error::DuplicateCommand => "This purchase has already been processed.",
        _ => "This purchase is unavailable. Return to the map and try again.",
    }
}
fn apply(a: Action, v: &mut Village, d: &mut Desktop, s: &mut Session, opts: &Options) {
    match a {
        Action::Map => v.nav.map(),
        Action::Home => v.nav = Navigation::default(),
        Action::Back => v.nav.back(),
        Action::Close => {
            v.nav = Navigation::default();
            d.shell.expanded = false;
            d.shell.recover();
        }
        Action::Visit(l) => {
            v.nav.visit(l);
            v.category = 0;
            v.quantity = 1;
        }
        Action::Category(i) => v.category = i,
        Action::Quantity(delta) => v.quantity = (v.quantity as i32 + delta).clamp(1, 99) as u32,
        Action::Buy(item, id) => {
            if s.disabled || v.nav.screen != Screen::Destination(item.vendor()) {
                return;
            }
            if id > s.save.state.command_id && v.category == 0 {
                s.status = match s.save.state.command(
                    id,
                    Command::Purchase {
                        vendor: item.vendor(),
                        item,
                        quantity: v.quantity,
                    },
                ) {
                    Ok(()) => format!("Purchased {} x{}", item.label(), v.quantity),
                    Err(e) => purchase_error(&e).to_owned(),
                };
                crate::persist(s, opts);
            }
        }
    }
    v.hover = None;
    v.nav.focus = None;
    v.redraw = true;
}
#[allow(clippy::too_many_arguments)]
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut v: ResMut<Village>,
    mut d: ResMut<Desktop>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    surface: Query<(&RelativeCursorPosition, &ComputedNode), With<MapSurface>>,
) {
    if title.active {
        return;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        d.shell.recover();
        d.shell.expanded = true;
        v.nav.map();
        v.redraw = true;
    }
    if !d.shell.expanded {
        if v.nav.screen != Screen::Home {
            v.nav = Navigation::default();
            v.redraw = true;
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        let a = if v.nav.screen == Screen::Home {
            Action::Close
        } else {
            Action::Back
        };
        apply(a, &mut v, &mut d, &mut s, &opts);
        return;
    }
    if v.nav.screen == Screen::Map {
        let hover = surface
            .single()
            .ok()
            .and_then(|(cursor, node)| {
                cursor
                    .normalized
                    .filter(|_| cursor.cursor_over)
                    .and_then(|p| {
                        v.registry.image_point(
                            [(p.x + 0.5) * node.size().x, (p.y + 0.5) * node.size().y],
                            node.size().to_array(),
                        )
                    })
            })
            .and_then(|p| {
                if v.art_available {
                    v.registry.pick(p)
                } else {
                    None
                }
            });
        if hover != v.hover {
            v.hover = hover;
            if hover.is_some() {
                v.nav.focus = None;
            }
        }
        if keys.just_pressed(KeyCode::ArrowDown)
            || keys.just_pressed(KeyCode::ArrowRight)
            || keys.just_pressed(KeyCode::Tab)
        {
            v.nav
                .cycle(keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));
            v.hover = None;
        }
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowLeft) {
            v.nav.cycle(true);
            v.hover = None;
        }
        let digit = [
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
        .position(|k| keys.just_pressed(*k));
        let visit = digit.or_else(|| {
            if keys.just_pressed(KeyCode::Enter) {
                v.nav.focus.or(v.hover)
            } else if mouse.just_pressed(MouseButton::Left) {
                v.hover
            } else {
                None
            }
        });
        if let Some(i) = visit {
            let l = v.registry.locations[i].id;
            apply(Action::Visit(l), &mut v, &mut d, &mut s, &opts);
        }
    } else if let Screen::Destination(l) = v.nav.screen {
        if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowRight) {
            v.category = 1 - v.category;
            v.redraw = true;
        }
        if keys.just_pressed(KeyCode::Minus) {
            apply(Action::Quantity(-1), &mut v, &mut d, &mut s, &opts);
        }
        if keys.just_pressed(KeyCode::Equal) {
            apply(Action::Quantity(1), &mut v, &mut d, &mut s, &opts);
        }
        if keys.just_pressed(KeyCode::Enter)
            && v.category == 0
            && let Some(item) = Item::ALL.into_iter().find(|i| i.vendor() == l)
        {
            apply(
                Action::Buy(item, s.save.state.command_id.saturating_add(1)),
                &mut v,
                &mut d,
                &mut s,
                &opts,
            );
        }
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn draw(
    mut commands: Commands,
    mut v: ResMut<Village>,
    d: Res<Desktop>,
    s: Res<Session>,
    server: Res<AssetServer>,
    assets: Res<AssetRoot>,
    root: Query<Entity, With<Root>>,
    mut last: Local<Option<(Screen, bool, bool, u64, u64, u32)>>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
) {
    let key = (
        v.nav.screen,
        d.shell.expanded,
        title.active,
        s.save.state.command_id,
        s.save.state.gold,
        s.save.state.life.pantry.staples(),
    );
    if *last == Some(key) && !v.redraw {
        return;
    }
    *last = Some(key);
    v.redraw = false;
    let Ok(root) = root.single() else {
        return;
    };
    commands.entity(root).despawn_children();
    let show = d.shell.expanded
        && !title.active
        && opts
            .qa
            .as_deref()
            .is_none_or(|q| q.starts_with("village-") || q.starts_with("life-"));
    commands.entity(root).insert((
        Node {
            display: if show { Display::Flex } else { Display::None },
            position_type: PositionType::Absolute,
            top: if v.nav.screen == Screen::Home {
                px(78)
            } else {
                px(0)
            },
            left: px(12),
            right: px(12),
            bottom: if v.nav.screen == Screen::Home {
                Val::Auto
            } else {
                px(180)
            },
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(Color::srgb(0.38, 0.28, 0.18)),
        BackgroundColor(Color::srgb(0.045, 0.055, 0.052)),
    ));
    commands.entity(root).with_children(|parent| {
        parent
            .spawn(Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                button(row, "Village [M]", Action::Map);
                button(row, "Forge", Action::Home);
                button(row, "Back [Esc]", Action::Back);
                button(row, "Close", Action::Close);
                row.spawn((text("", 13.), Wallet));
            });
        if v.nav.screen == Screen::Home {
            return;
        }
        parent.spawn((text("", 13.), Status));
        match v.nav.screen {
            Screen::Map => draw_map(parent, &v, &server),
            Screen::Destination(location) => {
                draw_destination(parent, location, &v, &s, &server, &assets)
            }
            Screen::Home => {}
        }
    });
}

fn draw_map(parent: &mut ChildSpawnerCommands, village: &Village, server: &AssetServer) {
    parent
        .spawn(Node {
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(16),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                MapSurface,
                RelativeCursorPosition::default(),
                Node {
                    width: percent(53),
                    height: percent(100),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.07, 0.08, 0.075)),
            ))
            .with_children(|map| {
                if village.art_available {
                    let layers = std::iter::once((None, "map/ashbrook/base.png".to_owned())).chain(
                        village
                            .registry
                            .locations
                            .iter()
                            .enumerate()
                            .map(|(i, r)| (Some(i), r.overlay.clone())),
                    );
                    for (index, path) in layers {
                        let image = server
                            .load_builder()
                            .with_settings::<bevy::image::ImageLoaderSettings>(|settings| {
                                settings.sampler = bevy::image::ImageSampler::linear()
                            })
                            .load::<Image>(path);
                        map.spawn((
                            ImageNode::new(image),
                            MapLayer(index),
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            Visibility::Hidden,
                        ));
                    }
                } else {
                    map.spawn(text(
                        "Map artwork unavailable.\nUse the eight destinations at right.",
                        16.,
                    ));
                }
            });
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                flex_grow: 1.,
                ..default()
            })
            .with_children(|list| {
                list.spawn(text("ASHBROOK  /  Choose a destination", 16.));
                for (i, region) in village.registry.locations.iter().enumerate() {
                    button(
                        list,
                        format!("{}  {}", i + 1, region.label),
                        Action::Visit(region.id),
                    );
                }
                list.spawn((text("", 12.), Caption));
            });
        });
}

fn destination_copy(location: Location) -> ([&'static str; 2], &'static str) {
    match location {
        Location::Carpenter => (
            ["Materials", "Furniture"],
            "Furniture requires the carpenter's tool commission. Commissions are not available yet.",
        ),
        Location::Leatherworker => (
            ["Materials", "Work gear"],
            "Work gear requires the leatherworker's tool commission. Commissions are not available yet.",
        ),
        Location::Farm => (
            ["Provisions", "Basic materials"],
            "Fiber supply requires the farmer's tool commission. No seeds or livestock in this milestone.",
        ),
        Location::Tavern => (
            ["Meals", "Recruit"],
            "Recruitment awaits the expedition system. No adventurers can be dispatched yet.",
        ),
        Location::VillageChief => (
            ["Village", ""],
            "The chief's village progression and dialogue are not implemented yet. No quest reward can be claimed.",
        ),
        Location::MerchantGuild => (
            ["Trade", ""],
            "Guild commissions and trade progression are not implemented yet. Local shops remain available.",
        ),
        _ => (
            ["Expeditions", ""],
            "Expeditions unavailable: recruitment, supplies and destinations are not implemented yet. Return to the village to prepare.",
        ),
    }
}

fn draw_destination(
    parent: &mut ChildSpawnerCommands,
    location: Location,
    village: &Village,
    session: &Session,
    server: &AssetServer,
    assets: &AssetRoot,
) {
    let state = &session.save.state;
    parent.spawn(Node {
        column_gap: px(18), flex_grow: 1., min_height: px(0), ..default()
    }).with_children(|row| {
        let item = Item::ALL.into_iter().find(|item| item.vendor() == location);
        if item.is_some() {
            let (id, width) = match location {
                Location::Carpenter => ("carpenter", 116.),
                Location::Leatherworker => ("leatherworker", 116.),
                Location::Farm => ("farm", 125.),
                _ => ("tavern", 103.),
            };
            let path = format!("village/{id}.png");
            if assets.0.join(&path).is_file() {
                row.spawn((ImageNode::new(server.load(path)), Node {
                    width: px(width), height: px(155), ..default()
                }));
            } else {
                row.spawn(text("Merchant\nportrait\nunavailable", 13.));
            }
        }
        row.spawn(Node {
            flex_direction: FlexDirection::Column, row_gap: px(10), flex_grow: 1.,
            min_width: px(0), width: percent(80), ..default()
        }).with_children(|body| {
            body.spawn(text(location.label(), 22.));
            let (tabs, locked) = destination_copy(location);
            if let Some(item) = item {
                body.spawn(Node { column_gap: px(8), ..default() }).with_children(|row| {
                    for (i, label) in tabs.iter().enumerate() {
                        button(row, format!("{}{}", if village.category == i { "* " } else { "" }, label), Action::Category(i));
                    }
                });
                if village.category == 0 {
                    let owned = match item {
                        Item::Timber => state.commerce.timber,
                        Item::Leather => state.commerce.leather,
                        Item::Staple => state.life.pantry.staples(),
                        Item::Meal => state.life.pantry.slots.iter().flatten()
                            .filter(|stack| stack.food == forge_core::life::Food::Premium)
                            .map(|stack| stack.count).sum(),
                    };
                    body.spawn(text(format!("{}\n{} gold each  /  Stock {}  /  Owned {}",
                        item.label(), item.price(state), state.commerce.stock[item.index()], owned), 16.));
                    let reason = match state.quote(location, item, village.quantity) {
                        Err(error) => purchase_error(&error).to_owned(),
                        Ok(price) if price > state.gold => format!("Need {price} gold; wallet has {}", state.gold),
                        Ok(price) => format!("Total: {price} gold"),
                    };
                    body.spawn(Node { column_gap: px(8), align_items: AlignItems::Center, ..default() })
                        .with_children(|row| {
                            button(row, "- [-]", Action::Quantity(-1));
                            row.spawn(text(format!("Quantity {}", village.quantity), 14.));
                            button(row, "+ [=]", Action::Quantity(1));
                            button(row, "Buy [Enter]", Action::Buy(item, state.command_id.saturating_add(1)));
                        });
                    body.spawn(text(reason, 14.));
                    body.spawn(text("[Left / Right] category  |  [- / =] quantity  |  [Esc] map", 12.));
                    let note = match item {
                        Item::Meal => "Premium meal stays in the pantry until explicitly consumed at the forge.",
                        Item::Staple => "Staples use the existing pantry and automatic eating rules. Well water remains free.",
                        _ => "Materials are stored locally. Crafting recipes and commissions are future progression.",
                    };
                    body.spawn(text(note, 13.));
                } else {
                    body.spawn(text(locked, 16.));
                }
            } else {
                body.spawn(text(locked, 16.));
                button(body, "Return to map", Action::Map);
            }
        });
    });
}
#[allow(clippy::type_complexity)]
pub fn update(
    v: Res<Village>,
    s: Res<Session>,
    surface: Query<&ComputedNode, With<MapSurface>>,
    mut layers: Query<(&MapLayer, &mut Node, &mut Visibility)>,
    mut buttons: Query<(&Action, &mut BackgroundColor)>,
    mut labels: ParamSet<(
        Query<&mut Text, With<Caption>>,
        Query<&mut Text, With<Wallet>>,
        Query<&mut Text, With<Status>>,
    )>,
) {
    let active = v.hover.or(v.nav.focus);
    for (action, mut color) in &mut buttons {
        let focused = matches!(action,Action::Visit(l) if active.is_some_and(|i|v.registry.locations[i].id==*l));
        let disabled = matches!(action,Action::Buy(item,_) if s.disabled || s.save.state.quote(item.vendor(),*item,v.quantity).map_or(true,|p|p>s.save.state.gold));
        color.0 = if focused {
            Color::srgb(0.32, 0.29, 0.20)
        } else if disabled {
            Color::srgb(0.12, 0.12, 0.12)
        } else {
            Color::srgb(0.14, 0.18, 0.17)
        };
    }
    if let Ok(surface) = surface.single() {
        let size = surface.size() * surface.inverse_scale_factor;
        let scale = (size.x / v.registry.width as f32).min(size.y / v.registry.height as f32);
        let draw = Vec2::new(v.registry.width as f32, v.registry.height as f32) * scale;
        for (layer, mut node, mut visible) in &mut layers {
            node.width = px(draw.x);
            node.height = px(draw.y);
            node.left = px((size.x - draw.x) / 2.);
            node.top = px((size.y - draw.y) / 2.);
            *visible = if layer.0.is_none() || layer.0 == active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    for mut t in &mut labels.p0() {
        **t = active
            .map(|i| format!("> {}  /  Enter to visit", v.registry.locations[i].label))
            .unwrap_or("Arrows / Tab: focus * 1-8: visit\nThe central well is a landmark.".into());
    }
    for mut t in &mut labels.p1() {
        **t = format!(
            "Gold {}  |  Timber {}  |  Leather {}  |  F {}%  H {}%  T {}%",
            s.save.state.gold,
            s.save.state.commerce.timber,
            s.save.state.commerce.leather,
            s.save.state.life.fatigue / 100,
            s.save.state.life.hunger / 100,
            s.save.state.life.thirst / 100
        );
    }
    for mut t in &mut labels.p2() {
        **t = s.status.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_observer_map_shop_buy_duplicate_back_and_hidden_guard() {
        let dir = std::env::temp_dir().join(format!("village-ui-{}", std::process::id()));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut state = forge_core::State::new(1);
        state.gold = 30;
        state.command(1, Command::Craft).unwrap();
        app.insert_resource(Session {
            save: forge_core::save::Save::new(state, 100),
            dir: dir.clone(),
            _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
            last: std::time::Instant::now(),
            fraction: 0.,
            status: String::new(),
            disabled: false,
            trace: vec![],
        });
        app.insert_resource(Options {
            qa: Some("village-map".into()),
            ..default()
        });
        app.insert_resource(Desktop::new(false, true));
        app.init_resource::<Village>();
        app.init_resource::<TitleScreen>();
        app.add_observer(activate);
        let map = app.world_mut().spawn(Action::Map).id();
        let shop = app
            .world_mut()
            .spawn(Action::Visit(Location::Carpenter))
            .id();
        let buy = app.world_mut().spawn(Action::Buy(Item::Timber, 2)).id();
        let back = app.world_mut().spawn(Action::Back).id();
        let close = app.world_mut().spawn(Action::Close).id();
        for e in [map, shop, buy, buy] {
            app.world_mut().trigger(Activate { entity: e });
        }
        let s = &app.world().resource::<Session>().save.state;
        assert_eq!((s.gold, s.commerce.timber, s.command_id), (27, 1, 2));
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(5)
            .unwrap();
        assert_eq!(app.world().resource::<Session>().save.state.crafted, 1);
        app.world_mut().trigger(Activate { entity: back });
        assert_eq!(app.world().resource::<Village>().nav.screen, Screen::Map);
        app.world_mut().trigger(Activate { entity: close });
        assert!(!app.world().resource::<Desktop>().shell.expanded);
        let old = app.world().resource::<Session>().save.state.clone();
        app.world_mut().trigger(Activate { entity: buy });
        assert_eq!(app.world().resource::<Session>().save.state, old);
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
