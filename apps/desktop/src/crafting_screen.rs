//! Crafting catalog (#110) in the kit-v2 host: recipe tabs, a recipe grid, the selected recipe's
//! inputs with explicit ways to cover missing ones, and the work queue.
//!
//! Every button issues a domain command (`QueueRecipe`, `Buy`). Whether a button works is decided
//! by running that same command on a copy of the state; a refused button is disabled with the
//! command's own reason. Nothing is queued or bought automatically.
use crate::Session;
use crate::forge_view::Host;
use crate::lifestyle::Action;
use crate::screens::{DetailBody, Nav, item_icon, item_name, parchment, rail, row, screen_root};
use crate::text::{self, Lang};
use bevy::prelude::*;
use forge_core::crafting::{Book, Recipe};
use forge_core::items::{Id, Kind as ItemKind, Registry};
use forge_core::life::WorkKind;
use forge_core::village::Location;
use forge_core::{Command, State};
use forge_ui::kit2::{self, Kind};

/// Most jobs the queue holds (`State::quote_recipe`).
pub const QUEUE_CAP: usize = 32;
/// Index of the research-gated tab (no recipes until #75/#76).
pub const RESEARCH_TAB: usize = 3;

// ---- view model -------------------------------------------------------------------------------

/// Which tab a recipe belongs to: by the catalog kind of what it makes.
pub fn tab_of(r: &Recipe) -> usize {
    match Registry::bundled().get(r.output.item).map(|d| d.kind) {
        Some(ItemKind::Material) => 0,
        Some(ItemKind::Part) => 1,
        _ => 2,
    }
}

pub fn recipes(tab: usize) -> Vec<&'static Recipe> {
    if tab == RESEARCH_TAB {
        return vec![];
    }
    Book::bundled()
        .recipes
        .iter()
        .filter(|r| tab_of(r) == tab)
        .collect()
}

/// The recipe shown in the detail panel: the chosen one if it is on this tab, else the first.
pub fn selected(host: &Host) -> Option<&'static Recipe> {
    let list = recipes(host.craft_tab);
    host.recipe
        .and_then(|id| list.iter().copied().find(|r| r.id == id))
        .or_else(|| list.first().copied())
}

/// A way to cover a missing input. Each is an explicit button or a pointer, never automatic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Queue the recipe that makes this input.
    Make(Id),
    /// Buy the shortfall from the forge's own supplier (the existing ore channel).
    Buy { qty: u32, total: u64 },
    /// Only a village shop sells it.
    Shop(Location),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputView {
    pub item: Id,
    pub have: u32,
    pub need: u32,
    pub fixes: Vec<Fix>,
}

pub fn inputs(s: &State, r: &Recipe) -> Vec<InputView> {
    let book = Book::bundled();
    r.inputs
        .iter()
        .map(|i| {
            let have = s.stock(i.item);
            let mut fixes = vec![];
            if have < i.qty {
                if let Some(sub) = book.recipes.iter().find(|x| x.output.item == i.item) {
                    fixes.push(Fix::Make(sub.id));
                }
                let qty = (i.qty - have).min(99);
                if let Ok(q) = s.quote_buy(Location::Blacksmith, i.item, qty) {
                    fixes.push(Fix::Buy {
                        qty,
                        total: q.total,
                    });
                } else if let Some(v) = book.vendors.iter().find(|v| {
                    v.location != Location::Blacksmith && v.offers.iter().any(|o| o.item == i.item)
                }) {
                    fixes.push(Fix::Shop(v.location));
                }
            }
            InputView {
                item: i.item,
                have,
                need: i.qty,
                fixes,
            }
        })
        .collect()
}

/// The command a fix button issues, if it is a button.
pub fn fix_command(item: Id, fix: &Fix) -> Option<Command> {
    match *fix {
        Fix::Make(id) => Some(Command::QueueRecipe(id)),
        Fix::Buy { qty, .. } => Some(Command::Buy {
            vendor: Location::Blacksmith,
            item,
            quantity: qty,
        }),
        Fix::Shop(_) => None,
    }
}

/// Whether a command would be accepted now (the same check the command runs).
pub fn refusal(s: &State, command: Command) -> Option<forge_core::Error> {
    s.clone()
        .command(s.command_id.saturating_add(1), command)
        .err()
}

pub fn job_name(l: Lang, job: WorkKind) -> String {
    match job {
        WorkKind::Recipe(id) => Book::bundled()
            .recipe(id)
            .map_or_else(|| id.to_string(), |r| item_name(l, r.output.item)),
        WorkKind::ChefOrder | WorkKind::KitchenKnife => text::CHEF_ITEM.get(l).into(),
        WorkKind::FarmerSickleRepair => text::FARMER_TITLE.get(l).into(),
        other => other.label().into(),
    }
}

/// The rail's queue summary: waiting count, and the job at the bench with its time left.
pub fn queue_text(l: Lang, s: &State) -> String {
    let life = &s.life;
    let mut out = text::queue_line(l, life.queue.len(), QUEUE_CAP);
    match life.active_job {
        Some(job) => {
            let secs = life.work.div_ceil(life.speed().max(1));
            out.push('\n');
            out.push_str(&text::working_on(l, &job_name(l, job), secs));
        }
        None if life.queue.is_empty() => {
            out.push('\n');
            out.push_str(text::QUEUE_IDLE.get(l));
        }
        None => {}
    }
    if let Some(next) = life.queue.front() {
        out.push('\n');
        out.push_str(&text::up_next(l, &job_name(l, *next)));
    }
    out
}

fn recipe_label(l: Lang, r: &Recipe) -> String {
    let name = item_name(l, r.output.item);
    if r.output.qty > 1 {
        format!("{name} ×{}", r.output.qty)
    } else {
        name
    }
}

// ---- drawing ----------------------------------------------------------------------------------

/// Queue text in the rail, patched in place every frame so the countdown never rebuilds the page.
#[derive(Component)]
pub struct QueueText;

pub fn live_queue(s: Res<Session>, lang: Res<Lang>, mut q: Query<&mut Text, With<QueueText>>) {
    for mut t in &mut q {
        let next = queue_text(*lang, &s.save.state);
        if t.0 != next {
            t.0 = next;
        }
    }
}

pub fn draw(p: &mut ChildSpawnerCommands, server: &AssetServer, host: &Host, s: &State, l: Lang) {
    let shown = recipes(host.craft_tab);
    let chosen = selected(host);
    screen_root(p, server, |p| {
        let tabs: Vec<(String, String)> = text::CRAFT_TABS
            .iter()
            .enumerate()
            .map(|(i, t)| (t.get(l).to_string(), recipes(i).len().to_string()))
            .collect();
        rail(
            p,
            server,
            l,
            &tabs,
            host.craft_tab,
            44.,
            Nav::CraftTab,
            |r| {
                r.spawn(kit2::strong(text::QUEUE_TITLE.get(l), 13., kit2::INK_WOOD));
                r.spawn((
                    kit2::label(queue_text(l, s), 11., kit2::INK_WOOD_MUTED),
                    QueueText,
                ));
                r.spawn(Node {
                    height: px(6),
                    ..default()
                });
            },
        );
        parchment(p, server, Some(392.), |g| {
            let ready = shown
                .iter()
                .filter(|r| s.quote_recipe(r.id).is_ok())
                .count();
            g.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|h| {
                h.spawn(kit2::strong(
                    text::CRAFT_TABS[host.craft_tab.min(RESEARCH_TAB)].get(l),
                    17.,
                    kit2::INK,
                ));
                h.spawn(kit2::label(
                    format!("{}  {ready} / {}", text::READY_COUNT.get(l), shown.len()),
                    12.,
                    kit2::INK,
                ));
            });
            kit2::divider(g, server);
            if shown.is_empty() {
                g.spawn(kit2::label(
                    text::CRAFT_RESEARCH_EMPTY.get(l),
                    12.,
                    kit2::INK_MUTED,
                ));
            }
            g.spawn(Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(4),
                row_gap: px(4),
                ..default()
            })
            .with_children(|grid| {
                for r in &shown {
                    recipe_card(grid, server, r, chosen.is_some_and(|c| c.id == r.id), s, l);
                }
            });
        });
        let d = parchment(p, server, None, |d| {
            if let Some(r) = chosen {
                detail(d, server, r, s, l);
            }
        });
        p.commands().entity(d).insert(DetailBody);
    });
}

fn recipe_card(
    grid: &mut ChildSpawnerCommands,
    server: &AssetServer,
    r: &Recipe,
    selected: bool,
    s: &State,
    l: Lang,
) {
    let ready = s.quote_recipe(r.id).is_ok();
    let e = kit2::control(
        grid,
        server,
        Kind::Card,
        selected,
        format!("recipe:{}", r.id),
        Node {
            width: px(68),
            height: px(78),
            padding: UiRect::top(px(6)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            ..default()
        },
        Nav::Recipe(r.id),
    );
    grid.commands().entity(e).with_children(|card| {
        kit2::slot(
            card,
            server,
            "normal",
            40.,
            Some((&item_icon(r.output.item), None)),
            1.,
        );
        card.spawn((
            kit2::label(
                recipe_label(l, r),
                10.,
                if ready { kit2::INK } else { kit2::INK_MUTED },
            ),
            TextLayout::justify(Justify::Center),
        ));
    });
}

/// Right-hand panel: what the recipe makes, its inputs with fixes, and the queue button.
pub fn detail(d: &mut ChildSpawnerCommands, server: &AssetServer, r: &Recipe, s: &State, l: Lang) {
    d.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|h| {
        h.spawn(kit2::strong(recipe_label(l, r), 18., kit2::INK));
        kit2::tag(h, server, "rank", text::station(l, r.station));
    });
    kit2::divider(d, server);
    d.spawn(row(12.)).with_children(|top| {
        kit2::slot(
            top,
            server,
            "selected",
            56.,
            Some((&item_icon(r.output.item), None)),
            1.,
        );
        top.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        })
        .with_children(|c| {
            for (k, v) in [
                (
                    text::RESULT,
                    format!(
                        "{} ×{}  ({} {})",
                        item_name(l, r.output.item),
                        r.output.qty,
                        text::OWNED.get(l),
                        s.stock(r.output.item)
                    ),
                ),
                (text::WORK, text::seconds(l, r.seconds)),
            ] {
                c.spawn(row(10.)).with_children(|t| {
                    t.spawn((
                        kit2::label(k.get(l), 12., kit2::INK_MUTED),
                        Node {
                            width: px(56),
                            ..default()
                        },
                    ));
                    t.spawn(kit2::label(v, 12., kit2::INK));
                });
            }
        });
    });
    kit2::divider(d, server);
    d.spawn(kit2::label(text::NEEDS.get(l), 12., kit2::INK));
    for input in inputs(s, r) {
        d.spawn(row(6.)).with_children(|line| {
            kit2::slot(
                line,
                server,
                "normal",
                32.,
                Some((&item_icon(input.item), None)),
                1.,
            );
            line.spawn(Node {
                flex_grow: 1.,
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|c| {
                c.spawn(kit2::label(item_name(l, input.item), 12., kit2::INK));
                c.spawn(kit2::label(
                    text::have_need(l, input.have, input.need),
                    11.,
                    if input.have >= input.need {
                        kit2::INK_MUTED
                    } else {
                        kit2::DEADLINE
                    },
                ));
            });
            for fix in &input.fixes {
                match (fix, fix_command(input.item, fix)) {
                    (Fix::Shop(v), _) => {
                        line.spawn(kit2::label(text::sold_at(l, *v), 11., kit2::INK_MUTED));
                    }
                    (_, Some(command)) => {
                        let label = match fix {
                            Fix::Buy { qty, total } => text::buy_short(l, *qty, *total),
                            _ => text::MAKE_ONE.get(l).into(),
                        };
                        let width = if matches!(fix, Fix::Buy { .. }) {
                            104.
                        } else {
                            64.
                        };
                        let e =
                            kit2::button(line, server, Kind::Plain, label, width, Action(command));
                        disable_if_refused(line, e, s, command, l);
                    }
                    _ => {}
                }
            }
        });
    }
    d.spawn(Node {
        flex_grow: 1.,
        ..default()
    });
    d.spawn(kit2::label(text::CRAFT_HINT.get(l), 11., kit2::INK_MUTED));
    d.spawn(row(8.)).with_children(|b| {
        let command = Command::QueueRecipe(r.id);
        let e = kit2::button(
            b,
            server,
            Kind::Primary,
            text::QUEUE_RECIPE.get(l),
            140.,
            Action(command),
        );
        disable_if_refused(b, e, s, command, l);
        if let Some(err) = refusal(s, command) {
            b.spawn(kit2::label(text::error(l, err), 11., kit2::DEADLINE));
        }
    });
}

fn disable_if_refused(
    p: &mut ChildSpawnerCommands,
    e: Entity,
    s: &State,
    command: Command,
    l: Lang,
) {
    if let Some(err) = refusal(s, command) {
        p.commands().entity(e).insert((
            bevy::ui::InteractionDisabled,
            forge_ui::Unavailable(text::error(l, err).into()),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> State {
        let mut s = State::new(7);
        s.progression.rules = Some(forge_content::progression_rules().unwrap());
        s
    }

    #[test]
    fn tabs_split_every_recipe_once_and_research_is_empty() {
        let total: usize = (0..4).map(|t| recipes(t).len()).sum();
        assert_eq!(total, Book::bundled().recipes.len());
        assert_eq!(
            (recipes(0).len(), recipes(1).len(), recipes(2).len()),
            (1, 11, 10)
        );
        assert!(recipes(RESEARCH_TAB).is_empty());
        let mut host = Host {
            craft_tab: 2,
            ..default()
        };
        assert_eq!(selected(&host).map(|r| r.id), Some(recipes(2)[0].id));
        host.recipe = Some(recipes(0)[0].id);
        assert_eq!(
            selected(&host).map(|r| r.id),
            Some(recipes(2)[0].id),
            "a recipe from another tab is not shown"
        );
        host.craft_tab = RESEARCH_TAB;
        assert!(selected(&host).is_none());
    }

    #[test]
    fn missing_inputs_offer_make_buy_or_point_to_a_shop() {
        let s = fresh();
        let book = Book::bundled();
        let knife = book.recipe(Id::new("assemble_kitchen_knife")).unwrap();
        let v = inputs(&s, knife);
        assert_eq!(
            v.iter().map(|i| (i.have, i.need)).collect::<Vec<_>>(),
            vec![(0, 1), (0, 1)]
        );
        assert_eq!(v[0].fixes, vec![Fix::Make(Id::new("forge_straight_blade"))]);
        assert_eq!(
            v[1].fixes,
            vec![
                Fix::Make(Id::new("make_wooden_handle")),
                Fix::Shop(Location::Carpenter)
            ]
        );
        let blade = book.recipe(Id::new("forge_straight_blade")).unwrap();
        let v = inputs(&s, blade);
        assert_eq!(
            v[0].fixes,
            vec![
                Fix::Make(Id::new("smelt_iron_ingot")),
                Fix::Buy { qty: 1, total: 7 }
            ]
        );
        let strap = book.recipe(Id::new("cut_leather_strap")).unwrap();
        assert_eq!(
            inputs(&s, strap)[0].fixes,
            vec![Fix::Shop(Location::Leatherworker)]
        );
        // Inputs on hand need no fix.
        let smelt = book.recipe(Id::new("smelt_iron_ingot")).unwrap();
        assert!(inputs(&s, smelt).iter().all(|i| i.fixes.is_empty()));
    }

    #[test]
    fn buttons_issue_exactly_the_command_they_show() {
        let mut s = fresh();
        s.gold = 7;
        let item = Id::new("iron_ingot");
        let cmd = fix_command(item, &Fix::Buy { qty: 1, total: 7 }).unwrap();
        assert_eq!(refusal(&s, cmd), None);
        s.command(s.command_id + 1, cmd).unwrap();
        assert_eq!((s.gold, s.stock(item)), (0, 1));
        // Refusals are the command's own errors, and leave nothing changed.
        let before = s.clone();
        assert_eq!(
            refusal(&s, cmd),
            Some(forge_core::Error::InsufficientResources)
        );
        assert_eq!(s, before);
        let queue = Command::QueueRecipe(Id::new("forge_straight_blade"));
        assert_eq!(refusal(&s, queue), None);
        s.command(s.command_id + 1, queue).unwrap();
        assert_eq!(s.stock(item), 0);
        assert!(queue_text(Lang::En, &s).starts_with("1 / 32 waiting"));
    }

    #[test]
    fn queue_text_names_the_job_at_the_bench() {
        let mut s = fresh();
        s.command(
            s.command_id + 1,
            Command::QueueRecipe(Id::new("smelt_iron_ingot")),
        )
        .unwrap();
        s.advance(1).unwrap();
        let t = queue_text(Lang::En, &s);
        assert!(t.contains("Working: Iron Ingot"), "{t}");
        s.advance(60).unwrap();
        assert!(queue_text(Lang::En, &s).contains("idle"));
        assert_eq!(s.stock(Id::new("iron_ingot")), 1);
    }
}
