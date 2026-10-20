use crate::scenario::{Driver, Offline, Player, Preset, Start, Timed};
use crate::*;

fn scenario(name: &str, preset: Preset, driver: Driver, secs: u64) -> Scenario {
    Scenario {
        name: name.into(),
        seed: 42,
        duration_secs: secs,
        step_secs: 1,
        sample_every_secs: 600,
        start: Start {
            preset,
            ..Default::default()
        },
        driver,
        offline: vec![],
    }
}

fn craft_once() -> Driver {
    Driver::Script {
        commands: vec![Timed {
            at: 0,
            command: "Craft".into(),
        }],
    }
}

#[test]
fn same_seed_same_bytes_and_seed_matters() {
    let sc = scenario(
        "player",
        Preset::Fresh,
        Driver::Player(Player::default()),
        4 * 3600,
    );
    let a = serde_json::to_string(&run(&sc).unwrap()).unwrap();
    let b = serde_json::to_string(&run(&sc).unwrap()).unwrap();
    assert_eq!(a, b);
    let mut other = sc.clone();
    other.seed = 7;
    assert_ne!(a, serde_json::to_string(&run(&other).unwrap()).unwrap());
}

#[test]
fn craft_and_sale_matches_the_core_reference() {
    // forge-qa's reference: one dagger crafted and sold within the hour for 12 gold.
    let r = run(&scenario(
        "craft-and-sale",
        Preset::LegacyDagger,
        craft_once(),
        3600,
    ))
    .unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!((r.end.crafted, r.end.sold, r.end.gold), (1, 1, 12));
    assert_eq!(r.ledger.get("sale"), Some(&12));
}

#[test]
fn offline_settles_like_staying_online() {
    let online = scenario("online", Preset::LegacyDagger, craft_once(), 2 * 3600);
    let mut away = online.clone();
    away.offline = vec![Offline {
        at: 5,
        secs: 7200 - 5,
    }];
    let (a, b) = (run(&online).unwrap(), run(&away).unwrap());
    assert!(a.failures.is_empty() && b.failures.is_empty());
    let strip = |s: &runner::Sample| (s.tick, s.gold, s.ore, s.crafted, s.sold, s.staples);
    assert_eq!(strip(&a.end), strip(&b.end));
    assert_eq!(b.offline[0].uncredited, 0);
}

#[test]
fn offline_is_capped_like_the_game() {
    let mut sc = scenario("away", Preset::LegacyDagger, Driver::Idle, 20 * 3600);
    sc.offline = vec![Offline {
        at: 0,
        secs: 20 * 3600,
    }];
    let r = run(&sc).unwrap();
    assert_eq!(r.offline[0].settled, forge_core::OFFLINE_CAP_SECS);
    assert_eq!(
        r.offline[0].uncredited,
        20 * 3600 - forge_core::OFFLINE_CAP_SECS
    );
    assert_eq!(r.end.tick, forge_core::OFFLINE_CAP_SECS);
}

#[test]
fn diligent_player_finishes_the_first_loop() {
    let r = run(&scenario(
        "player",
        Preset::Fresh,
        Driver::Player(Player::default()),
        6 * 3600,
    ))
    .unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(r.end.chef, "Delivered");
    assert_eq!(r.end.farmer, "Delivered");
    assert!(r.end.crafted > 0 && r.end.sold > 0);
    // Each order paid once, through its own command.
    assert_eq!(r.commands["DeliverChef"].ok, 1);
    assert_eq!(r.commands["DeliverFarmerRepair"].ok, 1);
}

#[test]
fn matrix_expands_every_combination_and_seed() {
    let m: Matrix = serde_json::from_value(serde_json::json!({
        "base": {"name": "m", "seed": 0, "duration_secs": 60, "driver": {"kind": "player"}},
        "seeds": {"from": 1, "count": 3},
        "vary": {"/start/gold": [0, 40], "/driver/buy_ore": [true, false]}
    }))
    .unwrap();
    let cases = expand(&m).unwrap();
    assert_eq!(cases.len(), 2 * 2 * 3);
    assert_eq!(cases[0].scenario.start.gold, Some(0));
    assert!(matches!(&cases[0].scenario.driver, Driver::Player(p) if p.buy_ore));
    assert_eq!(cases[2].scenario.seed, 3);
    assert!(cases[0].scenario.name.contains("/start/gold=0"));
}

#[test]
fn smoke_matrix_has_no_invariant_failures() {
    let m: Matrix = serde_json::from_str(include_str!("../scenarios/matrix-smoke.json")).unwrap();
    let cases = expand(&m).unwrap();
    let results = run_all(&cases, 4);
    for (c, r) in cases.iter().zip(&results) {
        let r = r.as_ref().unwrap();
        assert!(
            r.failures.is_empty(),
            "{}: {:?}",
            c.scenario.name,
            r.failures
        );
    }
    let csv = summary_csv(&cases, &results);
    assert_eq!(csv.lines().count(), cases.len() + 1);
}

#[test]
fn bundled_scenarios_load() {
    for text in [
        include_str!("../scenarios/craft-and-sale.json"),
        include_str!("../scenarios/player-day.json"),
        include_str!("../scenarios/offline-8h.json"),
    ] {
        Scenario::from_json(text).unwrap();
    }
    for text in [
        include_str!("../scenarios/matrix-smoke.json"),
        include_str!("../scenarios/matrix-v0.json"),
    ] {
        expand(&serde_json::from_str(text).unwrap()).unwrap();
    }
}

#[test]
fn unknown_fields_and_commands_are_rejected() {
    assert!(Scenario::from_json(r#"{"name":"x","seed":1,"duration_secs":1,"typo":1}"#).is_err());
    assert!(
        Scenario::from_json(
            r#"{"name":"x","seed":1,"duration_secs":1,"driver":{"kind":"script","commands":[{"at":0,"command":"MakeGold"}]}}"#
        )
        .is_err()
    );
}
