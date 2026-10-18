//! Finished goods and authoritative work stages. Presentation maps stages to stations and clips;
//! the stage is derived from remaining work, so view changes and offline catch-up cannot replay it.
use crate::{CRAFT_SECS, State, spaces::Workstation};
use serde::{Deserialize, Serialize};

/// Goods that can occupy a showroom slot. Item IDs are shared with the content catalog (#74/#77);
/// only the Foundation dagger exists today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Good {
    IronDagger,
}
impl Good {
    pub const ALL: [Self; 1] = [Self::IronDagger];
    pub fn id(self) -> &'static str {
        match self {
            Self::IronDagger => "iron_dagger",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::IronDagger => "Iron dagger",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Self::IronDagger => "dagger",
        }
    }
    pub fn price(self) -> u64 {
        match self {
            Self::IronDagger => crate::SALE_PRICE,
        }
    }
    /// Ordered recipe stages in permille of logical work time. Not every recipe needs all three.
    pub fn stages(self) -> &'static [(Stage, u32)] {
        match self {
            Self::IronDagger => &[
                (Stage::Heating, 300),
                (Stage::Hammering, 500),
                (Stage::Quenching, 200),
            ],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Heating,
    Hammering,
    Quenching,
    Researching,
}
impl Stage {
    pub fn station(self) -> Workstation {
        match self {
            Self::Researching => Workstation::ResearchTable,
            Self::Heating => Workstation::Forge,
            Self::Hammering => Workstation::Anvil,
            Self::Quenching => Workstation::QuenchingTub,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Researching => "Researching",
            Self::Heating => "Heating",
            Self::Hammering => "Hammering",
            Self::Quenching => "Quenching",
        }
    }
}
/// Stage for a job at `progress` permille (0 = just started, 1000 = complete).
pub fn stage_at(good: Good, progress: u32) -> Stage {
    let mut acc = 0;
    let stages = good.stages();
    for (stage, share) in stages {
        acc += share;
        if progress < acc {
            return *stage;
        }
    }
    stages.last().map(|s| s.0).unwrap_or(Stage::Hammering)
}
impl State {
    /// The good currently being made. One recipe exists; the queue is homogeneous.
    pub fn current_good(&self) -> Good {
        Good::IronDagger
    }
    /// Permille progress of the active job, if any. Independent of fatigue speed scaling.
    pub fn job_progress(&self) -> Option<u32> {
        if self.life.work == 0 {
            None
        } else {
            let total = self
                .life
                .active_job
                .map_or(CRAFT_SECS, |job| self.work_seconds(job))
                * 1000;
            Some(((total - self.life.work.min(total)) * 1000 / total) as u32)
        }
    }
    /// Entrusted repairs are anvil work throughout; general crafting follows the recipe stages.
    pub fn work_stage(&self) -> Option<Stage> {
        let progress = self.job_progress()?;
        Some(match self.life.active_job {
            Some(crate::life::WorkKind::FarmerSickleRepair) => Stage::Hammering,
            Some(crate::life::WorkKind::DaggerStudy) => Stage::Researching,
            _ => stage_at(self.current_good(), progress),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stages_cover_full_progress_in_order() {
        assert_eq!(stage_at(Good::IronDagger, 0), Stage::Heating);
        assert_eq!(stage_at(Good::IronDagger, 299), Stage::Heating);
        assert_eq!(stage_at(Good::IronDagger, 300), Stage::Hammering);
        assert_eq!(stage_at(Good::IronDagger, 799), Stage::Hammering);
        assert_eq!(stage_at(Good::IronDagger, 800), Stage::Quenching);
        assert_eq!(stage_at(Good::IronDagger, 1000), Stage::Quenching);
        let total: u32 = Good::IronDagger.stages().iter().map(|s| s.1).sum();
        assert_eq!(total, 1000);
    }
    #[test]
    fn stage_is_derived_from_authoritative_work_only() {
        let mut s = State::foundation(1);
        assert_eq!(s.work_stage(), None);
        s.command(1, crate::Command::Craft).unwrap();
        assert_eq!(s.work_stage(), Some(Stage::Heating));
        let mut slow = s.clone();
        slow.life.fatigue = 9000;
        s.advance(2).unwrap();
        slow.advance(2).unwrap();
        assert_eq!(s.work_stage(), Some(Stage::Hammering));
        // A tired smith reaches the same stage later, but the stage never runs ahead of work.
        assert!(slow.job_progress().unwrap() < s.job_progress().unwrap());
        s.advance(2).unwrap();
        assert_eq!(s.work_stage(), Some(Stage::Quenching));
        s.advance(1).unwrap();
        assert_eq!(s.work_stage(), None);
        assert_eq!(s.crafted, 1);
    }
}
