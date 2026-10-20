//! Orders (#109) and Weapon Collection (#115) in the 960x360 management host, drawn with kit v2.
//!
//! Every value on screen is read from `forge_core::State`; buttons issue the same domain commands
//! as the older panels (through `lifestyle::Action`). View models are pure functions of the state so
//! they are tested without a window.
use crate::forge_view::{Host, HostRoot, HostScreen};
use crate::lifestyle::Action;
use crate::text::{self, Lang, T};
use crate::{Session, forge_view};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use forge_core::{Command, State, commission::OrderPhase, expedition::Gear, items::Id};
use forge_ui::kit2::{self, Kind};

// ---- view models ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderId {
    Chef,
    Farmer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Due {
    None,
    AfterAccept(u64),
    Left(u64),
    Overdue,
    OnTime,
    Late,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderAction {
    pub label: T,
    pub command: Command,
    pub primary: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderView {
    pub id: OrderId,
    pub npc: &'static str,
    pub portrait: Rect,
    pub requester: T,
    pub title: T,
    pub item: T,
    pub item_icon: &'static str,
    pub quote: T,
    pub phase: OrderPhase,
    pub gold: u64,
    pub favor: Option<u8>,
    pub due: Due,
    /// Inputs the order consumes when queued: (item, have, need).
    pub needs: Vec<(Id, u32, u32)>,
    /// Recipes the player can queue now to make missing parts (explicit buttons, never automatic).
    pub steps: Vec<Id>,
    pub work_secs: u64,
    pub actions: Vec<OrderAction>,
}

fn act(label: T, command: Command, primary: bool) -> OrderAction {
    OrderAction {
        label,
        command,
        primary,
    }
}

/// The Chef's knife is assembled from the chef recipe's parts (schema 8).
fn chef_needs(s: &State) -> Vec<(Id, u32, u32)> {
    let book = forge_core::crafting::Book::bundled();
    book.recipe(book.chef_order_recipe)
        .map(|r| {
            r.inputs
                .iter()
                .map(|i| (i.item, s.stock(i.item), i.qty))
                .collect()
        })
        .unwrap_or_default()
}

/// Part recipes still useful for the Chef's knife: one per missing part, plus smelting while a
/// blade is missing and no ingot is on hand. Whether each can run now is the command's own check.
fn part_steps(s: &State) -> Vec<Id> {
    let book = forge_core::crafting::Book::bundled();
    let mut steps = vec![];
    for (item, have, need) in chef_needs(s) {
        if have >= need {
            continue;
        }
        if let Some(r) = book.recipes.iter().find(|r| r.output.item == item) {
            for input in &r.inputs {
                if s.stock(input.item) < input.qty
                    && let Some(sub) = book.recipes.iter().find(|x| x.output.item == input.item)
                {
                    steps.push(sub.id);
                }
            }
            steps.push(r.id);
        }
    }
    steps
}

pub(crate) fn item_name(l: Lang, id: Id) -> String {
    forge_core::items::Registry::bundled().get(id).map_or_else(
        || id.to_string(),
        |d| match l {
            Lang::Ko => d.name_ko.clone(),
            Lang::En => d.name_en.clone(),
        },
    )
}
pub(crate) fn item_icon(id: Id) -> String {
    forge_core::items::Registry::bundled()
        .get(id)
        .map_or_else(String::new, |d| d.icon.clone())
}

/// The orders that exist in the current state, in a stable order.
pub fn orders(s: &State) -> Vec<OrderView> {
    let mut out = vec![];
    if let Some(rules) = &s.progression.rules {
        let phase = s.progression.chef_phase;
        let actions = match phase {
            OrderPhase::Offered => vec![act(text::ACCEPT, Command::AcceptChef, true)],
            OrderPhase::Accepted => vec![
                act(text::QUEUE, Command::QueueChef, true),
                act(text::BUY_ORE, Command::BuyOre, false),
            ],
            OrderPhase::Ready => vec![act(text::DELIVER, Command::DeliverChef, true)],
            _ => vec![],
        };
        out.push(OrderView {
            id: OrderId::Chef,
            npc: "chef",
            portrait: Rect::new(30., 4., 78., 52.),
            requester: text::CHEF,
            title: text::CHEF_TITLE,
            item: text::CHEF_ITEM,
            item_icon: "items/finished/icons/kitchen_knife.png",
            quote: text::CHEF_QUOTE,
            phase,
            gold: rules.chef.reward_gold,
            favor: None,
            due: Due::None,
            needs: chef_needs(s),
            steps: if phase == OrderPhase::Accepted {
                part_steps(s)
            } else {
                vec![]
            },
            work_secs: rules.recipes[0].seconds,
            actions,
        });
    }
    {
        use forge_core::commission::*;
        let o = &s.farmer_order;
        let (gold, favor) = match o.phase {
            OrderPhase::Delivered => (o.settled_gold, Some(o.favor).filter(|f| *f > 0)),
            _ if o.completed_at.is_some() => {
                let (g, f) = o.reward();
                (g, Some(f).filter(|f| *f > 0))
            }
            _ => (REWARD_GOLD, Some(REWARD_FAVOR)),
        };
        let due = match o.deadline_at {
            None => Due::AfterAccept(DEADLINE_SECS),
            Some(_) if o.completed_at.is_some() => {
                if o.on_time() {
                    Due::OnTime
                } else {
                    Due::Late
                }
            }
            Some(at) if at > s.tick => Due::Left(at - s.tick),
            Some(_) => Due::Overdue,
        };
        let actions = match o.phase {
            OrderPhase::Offered => vec![act(text::ACCEPT, Command::AcceptFarmerRepair, true)],
            OrderPhase::Accepted => vec![
                act(text::QUEUE, Command::QueueFarmerRepair, true),
                act(text::BUY_ORE, Command::BuyOre, false),
            ],
            OrderPhase::Ready => vec![act(text::DELIVER, Command::DeliverFarmerRepair, true)],
            _ => vec![],
        };
        out.push(OrderView {
            id: OrderId::Farmer,
            npc: "farmer",
            portrait: Rect::new(21., 4., 69., 52.),
            requester: text::FARMER,
            title: text::FARMER_TITLE,
            item: text::FARMER_ITEM,
            item_icon: "items/finished/icons/sickle.png",
            quote: text::FARMER_QUOTE,
            phase: o.phase,
            gold,
            favor,
            due,
            needs: vec![(forge_core::items::IRON_ORE, s.ore(), REPAIR_ORE)],
            steps: vec![],
            work_secs: REPAIR_SECS,
            actions,
        });
    }
    out
}

/// The order the detail panel shows: the chosen one if it is in the current filter, else the first.
pub fn selected_order(host: &Host, s: &State) -> Option<OrderId> {
    let shown: Vec<_> = orders(s)
        .into_iter()
        .filter(|o| in_filter(o, host.order_filter))
        .map(|o| o.id)
        .collect();
    host.order
        .filter(|id| shown.contains(id))
        .or(shown.first().copied())
}

/// The weapon the detail panel shows: the chosen one if it is in the family, else the first found.
pub fn selected_weapon(host: &Host, s: &State) -> Option<usize> {
    let shown: Vec<usize> = (0..WEAPONS.len())
        .filter(|i| in_family(WEAPONS[*i].0, host.family))
        .collect();
    host.weapon
        .filter(|i| shown.contains(i))
        .or_else(|| {
            shown
                .iter()
                .copied()
                .find(|i| record(s, WEAPONS[*i].0).found)
        })
        .or(shown.first().copied())
}

pub const ORDER_FILTERS: [T; 3] = [text::ALL, text::ACTIVE, text::DONE];
pub fn in_filter(o: &OrderView, filter: usize) -> bool {
    match filter {
        1 => o.phase != OrderPhase::Delivered,
        2 => o.phase == OrderPhase::Delivered,
        _ => true,
    }
}

/// Weapon catalog for the collection: id, family. Icons are the finished-goods icons; the
/// shortbow uses the hunter's bow layer.
pub const WEAPONS: [(&str, &str); 10] = [
    ("dagger", "dagger"),
    ("hunting_knife", "dagger"),
    ("hand_axe", "axe"),
    ("spear", "polearm"),
    ("pike", "polearm"),
    ("halberd", "polearm"),
    ("wooden_club", "blunt"),
    ("wooden_staff", "blunt"),
    ("wooden_shield", "shield"),
    ("shortbow", "bow"),
];
pub const FAMILIES: [&str; 6] = ["dagger", "axe", "polearm", "blunt", "shield", "bow"];
const BOW: (&str, Rect) = (
    "npc/hunter/equipment/bow.png",
    Rect {
        min: Vec2::new(28., 47.),
        max: Vec2::new(45., 78.),
    },
);

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WeaponRecord {
    pub found: bool,
    /// The game can make it today (otherwise it is only in the item catalog).
    pub craftable: bool,
    pub owned: u32,
    pub made: Option<u32>,
    pub sold: Option<u32>,
    pub best_quality: Option<u8>,
    pub appearance: Option<forge_core::weapon::Appearance>,
    pub field_uses: u32,
    pub kills: u32,
    pub best_wave: u8,
    pub ore: Option<u32>,
}

pub fn record(s: &State, id: &str) -> WeaponRecord {
    let gear = match id {
        "hunting_knife" => Some(Gear::HuntingKnife),
        "shortbow" => Some(Gear::Shortbow),
        _ => None,
    };
    if let Some(g) = gear {
        let ws: Vec<_> = s
            .expedition
            .weapons
            .iter()
            .filter(|w| w.kind == g)
            .collect();
        let best = ws
            .iter()
            .filter_map(|w| w.craftsmanship.as_ref())
            .max_by_key(|c| c.quality);
        return WeaponRecord {
            found: !ws.is_empty(),
            craftable: true,
            owned: ws.len() as u32,
            best_quality: best.map(|c| c.quality),
            appearance: best.map(|c| c.appearance),
            ore: best.map(|c| c.ore),
            field_uses: ws.iter().map(|w| w.field_uses).sum(),
            kills: ws.iter().map(|w| w.kills).sum(),
            best_wave: ws.iter().map(|w| w.highest_wave).max().unwrap_or(0),
            ..default()
        };
    }
    if id == "dagger" {
        return WeaponRecord {
            found: s.crafted > 0 || s.daggers > 0,
            craftable: true,
            owned: s.daggers,
            made: Some(s.crafted),
            sold: Some(s.sold),
            ..default()
        };
    }
    WeaponRecord::default()
}

pub fn in_family(id: &str, family: usize) -> bool {
    family == 0
        || WEAPONS
            .iter()
            .any(|(w, f)| *w == id && FAMILIES.get(family - 1) == Some(f))
}

fn stars(a: forge_core::weapon::Appearance) -> u8 {
    use forge_core::weapon::Appearance::*;
    match a {
        Common => 1,
        Refined => 2,
        Masterwork => 3,
    }
}

// ---- navigation ---------------------------------------------------------------------------

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nav {
    Screen(HostScreen),
    OrderFilter(usize),
    Order(OrderId),
    Family(usize),
    Weapon(usize),
    CraftTab(usize),
    Recipe(Id),
}

pub fn navigate(
    e: On<Activate>,
    navs: Query<&Nav>,
    mut host: ResMut<Host>,
    dialog: Res<forge_ui::DialogState>,
) {
    if dialog.is_open() {
        return;
    }
    let Ok(nav) = navs.get(e.entity) else { return };
    match *nav {
        Nav::Screen(screen) => host.screen = screen,
        Nav::OrderFilter(f) => host.order_filter = f,
        Nav::Order(id) => host.order = Some(id),
        Nav::Family(f) => {
            host.family = f;
            host.weapon = None;
        }
        Nav::Weapon(i) => host.weapon = Some(i),
        Nav::CraftTab(t) => {
            host.craft_tab = t;
            host.recipe = None;
        }
        Nav::Recipe(id) => host.recipe = Some(id),
    }
}

// ---- drawing ------------------------------------------------------------------------------

#[derive(Component)]
pub struct ScreenPage;
#[derive(Component)]
pub struct HomePage;
/// Countdown text patched in place so a ticking deadline never rebuilds the screen.
#[derive(Component)]
pub struct DueText(OrderId);

fn due_text(l: Lang, d: &Due) -> String {
    match d {
        Due::None => text::NO_DEADLINE.get(l).into(),
        Due::AfterAccept(s) => text::after_accept(l, *s),
        Due::Left(s) => text::left(l, *s),
        Due::Overdue => text::OVERDUE.get(l).into(),
        Due::OnTime => text::ON_TIME.get(l).into(),
        Due::Late => text::LATE.get(l).into(),
    }
}
fn due_urgent(d: &Due) -> bool {
    matches!(d, Due::Left(_) | Due::Overdue)
}

pub fn show_pages(
    host: Res<Host>,
    mut home: Query<&mut Node, (With<HomePage>, Without<ScreenPage>)>,
    mut page: Query<&mut Node, (With<ScreenPage>, Without<HomePage>)>,
) {
    let on = host.screen != HostScreen::Home;
    for mut n in &mut home {
        let d = if on { Display::None } else { Display::Flex };
        if n.display != d {
            n.display = d;
        }
    }
    for mut n in &mut page {
        let d = if on { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
    }
}

pub fn live_due(s: Res<Session>, lang: Res<Lang>, mut q: Query<(&DueText, &mut Text)>) {
    if q.is_empty() {
        return;
    }
    let views = orders(&s.save.state);
    for (due, mut t) in &mut q {
        if let Some(v) = views.iter().find(|v| v.id == due.0) {
            let next = due_text(*lang, &v.due);
            if t.0 != next {
                t.0 = next;
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn draw(
    mut c: Commands,
    host: Res<Host>,
    s: Res<Session>,
    lang: Res<Lang>,
    server: Res<AssetServer>,
    page: Query<Entity, With<ScreenPage>>,
    detail: Query<Entity, With<DetailBody>>,
    cards: Query<(Entity, &Nav, Has<forge_ui::Selected>)>,
    mut last: Local<(String, String)>,
) {
    let Ok(page) = page.single() else { return };
    let st = &s.save.state;
    // Whole-screen key: what the rail and lists show. Selection is kept out of it so choosing a
    // row or card only redraws the detail panel and moves the gold rim.
    let page_key = format!(
        "{:?}:{}:{:?}:{}:{:?}:{}:{}:{:?}:{}:{}:{}:{}:{}:{:?}:{:?}:{}",
        st.inventory,
        st.life.queue.len(),
        st.life.active_job,
        host.craft_tab,
        host.screen,
        host.order_filter,
        host.family,
        *lang,
        st.command_id,
        st.ore(),
        st.gold,
        st.crafted,
        st.expedition.weapons.len(),
        st.progression.chef_phase,
        st.farmer_order.phase,
        st.expedition
            .weapons
            .iter()
            .map(|w| w.field_uses + w.kills)
            .sum::<u32>(),
    );
    let order = selected_order(&host, st);
    let weapon = selected_weapon(&host, st);
    let recipe = crate::crafting_screen::selected(&host);
    let sel_key = format!("{order:?}:{weapon:?}:{:?}", recipe.map(|r| r.id));
    let l = *lang;
    if last.0 != page_key {
        last.0 = page_key;
        last.1 = sel_key;
        c.entity(page).despawn_children();
        c.entity(page).with_children(|p| match host.screen {
            HostScreen::Home => {}
            HostScreen::Orders => draw_orders(p, &server, &host, st, l),
            HostScreen::Collection => draw_collection(p, &server, &host, st, l),
            HostScreen::Crafting => crate::crafting_screen::draw(p, &server, &host, st, l),
        });
        return;
    }
    if last.1 == sel_key {
        return;
    }
    last.1 = sel_key;
    for (e, nav, selected) in &cards {
        let want = match nav {
            Nav::Order(id) => Some(order == Some(*id)),
            Nav::Weapon(i) => Some(weapon == Some(*i)),
            Nav::Recipe(id) => Some(recipe.is_some_and(|r| r.id == *id)),
            _ => None,
        };
        match want {
            Some(true) if !selected => {
                c.entity(e).insert(forge_ui::Selected);
            }
            Some(false) if selected => {
                c.entity(e).remove::<forge_ui::Selected>();
            }
            _ => {}
        }
    }
    for d in &detail {
        c.entity(d).despawn_children();
        c.entity(d)
            .with_children(|d| fill_detail(d, &server, &host, st, l));
    }
}

/// Right-hand panel contents for the current screen and selection.
fn fill_detail(
    d: &mut ChildSpawnerCommands,
    server: &AssetServer,
    host: &Host,
    s: &State,
    l: Lang,
) {
    match host.screen {
        HostScreen::Orders => {
            if let Some(o) =
                selected_order(host, s).and_then(|id| orders(s).into_iter().find(|o| o.id == id))
            {
                order_detail(d, server, &o, s, l);
            }
        }
        HostScreen::Collection => {
            if let Some(i) = selected_weapon(host, s) {
                let (id, family) = WEAPONS[i];
                weapon_detail(d, server, id, family, &record(s, id), l);
            }
        }
        HostScreen::Crafting => {
            if let Some(r) = crate::crafting_screen::selected(host) {
                crate::crafting_screen::detail(d, server, r, s, l);
            }
        }
        HostScreen::Home => {}
    }
}

#[derive(Component)]
pub struct DetailBody;

fn column(width: Option<f32>) -> Node {
    Node {
        width: width.map_or(Val::Auto, px),
        flex_grow: if width.is_some() { 0. } else { 1. },
        flex_shrink: if width.is_some() { 0. } else { 1. },
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

pub(crate) fn parchment(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    width: Option<f32>,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    let mut node = column(width);
    node.height = percent(100);
    node.padding = UiRect::new(px(16), px(16), px(12), px(12));
    node.row_gap = px(6);
    node.overflow = Overflow::clip();
    p.spawn(kit2::frame(server, "panel-parchment", node))
        .with_children(body)
        .id()
}

/// Left rail: one tab per entry, the selected one lit, a Back button at the bottom.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rail(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    l: Lang,
    tabs: &[(String, String)],
    selected: usize,
    tab_h: f32,
    nav: impl Fn(usize) -> Nav,
    footer: impl FnOnce(&mut ChildSpawnerCommands),
) {
    p.spawn(Node {
        width: px(172),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        row_gap: px(4),
        flex_shrink: 0.,
        ..default()
    })
    .with_children(|r| {
        for (i, (name, count)) in tabs.iter().enumerate() {
            let sel = i == selected;
            let e = kit2::control(
                r,
                server,
                Kind::Tab,
                sel,
                format!("rail:{name}"),
                Node {
                    width: percent(100),
                    height: px(tab_h),
                    padding: UiRect::axes(px(14), px(0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    flex_shrink: 0.,
                    ..default()
                },
                nav(i),
            );
            r.commands().entity(e).with_children(|t| {
                t.spawn(kit2::strong(name.clone(), 14., kit2::INK_WOOD));
                t.spawn(kit2::label(
                    count.clone(),
                    12.,
                    if sel {
                        kit2::INK_WOOD
                    } else {
                        kit2::INK_WOOD_MUTED
                    },
                ));
            });
        }
        r.spawn(Node {
            flex_grow: 1.,
            ..default()
        });
        footer(r);
        kit2::button(
            r,
            server,
            Kind::Plain,
            text::BACK.get(l),
            172.,
            Nav::Screen(HostScreen::Home),
        );
    });
}

pub(crate) fn screen_root(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    p.spawn(kit2::frame(
        server,
        "window-frame",
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(14)),
            column_gap: px(6),
            ..default()
        },
    ))
    .with_children(body);
}

pub(crate) fn row(gap: f32) -> Node {
    Node {
        column_gap: px(gap),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn draw_orders(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    host: &Host,
    s: &State,
    l: Lang,
) {
    let all = orders(s);
    let shown: Vec<_> = all
        .iter()
        .filter(|o| in_filter(o, host.order_filter))
        .collect();
    let selected =
        selected_order(host, s).and_then(|id| shown.iter().find(|o| o.id == id).copied());
    screen_root(p, server, |p| {
        let tabs: Vec<(String, String)> = ORDER_FILTERS
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let n = all.iter().filter(|o| in_filter(o, i)).count();
                (t.get(l).to_string(), n.to_string())
            })
            .collect();
        rail(
            p,
            server,
            l,
            &tabs,
            host.order_filter,
            52.,
            Nav::OrderFilter,
            |_| {},
        );
        parchment(p, server, Some(392.), |list| {
            list.spawn(kit2::strong(text::ORDER_LIST.get(l), 17., kit2::INK));
            kit2::divider(list, server);
            if shown.is_empty() {
                list.spawn(kit2::label(text::NO_ORDERS.get(l), 13., kit2::INK_MUTED));
            }
            for o in &shown {
                order_row(list, server, o, selected.is_some_and(|x| x.id == o.id), l);
            }
        });
        let d = parchment(p, server, None, |d| fill_detail(d, server, host, s, l));
        p.commands().entity(d).insert(DetailBody);
    });
}

fn portrait(p: &mut ChildSpawnerCommands, server: &AssetServer, o: &OrderView, size: f32) {
    let path = format!("npc/{}/neutral.png", o.npc);
    kit2::slot(
        p,
        server,
        "normal",
        size,
        Some((&path, Some(o.portrait))),
        1.,
    );
}

fn order_row(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    o: &OrderView,
    selected: bool,
    l: Lang,
) {
    let e = kit2::control(
        p,
        server,
        Kind::Card,
        selected,
        format!("order:{:?}", o.id),
        Node {
            width: percent(100),
            height: px(80),
            padding: UiRect::all(px(8)),
            column_gap: px(10),
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        },
        Nav::Order(o.id),
    );
    p.commands().entity(e).with_children(|r| {
        portrait(r, server, o, 56.);
        r.spawn(Node {
            flex_grow: 1.,
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(|c| {
            c.spawn(row(6.)).with_children(|t| {
                t.spawn(kit2::strong(o.title.get(l), 15., kit2::INK));
                kit2::tag(t, server, "daily", text::DAILY.get(l));
            });
            c.spawn(kit2::label(
                format!("{} · {}", o.requester.get(l), text::phase(l, o.phase)),
                11.,
                kit2::INK_MUTED,
            ));
            c.spawn(row(4.)).with_children(|t| {
                t.spawn(kit2::glyph(server, "glyph-hourglass", 8., 9.));
                t.spawn((
                    kit2::label(
                        due_text(l, &o.due),
                        11.,
                        if due_urgent(&o.due) {
                            kit2::DEADLINE
                        } else {
                            kit2::INK_MUTED
                        },
                    ),
                    DueText(o.id),
                ));
            });
        });
        r.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|c| {
            c.spawn(row(3.)).with_children(|t| {
                t.spawn(kit2::glyph(server, "glyph-coin", 10., 10.));
                t.spawn(kit2::label(o.gold.to_string(), 12., kit2::INK));
            });
            kit2::slot(c, server, "normal", 44., Some((o.item_icon, None)), 1.);
        });
    });
}

fn order_detail(
    d: &mut ChildSpawnerCommands,
    server: &AssetServer,
    o: &OrderView,
    s: &State,
    l: Lang,
) {
    d.spawn(row(8.)).with_children(|t| {
        t.spawn(kit2::strong(o.title.get(l), 18., kit2::INK));
        kit2::tag(t, server, "daily", text::DAILY.get(l));
    });
    kit2::divider(d, server);
    d.spawn(Node {
        column_gap: px(10),
        ..default()
    })
    .with_children(|r| {
        portrait(r, server, o, 60.);
        r.spawn(Node {
            flex_grow: 1.,
            min_width: px(0),
            ..default()
        })
        .with_children(|q| {
            q.spawn(kit2::label(o.quote.get(l), 12., kit2::INK_MUTED));
        });
    });
    d.spawn(Node {
        column_gap: px(10),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|r| {
        r.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(2),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|c| {
            kit2::slot(c, server, "selected", 48., Some((o.item_icon, None)), 1.);
            c.spawn(kit2::label(o.item.get(l), 11., kit2::INK));
        });
        r.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        })
        .with_children(|c| {
            let reward = match o.favor {
                Some(f) => format!("{} · {}", text::gold(l, o.gold), text::favor(l, f)),
                None => text::gold(l, o.gold),
            };
            for (k, v, col) in [
                (text::STATUS, text::phase(l, o.phase).to_string(), kit2::INK),
                (
                    text::DEADLINE,
                    due_text(l, &o.due),
                    if due_urgent(&o.due) {
                        kit2::DEADLINE
                    } else {
                        kit2::INK
                    },
                ),
                (text::REWARD, reward, kit2::INK),
                (text::WORK, text::seconds(l, o.work_secs), kit2::INK),
            ] {
                c.spawn(row(10.)).with_children(|t| {
                    t.spawn((
                        kit2::label(k.get(l), 12., kit2::INK_MUTED),
                        Node {
                            width: px(64),
                            ..default()
                        },
                    ));
                    let mut e = t.spawn(kit2::label(v, 12., col));
                    if k == text::DEADLINE {
                        e.insert(DueText(o.id));
                    }
                });
            }
        });
    });
    kit2::divider(d, server);
    d.spawn(row(8.)).with_children(|r| {
        r.spawn(kit2::label(text::NEEDS.get(l), 12., kit2::INK));
        let open = matches!(o.phase, OrderPhase::Offered | OrderPhase::Accepted);
        for &(item, have, need) in &o.needs {
            kit2::slot(r, server, "normal", 40., Some((&item_icon(item), None)), 1.);
            r.spawn(kit2::label(
                format!("{}\n{}", item_name(l, item), text::have_need(l, have, need)),
                11.,
                if have >= need || !open {
                    kit2::INK
                } else {
                    kit2::DEADLINE
                },
            ));
        }
    });
    if !o.steps.is_empty() {
        d.spawn(Node {
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|r| {
            r.spawn(kit2::label(text::MAKE.get(l), 11., kit2::INK_MUTED));
            for &step in &o.steps {
                let out = forge_core::crafting::Book::bundled()
                    .recipe(step)
                    .map(|x| x.output.item)
                    .unwrap_or(step);
                let command = Command::QueueRecipe(step);
                let e = kit2::button(
                    r,
                    server,
                    Kind::Plain,
                    item_name(l, out),
                    92.,
                    Action(command),
                );
                if let Err(err) = s.clone().command(s.command_id.saturating_add(1), command) {
                    r.commands().entity(e).insert((
                        bevy::ui::InteractionDisabled,
                        forge_ui::Unavailable(text::error(l, err).into()),
                    ));
                }
            }
        });
    }
    d.spawn(Node {
        flex_grow: 1.,
        ..default()
    });
    match o.phase {
        OrderPhase::Queued => {
            d.spawn(kit2::label(text::IN_QUEUE.get(l), 12., kit2::INK_MUTED));
        }
        OrderPhase::Working => {
            d.spawn(kit2::label(text::AT_BENCH.get(l), 12., kit2::INK_MUTED));
        }
        OrderPhase::Delivered => {
            d.spawn(kit2::label(text::THANKS.get(l), 12., kit2::INK_MUTED));
        }
        _ => {}
    }
    d.spawn(Node {
        column_gap: px(4),
        ..default()
    })
    .with_children(|r| {
        for a in &o.actions {
            let e = kit2::button(
                r,
                server,
                if a.primary {
                    Kind::Primary
                } else {
                    Kind::Plain
                },
                a.label.get(l),
                if a.primary { 140. } else { 120. },
                Action(a.command),
            );
            // Same eligibility check the command runs; a refused button is disabled with its reason.
            if let Err(err) = s.clone().command(s.command_id.saturating_add(1), a.command) {
                r.commands().entity(e).insert((
                    bevy::ui::InteractionDisabled,
                    forge_ui::Unavailable(text::error(l, err).into()),
                ));
            }
        }
    });
}

fn weapon_icon(id: &str, found: bool) -> (String, Option<Rect>) {
    match (id, found) {
        ("shortbow", true) => (BOW.0.into(), Some(BOW.1)),
        (_, true) => (format!("items/finished/icons/{id}.png"), None),
        (_, false) => (format!("items/finished/silhouettes/{id}.png"), None),
    }
}

fn draw_collection(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    host: &Host,
    s: &State,
    l: Lang,
) {
    let records: Vec<_> = WEAPONS.iter().map(|(id, _)| record(s, id)).collect();
    let count = |family: usize| {
        let ids: Vec<_> = WEAPONS
            .iter()
            .enumerate()
            .filter(|(_, (id, _))| in_family(id, family))
            .collect();
        let found = ids.iter().filter(|(i, _)| records[*i].found).count();
        format!("{found} / {}", ids.len())
    };
    let shown: Vec<usize> = (0..WEAPONS.len())
        .filter(|i| in_family(WEAPONS[*i].0, host.family))
        .collect();
    let selected = selected_weapon(host, s);
    screen_root(p, server, |p| {
        let mut tabs = vec![(text::ALL_WEAPONS.get(l).to_string(), count(0))];
        tabs.extend(
            FAMILIES
                .iter()
                .enumerate()
                .map(|(i, f)| (text::family(l, f).to_string(), count(i + 1))),
        );
        rail(p, server, l, &tabs, host.family, 38., Nav::Family, |_| {});
        parchment(p, server, Some(392.), |g| {
            g.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|h| {
                h.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|t| {
                    t.spawn(kit2::strong(tabs[host.family].0.clone(), 17., kit2::INK));
                    t.spawn(kit2::label(
                        text::COLLECTION_HINT.get(l),
                        11.,
                        kit2::INK_MUTED,
                    ));
                });
                h.spawn(kit2::label(
                    format!("{}  {}", text::PROGRESS.get(l), tabs[host.family].1),
                    12.,
                    kit2::INK,
                ));
            });
            kit2::divider(g, server);
            g.spawn(Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(4),
                row_gap: px(4),
                ..default()
            })
            .with_children(|grid| {
                for &i in &shown {
                    let (id, _) = WEAPONS[i];
                    let r = &records[i];
                    let e = kit2::control(
                        grid,
                        server,
                        Kind::Card,
                        selected == Some(i),
                        format!("weapon:{id}"),
                        Node {
                            width: px(68),
                            height: px(112),
                            padding: UiRect::top(px(8)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(6),
                            ..default()
                        },
                        Nav::Weapon(i),
                    );
                    grid.commands().entity(e).with_children(|card| {
                        let (path, rect) = weapon_icon(id, r.found);
                        kit2::slot(
                            card,
                            server,
                            if r.found { "normal" } else { "locked" },
                            48.,
                            Some((&path, rect)),
                            1.,
                        );
                        card.spawn((
                            kit2::label(
                                if r.found {
                                    text::weapon_name(l, id)
                                } else {
                                    text::UNKNOWN.get(l)
                                },
                                11.,
                                if r.found { kit2::INK } else { kit2::INK_MUTED },
                            ),
                            TextLayout::justify(Justify::Center),
                        ));
                        if let Some(a) = r.appearance {
                            kit2::stars(card, server, stars(a));
                        }
                    });
                }
            });
        });
        let d = parchment(p, server, None, |d| fill_detail(d, server, host, s, l));
        p.commands().entity(d).insert(DetailBody);
    });
}

fn weapon_detail(
    d: &mut ChildSpawnerCommands,
    server: &AssetServer,
    id: &str,
    family: &str,
    r: &WeaponRecord,
    l: Lang,
) {
    d.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|h| {
        h.spawn(kit2::strong(
            if r.found {
                text::weapon_name(l, id)
            } else {
                text::UNKNOWN.get(l)
            },
            18.,
            kit2::INK,
        ));
        kit2::tag(h, server, "rank", text::family(l, family));
    });
    kit2::divider(d, server);
    d.spawn(Node {
        column_gap: px(12),
        ..default()
    })
    .with_children(|r0| {
        let (path, rect) = weapon_icon(id, r.found);
        kit2::slot(
            r0,
            server,
            if r.found { "normal" } else { "locked" },
            84.,
            Some((&path, rect)),
            2.,
        );
        r0.spawn(Node {
            flex_grow: 1.,
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .with_children(|c| {
            if let Some(a) = r.appearance {
                c.spawn(row(6.)).with_children(|t| {
                    kit2::stars(t, server, stars(a));
                    t.spawn(kit2::label(text::appearance(l, a), 12., kit2::INK));
                });
            }
            let blurb = if !r.found && !r.craftable {
                text::NOT_IN_GAME.get(l)
            } else if !r.found {
                text::UNDISCOVERED.get(l)
            } else {
                text::weapon_blurb(l, id)
            };
            c.spawn(kit2::label(blurb, 12., kit2::INK_MUTED));
        });
    });
    if !r.found {
        return;
    }
    kit2::divider(d, server);
    let mut facts: Vec<(T, String)> = vec![(text::OWNED, r.owned.to_string())];
    if let Some(m) = r.made {
        facts.push((text::MADE, m.to_string()));
    }
    if let Some(sold) = r.sold {
        facts.push((text::SOLD, sold.to_string()));
    }
    if let Some(q) = r.best_quality {
        facts.push((text::BEST_QUALITY, q.to_string()));
    }
    if let Some(o) = r.ore {
        facts.push((text::ORE_USED, o.to_string()));
    }
    if r.made.is_none() {
        facts.push((text::FIELD_USES, r.field_uses.to_string()));
        facts.push((text::KILLS, r.kills.to_string()));
        facts.push((text::BEST_WAVE, r.best_wave.to_string()));
    }
    d.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        row_gap: px(4),
        ..default()
    })
    .with_children(|g| {
        for (k, v) in facts {
            g.spawn(Node {
                width: percent(50),
                column_gap: px(8),
                ..default()
            })
            .with_children(|t| {
                t.spawn((
                    kit2::label(k.get(l), 12., kit2::INK_MUTED),
                    Node {
                        width: px(72),
                        ..default()
                    },
                ));
                t.spawn(kit2::label(v, 12., kit2::INK));
            });
        }
    });
    if id == "hunting_knife" {
        d.spawn(Node {
            flex_grow: 1.,
            ..default()
        });
        d.spawn(kit2::label(text::SOLD_NOTE.get(l), 11., kit2::INK_MUTED));
    }
}

/// Spawns the page container inside the host root (called from `forge_view::setup`).
pub fn page() -> impl Bundle {
    (
        ScreenPage,
        Node {
            display: Display::None,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(kit2::BACKING),
    )
}

/// QA: click a card, a family tab and another card, capturing the frames right after each click
/// (the redraw must never show an empty or half-built panel).
fn qa_clicks(mut c: Commands, opts: Res<crate::Options>, navs: Query<(Entity, &Nav)>) {
    if opts.qa.as_deref() != Some("screen-collection-clicks") {
        return;
    }
    let target = match opts.frames {
        100 => Some(Nav::Weapon(9)),
        130 => Some(Nav::Family(1)),
        160 => Some(Nav::Weapon(0)),
        _ => None,
    };
    if let Some(t) = target
        && let Some((e, _)) = navs.iter().find(|(_, n)| **n == t)
    {
        c.trigger(Activate { entity: e });
    }
    if let Some(path) = &opts.capture
        && [101, 102, 103, 131, 132, 133, 161, 162].contains(&opts.frames)
    {
        let extra = format!("{}-{:03}.png", path.trim_end_matches(".png"), opts.frames);
        c.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(crate::save_capture(extra));
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, qa_clicks);
    let qa_en = std::env::args().any(|a| a.starts_with("screen-") && a.ends_with("-en"));
    app.insert_resource(if qa_en { Lang::En } else { Lang::from_env() })
        .add_observer(navigate)
        .add_systems(
            Update,
            (
                show_pages,
                draw,
                live_due,
                crate::crafting_screen::live_queue,
            )
                .chain()
                .after(forge_view::update_host)
                .run_if(|q: Query<(), With<HostRoot>>| !q.is_empty()),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::weapon;

    fn fresh() -> State {
        let mut s = State::new(42);
        s.progression.rules = Some(forge_content::progression_rules().unwrap());
        s
    }
    fn cmd(s: &mut State, c: Command) {
        s.command(s.command_id + 1, c).unwrap();
    }

    #[test]
    fn orders_follow_domain_phases_and_actions() {
        let mut s = fresh();
        let v = orders(&s);
        assert_eq!(
            v.iter().map(|o| o.id).collect::<Vec<_>>(),
            [OrderId::Chef, OrderId::Farmer]
        );
        assert_eq!(
            v[1].due,
            Due::AfterAccept(forge_core::commission::DEADLINE_SECS)
        );
        assert_eq!(v[1].actions[0].command, Command::AcceptFarmerRepair);
        cmd(&mut s, Command::AcceptFarmerRepair);
        let v = orders(&s);
        assert_eq!(v[1].phase, OrderPhase::Accepted);
        assert!(matches!(v[1].due, Due::Left(_)));
        assert_eq!(
            v[1].actions.iter().map(|a| a.command).collect::<Vec<_>>(),
            [Command::QueueFarmerRepair, Command::BuyOre]
        );
        // Rewards and costs are the domain constants, not display copies.
        assert_eq!(v[1].gold, forge_core::commission::REWARD_GOLD);
        assert_eq!(v[1].needs[0].2, forge_core::commission::REPAIR_ORE);
        // The Chef needs the chef recipe's parts, not ore.
        let parts: Vec<_> = v[0].needs.iter().map(|n| n.0.to_string()).collect();
        assert_eq!(parts, ["straight_blade", "wooden_handle"]);
        assert_eq!(
            v[0].gold,
            s.progression.rules.as_ref().unwrap().chef.reward_gold
        );
    }

    #[test]
    fn filters_split_active_and_completed() {
        let mut s = weapon::qa_smith();
        let v = orders(&s);
        let chef = v.iter().find(|o| o.id == OrderId::Chef).unwrap();
        assert_eq!(chef.phase, OrderPhase::Delivered);
        assert!(in_filter(chef, 2) && !in_filter(chef, 1) && in_filter(chef, 0));
        cmd(&mut s, Command::AcceptFarmerRepair);
        let farmer = orders(&s)
            .into_iter()
            .find(|o| o.id == OrderId::Farmer)
            .unwrap();
        assert!(in_filter(&farmer, 1) && !in_filter(&farmer, 2));
    }

    #[test]
    fn collection_reads_real_weapon_records() {
        let mut s = fresh();
        assert!(WEAPONS.iter().all(|(id, _)| !record(&s, id).found));
        s = weapon::qa_smith();
        let bow = record(&s, "shortbow");
        assert!(bow.found && bow.owned == 1 && bow.best_quality.is_none());
        assert!(!record(&s, "hunting_knife").found);
        cmd(
            &mut s,
            Command::ForgeWeapon(weapon::Plan {
                approach: weapon::Approach::Careful,
                ..default()
            }),
        );
        s.advance(60).unwrap();
        let knife = record(&s, "hunting_knife");
        let c = s
            .expedition
            .weapons
            .iter()
            .find_map(|w| w.craftsmanship.as_ref())
            .unwrap();
        assert!(knife.found);
        assert_eq!(knife.best_quality, Some(c.quality));
        assert_eq!(knife.appearance, Some(c.appearance));
        assert_eq!(knife.ore, Some(c.ore));
        // Catalog-only weapons stay undiscovered and say so.
        let pike = record(&s, "pike");
        assert!(!pike.found && !pike.craftable);
    }

    #[test]
    fn families_cover_every_weapon_once() {
        for (id, _) in WEAPONS {
            assert_eq!(
                (1..=FAMILIES.len()).filter(|f| in_family(id, *f)).count(),
                1,
                "{id}"
            );
            assert!(in_family(id, 0));
        }
    }

    #[test]
    fn activating_controls_navigates_and_issues_domain_commands() {
        use crate::{Options, Session, window::Desktop};
        let dir = std::env::temp_dir().join(format!("forge-screens-{}", std::process::id()));
        let mut app = App::new();
        app.insert_resource(Session {
            save: forge_core::save::Save::new(fresh(), 100),
            dir: dir.clone(),
            _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
            last: std::time::Instant::now(),
            fraction: 0.,
            status: String::new(),
            disabled: false,
            trace: vec![],
        })
        .insert_resource(Options::default())
        .insert_resource(crate::title::TitleScreen::default())
        .insert_resource(Desktop::new(false, true))
        .init_resource::<forge_ui::DialogState>()
        .init_resource::<Host>()
        .add_observer(navigate)
        .add_observer(crate::lifestyle::buttons);
        let press = |app: &mut App, b: (Nav,)| {
            let e = app.world_mut().spawn(b).id();
            app.world_mut().trigger(Activate { entity: e });
        };
        press(&mut app, (Nav::Screen(HostScreen::Orders),));
        press(&mut app, (Nav::OrderFilter(1),));
        press(&mut app, (Nav::Order(OrderId::Farmer),));
        let host = app.world().resource::<Host>();
        assert_eq!(
            (host.screen, host.order_filter, host.order),
            (HostScreen::Orders, 1, Some(OrderId::Farmer))
        );
        // The detail's primary action is the farmer's real Accept command.
        let view = orders(&app.world().resource::<Session>().save.state).remove(1);
        let e = app.world_mut().spawn(Action(view.actions[0].command)).id();
        app.world_mut().trigger(Activate { entity: e });
        assert_eq!(
            app.world()
                .resource::<Session>()
                .save
                .state
                .farmer_order
                .phase,
            OrderPhase::Accepted
        );
        // Crafting: tab and recipe selection, then the detail's queue button queues that recipe.
        press(&mut app, (Nav::Screen(HostScreen::Crafting),));
        press(&mut app, (Nav::CraftTab(2),));
        let club = Id::new("carve_wooden_club");
        press(&mut app, (Nav::Recipe(club),));
        let host = app.world().resource::<Host>();
        assert_eq!(
            crate::crafting_screen::selected(host).map(|r| r.id),
            Some(club)
        );
        let logs = app
            .world()
            .resource::<Session>()
            .save
            .state
            .stock(forge_core::items::ASH_LOG);
        let e = app
            .world_mut()
            .spawn(Action(Command::QueueRecipe(club)))
            .id();
        app.world_mut().trigger(Activate { entity: e });
        let st = &app.world().resource::<Session>().save.state;
        assert_eq!(
            (st.stock(forge_core::items::ASH_LOG), st.life.queue.len()),
            (logs - 1, 1)
        );
        press(&mut app, (Nav::CraftTab(0),));
        assert_eq!(app.world().resource::<Host>().recipe, None);
        press(&mut app, (Nav::Screen(HostScreen::Orders),));
        // A modal blocks navigation.
        app.world_mut()
            .resource_mut::<forge_ui::DialogState>()
            .open("t", "t", "t", None);
        press(&mut app, (Nav::Screen(HostScreen::Collection),));
        assert_eq!(app.world().resource::<Host>().screen, HostScreen::Orders);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn every_referenced_icon_exists() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/");
        let s = fresh();
        for o in orders(&s) {
            assert!(std::path::Path::new(&format!("{root}{}", o.item_icon)).exists());
            assert!(std::path::Path::new(&format!("{root}npc/{}/neutral.png", o.npc)).exists());
        }
        for (id, _) in WEAPONS {
            for found in [true, false] {
                let (p, _) = weapon_icon(id, found);
                assert!(std::path::Path::new(&format!("{root}{p}")).exists(), "{p}");
            }
        }
    }
}
