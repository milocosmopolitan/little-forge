//! Alpha progression uses the same command transaction and work clock as ordinary crafting.
use crate::{Command, Error, Event, EventKind, State, commission::OrderPhase, life::WorkKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Technique {
    pub id: String,
    pub label: String,
    pub level_xp: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub label: String,
    pub ore: u32,
    pub seconds: u64,
    pub general_xp: u64,
    pub technique_xp: [u64; 2],
    pub research_xp: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChefDefinition {
    pub id: String,
    pub reward_gold: u64,
    pub request: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchDefinition {
    pub id: String,
    pub goal_xp: u64,
    pub technique_levels: [u64; 2],
    pub prerequisites: Vec<String>,
    pub study_seconds: u64,
    pub study_xp: u64,
    pub study_gold: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rules {
    pub version: u32,
    pub techniques: [Technique; 2],
    pub recipes: [Recipe; 2],
    pub chef: ChefDefinition,
    pub dagger: ResearchDefinition,
}
impl Rules {
    pub fn validate(&self) -> Result<(), Error> {
        // This slice deliberately has one root research node. Unknown dependencies and
        // self-dependencies are rejected instead of pretending a larger tree is implemented.
        if self.version != 1
            || self.techniques[0].id != "blade_forging"
            || self.techniques[1].id != "handle_fitting"
            || self.recipes[0].id != "kitchen_knife"
            || self.recipes[1].id != "iron_dagger"
            || self.chef.id != "chef_first_knife"
            || self.dagger.id != "dagger_structure"
            || !self.dagger.prerequisites.is_empty()
            || self.dagger.goal_xp == 0
            || self.dagger.study_xp == 0
            || !(1..=3600).contains(&self.dagger.study_seconds)
            || self.dagger.study_gold > self.chef.reward_gold
            || self
                .recipes
                .iter()
                .any(|r| r.ore == 0 || r.ore > 5 || !(1..=3600).contains(&r.seconds))
        {
            return Err(Error::InvalidState);
        }
        for (i, t) in self.techniques.iter().enumerate() {
            if t.level_xp == 0
                || self.dagger.technique_levels[i] > 2
                || self.recipes[0].technique_xp[i] / t.level_xp < self.dagger.technique_levels[i]
            {
                return Err(Error::InvalidState);
            }
        }
        // Chef plus study must always reach completion without repeat orders or RNG.
        if self.recipes[0]
            .research_xp
            .saturating_add(self.dagger.study_xp)
            < self.dagger.goal_xp
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub first_tick: u64,
    pub first_item_id: u64,
    pub count: u64,
    /// No quality calculator exists yet. Never invent a best-quality result.
    pub best_quality: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Milestone {
    pub id: String,
    pub tick: u64,
    pub text: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progression {
    pub weapons: Vec<crate::expedition::Weapon>,
    pub rules: Option<Rules>,
    pub legacy_dagger: bool,
    pub chef_phase: OrderPhase,
    pub general_xp: u64,
    pub technique_xp: [u64; 2],
    pub clues: Vec<String>,
    pub research_xp: u64,
    pub study_pending: bool,
    pub research_completed: bool,
    pub codex: BTreeMap<String, Record>,
    pub journal: Vec<Milestone>,
    pub next_item_id: u64,
}
impl Progression {
    pub fn level(&self, i: usize) -> u64 {
        self.rules.as_ref().map_or(0, |r| {
            (self.technique_xp[i] / r.techniques[i].level_xp).min(2)
        })
    }
    pub fn discovered(&self) -> bool {
        self.clues.iter().any(|c| c == "chef_balance")
    }
    pub fn eligible(&self) -> bool {
        self.discovered()
            && self
                .rules
                .as_ref()
                .is_some_and(|r| (0..2).all(|i| self.level(i) >= r.dagger.technique_levels[i]))
    }
    pub fn can_complete(&self) -> bool {
        self.eligible()
            && !self.study_pending
            && !self.research_completed
            && self
                .rules
                .as_ref()
                .is_some_and(|r| self.research_xp >= r.dagger.goal_xp)
    }
    pub fn dagger_unlocked(&self) -> bool {
        self.legacy_dagger || self.research_completed
    }
    pub(crate) fn note(&mut self, tick: u64, id: &str, text: String) {
        if !self.journal.iter().any(|m| m.id == id) {
            self.journal.push(Milestone {
                id: id.into(),
                tick,
                text,
            });
        }
    }
}
impl State {
    /// Content supplies the rules; core owns all subsequent rewards and eligibility.
    pub fn with_progression(seed: u64, rules: Rules) -> Result<Self, Error> {
        rules.validate()?;
        let mut state = Self::new(seed);
        state.progression.rules = Some(rules);
        Ok(state)
    }
    pub fn work_seconds(&self, job: WorkKind) -> u64 {
        match (job, self.progression.rules.as_ref()) {
            (WorkKind::KitchenKnife, Some(r)) => r.recipes[0].seconds,
            (WorkKind::DaggerStudy, Some(r)) => r.dagger.study_seconds,
            (WorkKind::IronDagger, Some(r)) => r.recipes[1].seconds,
            _ => job.seconds(),
        }
    }
    pub fn dagger_ore(&self) -> u32 {
        self.progression
            .rules
            .as_ref()
            .map_or(2, |r| r.recipes[1].ore)
    }
    pub(crate) fn progression_command(&mut self, command: Command) -> Result<(), Error> {
        let rules = self.progression.rules.clone().ok_or(Error::InvalidState)?;
        let p = &mut self.progression;
        match command {
            Command::AcceptChef => {
                if p.chef_phase != OrderPhase::Offered {
                    return Err(Error::InvalidOrderState);
                }
                p.chef_phase = OrderPhase::Accepted;
            }
            Command::QueueChef => {
                if p.chef_phase != OrderPhase::Accepted {
                    return Err(Error::InvalidOrderState);
                }
                if self.life.queue.len() >= 32 {
                    return Err(Error::Busy);
                }
                if self.ore < rules.recipes[0].ore {
                    return Err(Error::InsufficientResources);
                }
                self.ore -= rules.recipes[0].ore;
                self.life.queue.push_back(WorkKind::KitchenKnife);
                p.chef_phase = OrderPhase::Queued;
            }
            Command::DeliverChef => {
                if p.chef_phase != OrderPhase::Ready {
                    return Err(Error::InvalidOrderState);
                }
                self.gold = self
                    .gold
                    .checked_add(rules.chef.reward_gold)
                    .ok_or(Error::Overflow)?;
                p.chef_phase = OrderPhase::Delivered;
                p.note(self.tick, "chef_delivered", format!("Chef's first commission delivered. +{} gold. A knife for the village kitchen.", rules.chef.reward_gold));
                p.clues.push("chef_balance".into());
                p.note(
                    self.tick,
                    "dagger_discovered",
                    "Dagger research discovered: a balanced blade and a secure handle.".into(),
                );
            }
            Command::StudyDagger => {
                if !p.eligible()
                    || p.research_completed
                    || p.study_pending
                    || p.research_xp >= rules.dagger.goal_xp
                {
                    return Err(Error::ResearchLocked);
                }
                if self.life.queue.len() >= 32 {
                    return Err(Error::Busy);
                }
                if self.gold < rules.dagger.study_gold {
                    return Err(Error::InsufficientResources);
                }
                self.gold -= rules.dagger.study_gold;
                p.study_pending = true;
                self.life.queue.push_back(WorkKind::DaggerStudy);
            }
            Command::CompleteDaggerResearch => {
                if !p.can_complete() {
                    return Err(Error::ResearchLocked);
                }
                p.research_completed = true;
                p.note(
                    self.tick,
                    "dagger_researched",
                    "Dagger research completed. Your first weapon recipe is ready to forge.".into(),
                );
            }
            _ => return Err(Error::InvalidState),
        }
        Ok(())
    }
    pub(crate) fn record_creation(
        &mut self,
        recipe: usize,
        events: &mut Vec<Event>,
    ) -> Result<(), Error> {
        let Some(rules) = self.progression.rules.clone() else {
            return Ok(());
        };
        let r = &rules.recipes[recipe];
        let p = &mut self.progression;
        p.general_xp = p
            .general_xp
            .checked_add(r.general_xp)
            .ok_or(Error::Overflow)?;
        for i in 0..2 {
            let old = p.level(i);
            p.technique_xp[i] = p.technique_xp[i]
                .checked_add(r.technique_xp[i])
                .ok_or(Error::Overflow)?;
            let level = p.level(i);
            if level > old {
                p.note(
                    self.tick,
                    &format!("{}:{level}", rules.techniques[i].id),
                    format!("{} reached level {level}.", rules.techniques[i].label),
                );
            }
        }
        p.research_xp = p
            .research_xp
            .saturating_add(r.research_xp)
            .min(rules.dagger.goal_xp);
        p.next_item_id = p.next_item_id.checked_add(1).ok_or(Error::Overflow)?;
        if let Some(record) = p.codex.get_mut(&r.id) {
            record.count = record.count.checked_add(1).ok_or(Error::Overflow)?;
        } else {
            p.codex.insert(
                r.id.clone(),
                Record {
                    first_tick: self.tick,
                    first_item_id: p.next_item_id,
                    count: 1,
                    best_quality: None,
                },
            );
            p.note(
                self.tick,
                &format!("first:{}", r.id),
                format!("First {} forged and recorded in Smith's Codex.", r.label),
            );
        }
        self.emit(EventKind::ProgressionChanged, events)
    }
    pub(crate) fn validate_progression(&self) -> Result<(), Error> {
        let p = &self.progression;
        for (i, w) in p.weapons.iter().enumerate() {
            crate::weapon::validate_record(w)?;
            if w.craftsmanship.is_none()
                || w.id == 0
                || w.id > self.expedition.next_weapon
                || p.weapons[..i].iter().any(|old| old.id == w.id)
            {
                return Err(Error::InvalidState);
            }
        }
        if let Some(rules) = &p.rules {
            rules.validate()?;
            if p.research_xp > rules.dagger.goal_xp
                || (p.research_completed && (!p.eligible() || p.research_xp < rules.dagger.goal_xp))
                || p.discovered() != (p.chef_phase == OrderPhase::Delivered)
                || p.clues.len() != usize::from(p.discovered())
            {
                return Err(Error::InvalidState);
            }
        } else if p
            != &(Progression {
                legacy_dagger: p.legacy_dagger,
                ..Default::default()
            })
        {
            return Err(Error::InvalidState);
        }
        let queued = |job| self.life.queue.iter().filter(|&&j| j == job).count();
        if queued(WorkKind::KitchenKnife) != usize::from(p.chef_phase == OrderPhase::Queued)
            || (self.life.active_job == Some(WorkKind::KitchenKnife))
                != (p.chef_phase == OrderPhase::Working)
            || queued(WorkKind::DaggerStudy)
                + usize::from(self.life.active_job == Some(WorkKind::DaggerStudy))
                != usize::from(p.study_pending)
            || self
                .life
                .active_job
                .is_some_and(|j| self.life.work > self.work_seconds(j) * 1000)
            || p.codex.iter().any(|(id, r)| {
                !["kitchen_knife", "iron_dagger", "hunting_knife"].contains(&id.as_str())
                    || r.count == 0
                    || r.first_tick > self.tick
                    || r.first_item_id > p.next_item_id
            })
            || p.journal.iter().any(|m| m.tick > self.tick)
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Smith,
        save::{self, Save},
    };
    fn rules() -> Rules {
        serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap()
    }
    fn game(seed: u64) -> State {
        let mut s = State::with_progression(seed, rules()).unwrap();
        s.life.config.sleep_seconds = 2;
        s
    }
    fn cmd(s: &mut State, c: Command) {
        s.command(s.command_id + 1, c).unwrap();
    }
    fn knife(s: &mut State) {
        cmd(s, Command::AcceptChef);
        cmd(s, Command::QueueChef);
        s.advance(20).unwrap();
        assert_eq!(s.progression.chef_phase, OrderPhase::Ready);
    }
    #[test]
    fn both_smiths_finish_first_legacy_without_rng_and_continue_idle() {
        for smith in Smith::ALL {
            for seed in 1..=32 {
                let mut s = game(seed);
                cmd(&mut s, Command::ChooseSmith(smith));
                assert_eq!(
                    s.command(s.command_id + 1, Command::Craft),
                    Err(Error::ResearchLocked)
                );
                knife(&mut s);
                assert_eq!(s.progression.general_xp, 10);
                assert_eq!(s.progression.technique_xp, [6, 3]);
                assert_eq!([s.progression.level(0), s.progression.level(1)], [1, 1]);
                assert!(!s.progression.discovered());
                assert_eq!(s.daggers, 0);
                assert_eq!(s.showroom.goods(), 0);
                cmd(&mut s, Command::DeliverChef);
                assert_eq!(s.gold, 24);
                assert!(s.progression.discovered());
                assert!(!s.progression.dagger_unlocked());
                assert_eq!(
                    s.command(s.command_id + 1, Command::CompleteDaggerResearch),
                    Err(Error::ResearchLocked)
                );
                cmd(&mut s, Command::StudyDagger);
                s.advance(10).unwrap();
                assert!(s.progression.can_complete());
                assert!(!s.progression.dagger_unlocked());
                cmd(&mut s, Command::CompleteDaggerResearch);
                cmd(&mut s, Command::QueueCraft);
                s.advance(5).unwrap();
                assert_eq!(s.progression.codex["iron_dagger"].count, 1);
                assert_eq!(s.progression.general_xp, 22);
                s.advance(100).unwrap();
                assert_eq!(s.sold, 1);
                assert_eq!(s.progression.codex["iron_dagger"].count, 1);
                cmd(&mut s, Command::QueueCraft);
                s.advance(60).unwrap();
                assert_eq!(s.progression.codex["iron_dagger"].count, 2);
                assert_eq!(
                    s.progression
                        .journal
                        .iter()
                        .filter(|m| m.id == "first:iron_dagger")
                        .count(),
                    1
                );
            }
        }
    }
    #[test]
    fn duplicate_delivery_research_and_overflow_never_partially_reward() {
        let mut s = game(7);
        knife(&mut s);
        s.gold = u64::MAX;
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::DeliverChef),
            Err(Error::Overflow)
        );
        assert_eq!(s, before);
        s.gold = 0;
        cmd(&mut s, Command::DeliverChef);
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id, Command::DeliverChef),
            Err(Error::DuplicateCommand)
        );
        assert_eq!(
            s.command(s.command_id + 1, Command::DeliverChef),
            Err(Error::InvalidOrderState)
        );
        assert_eq!(s, before);
        cmd(&mut s, Command::StudyDagger);
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::StudyDagger),
            Err(Error::ResearchLocked)
        );
        assert_eq!(s, before);
        s.advance(10).unwrap();
        cmd(&mut s, Command::CompleteDaggerResearch);
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::CompleteDaggerResearch),
            Err(Error::ResearchLocked)
        );
        assert_eq!(s, before);
        let mut s = game(3);
        cmd(&mut s, Command::AcceptChef);
        cmd(&mut s, Command::QueueChef);
        s.progression.general_xp = u64::MAX;
        let before = s.clone();
        assert_eq!(s.advance(20), Err(Error::Overflow));
        assert_eq!(s, before);
    }
    #[test]
    fn save_reload_every_phase_partitioning_and_offline_replay_match() {
        let dir = std::env::temp_dir().join(format!("forge-alpha-{}", std::process::id()));
        let mut save = Save::new(game(2), 100);
        for (command, seconds) in [
            (Command::ChooseSmith(Smith::Female), 0),
            (Command::AcceptChef, 0),
            (Command::QueueChef, 1),
            (Command::Rest, 100),
            (Command::DeliverChef, 0),
            (Command::StudyDagger, 10),
            (Command::CompleteDaggerResearch, 0),
            (Command::QueueCraft, 5),
        ] {
            cmd(&mut save.state, command);
            save::store(&dir, &mut save).unwrap();
            save = save::load(&dir).unwrap().unwrap();
            let mut split = save.state.clone();
            let expected = save.state.advance(seconds).unwrap();
            let actual: Vec<_> = (0..seconds)
                .flat_map(|_| split.advance(1).unwrap())
                .collect();
            assert_eq!(save.state, split);
            assert_eq!(expected, actual);
        }
        save::store(&dir, &mut save).unwrap();
        let mut offline = save::load(&dir).unwrap().unwrap();
        let mut online = offline.clone();
        online.state.advance(100).unwrap();
        offline.catch_up(200).unwrap();
        assert_eq!(offline.state, online.state);
        let before = offline.clone();
        offline.catch_up(200).unwrap();
        assert_eq!(offline, before);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn content_rejects_circular_or_unreachable_first_progression() {
        for change in 0..5 {
            let mut r = rules();
            match change {
                0 => r.dagger.prerequisites.push(r.dagger.id.clone()),
                1 => r.dagger.technique_levels[0] = 3,
                2 => r.techniques[1].level_xp = 0,
                3 => r.dagger.study_gold = 25,
                _ => r.dagger.goal_xp = 11,
            }
            assert_eq!(r.validate(), Err(Error::InvalidState));
        }
    }
    #[test]
    fn current_schema_missing_progression_is_rejected_and_v4_keeps_recipe() {
        let dir =
            std::env::temp_dir().join(format!("forge-alpha-migration-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let original = Save::new(State::foundation(9), 100);
        let mut json = serde_json::to_value(&original).unwrap();
        json["state"].as_object_mut().unwrap().remove("progression");
        std::fs::write(dir.join("save-0.json"), serde_json::to_vec(&json).unwrap()).unwrap();
        assert!(save::load(&dir).is_err());
        json["schema_version"] = 4.into();
        std::fs::write(dir.join("save-0.json"), serde_json::to_vec(&json).unwrap()).unwrap();
        let loaded = save::load(&dir).unwrap().unwrap();
        assert!(loaded.state.progression.legacy_dagger);
        assert!(loaded.state.progression.codex.is_empty());
        assert_eq!(loaded.state.ore, original.state.ore);
        assert_eq!(loaded.schema_version, save::SCHEMA_VERSION);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn entrusted_knife_bypasses_full_showroom_and_sleep_resumes_work() {
        let mut s = game(2);
        s.progression.legacy_dagger = true;
        s.ore = 20;
        s.next_visitor = 10000;
        for _ in 0..4 {
            cmd(&mut s, Command::QueueCraft);
        }
        s.advance(20).unwrap();
        assert_eq!(s.showroom.free(), 0);
        cmd(&mut s, Command::QueueCraft);
        cmd(&mut s, Command::AcceptChef);
        cmd(&mut s, Command::QueueChef);
        cmd(&mut s, Command::Rest);
        s.advance(100).unwrap();
        assert_eq!(s.progression.chef_phase, OrderPhase::Ready);
        assert_eq!(s.life.queue.len(), 1);
        assert_eq!(s.showroom.goods(), 4);
    }
}
