//! Optional combat observation. All gameplay writes use the same core command transaction.
use crate::{Options, Session, title::TitleScreen, village::Village, window::Desktop};
use bevy::{prelude::*, ui_widgets::Activate};
use forge_core::{
    Command,
    expedition::{Action as CombatAction, Gear, Loadout},
    village::{Item, Location, Screen},
};
#[derive(Resource)]
pub struct ExpeditionUi {
    pub loadout: Loadout,
    pub plan: forge_core::weapon::Plan,
    tab: u8,
    record_page: usize,
    revision: u64,
    seen_return: u64,
}
impl Default for ExpeditionUi {
    fn default() -> Self {
        Self {
            loadout: Loadout {
                food: 5,
                arrows: 24,
                ..default()
            },
            plan: forge_core::weapon::Plan::default(),
            tab: 0,
            record_page: 0,
            revision: 0,
            seen_return: 0,
        }
    }
}
type DrawKey = (u64, u64, bool, usize, bool);
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Actor(bool);
#[derive(Component)]
struct Arrow;
#[derive(Component)]
struct Impact;
#[derive(Component)]
struct Summary;
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Tab(u8),
    FieldCodex,
    NextRecords,
    Material,
    Technique,
    Approach,
    Forge,
    Domain(Command),
    Close,
    Craft(Gear),
    Primary,
    Secondary,
    Food,
    Arrows,
    Launch,
    Recall,
    BuyTimber,
    BuyFood,
    Dismiss,
    Prepare,
}
fn visible(v: &Village, d: &Desktop, t: &TitleScreen) -> bool {
    !t.active && d.shell.expanded && v.nav.screen == Screen::Destination(Location::Outside)
}
pub fn plugin(app: &mut App) {
    app.init_resource::<ExpeditionUi>()
        .add_observer(activate)
        .add_systems(Startup, (restore_loadout, preload))
        .add_systems(
            Update,
            (keyboard, draw, animate, return_dialogue)
                .chain()
                .after(crate::village::update),
        );
}
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut v: ResMut<Village>,
    mut d: ResMut<Desktop>,
    t: Res<TitleScreen>,
    dialog: Res<forge_ui::DialogState>,
) {
    if !t.active && !dialog.is_open() && keys.just_pressed(KeyCode::KeyX) {
        v.nav.visit(Location::Outside);
        d.shell.recover();
        d.shell.expanded = true;
    }
}
#[allow(clippy::too_many_arguments)]
fn activate(
    e: On<Activate>,
    q: Query<&Action>,
    mut ui: ResMut<ExpeditionUi>,
    mut s: ResMut<Session>,
    mut v: ResMut<Village>,
    d: Res<Desktop>,
    t: Res<TitleScreen>,
    opts: Res<Options>,
) {
    if t.active || s.disabled {
        return;
    }
    let Ok(a) = q.get(e.entity) else {
        return;
    };
    if matches!(a, Action::Dismiss | Action::Prepare | Action::FieldCodex) {
        ui.seen_return = s.save.state.expedition.active.as_ref().map_or(0, |b| b.id);
        if matches!(a, Action::Prepare | Action::FieldCodex) {
            if matches!(a, Action::FieldCodex) {
                ui.tab = 2;
            }
            v.nav.visit(Location::Outside);
        }
        return;
    }
    if !visible(&v, &d, &t) {
        return;
    }
    if matches!(a, Action::Close) {
        v.nav.screen = Screen::Home;
        return;
    }
    let running = s.save.state.expedition.running();
    let cmd = match *a {
        Action::NextRecords => {
            ui.record_page =
                (ui.record_page + 1) % s.save.state.progression.weapons.len().max(1).div_ceil(2);
            None
        }
        Action::Tab(tab) => {
            ui.tab = tab;
            None
        }
        Action::Material => {
            ui.plan.material = if ui.plan.material == forge_core::weapon::Material::Iron {
                forge_core::weapon::Material::Steel
            } else {
                forge_core::weapon::Material::Iron
            };
            None
        }
        Action::Technique => {
            ui.plan.technique = if ui.plan.technique == forge_core::weapon::Technique::LightEdge {
                forge_core::weapon::Technique::ReinforcedPoint
            } else {
                forge_core::weapon::Technique::LightEdge
            };
            None
        }
        Action::Approach => {
            ui.plan.approach = match ui.plan.approach {
                forge_core::weapon::Approach::Quick => forge_core::weapon::Approach::Standard,
                forge_core::weapon::Approach::Standard => forge_core::weapon::Approach::Careful,
                forge_core::weapon::Approach::Careful => forge_core::weapon::Approach::Quick,
            };
            None
        }
        Action::Forge => Some(Command::ForgeWeapon(ui.plan)),
        Action::Domain(cmd) => Some(cmd),
        Action::Craft(g) => Some(Command::CraftExpeditionGear(g)),
        Action::Launch => Some(Command::LaunchExpedition(ui.loadout)),
        Action::Recall => Some(Command::RecallExpedition),
        Action::BuyTimber => Some(Command::Purchase {
            vendor: Location::Carpenter,
            item: Item::Timber,
            quantity: 1,
        }),
        Action::BuyFood => Some(Command::BuyFood),
        Action::Primary | Action::Secondary if !running => {
            let mut choices = vec![None];
            choices.extend(s.save.state.expedition.weapons.iter().map(|w| Some(w.id)));
            let slot = if matches!(a, Action::Primary) {
                &mut ui.loadout.primary
            } else {
                &mut ui.loadout.secondary
            };
            let i = choices.iter().position(|v| v == slot).unwrap_or(0);
            *slot = choices[(i + 1) % choices.len()];
            None
        }
        Action::Food if !running => {
            ui.loadout.food = (ui.loadout.food + 1) % 6;
            None
        }
        Action::Arrows if !running => {
            ui.loadout.arrows = (ui.loadout.arrows + 12) % 48;
            None
        }
        _ => None,
    };
    if let Some(cmd) = cmd {
        let id = s.save.state.command_id + 1;
        s.status = match s.save.state.command(id, cmd) {
            Ok(()) => "Hunter's ledger updated.".into(),
            Err(err) => {
                format!("Cannot proceed: {err:?}. Check gear, ore, timber, food and arrows.")
            }
        };
        crate::persist(&mut s, &opts);
    }
    ui.revision += 1;
}
fn node(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        top: px(top),
        width: px(width),
        height: px(height),
        ..default()
    }
}
fn picture(p: &mut ChildSpawnerCommands, server: &AssetServer, path: &str, n: Node) {
    p.spawn((ImageNode::new(server.load(path.to_owned())), n));
}
#[allow(clippy::too_many_arguments)]
fn draw(
    mut c: Commands,
    server: Res<AssetServer>,
    s: Res<Session>,
    ui: Res<ExpeditionUi>,
    v: Res<Village>,
    d: Res<Desktop>,
    t: Res<TitleScreen>,
    roots: Query<Entity, With<Root>>,
    mut last: Local<Option<DrawKey>>,
) {
    let show = visible(&v, &d, &t);
    let key = (
        s.save.state.command_id,
        ui.revision,
        show,
        s.save.state.expedition.weapons.len(),
        s.save.state.expedition.running(),
    );
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    for e in &roots {
        c.entity(e).despawn();
    }
    if !show {
        return;
    }
    let state = &s.save.state;
    let ex = &state.expedition;
    if ui.tab != 0 {
        draw_workshop(&mut c, state, &ui);
        return;
    }
    c.spawn((Root,GlobalZIndex(90),node(0.,0.,960.,540.),BackgroundColor(Color::srgb(0.08,0.10,0.09)))).with_children(|p|{
  p.spawn((forge_ui::text("OLD FOREST PATH",22.),node(20.,12.,300.,30.)));
  p.spawn(node(350.,8.,400.,40.)).with_children(|p| { forge_ui::button(p,"Forge equipment",Action::Tab(1)); forge_ui::button(p,"Field Codex",Action::Tab(2)); });
  p.spawn((Node{position_type:PositionType::Absolute,right:px(16),top:px(8),..default()},)).with_children(|p|{forge_ui::button(p,"Back to forge",Action::Close);});
  p.spawn((Node{overflow:Overflow::clip(),..node(0.,50.,960.,300.)},)).with_children(|p|{
   for path in ["distant", "midground", "foreground"] {picture(p,&server,&format!("expedition/forest/{path}.png"),node(0.,-100.,960.,540.));}
   for i in 0..8 {picture(p,&server,"expedition/forest/ground.png",node(i as f32*128.,276.,128.,64.));}
   p.spawn((Actor(true),ImageNode::new(server.load("expedition/hunter/frames/neutral.png")),node(120.,0.,256.,320.)));
   p.spawn((Actor(false),ImageNode::new(server.load("expedition/wolf/frames/idle.png")),node(680.,104.,256.,192.)));
   p.spawn((Arrow,ImageNode::new(server.load("expedition/effects/arrow.png")),node(400.,130.,64.,32.),Visibility::Hidden));
   p.spawn((Impact,ImageNode::new(server.load("expedition/effects/hit.png")),node(600.,180.,64.,64.),Visibility::Hidden));
   picture(p,&server,"expedition/forest/rock.png",node(800.,240.,160.,112.));
   p.spawn((Summary,forge_ui::text("",16.),node(18.,8.,920.,62.),BackgroundColor(Color::srgba(0.04,0.06,0.05,0.90))));
  });
  p.spawn((Node{position_type:PositionType::Absolute,left:px(16),top:px(356),width:px(928),flex_direction:FlexDirection::Column,row_gap:px(5),..default()},)).with_children(|p|{
   let label=|id|ex.weapon(id).map_or("Empty".to_owned(),|w|format!("{} #{}",w.kind.label(),w.id));
   p.spawn((Node{column_gap:px(5),..default()},)).with_children(|p|{
    forge_ui::button(p,format!("Primary: {}",label(ui.loadout.primary)),Action::Primary);forge_ui::button(p,format!("Secondary: {}",label(ui.loadout.secondary)),Action::Secondary);forge_ui::button(p,format!("Food: {}",ui.loadout.food),Action::Food);forge_ui::button(p,format!("Arrows: {}",ui.loadout.arrows),Action::Arrows);
    forge_ui::button(p,if ex.running(){"Recall"}else{"Launch / retry"},if ex.running(){Action::Recall}else{Action::Launch});
   });
   p.spawn((Node{column_gap:px(5),..default()},)).with_children(|p|{
    forge_ui::button(p,"Bow: 1 ore + 2 timber",Action::Craft(Gear::Shortbow));forge_ui::button(p,"Knife: 2 ore",Action::Craft(Gear::HuntingKnife));forge_ui::button(p,"12 arrows: 1 ore + 1 timber",Action::Craft(Gear::Arrows));forge_ui::button(p,"Timber: 3g",Action::BuyTimber);forge_ui::button(p,"Food: 2g",Action::BuyFood);
   });
   p.spawn(forge_ui::text("Click slots to cycle. Inspect penetration for boars and handling for wolves in Forge equipment.",13.));
   p.spawn((Ledger,forge_ui::text("",13.)));
   p.spawn((Status,forge_ui::text("",13.)));
  });
 });
}
#[derive(Component)]
struct Ledger;
#[derive(Component)]
struct Status;
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn animate(
    s: Res<Session>,
    opts: Res<Options>,
    time: Res<Time>,
    server: Res<AssetServer>,
    mut actors: Query<(&Actor, &mut ImageNode, &mut Node, &mut Visibility)>,
    mut arrow: Query<(&mut Node, &mut Visibility), (With<Arrow>, Without<Actor>, Without<Impact>)>,
    mut impact: Query<(&mut Node, &mut Visibility), (With<Impact>, Without<Actor>, Without<Arrow>)>,
    mut summary: Query<&mut Text, (With<Summary>, Without<Ledger>, Without<Status>)>,
    mut ledger: Query<&mut Text, (With<Ledger>, Without<Summary>, Without<Status>)>,
    mut status: Query<&mut Text, (With<Status>, Without<Summary>, Without<Ledger>)>,
    mut action_clock: Local<(u64, u32, f32)>,
) {
    let st = &s.save.state;
    let b = st.expedition.active.as_ref();
    let tm = if opts.qa.is_some() {
        opts.frames as f32 / 30.
    } else {
        time.elapsed_secs()
    };
    let phase = (tm * 8.) as usize;
    let event = b.map_or((0, 0), |b| (b.id, b.second));
    if (action_clock.0, action_clock.1) != event {
        *action_clock = (event.0, event.1, time.elapsed_secs());
    }
    // An action advances once, then holds. Global modulo time made draw/aim oscillate.
    let action_age = if opts.qa.as_deref() == Some("expedition-release") {
        0.1
    } else if opts.qa.is_some() {
        0.75
    } else {
        (time.elapsed_secs() - action_clock.2).clamp(0., 1.)
    };
    let action = b.map_or(CombatAction::Entry, |b| b.last_action());
    let attack = b
        .and_then(|b| {
            b.trace
                .iter()
                .rev()
                .take_while(|e| e.second == b.second)
                .find(|e| matches!(e.action, CombatAction::BowRelease | CombatAction::Slash))
        })
        .map(|e| e.action);
    for (actor, mut image, mut n, mut vis) in &mut actors {
        let (kind, frame) = if actor.0 {
            let frame = match action {
                CombatAction::Retreat => format!("walk-{}", phase % 6),
                CombatAction::Victory => "victory".into(),
                CombatAction::Hit => "hit".into(),
                _ => match attack.unwrap_or(action) {
                    CombatAction::BowDraw => bow_frame(false, action_age).into(),
                    CombatAction::BowRelease => bow_frame(true, action_age).into(),
                    CombatAction::Slash => "dagger-slash".into(),
                    CombatAction::KnifeDraw => "dagger-draw".into(),
                    CombatAction::Walk => format!("walk-{}", phase % 6),
                    _ => "neutral".into(),
                },
            };
            image.flip_x = action == CombatAction::Retreat;
            n.left = px(if action == CombatAction::Retreat {
                40.
            } else if b.is_some_and(|b| b.second < 4) {
                40. + b.map_or(0., |b| b.second as f32 * 26.)
            } else {
                120.
            });
            ("hunter", frame)
        } else {
            *vis = if b.is_none()
                || action == CombatAction::Victory
                || action == CombatAction::Retreat
            {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
            n.left = px(b.map_or(680., |b| 330. + f32::from(b.distance) * 55.));
            (
                if b.is_some_and(|b| b.enemy() == forge_core::expedition::Enemy::Boar) {
                    "boar"
                } else {
                    "wolf"
                },
                if b.is_some_and(|b| b.enemy_hp == 0) {
                    "defeat".into()
                } else if action == CombatAction::Hit {
                    "attack".into()
                } else if attack.is_some() {
                    "hit".into()
                } else {
                    format!("run-{}", phase % 4)
                },
            )
        };
        image.image = server.load(format!("expedition/{kind}/frames/{frame}.png"));
    }
    for (mut n, mut v) in &mut arrow {
        *v = if attack == Some(CombatAction::BowRelease) && action_age < 0.35 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let target = b.map_or(600., |b| 430. + f32::from(b.distance) * 55.);
        n.left = px(340. + (target - 340.) * (action_age / 0.35).min(1.));
    }
    for (mut n, mut v) in &mut impact {
        let hit_age = if attack == Some(CombatAction::BowRelease) {
            0.30
        } else {
            0.
        };
        *v = if attack.is_some() && (hit_age..hit_age + 0.2).contains(&action_age) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        n.left = px(b.map_or(600., |b| 350. + f32::from(b.distance) * 55.));
    }
    for mut text in &mut summary {
        **text = if let Some(b) = b {
            format!(
                "Encounter {} / 5   |   Condition {} / 60   |   Food {}   Arrows {}\n{}",
                b.wave + 1,
                b.health,
                b.food,
                b.arrows,
                if b.done {
                    b.story()
                } else {
                    format!(
                        "{}  |  {:?}  |  {} encounters cleared",
                        b.enemy().label(),
                        action,
                        b.cleared
                    )
                }
            )
        } else {
            "Hunter: Forge a bow and knife, then click the slots to lend them to me.\nFive encounters. Equipment returns safely. Watching is optional.".into()
        };
    }
    for mut text in &mut ledger {
        **text = format!(
            "Gold {}  Ore {}  Timber {}  Pantry {}  Arrows {}  Queue {}  |  Codex: {}",
            st.gold,
            st.ore(),
            st.stock(forge_core::items::ASH_LOG),
            st.life.pantry.staples(),
            st.expedition.arrows,
            st.life.queue.len(),
            st.expedition
                .weapon(b.and_then(|b| b.knife))
                .and_then(|w| w.craftsmanship.as_ref())
                .map_or("Legacy equipment".into(), |c| format!(
                    "{} · Pen {} / handling {} / {}s",
                    c.label(),
                    c.properties.penetration,
                    c.properties.handling,
                    c.properties.interval
                ))
        );
    }
    for mut text in &mut status {
        **text = format!(
            "{}\nGold {} · Ore {} · Timber {} · Food {} · Work {} · Queue {}",
            s.status,
            st.gold,
            st.ore(),
            st.stock(forge_core::items::ASH_LOG),
            st.life.pantry.staples(),
            st.life.active_job.map_or("Idle", |j| j.label()),
            st.life.queue.len()
        );
    }
}

fn bow_frame(released: bool, age: f32) -> &'static str {
    if released {
        if age < 0.18 {
            "bow-release"
        } else if age < 0.65 {
            "bow-recovery"
        } else {
            "bow-ready"
        }
    } else if age < 0.12 {
        "bow-ready"
    } else if age < 0.30 {
        "bow-draw"
    } else if age < 0.52 {
        "bow-pull"
    } else {
        "bow-aim"
    }
}

fn restore_loadout(s: Res<Session>, mut ui: ResMut<ExpeditionUi>) {
    if let Some(b) = &s.save.state.expedition.active {
        ui.loadout = b.loadout;
    }
}
#[derive(Component)]
struct ReturnRoot;
#[allow(clippy::too_many_arguments)]
fn return_dialogue(
    mut c: Commands,
    server: Res<AssetServer>,
    s: Res<Session>,
    ui: Res<ExpeditionUi>,
    v: Res<Village>,
    d: Res<Desktop>,
    t: Res<TitleScreen>,
    q: Query<Entity, With<ReturnRoot>>,
    mut last: Local<Option<u64>>,
) {
    let battle = s.save.state.expedition.active.as_ref().filter(|b| {
        b.done
            && b.id != ui.seen_return
            && !t.active
            && d.shell.expanded
            && v.nav.screen == Screen::Home
    });
    let id = battle.map(|b| b.id);
    if *last == id {
        return;
    }
    *last = id;
    for e in &q {
        c.entity(e).despawn();
    }
    let Some(b) = battle else {
        return;
    };
    c.spawn((ReturnRoot,GlobalZIndex(95),node(110.,120.,740.,260.),BackgroundColor(Color::srgb(0.13,0.15,0.12)),BorderColor::all(forge_ui::EDGE))).with_children(|p|{
  picture(p,&server,"npc/hunter/frames/talk-small.png",node(12.,50.,128.,160.));
  p.spawn((forge_ui::text("HUNTER'S RETURN",23.),node(150.,18.,560.,36.)));
  p.spawn((forge_ui::text(b.story(),18.),node(150.,62.,560.,110.)));
  p.spawn((forge_ui::text(format!("Equipment returned safely. {} gold from the trail. Field notes saved in the Codex.",b.reward_gold()),14.),node(150.,162.,560.,48.)));
  p.spawn((node(150.,214.,550.,40.),)).with_children(|p|{forge_ui::button(p,"Change equipment and retry",Action::Prepare);forge_ui::button(p,"Back to work",Action::Dismiss);});
 });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bow_draw_holds_aim_without_reversing_or_replaying_release() {
        assert_eq!(
            [0., 0.15, 0.35, 0.6, 0.9, 10.].map(|age| bow_frame(false, age)),
            [
                "bow-ready",
                "bow-draw",
                "bow-pull",
                "bow-aim",
                "bow-aim",
                "bow-aim"
            ]
        );
        assert_eq!(
            [0., 0.2, 0.7, 10.].map(|age| bow_frame(true, age)),
            ["bow-release", "bow-recovery", "bow-ready", "bow-ready"]
        );
    }
    #[test]
    fn actual_ui_observer_craft_fail_change_gear_retry_and_view_switch() {
        let dir = std::env::temp_dir().join(format!("expedition-ui-{}", std::process::id()));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut state = forge_core::State::new(42);
        state.gold = 50;
        state.set_stock(forge_core::items::ASH_LOG, 8);
        state
            .life
            .pantry
            .add(forge_core::life::Food::Staple, 10, 20)
            .unwrap();
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
            qa: Some("expedition-input".into()),
            ..default()
        });
        app.insert_resource(Desktop::new(false, true));
        app.init_resource::<Village>();
        app.init_resource::<TitleScreen>();
        app.init_resource::<ExpeditionUi>();
        app.add_observer(activate);
        app.world_mut()
            .resource_mut::<Village>()
            .nav
            .visit(Location::Outside);
        let press = |app: &mut App, action| {
            let e = app.world_mut().spawn(action).id();
            app.world_mut().trigger(Activate { entity: e });
            app.world_mut().despawn(e);
        };
        for gear in [
            Gear::Shortbow,
            Gear::HuntingKnife,
            Gear::Arrows,
            Gear::Arrows,
            Gear::Arrows,
        ] {
            press(&mut app, Action::Craft(gear));
        }
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(45)
            .unwrap();
        press(&mut app, Action::Primary);
        press(&mut app, Action::Launch);
        assert!(
            app.world()
                .resource::<Session>()
                .save
                .state
                .expedition
                .running()
        );
        let mut expected = app.world().resource::<Session>().save.state.clone();
        expected.advance(600).unwrap();
        press(&mut app, Action::Close);
        // A hidden launch button cannot write, while simulation continues in the forge.
        press(&mut app, Action::Launch);
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(600)
            .unwrap();
        assert_eq!(expected, app.world().resource::<Session>().save.state);
        assert_eq!(expected.expedition.active.unwrap().cleared, 2);
        press(&mut app, Action::Prepare);
        press(&mut app, Action::Secondary);
        press(&mut app, Action::Secondary);
        press(&mut app, Action::Craft(Gear::Arrows));
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(10)
            .unwrap();
        press(&mut app, Action::Launch);
        app.world_mut()
            .resource_mut::<Session>()
            .save
            .state
            .advance(600)
            .unwrap();
        let state = &app.world().resource::<Session>().save.state;
        assert_eq!(state.expedition.active.as_ref().unwrap().cleared, 5);
        assert!(state.expedition.path_reopened);
        assert_eq!(state.expedition.weapons.len(), 2);
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[derive(Resource)]
pub(crate) struct ExpeditionArt {
    pub images: Vec<Handle<Image>>,
}
fn preload(mut commands: Commands, server: Res<AssetServer>) {
    let mut images = vec![];
    for (name, json) in [
        (
            "hunter",
            include_str!("../../../assets/expedition/hunter/animation.json"),
        ),
        (
            "wolf",
            include_str!("../../../assets/expedition/wolf/animation.json"),
        ),
        (
            "boar",
            include_str!("../../../assets/expedition/boar/animation.json"),
        ),
    ] {
        let data: serde_json::Value =
            serde_json::from_str(json).expect("validated combat animation");
        for frame in data["frames"].as_array().expect("animation frames") {
            images.push(server.load(format!(
                "expedition/{name}/frames/{}.png",
                frame.as_str().expect("frame name")
            )));
        }
    }
    for path in [
        "forest/distant",
        "forest/midground",
        "forest/foreground",
        "forest/ground",
        "forest/rock",
        "effects/arrow",
        "effects/hit",
    ] {
        images.push(server.load(format!("expedition/{path}.png")));
    }
    commands.insert_resource(ExpeditionArt { images });
}

fn draw_workshop(c: &mut Commands, s: &forge_core::State, ui: &ExpeditionUi) {
    c.spawn((Root,GlobalZIndex(90),node(0.,0.,960.,540.),BackgroundColor(Color::srgb(0.08,0.10,0.09)))).with_children(|p| {
        p.spawn(node(18.,12.,924.,40.)).with_children(|p| {
            forge_ui::button(p,"Watch expedition",Action::Tab(0));
            forge_ui::button(p,"Forge equipment",Action::Tab(1));
            forge_ui::button(p,"Field Codex",Action::Tab(2));
            forge_ui::button(p,"Back to forge",Action::Close);
        });
        p.spawn((Node{flex_direction:FlexDirection::Column,row_gap:px(10),..node(24.,66.,912.,450.)},)).with_children(|p| {
            if ui.tab == 1 {
                let craft=forge_core::weapon::Rules::bundled().forge(ui.plan,s.progression.level(0),s.smith);
                p.spawn(forge_ui::text("HUNTING KNIFE / Choose the tool for the threat",22.));
                p.spawn(Node{column_gap:px(8),..default()}).with_children(|p| {
                    forge_ui::button(p,format!("Material: {:?}",ui.plan.material),Action::Material);
                    forge_ui::button(p,format!("Technique: {}",ui.plan.technique.label()),Action::Technique);
                    forge_ui::button(p,format!("Approach: {}",ui.plan.approach.label()),Action::Approach);
                });
                let v=craft.properties;
                p.spawn(forge_ui::text(format!("{}
Force {} · Penetration {} · Recovery {}s · Handling {}
Investment: {} ore / {} effective work seconds",craft.label(),v.force,v.penetration,v.interval,v.handling,craft.ore,craft.seconds),19.));
                p.spawn(forge_ui::text("Boar hide resists 5 force; penetration bypasses resistance.
Light edges recover quickly and reduce wolf damage. Reinforced points trade speed for penetration.
Steel uses extra ore and recovers slower. Quality measures precision, never a damage multiplier.",15.));
                forge_ui::button(p,"Forge this knife",Action::Forge);
                p.spawn(Node{column_gap:px(6),..default()}).with_children(|p| {
                    forge_ui::button(p,"Buy 2 ore / 4g",Action::Domain(Command::BuyOre));
                    let progress=&s.progression;
                    let next=match progress.chef_phase {
                        forge_core::commission::OrderPhase::Offered=>Some(("Accept Chef commission",Command::AcceptChef)),
                        forge_core::commission::OrderPhase::Accepted=>Some(("Assemble Kitchen Knife (blade + handle)",Command::QueueChef)),
                        forge_core::commission::OrderPhase::Ready=>Some(("Deliver Chef knife",Command::DeliverChef)),
                        forge_core::commission::OrderPhase::Delivered if progress.can_complete()=>Some(("Complete Dagger research",Command::CompleteDaggerResearch)),
                        forge_core::commission::OrderPhase::Delivered if !progress.research_completed && !progress.study_pending=>Some(("Study Dagger / 4g",Command::StudyDagger)),
                        _=>None,
                    };
                    if let Some((label,cmd))=next {forge_ui::button(p,label,Action::Domain(cmd));}
                });
                p.spawn(forge_ui::text(format!("Blade forging Lv{} · Handle fitting Lv{} · Research {}/10 {}
Common uses Quick Craft. Standard/Careful require both techniques Lv1; all new knives require the researched recipe.",s.progression.level(0),s.progression.level(1),s.progression.research_xp,if s.progression.research_completed {"completed"}else{"not completed"}),14.));
            } else {
                p.spawn(forge_ui::text("SMITH'S CODEX / Permanent field records",22.));
                forge_ui::button(p,"Next records",Action::NextRecords);
                if let Some(b)=&s.expedition.active {
                    p.spawn(forge_ui::text(format!("Previous best {} / Current cleared {} / Reached wave {}
{}",b.previous_best,b.cleared,b.wave+1,b.story()),16.));
                    if b.done { p.spawn(forge_ui::text(format!("Equipment returned · Reward {} gold · {} arrows returned",b.reward_gold(),b.arrows),14.)); }
                }
                if let Some(b)=&s.expedition.active {
                    let mut contributions=std::collections::BTreeMap::<u64,u32>::new();
                    for t in &b.trace {if let Some(id)=t.weapon { *contributions.entry(id).or_default()+=u32::from(t.amount); }}
                    if let Some((id,damage))=contributions.iter().max_by_key(|(_,damage)|*damage) {p.spawn(forge_ui::text(format!("Largest contribution: weapon #{id}, {damage} attack force delivered"),14.));}
                }
                for w in s.progression.weapons.iter().rev().skip(ui.record_page*2).take(2) {
                    let owned=s.expedition.weapon(Some(w.id)).is_some();
                    let craft=w.craftsmanship.as_ref().expect("crafted Codex record");
                    p.spawn(forge_ui::text(format!("#{} {} · {}
{} trips / {} defeats / best wave {} / {} completions / breakthrough {}",w.id,craft.label(),if owned {"Owned"}else{"Sold, history retained"},w.field_uses,w.kills,w.highest_wave,w.completions,w.first_breakthrough.map_or("not yet".into(),|id|format!("expedition #{id}"))),15.));
                    if owned {forge_ui::button(p,format!("Sell #{} / 4g",w.id),Action::Domain(Command::SellWeapon(w.id)));}
                }
            }
            p.spawn((Status,forge_ui::text("",14.)));
        });
    });
}
