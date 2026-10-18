//! Presentation reads authoritative life state; buttons issue atomic domain commands.
use crate::{Options, Session, window::Desktop};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use forge_core::Command;
#[derive(Component)]
pub struct Management;
#[derive(Component)]
pub struct Needs;
#[derive(Component)]
pub struct Ledger;
#[derive(Component)]
pub struct Action(pub Command);
/// Status-column nodes whose text or bar width follows the simulation without a rebuild.
#[derive(Component, Clone, Copy, PartialEq)]
pub enum Live {
    Action,
    Queue,
    Status,
    Fatigue,
    Hunger,
    Thirst,
    WorkBar,
    FatigueBar,
    HungerBar,
    ThirstBar,
}
fn live_text(s: &Session, kind: Live) -> String {
    let l = &s.save.state.life;
    match kind {
        Live::Action => format!("{:?}", l.action),
        Live::Queue => format!(
            "{} waiting · {}s remaining",
            l.queue,
            l.work.div_ceil(l.speed()) + l.remaining as u64
        ),
        Live::Status => s
            .status
            .split(". C craft")
            .next()
            .unwrap_or(&s.status)
            .to_string(),
        Live::Fatigue => format!("Fatigue {}%", l.fatigue / 100),
        Live::Hunger => format!("Hunger {}%", l.hunger / 100),
        Live::Thirst => format!("Thirst {}%", l.thirst / 100),
        _ => String::new(),
    }
}
fn live_fraction(s: &Session, kind: Live) -> f32 {
    let l = &s.save.state.life;
    match kind {
        Live::WorkBar if l.work == 0 => 0.,
        Live::WorkBar => 1. - l.work as f32 / (forge_core::CRAFT_SECS * 1000) as f32,
        Live::FatigueBar => l.fatigue as f32 / 10000.,
        Live::HungerBar => l.hunger as f32 / 10000.,
        _ => l.thirst as f32 / 10000.,
    }
    .clamp(0., 1.)
}
fn meter(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    s: &Session,
    label: Option<Live>,
    bar: Live,
) {
    match label {
        Some(kind) => parent.spawn((forge_ui::text(live_text(s, kind), 13.), kind)),
        None => parent.spawn(forge_ui::text("Current blade", 13.)),
    };
    let fill = if label.is_some() {
        forge_ui::Fill::Amber
    } else {
        forge_ui::Fill::Blue
    };
    let id = forge_ui::progress_bar(parent, server, live_fraction(s, bar), fill);
    parent.commands().entity(id).insert(bar);
}
/// Writes only on change so Bevy does not re-layout text or nodes every frame.
pub fn live(
    s: Res<Session>,
    mut texts: Query<(&Live, &mut Text)>,
    mut bars: Query<(&Live, &mut Node), Without<Text>>,
) {
    for (kind, mut text) in &mut texts {
        let next = live_text(&s, *kind);
        if text.0 != next {
            text.0 = next;
        }
    }
    for (kind, mut node) in &mut bars {
        let next = percent(live_fraction(&s, *kind) * 100.);
        if node.width != next {
            node.width = next;
        }
    }
}
#[derive(Resource, Default)]
pub struct HomeUi {
    pub tab: usize,
    selected: usize,
}
#[derive(Component)]
pub struct HomeTab(pub usize);
#[derive(Component)]
pub struct HomeItem(pub usize);
#[allow(clippy::too_many_arguments)]
pub fn tab(
    event: On<Activate>,
    tabs: Query<&HomeTab>,
    items: Query<&HomeItem>,
    mut home: ResMut<HomeUi>,
    desktop: Res<Desktop>,
    village: Res<crate::village::Village>,
    title: Res<crate::title::TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
) {
    if title.active
        || dialog.is_open()
        || !desktop.shell.expanded
        || village.nav.screen != forge_core::village::Screen::Home
    {
        return;
    }
    if let Ok(tab) = tabs.get(event.entity) {
        home.tab = tab.0;
        home.selected = 0;
    }
    if let Ok(item) = items.get(event.entity) {
        home.selected = item.0;
    }
}
pub fn setup(mut commands: Commands, opts: Res<Options>, mut home: ResMut<HomeUi>) {
    home.tab = match opts.qa.as_deref() {
        Some("village-materials") => 1,
        Some("village-provisions") => 2,
        _ => 0,
    };
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 13.0.into(),
            ..default()
        },
        TextColor(forge_ui::INK),
        Node {
            position_type: PositionType::Absolute,
            top: px(43),
            left: px(12),
            ..default()
        },
        Needs,
    ));
    // Retained status marker for existing diagnostic/test consumers; never player-facing.
    commands.spawn((Text::new(""), Visibility::Hidden, Ledger));
}
pub fn draw(
    mut commands: Commands,
    root: Query<Entity, With<Management>>,
    s: Res<Session>,
    home: Res<HomeUi>,
    server: Res<AssetServer>,
    mut last: Local<String>,
) {
    let Ok(root) = root.single() else {
        return;
    };
    let l = &s.save.state.life;
    // Fast-changing needs, timers and status text are patched in place by `live`;
    // rebuilding them every second made the whole panel flicker.
    let st = &s.save.state;
    let key = format!(
        "{}:{}:{}:{}:{}:{}:{}:{root:?}",
        home.tab,
        l.pantry.staples(),
        st.ore,
        st.daggers,
        st.commerce.timber,
        st.commerce.leather,
        home.selected,
    );
    if *last == key {
        return;
    }
    *last = key;
    commands.entity(root).despawn_children();
    commands.entity(root).insert((
        Node {
            width: percent(100),
            flex_grow: 1.,
            min_height: px(0),
            padding: UiRect::all(px(14)),
            border: UiRect::all(px(2)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(forge_ui::SURFACE),
        BorderColor::all(forge_ui::EDGE),
    ));
    commands.entity(root).with_children(|p|{
        p.spawn(forge_ui::text("Workshop ledger",21.));
        p.spawn(Node{flex_grow:1.,min_height:px(0),column_gap:px(12),..default()}).with_children(|row|{
            row.spawn(forge_ui::panel(percent(18))).with_children(|side|{
                for (i,label) in ["Crafting","Materials","Provisions"].iter().enumerate(){let entity = forge_ui::button(side,*label,HomeTab(i)); side.commands().entity(entity).insert(forge_ui::SkinKind::Tab); if home.tab == i { side.commands().entity(entity).insert(forge_ui::Selected); }}
                side.spawn(forge_ui::text("Work continues while you browse.",12.));
                forge_ui::button(side,"Rest [R]",Action(Command::Rest));
            });
            row.spawn(forge_ui::panel(percent(48))).with_children(|body|{
                match home.tab {
                    0=>{
                        body.spawn(forge_ui::text("Iron dagger",22.));
                        forge_ui::icon(body,&server,"dagger",80.);
                        body.spawn(forge_ui::text("A simple blade, carefully made.",14.));
                        body.spawn(forge_ui::text(format!("Requires 2 ore · Available {} · Result slots free {}",s.save.state.ore,st.showroom.free()),14.));
                        if let Some(reason) = st.craft_block_reason() { body.spawn(forge_ui::text(reason.label(),13.)); }
                        forge_ui::button(body,"Add to queue [Q]",Action(Command::QueueCraft));
                        forge_ui::button(body,"Buy 2 ore · 4 gold [B]",Action(Command::BuyOre));
                    },
                    1=>{
                        body.spawn(forge_ui::text("Materials & finished goods",19.));
                        body.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()}).with_children(|grid|{
                            for (i,(icon,count)) in [("ore",s.save.state.ore),("timber",s.save.state.commerce.timber),("leather",s.save.state.commerce.leather),("dagger",s.save.state.daggers)].into_iter().enumerate() {forge_ui::slot(grid,&server,icon,count,home.selected==i,(HomeItem(i),forge_ui::FocusKey(format!("home-slot:{i}"))));}
                        });
                        body.spawn(forge_ui::text(["Iron ore · 2 required for an iron dagger.","Timber · Stored for recipes and display racks.","Leather · Stored for future recipes.","Iron daggers · Finished goods live on the showroom racks; only displayed ones sell."][home.selected.min(3)],16.));
                    },
                    _=>{
                        body.spawn(forge_ui::text("Provisions",20.));
                        body.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(5),row_gap:px(5),..default()}).with_children(|grid|{
                            for (i,slot) in l.pantry.slots.iter().enumerate() {let (icon,count)=slot.as_ref().map(|v|(if v.food==forge_core::life::Food::Staple {"staples"} else {"meal"},v.count)).unwrap_or(("provisions",0));
                                forge_ui::slot(grid,&server,icon,count,home.selected==i,(HomeItem(i),forge_ui::FocusKey(format!("home-slot:{i}"))));
                            }
                        });
                        let detail = l.pantry.slots.get(home.selected).and_then(Option::as_ref).map(|v| if v.food == forge_core::life::Food::Staple { "Staples · Eaten automatically when hungry." } else { "Premium meal · Use Eat meal when needed." }).unwrap_or("Empty slot · Purchase food at the farm or tavern.");
                        body.spawn(forge_ui::text(detail,13.));
                        body.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(5),row_gap:px(5),..default()}).with_children(|p|{for (label,c) in [("Eat staple",Command::Eat),("Eat meal",Command::EatMeal),("Free water",Command::Drink),("Buy staples",Command::BuyFood)]{forge_ui::button(p,label,Action(c));}});
                    }
                }
            });
            row.spawn(forge_ui::panel(percent(34))).with_children(|queue|{
                queue.spawn(forge_ui::text("At the workbench",20.));
                queue.spawn((forge_ui::text(live_text(&s, Live::Action), 17.), Live::Action));
                queue.spawn((forge_ui::text(live_text(&s, Live::Queue), 14.), Live::Queue));
                meter(queue, &server, &s, None, Live::WorkBar);
                meter(queue, &server, &s, Some(Live::Fatigue), Live::FatigueBar);
                meter(queue, &server, &s, Some(Live::Hunger), Live::HungerBar);
                meter(queue, &server, &s, Some(Live::Thirst), Live::ThirstBar);
                queue.spawn(forge_ui::text("Your smith rests, eats and resumes work automatically. Meals are used only when you choose.",13.));
                queue.spawn((forge_ui::text(live_text(&s, Live::Status), 13.), Live::Status));
            });
        });
    });
}
#[allow(clippy::too_many_arguments)]
pub fn buttons(
    event: On<Activate>,
    actions: Query<&Action>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    dialog: Option<Res<forge_ui::DialogState>>,
    desktop: Res<Desktop>,
    mut farmer: Option<ResMut<crate::npc::FarmerPresentation>>,
    village: Option<Res<crate::village::Village>>,
) {
    if dialog.as_ref().is_some_and(|d| d.is_open())
        || !desktop.shell.expanded
        || s.disabled
        || village
            .as_ref()
            .is_some_and(|v| v.nav.screen != forge_core::village::Screen::Home)
    {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        let id = s.save.state.command_id + 1;
        s.status = match s.save.state.command(id, action.0) {
            Ok(()) => {
                if let Some(farmer) = farmer.as_mut() {
                    farmer.accepted(id, action.0);
                }
                format!("{:?} accepted", action.0)
            }
            Err(e) => format!("{e:?}"),
        };
        crate::persist(&mut s, &opts);
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update(
    s: Res<Session>,
    opts: Res<Options>,
    desktop: Res<Desktop>,
    village: Option<Res<crate::village::Village>>,
    title: Option<Res<crate::title::TitleScreen>>,
    mut panel: Query<&mut Visibility, (With<Management>, Without<Needs>)>,
    mut needs: Query<(&mut Text, &mut Visibility), With<Needs>>,
    mut ledger: Query<&mut Text, (With<Ledger>, Without<Needs>)>,
) {
    let visible = !title.as_ref().is_some_and(|t| t.active)
        && opts
            .qa
            .as_deref()
            .is_none_or(|q| {
                q.starts_with("life-")
                    || q.starts_with("village-")
                    || q.starts_with("forge-")
                    || q == "expanded"
            });
    let l = &s.save.state.life;
    for mut v in &mut panel {
        *v = if visible
            && desktop.shell.expanded
            && village
                .as_ref()
                .is_none_or(|v| v.nav.screen == forge_core::village::Screen::Home)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut text, mut v) in &mut needs {
        *v = if visible && !desktop.shell.expanded {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        **text = format!(
            "Fatigue {}%  /  Hunger {}%  /  Thirst {}%   |   {:?}{}",
            l.fatigue / 100,
            l.hunger / 100,
            l.thirst / 100,
            l.action,
            if l.pantry.staples() == 0 {
                "  - pantry empty"
            } else {
                ""
            }
        );
    }
    for mut text in &mut ledger {
        let slots: Vec<_> = l
            .pantry
            .slots
            .iter()
            .enumerate()
            .map(|(i, s)| match s {
                Some(s) => format!("{}:{:?} x{}", i + 1, s.food, s.count),
                None => format!("{}: --", i + 1),
            })
            .collect();
        **text = format!(
            "{:?}  |  timer {}s  |  work speed {}%  |  queued {}\nFatigue slows remaining work. Sleep restores fatigue; food / water restore hunger / thirst.\n\nPANTRY  (12 stackable slots, {} per stack)\n{}\n{}\nFarmer staple: {} gold  |  Ore: 4 gold / 2  |  Well: always free",
            l.action,
            l.remaining,
            l.speed() / 10,
            l.queue,
            l.config.stack_limit,
            slots[..6].join("    "),
            slots[6..].join("    "),
            l.config.food_price
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_management_observer_uses_domain_and_hidden_panel_ignores_activation() {
        let dir = std::env::temp_dir().join(format!(
            "forge-ui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut state = forge_core::State::new(42);
        state.life.thirst = 9000;
        app.insert_resource(Session {
            save: forge_core::save::Save::new(state, 100),
            _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
            dir: dir.clone(),
            last: std::time::Instant::now(),
            fraction: 0.,
            status: String::new(),
            disabled: false,
            trace: vec![],
        });
        app.insert_resource(Options {
            qa: Some("life-loop".into()),
            ..default()
        });
        app.insert_resource(Desktop::new(false, true));
        app.add_observer(buttons);
        app.add_systems(Update, update);
        let action = app.world_mut().spawn(Action(Command::Drink)).id();
        let needs = app
            .world_mut()
            .spawn((Needs, Text::new(""), Visibility::Inherited))
            .id();
        app.world_mut().spawn((Management, Visibility::Hidden));
        app.world_mut().spawn((Ledger, Text::new("")));
        app.world_mut().trigger(Activate { entity: action });
        app.update();
        let s = app.world().resource::<Session>();
        assert_eq!(s.save.state.life.thirst, 0);
        assert_eq!(s.save.state.gold, 0);
        assert_eq!(s.save.state.command_id, 1);
        assert!(app.world().get::<Text>(needs).unwrap().contains("Drinking"));
        app.world_mut().resource_mut::<Desktop>().shell.expanded = false;
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .life
            .thirst = 5000;
        app.world_mut().trigger(Activate { entity: action });
        assert_eq!(
            app.world().resource::<Session>().save.state.life.thirst,
            5000
        );
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
