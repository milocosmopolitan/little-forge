//! Actual GPU lifecycle capture; deterministic core clock, production title/host controls.
use crate::{Options, Session, forge_view, spaces, title};
use bevy::{prelude::*, render::view::screenshot::Screenshot, ui_widgets::Activate};
use forge_npc::player::{Actor, CharacterId, Walker};
#[derive(Default)]
pub struct Evidence {
    actor: Option<Entity>,
    samples: Vec<serde_json::Value>,
    captures: std::collections::BTreeSet<String>,
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn drive(
    mut c: Commands,
    opts: Res<Options>,
    titles: Query<(Entity, &title::TitleAction)>,
    controls: Query<(Entity, &forge_view::Action)>,
    actors: Query<(Entity, &Actor, &Walker, &CharacterId), With<spaces::WorkshopActor>>,
    stations: Query<Entity, With<crate::interior::FixturePart>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    desktop: Res<crate::window::Desktop>,
    mut proof: Local<Evidence>,
    mut exit: MessageWriter<AppExit>,
) {
    if !opts.qa.as_deref().is_some_and(|q| q.starts_with("d01")) {
        return;
    }
    let f = opts.frames;
    let sales = opts.qa.as_deref() == Some("d01-sales");
    let smith = if opts.qa.as_deref().is_some_and(|q| q.contains("female")) {
        forge_core::Smith::Female
    } else {
        forge_core::Smith::Male
    };
    let title_action = match f {
        20 => Some(title::TitleAction::New),
        40 => Some(title::TitleAction::Begin),
        60 => Some(title::TitleAction::Pick(smith)),
        _ => None,
    };
    if let Some(action) = title_action {
        let (e, _) = titles
            .iter()
            .find(|(_, a)| **a == action)
            .expect("D01 title control");
        c.trigger(Activate { entity: e });
    }
    let stable = opts.qa.as_deref().is_some_and(|q| q.ends_with("-stable")) || sales;
    let action = if stable {
        (f == 80).then_some(forge_view::Action::Close)
    } else if f == 80 || (142..542).contains(&f) && (f - 142).is_multiple_of(4) {
        Some(forge_view::Action::Close)
    } else if (140..540).contains(&f) && (f - 140).is_multiple_of(4) || f == 600 {
        Some(forge_view::Action::Open)
    } else {
        None
    };
    if let Some(action) = action {
        let (e, _) = controls
            .iter()
            .find(|(_, a)| **a == action)
            .expect("D01 host control");
        c.trigger(Activate { entity: e });
    }
    if f == 120 && sales {
        // Explicit migrated-recipe fixture; work, slot reservation and sale use real commands.
        session.save.state.progression.legacy_dagger = true;
        let id = session.save.state.command_id + 1;
        session
            .save
            .state
            .command(id, forge_core::Command::Craft)
            .expect("paid sale good");
    }
    if f == 120 && !sales {
        for command in [
            forge_core::Command::AcceptChef,
            forge_core::Command::QueueChef,
        ] {
            let id = session.save.state.command_id + 1;
            session
                .save
                .state
                .command(id, command)
                .expect("D01 paid Chef work command");
        }
    }
    if f > 90 {
        let (e, a, w, id) = actors.single().expect("exactly one persistent smith");
        if let Some(expected) = proof.actor {
            assert_eq!(e, expected, "smith recreated");
        } else {
            proof.actor = Some(e);
        }
        assert_eq!(id.0, smith.id());
        // Seven fixtures x (back, front) x (art, outline).
        assert_eq!(stations.iter().count(), 28, "station layers duplicated");
        let win = windows.single().expect("one window");
        proof.samples.push(serde_json::json!({"frame":f,"entity":format!("{e:?}"),"clip":a.clip,"clip_frame":a.clip_frame,"elapsed":a.elapsed,"position":[w.pos.x,w.pos.y],"window":[win.width(),win.height()],"physical":[win.resolution.physical_width(),win.resolution.physical_height()],"dpi":win.resolution.scale_factor(),"desktop_position":format!("{:?}",win.position),"expanded":desktop.shell.expanded,"tick":session.save.state.tick,"work":session.save.state.life.work,"activity":format!("{:?}",session.save.state.activity())}));
        let stage = match a.clip.as_str() {
            "forging" => Some("heating"),
            "hammering" => Some("hammering"),
            "quenching" => Some("quenching"),
            "relaxed" => Some("rest"),
            _ => None,
        };
        if let Some(stage) = stage
            && (stage == "rest" || a.clip_frame.unwrap_or(0) > 0)
            && proof.captures.insert(stage.into())
        {
            capture(&mut c, &opts, stage);
        }
    }
    if f == 100 {
        capture(&mut c, &opts, "ambient");
    }
    if f == 145 {
        capture(&mut c, &opts, "active-expanded");
    }
    if f == 147 {
        capture(&mut c, &opts, "active-collapsed");
    }
    if f == 610 {
        capture(&mut c, &opts, "management");
    }
    if sales && [300, 400, 470, 525, 620].contains(&f) {
        capture(&mut c, &opts, &format!("sale-{f}"));
    }
    if f == 300 {
        let dir = session.dir.clone();
        forge_core::save::store(&dir, &mut session.save).expect("mid-work save");
        let restored = forge_core::save::load(&dir)
            .expect("read save")
            .expect("saved state");
        assert_eq!(restored, session.save);
        let snapshot = dir.join("mid-work");
        std::fs::create_dir_all(&snapshot).unwrap();
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            if entry.path().is_file() && entry.file_name() != "session.lock" {
                std::fs::copy(entry.path(), snapshot.join(entry.file_name())).unwrap();
            }
        }
    }
    if f == 850 {
        let id = session.save.state.command_id + 1;
        session
            .save
            .state
            .command(id, forge_core::Command::Rest)
            .expect("rest command");
    }
    if f == 1380 {
        if sales {
            assert_eq!(session.save.state.sold, 1);
            assert_eq!(session.save.state.gold, 12);
            assert_eq!(session.save.state.sellable(), 0);
        } else {
            assert!(proof.captures.contains("heating"));
            assert!(proof.captures.contains("hammering"));
            assert!(proof.captures.contains("quenching"));
            assert_eq!(
                session.save.state.progression.chef_phase,
                forge_core::commission::OrderPhase::Ready
            );
        }
        let dir = session.dir.clone();
        forge_core::save::store(&dir, &mut session.save).expect("save");
        assert_eq!(forge_core::save::load(&dir).unwrap().unwrap(), session.save);
        if let Some(path) = &opts.capture {
            std::fs::write(format!("{path}.json"),serde_json::to_vec_pretty(&serde_json::json!({"result":"passed","cycles":if stable { 0 } else { 100 },"clock":"core and animation both advance1/30 second per rendered frame","commands":if sales {"migrated recipe fixture; real Craft and scheduled sale; no injected sale event"} else {"real paid Chef knife work; production Activate title and host controls, not OS input"},"state":session.save.state,"samples":proof.samples,"captures":proof.captures,"save_reload":"equal during work and at end"})).unwrap()).unwrap();
        }
    }
    if f == 1400 {
        exit.write(AppExit::Success);
    }
}
fn capture(c: &mut Commands, opts: &Options, name: &str) {
    if std::env::var_os("LITTLE_FORGE_TRACE_DIR").is_some() {
        return;
    }
    if let Some(path) = &opts.capture {
        c.spawn(Screenshot::primary_window())
            .observe(crate::save_capture(format!(
                "{}-{name}.png",
                path.trim_end_matches(".png")
            )));
    }
}
