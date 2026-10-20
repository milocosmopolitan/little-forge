//! Real controls, accelerated authoritative clock, isolated journal and actual GPU captures.
use crate::{Options, Session, lifestyle, spaces, title};
use bevy::{prelude::*, render::view::screenshot::Screenshot, ui_widgets::Activate};
use forge_core::{Command, Smith};

#[allow(clippy::too_many_arguments)]
pub fn drive(
    mut commands: Commands,
    opts: Res<Options>,
    expedition: Query<(Entity, &crate::expedition::Action)>,
    titles: Query<(Entity, &title::TitleAction)>,
    tabs: Query<(Entity, &lifestyle::HomeTab)>,
    items: Query<(Entity, &lifestyle::HomeItem)>,
    actions: Query<(Entity, &lifestyle::Action)>,
    spaces: Query<(Entity, &spaces::Action)>,
    mut session: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
    mut boar_capture: Local<bool>,
    host: Query<(Entity, &crate::forge_view::Action)>,
) {
    if !opts.qa.as_deref().is_some_and(|q| q.starts_with("alpha")) {
        return;
    }
    let frame = opts.frames;
    if frame == 85 || frame == 90 {
        let (entity, _) = host
            .iter()
            .find(|(_, a)| {
                **a == if frame == 85 {
                    crate::forge_view::Action::Open
                } else {
                    crate::forge_view::Action::Legacy
                }
            })
            .expect("alpha: legacy controls");
        commands.trigger(Activate { entity });
    }
    if opts.qa.as_deref() == Some("alpha-crafting") && frame >= 620 {
        crafting(
            &mut commands,
            &opts,
            &expedition,
            &mut session,
            &mut exit,
            &mut boar_capture,
        );
        return;
    }
    let smith = if opts.qa.as_deref() == Some("alpha-female") {
        Smith::Female
    } else {
        Smith::Male
    };
    let title_action = match frame {
        20 => Some(title::TitleAction::New),
        40 => Some(title::TitleAction::Begin),
        60 => Some(title::TitleAction::Pick(smith)),
        _ => None,
    };
    let tab = match frame {
        100 | 240 => Some(3),
        420 => Some(4),
        620 => Some(0),
        720 => Some(5),
        800 => Some(6),
        _ => None,
    };
    if frame == 150 {
        // Schema 8: the knife needs a blade and a handle. Make them with real recipe commands
        // and simulated time before the UI's Queue button is pressed at frame 160.
        forge_core::crafting::make_chef_parts(&mut session.save.state).expect("alpha Chef parts");
    }
    let action = match frame {
        140 => Some(Command::AcceptChef),
        160 => Some(Command::QueueChef),
        400 => Some(Command::DeliverChef),
        460 => Some(Command::StudyDagger),
        600 => Some(Command::CompleteDaggerResearch),
        640 => Some(Command::QueueCraft),
        _ => None,
    };
    let close = [180, 840].contains(&frame);
    let target = if let Some(wanted) = title_action {
        Some(
            titles
                .iter()
                .find(|(_, a)| **a == wanted)
                .expect("alpha: title control missing")
                .0,
        )
    } else if let Some(wanted) = tab {
        Some(
            tabs.iter()
                .find(|(_, a)| a.0 == wanted)
                .expect("alpha: navigation control missing")
                .0,
        )
    } else if let Some(wanted) = action {
        Some(
            actions
                .iter()
                .find(|(_, a)| a.0 == wanted)
                .expect("alpha: gameplay control missing")
                .0,
        )
    } else if close {
        Some(
            spaces
                .iter()
                .find(|(_, a)| **a == spaces::Action::CloseDrawer)
                .expect("alpha: close control missing")
                .0,
        )
    } else if frame == 740 {
        Some(
            items
                .iter()
                .find(|(_, a)| a.0 == 1)
                .expect("alpha: codex selection missing")
                .0,
        )
    } else {
        None
    };
    if let Some(entity) = target {
        commands.trigger(Activate { entity });
    }
    let capture = match frame {
        15 => Some("title"),
        55 => Some("character-selection"),
        80 => Some("workshop"),
        120 => Some("chef-commission"),
        200 => Some("active-crafting"),
        380 => Some("chef-ready"),
        440 => Some("research-discovered"),
        580 => Some("research-ready"),
        610 => Some("research-completed"),
        760 => Some("codex"),
        820 => Some("journal"),
        860 => Some("workshop-return"),
        _ => None,
    };
    if let (Some(name), Some(path)) = (capture, &opts.capture) {
        commands
            .spawn(Screenshot::primary_window())
            .observe(crate::save_capture(format!(
                "{}-{name}.png",
                path.trim_end_matches(".png")
            )));
    }
    if frame == 880 {
        let s = &session.save.state;
        assert_eq!(s.smith, Some(smith));
        assert_eq!(
            s.progression.chef_phase,
            forge_core::commission::OrderPhase::Delivered
        );
        assert!(s.progression.research_completed);
        assert_eq!(s.progression.general_xp, 22);
        assert_eq!(s.progression.codex["iron_dagger"].count, 1);
        assert_eq!(s.progression.codex["kitchen_knife"].count, 1);
        assert_eq!(s.ore(), 6);
        assert!(!session.disabled);
        let dir = session.dir.clone();
        forge_core::save::store(&dir, &mut session.save).expect("alpha: save");
        let loaded = forge_core::save::load(&dir)
            .expect("alpha: reload")
            .expect("alpha: journal");
        assert_eq!(loaded, session.save);
        if let Some(path) = &opts.capture {
            std::fs::write(format!("{path}.json"), serde_json::to_vec_pretty(&serde_json::json!({"scenario":opts.qa,"clock":"0.1 simulation second per rendered frame","input":"Activate events on production controls","state":loaded.state,"save_reload":"equal","result":"passed"})).unwrap()).expect("alpha: report");
        }
    }
    if frame == 900 {
        exit.write(AppExit::Success);
    }
}

fn crafting(
    commands: &mut Commands,
    opts: &Options,
    actions: &Query<(Entity, &crate::expedition::Action)>,
    session: &mut Session,
    exit: &mut MessageWriter<AppExit>,
    boar_capture: &mut bool,
) {
    use crate::expedition::Action as A;
    use forge_core::expedition::Gear;
    let frame = opts.frames;
    let action = match frame {
        620 => Some(A::Prepare),
        640 => Some(A::Craft(Gear::Shortbow)),
        660 | 680 | 700 => Some(A::Craft(Gear::Arrows)),
        1040 | 1740 => Some(A::Tab(1)),
        1060 => Some(A::Technique),
        1080 | 1820 => Some(A::Forge),
        1180 | 2120 => Some(A::Tab(0)),
        1200 => Some(A::Primary),
        1220 | 1240 | 2140 => Some(A::Secondary),
        1260 | 1280 | 2160 | 2180 | 2200 | 2220 | 2240 => Some(A::BuyFood),
        1320 | 2300 => Some(A::Launch),
        1700 | 3200 => Some(A::Tab(2)),
        1760 | 1780 => Some(A::Approach),
        1800 => Some(A::Material),
        2260 => Some(A::Craft(Gear::Arrows)),
        _ => None,
    };
    if let Some(wanted) = action {
        let entity = actions
            .iter()
            .find(|(_, a)| **a == wanted)
            .unwrap_or_else(|| panic!("crafting frame {frame}: missing control {wanted:?}"))
            .0;
        commands.trigger(Activate { entity });
    }
    let mut name = match frame {
        1100 => Some("common-crafting"),
        1405 => Some("first-wolves"),
        1680 => Some("wave-three-retreat"),
        1720 => Some("failure-feedback"),
        1840 => Some("masterwork-crafting"),
        3100 => Some("victory"),
        3220 => Some("breakthrough-codex"),
        _ => None,
    };
    if frame == 1405 {
        assert_eq!(
            session
                .save
                .state
                .expedition
                .active
                .as_ref()
                .unwrap()
                .enemy(),
            forge_core::expedition::Enemy::Wolf
        );
    }
    if !*boar_capture
        && session
            .save
            .state
            .expedition
            .active
            .as_ref()
            .is_some_and(|b| b.id == 2 && b.wave == 2 && b.enemy_hp == 0)
    {
        *boar_capture = true;
        name = Some("retry-boar");
    }
    if let (Some(name), Some(path)) = (name, &opts.capture) {
        commands
            .spawn(Screenshot::primary_window())
            .observe(crate::save_capture(format!(
                "{}-{name}.png",
                path.trim_end_matches(".png")
            )));
    }
    if frame == 1700 {
        assert_eq!(
            session
                .save
                .state
                .expedition
                .active
                .as_ref()
                .expect("first expedition")
                .cleared,
            2
        );
    }
    if frame == 3240 {
        assert!(*boar_capture, "actual Wave 3 boar defeat must be captured");
        let s = &session.save.state;
        assert_eq!(
            s.expedition.active.as_ref().expect("retry").cleared,
            5,
            "{s:?}"
        );
        assert_eq!(s.expedition.records[0].cleared, 2);
        assert_eq!(s.progression.weapons.len(), 2);
        assert!(s.progression.weapons[1].first_breakthrough.is_some());
        let dir = session.dir.clone();
        forge_core::save::store(&dir, &mut session.save).unwrap();
        let loaded = forge_core::save::load(&dir).unwrap().unwrap();
        assert_eq!(loaded, session.save);
        if let Some(path) = &opts.capture {
            std::fs::write(format!("{path}.json"),serde_json::to_vec_pretty(&serde_json::json!({"result":"passed","input":"production Bevy Activate controls","clock":"0.1s per rendered frame","fixture":"initial ore40 gold80 timber8 only; research and weapons earned","save_reload":"equal","state":loaded.state})).unwrap()).unwrap();
        }
    }
    if frame == 3260 {
        exit.write(AppExit::Success);
    }
}
