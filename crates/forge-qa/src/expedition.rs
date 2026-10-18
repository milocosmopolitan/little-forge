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
