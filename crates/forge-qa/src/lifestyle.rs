use forge_core::{Command, EventKind, State, life::SmithAction, save::Save};
pub fn report() -> serde_json::Value {
    let mut s = State::new(42);
    s.life.config = forge_content::needs_config().unwrap();
    s.life.fatigue = 8990;
    s.life.hunger = 8000;
    s.life.thirst = 8000;
    s.command(1, Command::Craft).unwrap();
    s.command(2, Command::QueueCraft).unwrap();
    let mut events = vec![];
    for _ in 0..300 {
        events.extend(s.advance(1).unwrap());
        if s.sold > 0 && s.command_id == 2 {
            s.command(3, Command::BuyFood).unwrap();
        }
    }
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::ActionChanged(SmithAction::Sleeping))
    );
    assert!(events.iter().any(|e| e.kind == EventKind::Ate));
    assert!(events.iter().any(|e| e.kind == EventKind::Drank));
    assert_eq!(s.sold, 2);
    assert_eq!(s.life.pantry.staples(), 3);
    let start = std::time::Instant::now();
    let mut multiday = s.clone();
    let mut sleeps = 0;
    let mut foods = 0;
    for _ in 0..3 * 86400 {
        // Scripted player policy; production never automatically spends gold.
        if multiday.life.queue < 2 {
            if multiday.ore < 2 && multiday.gold >= 4 {
                multiday
                    .command(multiday.command_id + 1, Command::BuyOre)
                    .unwrap();
            }
            if multiday.ore >= 2 {
                multiday
                    .command(multiday.command_id + 1, Command::QueueCraft)
                    .unwrap();
            }
        }
        if multiday.life.pantry.staples() == 0 && multiday.gold >= 2 {
            multiday
                .command(multiday.command_id + 1, Command::BuyFood)
                .unwrap();
        }
        for e in multiday.advance(1).unwrap() {
            if e.kind == EventKind::ActionChanged(SmithAction::Sleeping) {
                sleeps += 1;
            }
            if e.kind == EventKind::Ate {
                foods += 1;
            }
        }
        if multiday.tick.is_multiple_of(600) {
            let saved = Save::new(multiday.clone(), multiday.tick);
            let mut decoded: Save =
                serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
            decoded.catch_up(multiday.tick).unwrap();
            assert_eq!(saved, decoded);
            multiday = decoded.state;
        }
    }
    assert!(sleeps > 0 && foods > 0 && multiday.sold > 2);
    multiday.validate().unwrap();
    serde_json::json!({"scenario":"life-loop","state":s,"events":events,"multiday":{"eligible_seconds":3*86400,"wall_ms":start.elapsed().as_secs_f64()*1000.,"state":multiday,"sleep_count":sleeps,"staples_eaten":foods,"policy":"scripted player purchases/queue, not production auto-spend"}})
}
