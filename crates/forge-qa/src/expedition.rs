use forge_core::{
    Command,
    expedition::{Loadout, qa_fixture},
};
pub fn report() -> serde_json::Value {
    let mut results = vec![];
    for (name, primary, secondary, food, arrows) in [
        ("bow-only", 1, None, 5, 24),
        ("knife-only", 2, None, 5, 0),
        ("combined", 1, Some(2), 5, 24),
        ("no-food", 1, Some(2), 0, 24),
        ("no-arrows", 1, Some(2), 5, 0),
    ] {
        let mut s = qa_fixture();
        s.command(
            s.command_id + 1,
            Command::LaunchExpedition(Loadout {
                primary: Some(primary),
                secondary,
                food,
                arrows,
            }),
        )
        .unwrap();
        s.advance(600).unwrap();
        let b = s.expedition.active.as_ref().unwrap();
        if name == "combined" {
            assert_eq!(b.cleared, 5);
        } else {
            assert!(b.cleared < 5);
        }
        results.push(serde_json::json!({"scenario":name,"battle":b,"weapons":s.expedition.weapons,"path_reopened":s.expedition.path_reopened}));
    }
    serde_json::json!({"combat_version":forge_core::expedition::VERSION,"results":results})
}

pub fn crafting_report() -> serde_json::Value {
    use forge_core::expedition::Action;
    use forge_core::weapon::{Approach, Material, Plan, Technique, qa_smith};
    let mut results = vec![];
    for material in [Material::Iron, Material::Steel] {
        for technique in [Technique::LightEdge, Technique::ReinforcedPoint] {
            for approach in [Approach::Quick, Approach::Standard, Approach::Careful] {
                for bow in [false, true] {
                    let mut s = qa_smith();
                    s.command(
                        s.command_id + 1,
                        Command::ForgeWeapon(Plan {
                            material,
                            technique,
                            approach,
                        }),
                    )
                    .unwrap();
                    s.advance(30).unwrap();
                    let craft = s.expedition.weapons[1].craftsmanship.clone().unwrap();
                    s.command(
                        s.command_id + 1,
                        Command::LaunchExpedition(Loadout {
                            primary: Some(if bow { 1 } else { 2 }),
                            secondary: bow.then_some(2),
                            food: 5,
                            arrows: if bow { 24 } else { 0 },
                        }),
                    )
                    .unwrap();
                    s.advance(600).unwrap();
                    let b = s.expedition.active.as_ref().unwrap();
                    let damage: u32 = b
                        .trace
                        .iter()
                        .filter(|t| t.action == Action::Hit)
                        .map(|t| u32::from(t.amount))
                        .sum();
                    let contribution: u32 = b
                        .trace
                        .iter()
                        .filter(|t| t.weapon == Some(2))
                        .map(|t| u32::from(t.amount))
                        .sum();
                    results.push(serde_json::json!({"craft":craft,"bow":bow,"waves_cleared":b.cleared,"duration_seconds":b.second,"hunter_damage":damage,"ammunition_consumed":b.loadout.arrows-b.arrows,"provisions_consumed":b.loadout.food-b.food,"knife_damage":contribution,"waves_per_ore":f64::from(b.cleared)/f64::from(craft.ore),"cause":b.cause}));
                }
            }
        }
    }
    serde_json::json!({"version":2,"seed":42,"runs_per_build":1,"note":"Deterministic duration is also the mean for repeated identical runs; knife investment excludes the fixed bow/provisions.","results":results})
}
