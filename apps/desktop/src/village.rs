//! Bevy presentation for the village. All money/stock changes go through State::command.
use crate::{Options, Session, title::TitleScreen, window::Desktop};
use bevy::{prelude::*, ui::RelativeCursorPosition, ui_widgets::Activate};
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
    selected_item: Option<Item>,
    pub redraw: bool,
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
            selected_item: None,
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
    SelectItem(Item),
    Quantity(i32),
    Buy(Item, u64),
    /// Carpenter furniture: rack purchase/upgrade with the transaction ID rendered on the button.
    Furniture(Command, u64),
}
fn text(s: impl Into<String>, size: f32) -> impl Bundle {
    forge_ui::text(s, size)
}
fn button(parent: &mut ChildSpawnerCommands, label: impl Into<String>, action: Action) {
    forge_ui::button(parent, label, action);
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
        let next = (window.width() / 960.)
            .min(window.height() / reference_height)
            .clamp(0.1, 1.);
        if scale.0 != next {
            scale.0 = next;
        }
    }
}
pub fn setup(
    mut commands: Commands,
    root: Res<AssetRoot>,
    opts: Res<Options>,
    mut v: ResMut<Village>,
) {
    if opts
        .qa
        .as_deref()
        .is_some_and(|q| q.starts_with("expedition-"))
    {
        v.nav.visit(Location::Outside);
    }
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
    if opts.qa.as_deref().is_some_and(|q| {
        matches!(
            q,
            "village-home" | "village-materials" | "village-provisions"
        )
    }) {
        v.nav = Navigation::default();
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
        forge_ui::WindowSkin,
        bevy::input_focus::tab_navigation::TabGroup::new(0),
        Node {
            display: Display::None,
            ..default()
        },
        GlobalZIndex(10),
    ));
}
#[allow(clippy::too_many_arguments)]
pub fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    mut v: ResMut<Village>,
    mut d: ResMut<Desktop>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    dialog: Option<Res<forge_ui::DialogState>>,
    mut farmer: Option<ResMut<crate::npc::FarmerPresentation>>,
) {
    if title.active || !d.shell.expanded || dialog.as_ref().is_some_and(|d| d.is_open()) {
        return;
    }
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    apply(
        *action,
        &mut v,
        &mut d,
        &mut s,
        &opts,
        farmer.as_deref_mut(),
    );
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
fn apply(
    a: Action,
    v: &mut Village,
    d: &mut Desktop,
    s: &mut Session,
    opts: &Options,
    farmer: Option<&mut crate::npc::FarmerPresentation>,
) {
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
            v.selected_item = None;
        }
        Action::Category(i) => v.category = i,
        Action::SelectItem(item) => {
            if v.nav.screen == Screen::Destination(item.vendor()) {
                v.selected_item = Some(item);
                v.quantity = 1;
            }
        }
        Action::Quantity(delta) => v.quantity = (v.quantity as i32 + delta).clamp(1, 99) as u32,
        Action::Furniture(command, id) => {
            if s.disabled || v.nav.screen != Screen::Destination(Location::Carpenter) {
                return;
            }
            if id > s.save.state.command_id && v.category == 1 {
                s.status = match s.save.state.command(id, command) {
                    Ok(()) => match command {
                        Command::BuyRack => "Installed a new display rack.".into(),
                        _ => "Display rack upgraded; more slots are available.".into(),
                    },
                    Err(forge_core::Error::InsufficientResources) => {
                        "You need more gold or timber for this furniture.".into()
                    }
                    Err(forge_core::Error::RackLimit) => {
                        "The showroom cannot hold more displays, or this rack is at its top tier."
                            .into()
                    }
                    Err(e) => purchase_error(&e).to_owned(),
                };
                crate::persist(s, opts);
            }
        }
        Action::Buy(item, id) => {
            if s.disabled || v.nav.screen != Screen::Destination(item.vendor()) {
                return;
            }
            if id > s.save.state.command_id && v.category == 0 {
                let command = Command::Purchase {
                    vendor: item.vendor(),
                    item,
                    quantity: v.quantity,
                };
                s.status = match s.save.state.command(id, command) {
                    Ok(()) => {
                        if let Some(farmer) = farmer {
                            farmer.accepted(id, command);
                        }
                        format!("Purchased {} x{}", item.label(), v.quantity)
                    }
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
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    widgets: Query<(), With<forge_ui::SkinButton>>,
    mut v: ResMut<Village>,
    mut d: ResMut<Desktop>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    dialog: Option<Res<forge_ui::DialogState>>,
    mut farmer: Option<ResMut<crate::npc::FarmerPresentation>>,
    mut spaces: Option<ResMut<crate::spaces::Spaces>>,
    surface: Query<(&RelativeCursorPosition, &ComputedNode), With<MapSurface>>,
    host: Option<Res<crate::forge_view::Host>>,
) {
    if title.active || dialog.as_ref().is_some_and(|d| d.is_open()) {
        return;
    }
    // Orders / Collection handle their own Escape (back to the host first), see forge_view::keyboard.
    let kit_screen = host
        .as_ref()
        .is_some_and(|h| !h.legacy && h.screen != crate::forge_view::HostScreen::Home);
    let widget_focused = focus
        .as_ref()
        .is_some_and(|f| f.get().is_some_and(|e| widgets.contains(e)));
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
    if keys.just_pressed(KeyCode::Escape) && !kit_screen {
        // A workshop drawer closes first; the overlay collapses on the next Escape.
        if v.nav.screen == Screen::Home
            && let Some(spaces) = spaces.as_deref_mut()
            && crate::spaces::drawer_open(spaces)
        {
            spaces.station = None;
            spaces.manage = false;
            spaces.redraw = true;
            return;
        }
        let a = if v.nav.screen == Screen::Home {
            Action::Close
        } else {
            Action::Back
        };
        apply(a, &mut v, &mut d, &mut s, &opts, farmer.as_deref_mut());
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
        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowRight) {
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
            if keys.just_pressed(KeyCode::Enter) && !widget_focused {
                v.nav.focus.or(v.hover)
            } else if mouse.just_pressed(MouseButton::Left) {
                v.hover
            } else {
                None
            }
        });
        if let Some(i) = visit {
            let l = v.registry.locations[i].id;
            apply(
                Action::Visit(l),
                &mut v,
                &mut d,
                &mut s,
                &opts,
                farmer.as_deref_mut(),
            );
        }
    } else if let Screen::Destination(l) = v.nav.screen {
        if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowRight) {
            v.category = 1 - v.category;
            v.redraw = true;
        }
        if keys.just_pressed(KeyCode::Minus) {
            apply(
                Action::Quantity(-1),
                &mut v,
                &mut d,
                &mut s,
                &opts,
                farmer.as_deref_mut(),
            );
        }
        if keys.just_pressed(KeyCode::Equal) {
            apply(
                Action::Quantity(1),
                &mut v,
                &mut d,
                &mut s,
                &opts,
                farmer.as_deref_mut(),
            );
        }
        if keys.just_pressed(KeyCode::Enter)
            && !widget_focused
            && v.category == 0
            && let Some(item) = v
                .selected_item
                .filter(|i| i.vendor() == l)
                .or_else(|| Item::ALL.into_iter().find(|i| i.vendor() == l))
        {
            apply(
                Action::Buy(item, s.save.state.command_id.saturating_add(1)),
                &mut v,
                &mut d,
                &mut s,
                &opts,
                farmer.as_deref_mut(),
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
    mut last: Local<Option<(Screen, bool, bool, u64, u64, u32, forge_core::spaces::Space)>>,
    opts: Res<Options>,
    title: Res<TitleScreen>,
    spaces: Option<Res<crate::spaces::Spaces>>,
    host: Res<crate::forge_view::Host>,
) {
    // Home content is owned by `spaces::draw` / `lifestyle::draw`; their buttons issue commands
    // on every click, so command/gold/pantry changes must not rebuild (and flash) the shell there.
    let home = v.nav.screen == Screen::Home;
    let space = spaces
        .as_ref()
        .map_or(forge_core::spaces::Space::Exterior, |s| s.space);
    let key = (
        v.nav.screen,
        d.shell.expanded,
        title.active,
        if home { 0 } else { s.save.state.command_id },
        if home { 0 } else { s.save.state.gold },
        if home {
            0
        } else {
            s.save.state.life.pantry.staples()
        },
        space,
    );
    if *last == Some(key) && !v.redraw && !host.is_changed() {
        return;
    }
    *last = Some(key);
    v.redraw = false;
    let Ok(root) = root.single() else {
        return;
    };
    commands.entity(root).despawn_children();
    let show = !crate::forge_view::shown(&title, &v, &d, &host)
        && d.shell.expanded
        && v.nav.screen != Screen::Destination(Location::Outside)
        && !title.active
        && opts.qa.as_deref().is_none_or(|q| {
            q.starts_with("village-")
                || q.starts_with("life-")
                || q.starts_with("forge-")
                || (q.starts_with("repair-") || q.starts_with("alpha"))
                || q == "expanded"
        });
    commands.entity(root).insert((
        Node {
            display: if show { Display::Flex } else { Display::None },
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(12),
            right: px(12),
            bottom: px(196),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BorderColor::all(Color::srgb(0.38, 0.28, 0.18)),
        BackgroundColor(forge_ui::SURFACE),
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
                if v.nav.screen == Screen::Home {
                    for (i, candidate) in forge_core::spaces::Space::ALL.into_iter().enumerate() {
                        let e = forge_ui::button(
                            row,
                            format!("{} [{}]", candidate.label(), i + 1),
                            crate::spaces::Action::Go(candidate),
                        );
                        if candidate == space {
                            row.commands().entity(e).insert(forge_ui::Selected);
                        }
                    }
                } else {
                    button(row, "Forge", Action::Home);
                }
                button(row, "Back [Esc]", Action::Back);
                button(row, "Close", Action::Close);
                row.spawn((text("", 13.), Wallet));
            });
        if v.nav.screen == Screen::Home {
            parent.spawn((
                crate::spaces::HomeRoot,
                bevy::input_focus::tab_navigation::TabGroup::new(1),
                Node {
                    flex_grow: 1.,
                    min_height: px(420),
                    flex_shrink: 0.,
                    position_type: PositionType::Relative,
                    ..default()
                },
            ));
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
                    width: percent(68),
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
                list.spawn(text("ASHBROOK", 16.));
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
            "Display racks grow the showroom. Further furniture waits for the carpenter's tool commission.",
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
    let (id, line) = match location {
        Location::Carpenter => ("carpenter", "Good timber. Tools made to last."),
        Location::Leatherworker => ("leatherworker", "A little care goes a long way."),
        Location::Farm => ("farmer", "Fresh provisions for a good day's work."),
        Location::Tavern => ("chef", "Come in. There is something warm on the stove."),
        _ => ("", ""),
    };
    if !id.is_empty() && assets.0.join(format!("npc/{id}/neutral.png")).is_file() {
        forge_ui::merchant_header(parent, server, id, location.label(), line);
    } else {
        parent.spawn(text(location.label(), 24.));
    }
    let (tabs, locked) = destination_copy(location);
    let items: Vec<_> = Item::ALL
        .into_iter()
        .filter(|i| i.vendor() == location)
        .collect();
    parent
        .spawn(Node {
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(12),
            ..default()
        })
        .with_children(|row| {
            row.spawn(forge_ui::panel(percent(18)))
                .with_children(|side| {
                    for (i, tab) in tabs.iter().enumerate().filter(|(_, t)| !t.is_empty()) {
                        button(
                            side,
                            format!("{}{}", if village.category == i { "> " } else { "" }, tab),
                            Action::Category(i),
                        );
                    }
                    side.spawn(text("Esc · Village", 12.));
                });
            if location == Location::Carpenter && village.category == 1 {
                draw_furniture(row, state, server);
                return;
            }
            if village.category != 0 || items.is_empty() {
                row.spawn(forge_ui::panel(percent(82))).with_children(|p| {
                    p.spawn(text("Not available yet", 20.));
                    p.spawn(text(locked, 14.));
                    button(p, "Return to map", Action::Map);
                });
                return;
            }
            row.spawn(forge_ui::panel(percent(38)))
                .with_children(|list| {
                    list.spawn(text("AVAILABLE GOODS", 13.));
                    for item in &items {
                        forge_ui::item_button(
                            list,
                            server,
                            item_icon(*item),
                            item.label(),
                            item.price(state),
                            village.selected_item.unwrap_or(items[0]) == *item,
                            Action::SelectItem(*item),
                        );
                    }
                    list.spawn(text("More goods unlock as the village grows.", 12.));
                });
            let item = village
                .selected_item
                .filter(|i| items.contains(i))
                .unwrap_or(items[0]);
            row.spawn(forge_ui::panel(percent(44)))
                .with_children(|body| {
                    body.spawn(Node {
                        column_gap: px(12),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|p| {
                        forge_ui::icon(p, server, item_icon(item), 40.);
                        p.spawn(text(item.label(), 18.));
                    });
                    let owned = match item {
                        Item::Timber => state.commerce.timber,
                        Item::Leather => state.commerce.leather,
                        Item::Staple => state.life.pantry.staples(),
                        Item::Meal => state
                            .life
                            .pantry
                            .slots
                            .iter()
                            .flatten()
                            .filter(|s| s.food == forge_core::life::Food::Premium)
                            .map(|s| s.count)
                            .sum(),
                    };
                    body.spawn(text(
                        format!(
                            "In stock {}   ·   Owned {}",
                            state.commerce.stock[item.index()],
                            owned
                        ),
                        13.,
                    ));
                    let reason = match state.quote(location, item, village.quantity) {
                        Err(e) => purchase_error(&e).to_owned(),
                        Ok(p) if p > state.gold => {
                            format!("Need {p} gold · You have {}", state.gold)
                        }
                        Ok(p) => format!("Total: {p} gold"),
                    };
                    body.spawn(Node {
                        column_gap: px(8),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|p| {
                        button(p, "-", Action::Quantity(-1));
                        p.spawn(text(village.quantity.to_string(), 17.));
                        button(p, "+", Action::Quantity(1));
                    });
                    body.spawn(text(reason, 14.));
                    button(
                        body,
                        "Purchase [Enter]",
                        Action::Buy(item, state.command_id.saturating_add(1)),
                    );
                    body.spawn(text(
                        if item == Item::Meal {
                            "A warm meal. Use it from your provisions."
                        } else if item == Item::Staple {
                            "Everyday provisions. Your smith eats these when needed."
                        } else {
                            "Stored for future recipes and commissions."
                        },
                        12.,
                    ));
                });
        });
}
/// Showroom furniture (#95): one more rack or a higher tier on an existing rack. Prices and
/// slot counts come from the save's showroom config (seeded from assets/showroom.json).
fn draw_furniture(row: &mut ChildSpawnerCommands, state: &forge_core::State, server: &AssetServer) {
    let room = &state.showroom;
    let next_id = state.command_id.saturating_add(1);
    row.spawn(forge_ui::panel(percent(82))).with_children(|p| {
        p.spawn(text("Display racks", 20.));
        p.spawn(text(
            format!(
                "Finished goods share the showroom's {} slots. Each rack adds slots; upgrades add slots on the same floor space. Gold {} · Timber {}",
                room.capacity(),
                state.gold,
                state.commerce.timber
            ),
            13.,
        ));
        for (i, rack) in room.racks.iter().enumerate() {
            p.spawn(Node {
                column_gap: px(12),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                forge_ui::icon(line, server, "craft", 28.);
                line.spawn(text(
                    format!(
                        "Display {} · tier {} of {} · {} slots",
                        (b'A' + i as u8) as char,
                        rack.tier,
                        room.config.max_tier(),
                        room.rack_range(i).len()
                    ),
                    14.,
                ));
                match room.upgrade_cost(i) {
                    Some((gold, timber)) => {
                        let gain = room
                            .config
                            .rack_slots
                            .get(rack.tier as usize)
                            .copied()
                            .unwrap_or(0)
                            .saturating_sub(room.rack_range(i).len() as u32);
                        button(
                            line,
                            format!("Upgrade · {gold} gold, {timber} timber · +{gain} slots"),
                            Action::Furniture(Command::UpgradeRack { rack: i }, next_id),
                        );
                    }
                    None => {
                        line.spawn(text("Top tier", 13.));
                    }
                }
            });
        }
        match room.rack_cost() {
            Some((gold, timber)) => {
                button(
                    p,
                    format!(
                        "Buy display {} · {gold} gold, {timber} timber · +{} slots",
                        (b'A' + room.racks.len() as u8) as char,
                        room.config.rack_slots.first().copied().unwrap_or(0)
                    ),
                    Action::Furniture(Command::BuyRack, next_id),
                );
            }
            None => {
                p.spawn(text(
                    format!("Installation limit reached ({} displays).", room.config.max_racks),
                    13.,
                ));
            }
        }
        p.spawn(text(
            "Prices and slot counts are draft balance values. Existing goods and reservations are never moved.",
            12.,
        ));
    });
}
fn item_icon(item: Item) -> &'static str {
    match item {
        Item::Timber => "timber",
        Item::Leather => "leather",
        Item::Staple => "staples",
        Item::Meal => "meal",
    }
}
#[allow(clippy::type_complexity)]
pub fn update(
    v: Res<Village>,
    s: Res<Session>,
    surface: Query<&ComputedNode, With<MapSurface>>,
    mut layers: Query<(&MapLayer, &mut Node, &mut Visibility)>,
    mut commands: Commands,
    buttons: Query<(
        Entity,
        &Action,
        Has<bevy::ui::InteractionDisabled>,
        Has<forge_ui::Selected>,
    )>,
    mut labels: ParamSet<(
        Query<&mut Text, With<Caption>>,
        Query<&mut Text, With<Wallet>>,
        Query<&mut Text, With<Status>>,
    )>,
) {
    let active = v.hover.or(v.nav.focus);
    for (entity, action, was_disabled, was_selected) in &buttons {
        let selected = matches!(action,Action::Visit(l) if active.is_some_and(|i|v.registry.locations[i].id==*l))
            || matches!(action,Action::Category(i) if v.category==*i)
            || matches!(action,Action::SelectItem(item) if v.selected_item.or_else(|| Item::ALL.into_iter().find(|candidate| candidate.vendor() == item.vendor()))==Some(*item));
        let disabled = matches!(action,Action::Buy(item,_) if s.disabled || s.save.state.quote(item.vendor(),*item,v.quantity).map_or(true,|p|p>s.save.state.gold))
            || matches!(action,Action::Furniture(command,_) if s.disabled || {
                let room=&s.save.state.showroom;
                let cost=match command{Command::BuyRack=>room.rack_cost(),Command::UpgradeRack{rack}=>room.upgrade_cost(*rack),_=>None};
                cost.is_none_or(|(gold,timber)|gold>s.save.state.gold||timber>s.save.state.commerce.timber)
            });
        if disabled != was_disabled {
            if disabled {
                commands.entity(entity).insert((
                    bevy::ui::InteractionDisabled,
                    forge_ui::Unavailable(
                        "Check gold, stock and storage before purchasing.".into(),
                    ),
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<(bevy::ui::InteractionDisabled, forge_ui::Unavailable)>();
            }
        }
        if selected != was_selected {
            if selected {
                commands.entity(entity).insert(forge_ui::Selected);
            } else {
                commands.entity(entity).remove::<forge_ui::Selected>();
            }
        }
    }
    if let Ok(surface) = surface.single() {
        let size = surface.size() * surface.inverse_scale_factor;
        let scale = (size.x / v.registry.width as f32).min(size.y / v.registry.height as f32);
        let draw = Vec2::new(v.registry.width as f32, v.registry.height as f32) * scale;
        for (layer, mut node, mut visible) in &mut layers {
            if node.width != px(draw.x)
                || node.height != px(draw.y)
                || node.left != px((size.x - draw.x) / 2.)
                || node.top != px((size.y - draw.y) / 2.)
            {
                node.width = px(draw.x);
                node.height = px(draw.y);
                node.left = px((size.x - draw.x) / 2.);
                node.top = px((size.y - draw.y) / 2.);
            }
            let next = if layer.0.is_none() || layer.0 == active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visible != next {
                *visible = next;
            }
        }
    }
    for mut t in &mut labels.p0() {
        let value = active
            .map(|i| format!("> {}  /  Enter to visit", v.registry.locations[i].label))
            .unwrap_or("Arrows / Tab: focus * 1-8: visit\nThe central well is a landmark.".into());
        if **t != value {
            **t = value;
        }
    }
    for mut t in &mut labels.p1() {
        let value = format!(
            "Gold {}  |  Timber {}  |  Leather {}  |  F {}%  H {}%  T {}%",
            s.save.state.gold,
            s.save.state.commerce.timber,
            s.save.state.commerce.leather,
            s.save.state.life.fatigue / 100,
            s.save.state.life.hunger / 100,
            s.save.state.life.thirst / 100
        );
        if **t != value {
            **t = value;
        }
    }
    for mut t in &mut labels.p2() {
        let value = s.status.split(". C craft").next().unwrap_or(&s.status);
        if **t != value {
            **t = value.to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_scale_does_not_invalidate_ui_every_frame() {
        #[derive(Resource, Default)]
        struct Changes(Vec<bool>);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<UiScale>();
        app.init_resource::<Changes>();
        app.insert_resource(Desktop::new(false, true));
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (960, 540).into(),
                    ..default()
                },
                bevy::window::PrimaryWindow,
            ))
            .id();
        app.add_systems(
            Update,
            (
                fit_ui,
                |scale: Res<UiScale>, mut changes: ResMut<Changes>| {
                    changes.0.push(scale.is_changed());
                },
            )
                .chain(),
        );
        app.update();
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Changes>().0, [true, false, false]);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(640., 368.);
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<Changes>().0,
            [true, false, false, true, false]
        );
        assert!((app.world().resource::<UiScale>().0 - 2. / 3.).abs() < 0.0001);
    }
    #[test]
    fn actual_observer_map_shop_buy_duplicate_back_and_hidden_guard() {
        let dir = std::env::temp_dir().join(format!("village-ui-{}", std::process::id()));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut state = forge_core::State::new(1);
        state.progression.legacy_dagger = true;
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
        app.init_resource::<crate::npc::FarmerPresentation>();
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
        let farm = app.world_mut().spawn(Action::Visit(Location::Farm)).id();
        let food = app.world_mut().spawn(Action::Buy(Item::Staple, 3)).id();
        let before = app
            .world()
            .resource::<Session>()
            .save
            .state
            .life
            .pantry
            .staples();
        app.world_mut().trigger(Activate { entity: farm });
        app.world_mut().trigger(Activate { entity: food });
        assert!(
            app.world()
                .resource::<crate::npc::FarmerPresentation>()
                .pending
        );
        assert_eq!(
            app.world()
                .resource::<Session>()
                .save
                .state
                .life
                .pantry
                .staples(),
            before + 1
        );
        app.world_mut()
            .resource_mut::<crate::npc::FarmerPresentation>()
            .pending = false;
        app.world_mut().trigger(Activate { entity: food });
        assert!(
            !app.world()
                .resource::<crate::npc::FarmerPresentation>()
                .pending
        );
        // Carpenter furniture: one rack per rendered ID; a replay cannot buy twice.
        app.world_mut().resource_mut::<Session>().save.state.gold = 100;
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .commerce
            .timber = 10;
        app.world_mut().trigger(Activate { entity: shop });
        app.world_mut().resource_mut::<Village>().category = 1;
        let next = app.world().resource::<Session>().save.state.command_id + 1;
        let rack = app
            .world_mut()
            .spawn(Action::Furniture(Command::BuyRack, next))
            .id();
        app.world_mut().trigger(Activate { entity: rack });
        app.world_mut().trigger(Activate { entity: rack });
        let st = &app.world().resource::<Session>().save.state;
        assert_eq!(
            (
                st.showroom.racks.len(),
                st.showroom.capacity(),
                st.gold,
                st.commerce.timber
            ),
            (2, 8, 70, 6)
        );
        let upgrade = app
            .world_mut()
            .spawn(Action::Furniture(
                Command::UpgradeRack { rack: 0 },
                next + 1,
            ))
            .id();
        app.world_mut().trigger(Activate { entity: upgrade });
        let st = &app.world().resource::<Session>().save.state;
        assert_eq!(
            (st.showroom.racks[0].tier, st.showroom.capacity(), st.gold),
            (2, 10, 50)
        );
        app.world_mut().resource_mut::<Village>().category = 0;
        app.world_mut().trigger(Activate { entity: back });
        app.world_mut().trigger(Activate { entity: close });
        assert!(!app.world().resource::<Desktop>().shell.expanded);
        let old = app.world().resource::<Session>().save.state.clone();
        app.world_mut().trigger(Activate { entity: buy });
        assert_eq!(app.world().resource::<Session>().save.state, old);
        // Even a direct observer event cannot purchase through an active modal.
        app.world_mut().resource_mut::<Desktop>().shell.expanded = true;
        app.world_mut().trigger(Activate { entity: farm });
        let mut dialog = forge_ui::DialogState::default();
        dialog.open("quit", "Quit?", "", None);
        app.insert_resource(dialog);
        let next = app.world().resource::<Session>().save.state.command_id + 1;
        let blocked = app.world_mut().spawn(Action::Buy(Item::Staple, next)).id();
        app.world_mut().trigger(Activate { entity: blocked });
        assert_eq!(app.world().resource::<Session>().save.state, old);
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
