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
fn ore_dagger_runaway_baseline_is_unchanged() {
    // Historical regression baseline (recorded on main 584a58d, before the crafting engine):
    // buy ore → dagger → visitor sale compounds without limit. Changing these numbers is a
    // deliberate balance change and must update docs/system/ECONOMY_SIM.md with it.
    let sc = Scenario::from_json(include_str!(
        "../scenarios/baseline-ore-dagger-runaway.json"
    ))
    .unwrap();
    let r = run(&sc).unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(
        (r.end.gold, r.end.crafted, r.end.sold),
        (28_446, 3_564, 3_560)
    );
    assert_eq!(r.ledger.get("sale"), Some(&42_720));
}

#[test]
fn crafting_chain_makes_goods_from_bought_materials() {
    let sc = Scenario::from_json(include_str!("../scenarios/chain-day.json")).unwrap();
    let r = run(&sc).unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(r.end.chef, "Delivered");
    for item in [
        "iron_ingot",
        "straight_blade",
        "wooden_handle",
        "kitchen_knife",
        "wooden_club",
        "wooden_pole",
        "wooden_staff",
        "frying_pan",
        "wooden_plank",
        "metal_rivet",
        "leather_strap",
        "wooden_shield",
        "farming_hoe",
        "sickle",
        "woodcutting_axe",
        "cleaver",
        "hand_saw",
    ] {
        assert!(
            r.produced.get(item).copied().unwrap_or(0) > 0,
            "{item} never produced"
        );
    }
    assert!(
        r.ledger.get("cmd:Buy").copied().unwrap_or(0) < 0,
        "materials were bought"
    );
}

#[test]
fn running_out_of_gold_stops_production_cleanly() {
    // Vendor stock is unlimited (owner, 2026-10-20), so gold is the brake: each knife needs
    // 2 ore (4g) + charcoal (1g) + ash log (3g) + pine resin (2g) = 10g of inputs.
    let sc = Scenario::from_json(include_str!("../scenarios/chain-gold-out.json")).unwrap();
    let r = run(&sc).unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert!(r.produced.get("kitchen_knife").copied().unwrap_or(0) > 1);
    assert!(
        r.end.gold < 10,
        "spent down to below one knife's inputs: {}",
        r.end.gold
    );
    assert!(r.commands.get("Buy").is_some_and(|t| t.ok > 0));
}

#[test]
fn zero_gold_without_purchases_still_finishes_the_chef_order() {
    let mut sc = Scenario::from_json(include_str!("../scenarios/chain-day.json")).unwrap();
    sc.duration_secs = 4 * 3600;
    if let Driver::Player(p) = &mut sc.driver {
        p.buy_materials = false;
        p.buy_ore = false;
        p.make.clear();
    }
    let r = run(&sc).unwrap();
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(
        r.end.chef, "Delivered",
        "the new-game kit covers the first knife"
    );
}

#[test]
fn bundled_scenarios_load() {
    for text in [
        include_str!("../scenarios/craft-and-sale.json"),
        include_str!("../scenarios/player-day.json"),
        include_str!("../scenarios/offline-8h.json"),
        include_str!("../scenarios/baseline-ore-dagger-runaway.json"),
        include_str!("../scenarios/chain-day.json"),
        include_str!("../scenarios/chain-gold-out.json"),
    ] {
        Scenario::from_json(text).unwrap();
    }
    for text in [
        include_str!("../scenarios/matrix-smoke.json"),
        include_str!("../scenarios/matrix-v0.json"),
        include_str!("../scenarios/matrix-crafting.json"),
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
