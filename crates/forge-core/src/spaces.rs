//! One forge, three views. The views are ephemeral routing; the activity below is derived from the
//! authoritative state so every view and the offline catch-up agree without replaying motion.
use crate::{State, life::SmithAction};
use serde::{Deserialize, Serialize};

/// Which part of the player's own forge is shown. Separate from the village map and from the
/// window mode (ambient/expanded), which the desktop shell owns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Space {
    #[default]
    Exterior,
    Workshop,
    Showroom,
}
impl Space {
    pub const ALL: [Self; 3] = [Self::Exterior, Self::Workshop, Self::Showroom];
    pub fn label(self) -> &'static str {
        match self {
            Self::Exterior => "Outside",
            Self::Workshop => "Workshop",
            Self::Showroom => "Showroom",
        }
    }
    pub fn interior(self) -> bool {
        self != Self::Exterior
    }
    pub fn next(self) -> Self {
        match self {
            Self::Exterior => Self::Workshop,
            Self::Workshop => Self::Showroom,
            Self::Showroom => Self::Exterior,
        }
    }
}
/// The five workshop stations. IDs match `station_id` in the character animation manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Workstation {
    Forge,
    Anvil,
    QuenchingTub,
    ResearchTable,
    SalvageTable,
}
impl Workstation {
    pub const ALL: [Self; 5] = [
        Self::Forge,
        Self::Anvil,
        Self::QuenchingTub,
        Self::ResearchTable,
        Self::SalvageTable,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Forge => "forge",
            Self::Anvil => "anvil",
            Self::QuenchingTub => "quenching-tub",
            Self::ResearchTable => "research-table",
            Self::SalvageTable => "salvage-table",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Forge => "Forge",
            Self::Anvil => "Anvil",
            Self::QuenchingTub => "Quenching tub",
            Self::ResearchTable => "Research table",
            Self::SalvageTable => "Salvage table",
        }
    }
    /// Work clip played at this station, as named in animation.json.
    pub fn clip(self) -> &'static str {
        match self {
            Self::Forge => "forging",
            Self::Anvil => "hammering",
            Self::QuenchingTub => "quenching",
            Self::ResearchTable => "researching",
            Self::SalvageTable => "salvaging",
        }
    }
    /// Research and salvage are not simulated yet (#72/#76). They are preview fixtures only.
    pub fn simulated(self) -> bool {
        matches!(self, Self::Forge | Self::Anvil | Self::QuenchingTub)
    }
}
/// Why a queued general craft cannot start. Never a reason to debit materials again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    ShowroomFull,
    LegacyOverflow,
}
impl BlockReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::ShowroomFull => "Showroom full: sell or remove a finished item to continue",
            Self::LegacyOverflow => {
                "Older stock exceeds the showroom: it sells first, then crafting resumes"
            }
        }
    }
}
/// Presentation-facing summary of what the smith is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Idle,
    Working { station: Workstation, progress: u32 },
    Waiting(BlockReason),
    Sleeping,
    Eating,
    Drinking,
}
impl State {
    pub fn craft_block_reason(&self) -> Option<BlockReason> {
        self.showroom.block_reason()
    }
    pub fn activity(&self) -> Activity {
        match self.life.action {
            SmithAction::Sleeping => Activity::Sleeping,
            SmithAction::Eating => Activity::Eating,
            SmithAction::Drinking => Activity::Drinking,
            SmithAction::Working | SmithAction::Idle => {
                match (self.work_stage(), self.job_progress()) {
                    (Some(stage), Some(progress)) => Activity::Working {
                        station: stage.station(),
                        progress,
                    },
                    _ => match (
                        self.life.queue.iter().any(|j| !self.can_start(*j)),
                        self.craft_block_reason(),
                    ) {
                        (true, Some(reason)) => Activity::Waiting(reason),
                        _ => Activity::Idle,
                    },
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    #[test]
    fn spaces_cycle_and_station_ids_round_trip() {
        let mut s = Space::default();
        for _ in 0..3 {
            s = s.next();
        }
        assert_eq!(s, Space::Exterior);
        for w in Workstation::ALL {
            assert_eq!(Workstation::from_id(w.id()), Some(w));
        }
        assert!(!Workstation::ResearchTable.simulated());
    }
    #[test]
    fn activity_follows_authoritative_state_and_full_showroom_waits() {
        let mut s = State::foundation(3);
        assert_eq!(s.activity(), Activity::Idle);
        s.command(1, Command::Craft).unwrap();
        assert!(matches!(
            s.activity(),
            Activity::Working {
                station: Workstation::Forge,
                ..
            }
        ));
        // Fill the showroom with queued jobs; the visitor clock is pushed out so nothing sells.
        s.next_visitor = 10_000;
        for id in 2..=5 {
            s.command(id, Command::QueueCraft).unwrap();
        }
        assert_eq!(s.ore, 0);
        s.advance(60).unwrap();
        assert_eq!(s.showroom.free(), 0);
        assert_eq!(s.life.queue.len(), 1);
        assert_eq!(s.activity(), Activity::Waiting(BlockReason::ShowroomFull));
        assert_eq!(s.life.action, SmithAction::Idle);
    }
}
