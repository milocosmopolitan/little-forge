//! Craft choices produce a measured tool, never a quality damage multiplier.
use crate::{Error, Smith};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Material {
    #[default]
    Iron,
    Steel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Technique {
    #[default]
    LightEdge,
    ReinforcedPoint,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Approach {
    #[default]
    Quick,
    Standard,
    Careful,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Appearance {
    #[default]
    Common,
    Refined,
    Masterwork,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Plan {
    pub material: Material,
    pub technique: Technique,
    pub approach: Approach,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Properties {
    pub force: u16,
    pub penetration: u16,
    pub interval: u32,
    pub handling: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Craftsmanship {
    pub recipe: String,
    pub plan: Plan,
    pub quality: u8,
    pub appearance: Appearance,
    pub creator: Option<Smith>,
    pub proficiency: u64,
    pub properties: Properties,
    pub ore: u32,
    pub seconds: u64,
    pub content_version: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialRule {
    pub material: Material,
    pub force: u16,
    pub penetration: u16,
    pub interval: u32,
    pub ore: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApproachRule {
    pub approach: Approach,
    pub quality: u8,
    pub precision: u16,
    pub seconds: u64,
    pub extra_ore: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub version: u32,
    pub materials: [MaterialRule; 2],
    pub approaches: [ApproachRule; 3],
    pub boar_resistance: u16,
    pub wolf_pressure: u16,
}
impl Rules {
    pub fn bundled() -> Self {
        serde_json::from_str(include_str!("../../../assets/weapon-crafting.json"))
            .expect("validated weapon content")
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 2
            || self.materials[0].material != Material::Iron
            || self.materials[1].material != Material::Steel
            || self
                .approaches
                .iter()
                .map(|a| a.approach)
                .collect::<Vec<_>>()
                != [Approach::Quick, Approach::Standard, Approach::Careful]
            || self.materials.iter().any(|m| {
                m.force == 0
                    || m.force > 12
                    || m.penetration > 8
                    || !(1..=4).contains(&m.interval)
                    || !(1..=6).contains(&m.ore)
            })
            || self.approaches.iter().any(|a| {
                a.quality > 95
                    || a.precision > 4
                    || !(1..=60).contains(&a.seconds)
                    || a.extra_ore > 3
            })
            || self.boar_resistance > 8
            || self.wolf_pressure > 8
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    pub fn forge(&self, plan: Plan, proficiency: u64, creator: Option<Smith>) -> Craftsmanship {
        let m = self
            .materials
            .iter()
            .find(|m| m.material == plan.material)
            .expect("validated materials");
        let a = self
            .approaches
            .iter()
            .find(|a| a.approach == plan.approach)
            .expect("validated approaches");
        let precision = a.precision + u16::from(proficiency >= 2);
        let properties = match plan.technique {
            Technique::LightEdge => Properties {
                force: m.force,
                penetration: m.penetration,
                interval: m.interval,
                handling: precision + 1,
            },
            Technique::ReinforcedPoint => Properties {
                force: m.force,
                penetration: m.penetration + precision + 1,
                interval: m.interval + 1,
                handling: 0,
            },
        };
        let quality = a.quality + (proficiency.min(2) as u8 * 2);
        Craftsmanship {
            recipe: "hunting_knife".into(),
            plan,
            quality,
            appearance: if quality >= 85 {
                Appearance::Masterwork
            } else if quality >= 60 {
                Appearance::Refined
            } else {
                Appearance::Common
            },
            creator,
            proficiency,
            properties,
            ore: m.ore + a.extra_ore,
            seconds: a.seconds,
            content_version: self.version,
        }
    }
}
impl Technique {
    pub fn label(self) -> &'static str {
        match self {
            Self::LightEdge => "Light edge",
            Self::ReinforcedPoint => "Reinforced point",
        }
    }
}
impl Approach {
    pub fn label(self) -> &'static str {
        match self {
            Self::Quick => "Quick Craft",
            Self::Standard => "Standard Craft",
            Self::Careful => "Careful Craft",
        }
    }
}
impl Craftsmanship {
    pub fn label(&self) -> String {
        format!(
            "{:?} {:?} {} Q{}",
            self.appearance,
            self.plan.material,
            self.plan.technique.label(),
            self.quality
        )
    }
}

impl crate::State {
    pub(crate) fn queue_weapon(&mut self, plan: Plan) -> Result<(), Error> {
        if self.progression.rules.is_none() || !self.progression.dagger_unlocked() {
            return Err(Error::ResearchLocked);
        }
        if plan.approach != Approach::Quick
            && (self.progression.level(0) < 1 || self.progression.level(1) < 1)
        {
            return Err(Error::ResearchLocked);
        }
        let reserved = self
            .life
            .queue
            .iter()
            .chain(self.life.active_job.iter())
            .filter(|j| {
                matches!(
                    j,
                    crate::life::WorkKind::ForgedWeapon(_)
                        | crate::life::WorkKind::ExpeditionGear(
                            crate::expedition::Gear::HuntingKnife
                                | crate::expedition::Gear::Shortbow
                        )
                )
            })
            .count();
        if self.life.queue.len() >= 32 || self.expedition.weapons.len() + reserved >= 32 {
            return Err(Error::Busy);
        }
        let craft = Rules::bundled().forge(plan, self.progression.level(0), self.smith);
        if self.ore < craft.ore {
            return Err(Error::InsufficientResources);
        }
        self.ore -= craft.ore;
        self.life
            .queue
            .push_back(crate::life::WorkKind::ForgedWeapon(plan));
        Ok(())
    }
    pub(crate) fn finish_weapon(
        &mut self,
        plan: Plan,
        events: &mut Vec<crate::Event>,
    ) -> Result<(), Error> {
        let craft = Rules::bundled().forge(plan, self.progression.level(0), self.smith);
        self.finish_expedition_gear(crate::expedition::Gear::HuntingKnife)?;
        let w = self
            .expedition
            .weapons
            .last_mut()
            .ok_or(Error::InvalidState)?;
        w.craftsmanship = Some(craft.clone());
        let w = w.clone();
        let best = self
            .progression
            .weapons
            .iter()
            .filter_map(|w| w.craftsmanship.as_ref())
            .map(|c| c.quality)
            .max()
            .unwrap_or(0);
        self.progression.weapons.push(w.clone());
        if craft.quality > best {
            self.progression.note(self.tick, &format!("quality:{}",craft.quality),format!("New craftsmanship record: Hunting knife #{} Q{} (previous {}). Inspect its penetration and handling before lending.",w.id,craft.quality,best));
        }
        // The same blade and handle techniques that produced the kitchen knife improve here.
        let p = &mut self.progression;
        p.general_xp = p.general_xp.checked_add(15).ok_or(Error::Overflow)?;
        for (i, xp) in [5, 4].into_iter().enumerate() {
            p.technique_xp[i] = p.technique_xp[i].checked_add(xp).ok_or(Error::Overflow)?;
        }
        p.next_item_id = p.next_item_id.checked_add(1).ok_or(Error::Overflow)?;
        let record = p
            .codex
            .entry("hunting_knife".into())
            .or_insert(crate::progression::Record {
                first_tick: self.tick,
                first_item_id: p.next_item_id,
                count: 0,
                best_quality: None,
            });
        record.count = record.count.checked_add(1).ok_or(Error::Overflow)?;
        record.best_quality = Some(format!("Q{}", best.max(craft.quality)));
        if craft.quality > best {
            self.emit(crate::EventKind::ProgressionChanged, events)?;
        }
        self.emit(crate::EventKind::Crafted, events)
    }
    pub(crate) fn sell_weapon(&mut self, id: u64) -> Result<(), Error> {
        if self
            .expedition
            .active
            .as_ref()
            .is_some_and(|b| !b.done && [b.bow, b.knife].contains(&Some(id)))
        {
            return Err(Error::Busy);
        }
        let i = self
            .expedition
            .weapons
            .iter()
            .position(|w| w.id == id)
            .ok_or(Error::InvalidState)?;
        let w = &self.expedition.weapons[i];
        // Legacy contracts still resolve against their registry; do not remove their equipment.
        if w.craftsmanship.is_none() {
            return Err(Error::InvalidState);
        }
        self.gold = self.gold.checked_add(4).ok_or(Error::Overflow)?;
        self.expedition.weapons.remove(i);
        Ok(())
    }
}

use crate::{Command, State, expedition::Gear, life::Food};
fn command(s: &mut State, c: Command) {
    s.command(s.command_id + 1, c).expect("valid QA command");
}
/// Isolated balance fixture. Progression is earned through real commands.
pub fn qa_smith() -> crate::State {
    let mut s = crate::State::with_progression(
        42,
        serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap(),
    )
    .unwrap();
    s.gold = 80;
    s.ore = 40;
    s.commerce.timber = 12;
    command(&mut s, Command::AcceptChef);
    command(&mut s, Command::QueueChef);
    s.advance(20).unwrap();
    command(&mut s, Command::DeliverChef);
    command(&mut s, Command::StudyDagger);
    s.advance(10).unwrap();
    command(&mut s, Command::CompleteDaggerResearch);
    for g in [Gear::Shortbow, Gear::Arrows, Gear::Arrows, Gear::Arrows] {
        command(&mut s, Command::CraftExpeditionGear(g));
    }
    s.advance(32).unwrap();
    s.life.pantry.add(Food::Staple, 20, 20).unwrap();
    s
}
pub(crate) fn validate_record(w: &crate::expedition::Weapon) -> Result<(), Error> {
    if w.highest_wave > 5 || w.completions > w.field_uses {
        return Err(Error::InvalidState);
    }
    if let Some(c) = &w.craftsmanship {
        let mut expected = Rules::bundled().forge(c.plan, c.proficiency, c.creator);
        expected.appearance = c.appearance;
        if c != &expected || w.kind != Gear::HuntingKnife {
            return Err(Error::InvalidState);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, State, expedition::Loadout};
    fn command(s: &mut State, c: Command) {
        s.command(s.command_id + 1, c).unwrap();
    }
    #[test]
    fn balance_matrix() {
        Rules::bundled().validate().unwrap();
        for material in [Material::Iron, Material::Steel] {
            for technique in [Technique::LightEdge, Technique::ReinforcedPoint] {
                for approach in [Approach::Quick, Approach::Standard, Approach::Careful] {
                    let mut s = qa_smith();
                    command(
                        &mut s,
                        Command::ForgeWeapon(Plan {
                            material,
                            technique,
                            approach,
                        }),
                    );
                    s.advance(30).unwrap();
                    command(
                        &mut s,
                        Command::LaunchExpedition(Loadout {
                            primary: Some(1),
                            secondary: Some(2),
                            food: 5,
                            arrows: 24,
                        }),
                    );
                    s.advance(600).unwrap();
                    let b = s.expedition.active.as_ref().unwrap();
                    println!(
                        "{material:?}/{technique:?}/{approach:?}: clear={} hp={} sec={} reason={:?}",
                        b.cleared, b.health, b.second, b.cause
                    );
                }
            }
        }
    }
    fn forge(s: &mut State, plan: Plan) -> u64 {
        command(s, Command::ForgeWeapon(plan));
        s.advance(30).unwrap();
        s.expedition.next_weapon
    }
    fn launch(s: &mut State, id: u64) {
        command(
            s,
            Command::LaunchExpedition(Loadout {
                primary: Some(1),
                secondary: Some(id),
                food: 5,
                arrows: 24,
            }),
        );
    }
    #[test]
    fn wave_three_failure_improves_through_craft_and_history_survives_sale() {
        let mut s = qa_smith();
        let weak = forge(
            &mut s,
            Plan {
                technique: Technique::ReinforcedPoint,
                ..Default::default()
            },
        );
        launch(&mut s, weak);
        s.advance(90).unwrap();
        let b = s.expedition.active.as_ref().unwrap();
        assert_eq!(b.cleared, 2);
        assert_eq!(b.cause, Some(crate::expedition::Cause::Penetration));
        let strong = forge(
            &mut s,
            Plan {
                material: Material::Steel,
                technique: Technique::ReinforcedPoint,
                approach: Approach::Careful,
            },
        );
        launch(&mut s, strong);
        let locked = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::SellWeapon(strong)),
            Err(Error::Busy)
        );
        assert_eq!(s, locked);
        s.advance(90).unwrap();
        let b = s.expedition.active.as_ref().unwrap();
        assert_eq!(b.cleared, 5);
        assert_eq!(b.previous_best, 2);
        let record = s
            .progression
            .weapons
            .iter()
            .find(|w| w.id == strong)
            .unwrap()
            .clone();
        assert_eq!(record.highest_wave, 5);
        assert!(record.first_breakthrough.is_some());
        assert!(record.kills > 0);
        command(&mut s, Command::SellWeapon(strong));
        assert!(s.expedition.weapon(Some(strong)).is_none());
        assert!(s.progression.weapons.contains(&record));
        let before = s.clone();
        assert!(
            s.command(s.command_id + 1, Command::SellWeapon(strong))
                .is_err()
        );
        assert_eq!(s, before);
        s.validate().unwrap();
    }
    #[test]
    fn cosmetics_do_not_change_combat_and_a_poorly_matched_masterwork_fails() {
        let mut s = qa_smith();
        let id = forge(
            &mut s,
            Plan {
                material: Material::Iron,
                technique: Technique::LightEdge,
                approach: Approach::Careful,
            },
        );
        let mut cosmetic = s.clone();
        cosmetic.expedition.weapons[1]
            .craftsmanship
            .as_mut()
            .unwrap()
            .appearance = Appearance::Common;
        launch(&mut s, id);
        launch(&mut cosmetic, id);
        s.advance(100).unwrap();
        cosmetic.advance(100).unwrap();
        let a = s.expedition.active.as_ref().unwrap();
        let b = cosmetic.expedition.active.as_ref().unwrap();
        assert_eq!(a.trace, b.trace);
        assert!(a.cleared < 5);
        assert_eq!(a.cause, Some(crate::expedition::Cause::Penetration));
    }
    #[test]
    fn crafted_combat_and_jobs_survive_journal_and_offline_without_double_settlement() {
        let mut s = qa_smith();
        command(
            &mut s,
            Command::ForgeWeapon(Plan {
                material: Material::Steel,
                technique: Technique::ReinforcedPoint,
                approach: Approach::Careful,
            }),
        );
        s.advance(7).unwrap();
        let dir = std::env::temp_dir().join(format!("forge-weapon-{}", std::process::id()));
        let mut saved = crate::save::Save::new(s, 1000);
        crate::save::store(&dir, &mut saved).unwrap();
        let mut offline = crate::save::load(&dir).unwrap().unwrap();
        saved.state.advance(30).unwrap();
        offline.catch_up(1030).unwrap();
        assert_eq!(saved.state, offline.state);
        launch(&mut saved.state, 2);
        saved.saved_at = 1030;
        offline = saved.clone();
        offline.catch_up(1150).unwrap();
        for now in 1031..=1150 {
            saved.catch_up(now).unwrap();
            crate::save::store(&dir, &mut saved).unwrap();
            saved = crate::save::load(&dir).unwrap().unwrap();
        }
        assert_eq!(saved.state, offline.state);
        let settled = saved.state.clone();
        saved.catch_up(1150).unwrap();
        assert_eq!(saved.state, settled);
        assert_eq!(saved.state.progression.weapons[0].field_uses, 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn a_light_common_blade_outperforms_masterwork_point_without_ranged_support() {
        let run = |plan| {
            let mut s = qa_smith();
            let id = forge(&mut s, plan);
            command(
                &mut s,
                Command::LaunchExpedition(Loadout {
                    primary: Some(id),
                    secondary: None,
                    food: 5,
                    arrows: 0,
                }),
            );
            s.advance(100).unwrap();
            s.expedition.active.unwrap()
        };
        let light = run(Plan::default());
        let heavy = run(Plan {
            material: Material::Steel,
            technique: Technique::ReinforcedPoint,
            approach: Approach::Careful,
        });
        assert!(light.cleared > heavy.cleared);
        assert_eq!(heavy.cause, Some(crate::expedition::Cause::Handling));
    }
    #[test]
    fn three_qualities_change_controlled_boar_performance() {
        let runs = [Approach::Quick, Approach::Standard, Approach::Careful].map(|approach| {
            let mut s = qa_smith();
            let id = forge(
                &mut s,
                Plan {
                    material: Material::Steel,
                    technique: Technique::ReinforcedPoint,
                    approach,
                },
            );
            launch(&mut s, id);
            s.advance(100).unwrap();
            s.expedition.active.unwrap()
        });
        assert_eq!(runs.each_ref().map(|b| b.cleared), [4, 5, 5]);
        assert!(runs[2].health > runs[1].health);
        assert!(runs[2].second < runs[1].second);
        assert_ne!(runs[0].trace, runs[1].trace);
    }
    #[test]
    fn schema_six_inflight_legacy_battle_keeps_its_rules() {
        let mut s = crate::expedition::qa_fixture();
        launch(&mut s, 2);
        s.advance(17).unwrap();
        let expected = s.clone();
        let mut value = serde_json::to_value(crate::save::Save::new(s, 100)).unwrap();
        value["schema_version"] = 6.into();
        value["state"]["progression"]
            .as_object_mut()
            .unwrap()
            .remove("weapons");
        for w in value["state"]["expedition"]["weapons"]
            .as_array_mut()
            .unwrap()
        {
            for name in [
                "craftsmanship",
                "highest_wave",
                "completions",
                "first_breakthrough",
            ] {
                w.as_object_mut().unwrap().remove(name);
            }
        }
        for name in ["weapon_snapshot", "rules", "previous_best"] {
            value["state"]["expedition"]["active"]
                .as_object_mut()
                .unwrap()
                .remove(name);
        }
        let dir = std::env::temp_dir().join(format!("forge-weapon-v6-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("save-0.json"), serde_json::to_vec(&value).unwrap()).unwrap();
        let mut loaded = crate::save::load(&dir).unwrap().unwrap().state;
        let mut expected = expected;
        expected
            .expedition
            .active
            .as_mut()
            .unwrap()
            .weapon_snapshot
            .clear();
        assert_eq!(loaded, expected);
        loaded.advance(100).unwrap();
        expected.advance(100).unwrap();
        assert_eq!(loaded, expected);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn queued_knives_reserve_armory_slots_against_bow_jobs() {
        let mut s = qa_smith();
        s.ore = 200;
        for _ in 0..31 {
            command(&mut s, Command::ForgeWeapon(Plan::default()));
        }
        let before = s.clone();
        assert_eq!(
            s.command(
                s.command_id + 1,
                Command::CraftExpeditionGear(Gear::Shortbow)
            ),
            Err(Error::Busy)
        );
        assert_eq!(s, before);
        s.advance(1000).unwrap();
        assert_eq!(s.expedition.weapons.len(), 32);
        s.validate().unwrap();
    }
    #[test]
    fn normal_start_can_fund_retry_with_one_ordinary_dagger_sale() {
        let mut s = State::with_progression(
            42,
            serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap(),
        )
        .unwrap();
        command(&mut s, Command::AcceptChef);
        command(&mut s, Command::QueueChef);
        s.advance(20).unwrap();
        command(&mut s, Command::DeliverChef);
        command(&mut s, Command::StudyDagger);
        s.advance(10).unwrap();
        command(&mut s, Command::CompleteDaggerResearch);
        command(
            &mut s,
            Command::Purchase {
                vendor: crate::village::Location::Carpenter,
                item: crate::village::Item::Timber,
                quantity: 5,
            },
        );
        for g in [Gear::Shortbow, Gear::Arrows, Gear::Arrows, Gear::Arrows] {
            command(&mut s, Command::CraftExpeditionGear(g));
        }
        s.advance(32).unwrap();
        let weak = forge(
            &mut s,
            Plan {
                technique: Technique::ReinforcedPoint,
                ..Default::default()
            },
        );
        for _ in 0..2 {
            command(&mut s, Command::BuyFood);
        }
        launch(&mut s, weak);
        s.advance(90).unwrap();
        assert_eq!(s.expedition.active.as_ref().unwrap().cleared, 2);
        command(&mut s, Command::QueueCraft);
        s.advance(100).unwrap();
        assert_eq!(s.sold, 1);
        for _ in 0..2 {
            command(&mut s, Command::BuyOre);
        }
        for _ in 0..5 {
            command(&mut s, Command::BuyFood);
        }
        let strong = forge(
            &mut s,
            Plan {
                material: Material::Steel,
                technique: Technique::ReinforcedPoint,
                approach: Approach::Careful,
            },
        );
        launch(&mut s, strong);
        s.advance(100).unwrap();
        assert_eq!(s.expedition.active.unwrap().cleared, 5);
    }
    #[test]
    fn costs_and_research_gates_are_atomic() {
        let mut s = qa_smith();
        s.ore = 0;
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::ForgeWeapon(Plan::default())),
            Err(Error::InsufficientResources)
        );
        assert_eq!(s, before);
        let mut locked = State::with_progression(
            42,
            serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap(),
        )
        .unwrap();
        let before = locked.clone();
        assert_eq!(
            locked.command(1, Command::ForgeWeapon(Plan::default())),
            Err(Error::ResearchLocked)
        );
        assert_eq!(locked, before);
    }
}
