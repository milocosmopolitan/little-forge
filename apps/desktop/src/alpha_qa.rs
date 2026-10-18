//! Real controls, accelerated authoritative clock, isolated journal and actual GPU captures.
use crate::{Options, Session, lifestyle, spaces, title};
use bevy::{prelude::*, render::view::screenshot::Screenshot, ui_widgets::Activate};
use forge_core::{Command, Smith};

#[allow(clippy::too_many_arguments)]
pub fn drive(
    mut commands: Commands,
    opts: Res<Options>,
    titles: Query<(Entity, &title::TitleAction)>,
    tabs: Query<(Entity, &lifestyle::HomeTab)>,
    items: Query<(Entity, &lifestyle::HomeItem)>,
    actions: Query<(Entity, &lifestyle::Action)>,
    spaces: Query<(Entity, &spaces::Action)>,
    mut session: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
) {
    if !opts.qa.as_deref().is_some_and(|q| q.starts_with("alpha")) {
        return;
    }
    let frame = opts.frames;
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
        assert_eq!(s.ore, 6);
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
