//! The shop customer is the hunter NPC with its own authored frames, never the player sheet.
use bevy::prelude::*;

/// Frame ids under `npc/hunter/frames/`. Walk 0..6, then neutral and the give clip.
pub const FRAMES: [&str; 11] = [
    "walk-0",
    "walk-1",
    "walk-2",
    "walk-3",
    "walk-4",
    "walk-5",
    "neutral",
    "give-mid",
    "give-full",
    "give-empty",
    "give-retract",
];
pub const NEUTRAL: usize = 6;
/// The hunter's walk clip: six frames, 120 ms each (assets/npc/hunter/animation.json).
const WALK_FRAME_SECS: f32 = 0.12;
/// The give clip from `animation.json`: neutral, mid, full, empty, retract (ms 120/120/180/120/120).
const GIVE: [(usize, f32); 5] = [(6, 0.12), (7, 0.12), (8, 0.18), (9, 0.12), (10, 0.12)];

pub fn paths() -> impl Iterator<Item = String> {
    FRAMES.iter().map(|f| format!("npc/hunter/frames/{f}.png"))
}

pub fn walk(elapsed: f32) -> usize {
    (elapsed / WALK_FRAME_SECS) as usize % 6
}

/// Workshop sale: walk in (0-3 s), hand over at the counter (3-4.5 s), walk out flipped.
pub fn sale_frame(elapsed: f32) -> (usize, bool) {
    if elapsed < 3. {
        return (walk(elapsed), false);
    }
    if elapsed >= 4.5 {
        return (walk(elapsed), true);
    }
    let mut t = elapsed - 3.;
    for (frame, secs) in GIVE {
        if t < secs {
            return (frame, false);
        }
        t -= secs;
    }
    (NEUTRAL, false)
}

/// Item hand per frame, relative to the foot pivot (y up). The give clip reaches with the left
/// hand; walking and standing carry in the right hand.
pub fn hands() -> Vec<Vec2> {
    let recipes: serde_json::Value = serde_json::from_str(include_str!(
        "../../../assets/npc/hunter/poses/recipes.json"
    ))
    .expect("bundled hunter pose recipes");
    FRAMES
        .iter()
        .map(|f| {
            let sockets = &recipes["poses"][f]["sockets_px"];
            let hand = if f.starts_with("give") {
                &sockets["left_hand"]
            } else {
                &sockets["right_hand"]
            };
            let x = hand[0].as_f64().expect("hand x") as f32;
            let y = hand[1].as_f64().expect("hand y") as f32;
            Vec2::new(x - crate::interior::PIVOT.x, crate::interior::PIVOT.y - y)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_frame_exists_and_has_a_hand() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        for path in paths() {
            assert!(root.join(&path).is_file(), "{path}");
        }
        assert_eq!(hands().len(), FRAMES.len());
    }
    #[test]
    fn sale_walks_in_hands_over_then_walks_out_flipped() {
        assert_eq!(sale_frame(0.), (0, false));
        assert!(sale_frame(2.9).0 < NEUTRAL);
        assert_eq!(sale_frame(3.3), (8, false));
        assert_eq!(sale_frame(4.4), (NEUTRAL, false));
        assert!(sale_frame(5.).1);
    }
}
