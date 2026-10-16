//! Headless reproducible scenario and CPU simulation benchmark (not renderer performance).
mod lifestyle;
use forge_core::{Command, State, save::Save};
use std::time::Instant;
fn main() {
    if std::env::args().any(|a| a == "life-loop") {
        println!(
            "{}",
            serde_json::to_string_pretty(&lifestyle::report()).unwrap()
        );
        return;
    }
    forge_content::Manifest::bundled().expect("valid content");
    let start = Instant::now();
    let mut checksum = 0;
    for seed in 1..=10_000 {
        let mut state = State::new(seed);
        state.command(1, Command::Craft).unwrap();
        state.advance(3600).unwrap();
        assert_eq!(state.gold, 12);
        assert_eq!(state.sold, 1);
        checksum += state.event_id;
    }
    let elapsed = start.elapsed();
    let mut state = State::new(42);
    state.command(1, Command::Craft).unwrap();
    let events = state.advance(20).unwrap();
    let report = serde_json::json!({"scenario":"craft-and-sale","state":state,"events":events,"benchmark":{"kind":"simulation-only","runs":10000,"simulated_seconds_per_run":3600,"elapsed_ms":elapsed.as_secs_f64()*1000.,"event_checksum":checksum},"platform":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH}});
    let original = Save::new(state, 100);
    let encoded = serde_json::to_vec(&original).unwrap();
    let decoded: Save = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(original, decoded);
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
