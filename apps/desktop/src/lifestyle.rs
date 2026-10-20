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
    WorkBar,
}
fn live_text(s: &Session, kind: Live) -> String {
    let l = &s.save.state.life;
    match kind {
        Live::Action => format!("{:?}", l.action),
        Live::Queue => format!(
            "{} waiting · {}s remaining",
            l.queue.len(),
            l.work.div_ceil(l.speed()) + l.remaining as u64
        ),
        Live::Status => s
            .status
            .split(". C craft")
            .next()
            .unwrap_or(&s.status)
            .to_string(),
        _ => String::new(),
    }
}
fn live_fraction(s: &Session, kind: Live) -> f32 {
    let l = &s.save.state.life;
    match kind {
        Live::WorkBar if l.work == 0 => 0.,
        Live::WorkBar => s.save.state.job_progress().unwrap_or(0) as f32 / 1000.,
        _ => 0.,
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
        None => parent.spawn(forge_ui::text(
            s.save
                .state
                .life
                .active_job
                .map_or("Workbench", |j| j.label()),
            13.,
        )),
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
    mut spaces: ResMut<crate::spaces::Spaces>,
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
        spaces.space = forge_core::spaces::Space::Workshop;
        spaces.manage = true;
        spaces.station = None;
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
        Some(q) if q.starts_with("repair-") => 7,
        Some(q) if q.starts_with("alpha") => 3,
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
        st.ore(),
        st.daggers,
        st.stock(forge_core::items::ASH_LOG),
        st.stock(forge_core::items::DEER_LEATHER),
        home.selected,
    );
    let key = format!(
        "{key}:{}:{:?}:{}",
        st.command_id,
        st.farmer_order.phase,
        if home.tab == 7 { st.tick } else { 0 }
    );
    let key = format!(
        "{key}:{:?}:{}:{}:{:?}",
        st.progression.chef_phase,
        st.progression.research_xp,
        st.progression.general_xp,
        st.life.active_job
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

        p.spawn(Node{flex_grow:1.,min_height:px(0),column_gap:px(12),..default()}).with_children(|row|{
            row.spawn((forge_ui::SkinPanel, Node { width: percent(18), padding: UiRect::all(px(8)), flex_direction: FlexDirection::Column, row_gap: px(4), min_width: px(0), ..default() }, BackgroundColor(forge_ui::SURFACE))).with_children(|side|{
                for (i,label) in ["Crafting","Materials","Provisions","Chef [O]","Research","Codex","Journal","Farmer"].iter().enumerate(){let entity = forge_ui::button(side,*label,HomeTab(i)); side.commands().entity(entity).insert((forge_ui::SkinKind::Tab, Node { min_height: px(26), height: px(26), padding: UiRect::axes(px(4), px(2)), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() })); if home.tab == i { side.commands().entity(entity).insert(forge_ui::Selected); }}
                forge_ui::button(side,"Hunter",crate::expedition::Action::Prepare);

            });
            row.spawn(forge_ui::panel(percent(52))).with_children(|body|{
                match home.tab {
                    0=>{
                        body.spawn(forge_ui::text("Iron dagger",22.));
                        forge_ui::icon(body,&server,"dagger",80.);
                        body.spawn(forge_ui::text("A simple blade, carefully made.",14.));
                        body.spawn(forge_ui::text(format!("Requires {} ore · Available {} · Result slots free {}",st.dagger_ore(),s.save.state.ore(),st.showroom.free()),14.));
                        if let Some(reason) = st.craft_block_reason() { body.spawn(forge_ui::text(reason.label(),13.)); }
                        crate::progression_ui::command_button(body, &s.save.state, "Add to queue [Q]", Command::QueueCraft);
                        if !st.progression.dagger_unlocked() { body.spawn(forge_ui::text("Complete Chef’s commission, then study Dagger research.",14.)); forge_ui::button(body,"Open research",HomeTab(4)); }
                        forge_ui::button(body,"Buy 2 ore · 4 gold [B]",Action(Command::BuyOre));
                    },
                    1=>{
                        body.spawn(forge_ui::text("Materials & finished goods",19.));
                        body.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()}).with_children(|grid|{
                            for (i,(icon,count)) in [("ore",s.save.state.ore()),("timber",s.save.state.stock(forge_core::items::ASH_LOG)),("leather",s.save.state.stock(forge_core::items::DEER_LEATHER)),("dagger",s.save.state.daggers)].into_iter().enumerate() {forge_ui::slot(grid,&server,icon,count,home.selected==i,(HomeItem(i),forge_ui::FocusKey(format!("home-slot:{i}"))));}
                        });
                        body.spawn(forge_ui::text(["Iron ore · 2 required for an iron dagger.","Timber · Stored for recipes and display racks.","Leather · Stored for future recipes.","Iron daggers · Finished goods live on the showroom racks; only displayed ones sell."][home.selected.min(3)],16.));
                    },
                    3..=6 => crate::progression_ui::draw(body, &server, &s.save.state, home.tab, home.selected),
                    7 => draw_order(body, &s.save.state),
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
            row.spawn((forge_ui::SkinPanel, Node { width: percent(30), padding: UiRect::all(px(12)), flex_direction: FlexDirection::Column, row_gap: px(6), min_width: px(0), ..default() }, BackgroundColor(forge_ui::SURFACE))).with_children(|queue|{
                queue.spawn(forge_ui::text("At the workbench",18.));
                queue.spawn((forge_ui::text(live_text(&s, Live::Action), 17.), Live::Action));
                queue.spawn((forge_ui::text(live_text(&s, Live::Queue), 14.), Live::Queue));
                meter(queue, &server, &s, None, Live::WorkBar);

                queue.spawn((forge_ui::text(live_text(&s, Live::Status), 13.), Live::Status));
                forge_ui::button(queue,"Rest [R]",Action(Command::Rest));
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
    title: Res<crate::title::TitleScreen>,
    dialog: Option<Res<forge_ui::DialogState>>,
    desktop: Res<Desktop>,
    mut farmer: Option<ResMut<crate::npc::FarmerPresentation>>,
    village: Option<Res<crate::village::Village>>,
) {
    if title.active
        || dialog.as_ref().is_some_and(|d| d.is_open())
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
                command_status(action.0, &s.save.state)
            }
            Err(e) => command_error(e),
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
        && opts.qa.as_deref().is_none_or(|q| {
            q.starts_with("life-")
                || q.starts_with("village-")
                || q.starts_with("forge-")
                || q == "expanded"
                || (q.starts_with("repair-") || q.starts_with("alpha"))
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
        *v = Visibility::Hidden;
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
            l.queue.len(),
            l.config.stack_limit,
            slots[..6].join("    "),
            slots[6..].join("    "),
            l.config.food_price
        );
    }
}

fn draw_order(body: &mut ChildSpawnerCommands, s: &forge_core::State) {
    use forge_core::commission::*;
    let order = &s.farmer_order;
    body.spawn(forge_ui::text("First commission / Farmer", 19.));
    body.spawn(forge_ui::text("A worn sickle", 22.));
    body.spawn(forge_ui::text(
        "My sickle has lost its edge. Can you get it ready for the fields?",
        14.,
    ));
    body.spawn(forge_ui::text(
        format!(
            "{} ore · {}s base work · {} gold + {} favor",
            REPAIR_ORE, REPAIR_SECS, REWARD_GOLD, REWARD_FAVOR
        ),
        14.,
    ));
    let timing = match order.deadline_at {
        None => "Due 2 hours after acceptance. Take your time deciding.".to_owned(),
        Some(_) if order.completed_at.is_some() => {
            if order.on_time() {
                "Finished on time. Your full reward is safe.".into()
            } else {
                format!(
                    "Finished late. {} gold, no favor. No further penalty.",
                    LATE_REWARD_GOLD
                )
            }
        }
        Some(due) => {
            let left = due.saturating_sub(s.tick);
            if left == 0 {
                format!(
                    "Overdue. Finish and deliver for {} gold; no favor.",
                    LATE_REWARD_GOLD
                )
            } else {
                format!(
                    "Due in {}m {:02}s. Late completion: {} gold, no favor.",
                    left / 60,
                    left % 60,
                    LATE_REWARD_GOLD
                )
            }
        }
    };
    body.spawn(forge_ui::text(timing, 13.));
    body.spawn(forge_ui::text(order.phase.label(), 17.));
    match order.phase {
        OrderPhase::Offered => {
            forge_ui::button(
                body,
                "Accept repair [A]",
                Action(Command::AcceptFarmerRepair),
            );
        }
        OrderPhase::Accepted => {
            body.spawn(forge_ui::text(
                format!(
                    "Available ore: {}. Materials are used when queued.",
                    s.ore()
                ),
                13.,
            ));
            forge_ui::button(body, "Queue repair [S]", Action(Command::QueueFarmerRepair));
            forge_ui::button(body, "Buy 2 ore · 4 gold [B]", Action(Command::BuyOre));
        }
        OrderPhase::Queued => {
            let ahead = s
                .life
                .queue
                .iter()
                .position(|&job| job == forge_core::life::WorkKind::FarmerSickleRepair)
                .unwrap_or(0)
                + usize::from(s.life.active_job.is_some());
            body.spawn(forge_ui::text(
                format!("{} job(s) ahead. Work starts automatically.", ahead),
                14.,
            ));
        }
        OrderPhase::Working => {
            body.spawn(forge_ui::text(
                "At the shared workbench. Resting pauses work, then it resumes.",
                14.,
            ));
        }
        OrderPhase::Ready => {
            let (gold, favor) = order.reward();
            forge_ui::button(
                body,
                format!("Deliver · {} gold + {} favor [L]", gold, favor),
                Action(Command::DeliverFarmerRepair),
            );
        }
        OrderPhase::Delivered => {
            body.spawn(forge_ui::text(
                format!(
                    "Thank you! Earned {} gold · Farmer favor {}/100",
                    order.settled_gold, order.favor
                ),
                16.,
            ));
        }
    }
}
pub fn command_status(command: Command, s: &forge_core::State) -> String {
    match command {
        Command::AcceptChef => "Chef commission accepted. Queue the Kitchen Knife here.".into(),
        Command::QueueChef => "Kitchen Knife queued. Materials reserved once.".into(),
        Command::DeliverChef => {
            "Chef commission delivered! Dagger research discovered. Open Research.".into()
        }
        Command::StudyDagger => "Dagger study queued at the research table.".into(),
        Command::CompleteDaggerResearch => {
            "Dagger recipe unlocked! Open Crafting to forge your first weapon.".into()
        }
        Command::QueueCraft | Command::Craft => "Iron Dagger queued at the workbench.".into(),
        Command::AcceptFarmerRepair => {
            "Farmer's repair accepted. Queue it in the Farmer tab.".into()
        }
        Command::QueueFarmerRepair => {
            "Sickle repair queued. 2 ore used; work begins in order.".into()
        }
        Command::DeliverFarmerRepair => format!(
            "Sickle delivered! +{} gold, +{} farmer favor.",
            s.farmer_order.settled_gold, s.farmer_order.favor
        ),
        _ => format!("{command:?} accepted"),
    }
}
pub fn command_error(error: forge_core::Error) -> String {
    match error {
        forge_core::Error::ResearchLocked => {
            "Dagger recipe locked. Check Research for the next step.".into()
        }
        forge_core::Error::InsufficientResources => {
            "Not enough materials or gold. Check your supplies.".into()
        }
        forge_core::Error::InvalidOrderState => "Check Orders [O] for the next step.".into(),
        forge_core::Error::Busy => {
            "The workbench or queue is busy. Try again when space is free.".into()
        }
        _ => format!("{error:?}"),
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
        app.init_resource::<crate::title::TitleScreen>();
        app.init_resource::<crate::npc::FarmerPresentation>();
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
        // The real observer must guard hidden/title/disabled input and settle once.
        app.world_mut().resource_mut::<Session>().save.state = forge_core::State::new(42);
        let accept = app
            .world_mut()
            .spawn(Action(Command::AcceptFarmerRepair))
            .id();
        let enqueue = app
            .world_mut()
            .spawn(Action(Command::QueueFarmerRepair))
            .id();
        let deliver = app
            .world_mut()
            .spawn(Action(Command::DeliverFarmerRepair))
            .id();
        app.world_mut().trigger(Activate { entity: accept });
        assert_eq!(app.world().resource::<Session>().save.state.command_id, 0);
        app.world_mut().resource_mut::<Desktop>().shell.expanded = true;
        app.world_mut()
            .resource_mut::<crate::title::TitleScreen>()
            .active = true;
        app.world_mut().trigger(Activate { entity: accept });
        assert_eq!(app.world().resource::<Session>().save.state.command_id, 0);
        app.world_mut()
            .resource_mut::<crate::title::TitleScreen>()
            .active = false;
        app.world_mut().resource_mut::<Session>().disabled = true;
        app.world_mut().trigger(Activate { entity: accept });
        assert_eq!(app.world().resource::<Session>().save.state.command_id, 0);
        app.world_mut().resource_mut::<Session>().disabled = false;
        app.world_mut().trigger(Activate { entity: accept });
        app.world_mut().trigger(Activate { entity: accept });
        assert_eq!(app.world().resource::<Session>().save.state.command_id, 1);
        app.world_mut().trigger(Activate { entity: enqueue });
        app.world_mut().trigger(Activate { entity: enqueue });
        assert_eq!(app.world().resource::<Session>().save.state.ore(), 8);
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(20)
            .unwrap();
        app.world_mut().trigger(Activate { entity: deliver });
        app.world_mut().trigger(Activate { entity: deliver });
        let state = &app.world().resource::<Session>().save.state;
        assert_eq!(state.gold, 24);
        assert_eq!(state.farmer_order.favor, 5);
        assert_eq!(state.command_id, 3);
        assert_eq!(
            app.world()
                .resource::<crate::npc::FarmerPresentation>()
                .reaction,
            Some(Command::DeliverFarmerRepair)
        );
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
