//! Equipment-driven integer combat. Rendering and wall clocks cannot resolve an action.
use crate::{Error, State, life::WorkKind};
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gear {
    Shortbow,
    HuntingKnife,
    Arrows,
}
impl Gear {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shortbow => "Shortbow",
            Self::HuntingKnife => "Hunting knife",
            Self::Arrows => "12 arrows",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub craftsmanship: Option<crate::weapon::Craftsmanship>,
    pub highest_wave: u8,
    pub completions: u32,
    pub first_breakthrough: Option<u64>,
    pub id: u64,
    pub kind: Gear,
    pub crafted_at: u64,
    pub field_uses: u32,
    pub kills: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Loadout {
    pub primary: Option<u64>,
    pub secondary: Option<u64>,
    pub food: u8,
    pub arrows: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enemy {
    Wolf,
    Boar,
}
impl Enemy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Wolf => "Dire wolf",
            Self::Boar => "Wild boar",
        }
    }
    pub fn health(self) -> u16 {
        match self {
            Self::Wolf => 8,
            Self::Boar => 18,
        }
    }
}
pub const WAVES: [&[Enemy]; 5] = [
    &[Enemy::Wolf],
    &[Enemy::Wolf, Enemy::Wolf],
    &[Enemy::Boar],
    &[Enemy::Wolf, Enemy::Boar],
    &[Enemy::Boar, Enemy::Boar],
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Entry,
    Walk,
    BowDraw,
    BowRelease,
    KnifeDraw,
    Slash,
    Hit,
    EnemyDefeat,
    Eat,
    WaveCleared,
    Retreat,
    Victory,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cause {
    Penetration,
    Handling,
    MissingKnife,
    MissingBow,
    NoArrows,
    NoFood,
    Wounded,
    Voluntary,
}
impl Cause {
    pub fn advice(self) -> &'static str {
        match self {
            Self::Penetration => {
                "The boar resisted my blade. Forge a reinforced point with more penetration; quality alone does not pierce hide."
            }
            Self::Handling => {
                "The wolves punished my slow recovery. Try a lighter blade with better handling."
            }
            Self::MissingKnife => {
                "The boar's hide resisted arrows. Lend me a hunting knife for close range."
            }
            Self::MissingBow => {
                "The wolves reached me before I could thin them out. Add a shortbow and arrows."
            }
            Self::NoArrows => "My quiver ran dry. Pack more arrows before we try again.",
            Self::NoFood => {
                "I ran out of food between encounters. Pack five rations for the whole path."
            }
            Self::Wounded => {
                "I could not withstand another attack. Bring both weapons and more food."
            }
            Self::Voluntary => "I returned when you recalled me. Our equipment is safe.",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    pub second: u32,
    pub wave: u8,
    pub enemy: Enemy,
    pub action: Action,
    pub amount: u16,
    pub weapon: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battle {
    pub weapon_snapshot: Vec<Weapon>,
    pub rules: Option<crate::weapon::Rules>,
    pub previous_best: u8,
    pub id: u64,
    pub seed: u64,
    pub version: u32,
    pub loadout: Loadout,
    pub bow: Option<u64>,
    pub knife: Option<u64>,
    pub second: u32,
    pub wave: u8,
    pub opponent: u8,
    pub enemy_hp: u16,
    pub distance: u8,
    pub health: u16,
    pub food: u8,
    pub arrows: u8,
    pub pause: u8,
    pub cleared: u8,
    pub cause: Option<Cause>,
    pub done: bool,
    pub settled: bool,
    pub trace: Vec<Trace>,
}
impl Battle {
    pub fn enemy(&self) -> Enemy {
        WAVES[self.wave.min(4) as usize][self.opponent as usize]
    }
    pub fn last_action(&self) -> Action {
        self.trace.last().map_or(Action::Entry, |e| e.action)
    }
    fn emit(&mut self, action: Action, amount: u16, weapon: Option<u64>) {
        self.trace.push(Trace {
            second: self.second,
            wave: self.wave,
            enemy: self.enemy(),
            action,
            amount,
            weapon,
        });
    }
    fn finish(&mut self, cause: Option<Cause>) {
        self.cause = cause;
        self.done = true;
        self.emit(
            if cause.is_some() {
                Action::Retreat
            } else {
                Action::Victory
            },
            0,
            None,
        );
    }
    fn failure(&self) -> Cause {
        if let Some(p) = self.knife_properties() {
            if self.enemy() == Enemy::Boar && p.penetration < 4 {
                return Cause::Penetration;
            }
            if self.enemy() == Enemy::Wolf && p.handling < 2 {
                return Cause::Handling;
            }
        }
        if self.enemy() == Enemy::Boar && self.knife.is_none() {
            Cause::MissingKnife
        } else if self.bow.is_none() {
            Cause::MissingBow
        } else if self.arrows == 0 {
            Cause::NoArrows
        } else if self.food == 0 && self.loadout.food < 5 {
            Cause::NoFood
        } else {
            Cause::Wounded
        }
    }
    fn knife_properties(&self) -> Option<crate::weapon::Properties> {
        self.weapon_snapshot
            .iter()
            .find(|w| Some(w.id) == self.knife)
            .and_then(|w| w.craftsmanship.as_ref())
            .map(|c| c.properties)
    }
    fn step(&mut self) {
        if self.done {
            return;
        }
        self.second += 1;
        if self.pause > 0 {
            self.pause -= 1;
            self.emit(Action::Walk, 0, None);
            return;
        }
        if self.enemy_hp == 0 {
            if self.opponent as usize + 1 < WAVES[self.wave as usize].len() {
                self.opponent += 1;
            } else {
                self.cleared += 1;
                self.emit(Action::WaveCleared, 0, None);
                if self.cleared == 5 {
                    self.finish(None);
                    return;
                }
                self.wave += 1;
                self.opponent = 0;
                if self.food > 0 {
                    self.food -= 1;
                    self.health = (self.health + 20).min(60);
                    self.emit(Action::Eat, 20, None);
                } else {
                    self.health = self.health.saturating_sub(12);
                    if self.health == 0 {
                        self.finish(Some(Cause::NoFood));
                        return;
                    }
                }
            }
            self.enemy_hp = self.enemy().health();
            self.distance = 6;
            self.pause = 2;
            self.emit(Action::Walk, 0, None);
            return;
        }
        let enemy = self.enemy();
        // Wolves close twice as fast; boars telegraph then charge the final gap.
        if self.distance > 0 {
            self.distance =
                self.distance
                    .saturating_sub(if enemy == Enemy::Wolf || self.distance <= 3 {
                        2
                    } else {
                        1
                    });
            if self.bow.is_some() && self.arrows > 0 {
                if self.second.is_multiple_of(2) {
                    self.arrows -= 1;
                    let damage = if enemy == Enemy::Wolf { 8 } else { 2 };
                    self.enemy_hp = self.enemy_hp.saturating_sub(damage);
                    self.emit(Action::BowRelease, damage, self.bow);
                } else {
                    self.emit(Action::BowDraw, 0, self.bow);
                }
            } else {
                self.emit(Action::Walk, 0, None);
            }
        } else if let Some(knife) = self.knife {
            let properties = self.knife_properties();
            if self
                .second
                .is_multiple_of(properties.map_or(2, |p| p.interval))
            {
                let damage = properties.map_or(if enemy == Enemy::Wolf { 4 } else { 6 }, |p| {
                    let resistance = if enemy == Enemy::Boar {
                        self.rules.as_ref().map_or(5, |r| r.boar_resistance)
                    } else {
                        0
                    };
                    p.force
                        .saturating_sub(resistance.saturating_sub(p.penetration))
                        .max(1)
                });
                self.enemy_hp = self.enemy_hp.saturating_sub(damage);
                self.emit(Action::Slash, damage, Some(knife));
            } else {
                self.emit(Action::KnifeDraw, 0, Some(knife));
            }
        } else if self.bow.is_some() && self.arrows > 0 {
            if self.second.is_multiple_of(2) {
                self.arrows -= 1;
                let damage = if enemy == Enemy::Wolf { 4 } else { 1 };
                self.enemy_hp = self.enemy_hp.saturating_sub(damage);
                self.emit(Action::BowRelease, damage, self.bow);
            } else {
                self.emit(Action::BowDraw, 0, self.bow);
            }
        } else {
            self.emit(Action::Hit, 0, None);
        }
        if self.enemy_hp == 0 {
            self.emit(Action::EnemyDefeat, 0, None);
            return;
        }
        if self.distance == 0 && self.second.is_multiple_of(3) {
            let damage = if enemy == Enemy::Wolf {
                self.knife_properties().map_or(12, |p| {
                    12 + self.rules.as_ref().map_or(4, |r| r.wolf_pressure) - p.handling.min(8)
                })
            } else {
                12
            };
            self.health = self.health.saturating_sub(damage);
            self.emit(Action::Hit, damage, None);
            if self.health == 0 {
                self.finish(Some(self.failure()));
            }
        }
    }
    pub fn reward_gold(&self) -> u64 {
        u64::from(if self.cause == Some(Cause::Voluntary) {
            self.cleared
        } else {
            self.cleared.max(1)
        }) * 2
    }
    pub fn story(&self) -> String {
        format!(
            "I cleared {} of 5 encounters. {}",
            self.cleared,
            if let Some(c) = self.cause {
                c.advice()
            } else if self.done {
                "Your weapons reopened the Old Forest Path. The timber supply is safe again."
            } else {
                "I am still on the path."
            }
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Expedition {
    pub weapons: Vec<Weapon>,
    pub arrows: u32,
    pub next_weapon: u64,
    pub next_contract: u64,
    pub active: Option<Battle>,
    pub records: Vec<Battle>,
    pub path_reopened: bool,
}
impl Expedition {
    pub fn running(&self) -> bool {
        self.active.as_ref().is_some_and(|b| !b.done)
    }
    pub fn weapon(&self, id: Option<u64>) -> Option<&Weapon> {
        self.weapons.iter().find(|w| Some(w.id) == id)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.weapons.len() > 32
            || self.records.len() > 16
            || self
                .weapons
                .iter()
                .any(|w| w.id == 0 || w.id > self.next_weapon || w.kind == Gear::Arrows)
            || self
                .weapons
                .iter()
                .enumerate()
                .any(|(i, w)| self.weapons[..i].iter().any(|o| o.id == w.id))
        {
            return Err(Error::InvalidState);
        }
        for w in &self.weapons {
            crate::weapon::validate_record(w)?;
        }
        for b in self.active.iter().chain(self.records.iter()) {
            if let Some(rules) = &b.rules {
                rules.validate()?;
            }
            if (b.version == 2) != b.rules.is_some() || b.previous_best > 5 {
                return Err(Error::InvalidState);
            }
            if b.version == 2
                && (b.weapon_snapshot.len()
                    != usize::from(b.bow.is_some()) + usize::from(b.knife.is_some())
                    || !b
                        .weapon_snapshot
                        .iter()
                        .any(|w| Some(w.id) == b.knife && w.craftsmanship.is_some()))
            {
                return Err(Error::InvalidState);
            }
            for w in &b.weapon_snapshot {
                crate::weapon::validate_record(w)?;
            }

            if ![VERSION, 2].contains(&b.version)
                || b.wave >= 5
                || b.opponent as usize >= WAVES[b.wave as usize].len()
                || b.health > 60
                || b.cleared > 5
                || b.trace.len() > 1024
                || b.second > 600
                || b.food > b.loadout.food
                || b.arrows > b.loadout.arrows
                || b.id == 0
                || b.id > self.next_contract
                || b.done != b.settled
                || b.loadout.food > 5
                || b.loadout.arrows > 36
            {
                return Err(Error::InvalidState);
            }
            if b.bow.is_some()
                && self
                    .weapon(b.bow)
                    .or_else(|| b.weapon_snapshot.iter().find(|w| Some(w.id) == b.bow))
                    .is_none_or(|w| w.kind != Gear::Shortbow)
                || b.knife.is_some()
                    && self
                        .weapon(b.knife)
                        .or_else(|| b.weapon_snapshot.iter().find(|w| Some(w.id) == b.knife))
                        .is_none_or(|w| w.kind != Gear::HuntingKnife)
            {
                return Err(Error::InvalidState);
            }
        }
        Ok(())
    }
}
impl State {
    pub(crate) fn queue_expedition_gear(&mut self, gear: Gear) -> Result<(), Error> {
        if gear == Gear::HuntingKnife && self.progression.rules.is_some() {
            return self.queue_weapon(crate::weapon::Plan::default());
        }
        if self.life.queue.len() >= 32
            || gear != Gear::Arrows
                && self.expedition.weapons.len()
                    + self
                        .life
                        .queue
                        .iter()
                        .filter(|j| {
                            matches!(j, WorkKind::ForgedWeapon(_))
                                || matches!(j,WorkKind::ExpeditionGear(g) if *g!=Gear::Arrows)
                        })
                        .count()
                    + usize::from(
                        matches!(self.life.active_job, Some(WorkKind::ForgedWeapon(_)))
                            || matches!(self.life.active_job,Some(WorkKind::ExpeditionGear(g)) if g!=Gear::Arrows),
                    )
                    >= 32
        {
            return Err(Error::Busy);
        }
        let (ore, timber) = match gear {
            Gear::Shortbow => (1, 2),
            Gear::HuntingKnife => (2, 0),
            Gear::Arrows => (1, 1),
        };
        if self.ore < ore || self.commerce.timber < timber {
            return Err(Error::InsufficientResources);
        }
        self.ore -= ore;
        self.commerce.timber -= timber;
        self.life.queue.push_back(WorkKind::ExpeditionGear(gear));
        Ok(())
    }
    pub(crate) fn finish_expedition_gear(&mut self, gear: Gear) -> Result<(), Error> {
        if gear == Gear::Arrows {
            self.expedition.arrows = self
                .expedition
                .arrows
                .checked_add(12)
                .ok_or(Error::Overflow)?;
        } else {
            self.expedition.next_weapon = self
                .expedition
                .next_weapon
                .checked_add(1)
                .ok_or(Error::Overflow)?;
            self.expedition.weapons.push(Weapon {
                craftsmanship: None,
                highest_wave: 0,
                completions: 0,
                first_breakthrough: None,
                id: self.expedition.next_weapon,
                kind: gear,
                crafted_at: self.tick,
                field_uses: 0,
                kills: 0,
            });
        }
        Ok(())
    }
    pub(crate) fn launch_expedition(&mut self, loadout: Loadout) -> Result<(), Error> {
        if self.expedition.running() {
            return Err(Error::Busy);
        }
        if loadout.primary.is_none()
            || loadout.primary == loadout.secondary
            || loadout.food > 5
            || loadout.arrows > 36
        {
            return Err(Error::InvalidState);
        }
        let mut bow = None;
        let mut knife = None;
        for id in [loadout.primary, loadout.secondary].into_iter().flatten() {
            let w = self
                .expedition
                .weapon(Some(id))
                .ok_or(Error::InvalidState)?;
            match w.kind {
                Gear::Shortbow if bow.is_none() => bow = Some(id),
                Gear::HuntingKnife if knife.is_none() => knife = Some(id),
                _ => return Err(Error::InvalidState),
            }
        }
        if self.life.pantry.staples() < u32::from(loadout.food)
            || self.expedition.arrows < u32::from(loadout.arrows)
        {
            return Err(Error::InsufficientResources);
        }
        for _ in 0..loadout.food {
            self.life.pantry.take_staple();
        }
        self.expedition.arrows -= u32::from(loadout.arrows);
        if let Some(old) = self.expedition.active.take() {
            if self.expedition.records.len() == 16 {
                self.expedition.records.remove(0);
            }
            self.expedition.records.push(old);
        }
        self.expedition.next_contract = self
            .expedition
            .next_contract
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        let weapon_snapshot: Vec<_> = [bow, knife]
            .into_iter()
            .flatten()
            .filter_map(|id| self.expedition.weapon(Some(id)).cloned())
            .collect();
        let is_crafted = weapon_snapshot.iter().any(|w| w.craftsmanship.is_some());
        let previous_best = self
            .expedition
            .records
            .iter()
            .map(|b| b.cleared)
            .chain(
                self.expedition
                    .weapons
                    .iter()
                    .chain(self.progression.weapons.iter())
                    .map(|w| w.highest_wave),
            )
            .max()
            .unwrap_or(0);
        let mut b = Battle {
            weapon_snapshot,
            rules: is_crafted.then(crate::weapon::Rules::bundled),
            previous_best,
            id: self.expedition.next_contract,
            seed: self.rng,
            version: if is_crafted { 2 } else { VERSION },
            loadout,
            bow,
            knife,
            second: 0,
            wave: 0,
            opponent: 0,
            enemy_hp: 8,
            distance: 6,
            health: if loadout.food > 0 { 60 } else { 48 },
            food: loadout.food.saturating_sub(1),
            arrows: loadout.arrows,
            pause: 3,
            cleared: 0,
            cause: None,
            done: false,
            settled: false,
            trace: vec![],
        };
        b.emit(Action::Entry, 0, None);
        self.expedition.active = Some(b);
        Ok(())
    }
    pub(crate) fn recall_expedition(&mut self) -> Result<(), Error> {
        let b = self
            .expedition
            .active
            .as_mut()
            .filter(|b| !b.done)
            .ok_or(Error::InvalidState)?;
        b.finish(Some(Cause::Voluntary));
        self.settle_expedition()
    }
    pub(crate) fn expedition_step(&mut self) -> Result<(), Error> {
        if let Some(b) = &mut self.expedition.active {
            b.step();
        }
        self.settle_expedition()
    }
    fn settle_expedition(&mut self) -> Result<(), Error> {
        let Some(b) = self
            .expedition
            .active
            .as_mut()
            .filter(|b| b.done && !b.settled)
        else {
            return Ok(());
        };
        b.settled = true;
        // Unused provisions return once. Equipment never leaves its owned instance registry.
        self.expedition.arrows = self
            .expedition
            .arrows
            .checked_add(u32::from(b.arrows))
            .ok_or(Error::Overflow)?;
        // A full pantry cannot block returning safely: unused rations reimburse their staple cost.
        let refund = u64::from(b.food) * self.life.config.food_price;
        self.gold = self
            .gold
            .checked_add(refund + b.reward_gold())
            .ok_or(Error::Overflow)?;
        if b.cleared == 5 && !self.expedition.path_reopened {
            self.expedition.path_reopened = true;
            self.commerce.timber = self.commerce.timber.checked_add(8).ok_or(Error::Overflow)?;
        }
        for w in &mut self.expedition.weapons {
            if Some(w.id) == b.bow || Some(w.id) == b.knife {
                w.highest_wave = w.highest_wave.max(b.cleared);
                w.completions = w
                    .completions
                    .checked_add(u32::from(b.cleared == 5))
                    .ok_or(Error::Overflow)?;
                if b.cleared > b.previous_best
                    && b.previous_best > 0
                    && w.first_breakthrough.is_none()
                {
                    w.first_breakthrough = Some(b.id);
                }
                w.field_uses = w.field_uses.checked_add(1).ok_or(Error::Overflow)?;
                w.kills = w
                    .kills
                    .checked_add(
                        b.trace
                            .windows(2)
                            .filter(|pair| {
                                pair[1].action == Action::EnemyDefeat
                                    && pair[0].weapon == Some(w.id)
                            })
                            .count() as u32,
                    )
                    .ok_or(Error::Overflow)?;
            }
        }
        for w in &self.expedition.weapons {
            if let Some(record) = self
                .progression
                .weapons
                .iter_mut()
                .find(|record| record.id == w.id)
            {
                *record = w.clone();
            }
        }
        Ok(())
    }
}

/// Real queued recipes, with resource grants confined to isolated QA fixtures.
pub fn qa_fixture() -> State {
    let mut s = State::new(42);
    s.gold = 40;
    s.commerce.timber = 8;
    for g in [
        Gear::Shortbow,
        Gear::HuntingKnife,
        Gear::Arrows,
        Gear::Arrows,
        Gear::Arrows,
    ] {
        s.command(s.command_id + 1, crate::Command::CraftExpeditionGear(g))
            .unwrap();
    }
    s.advance(45).unwrap();
    s.life
        .pantry
        .add(crate::life::Food::Staple, 10, 20)
        .unwrap();
    s
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command,
        save::{self, Save},
    };
    fn run(primary: u64, secondary: Option<u64>, food: u8, arrows: u8) -> State {
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
        s
    }
    #[test]
    fn equipment_changes_failure_and_retry_outcome() {
        let mut bow = run(1, None, 5, 24);
        let b = bow.expedition.active.as_ref().unwrap();
        assert!(b.cleared < 5);
        assert_eq!(b.cause, Some(Cause::MissingKnife));
        let first = b.cleared;
        bow.command(
            bow.command_id + 1,
            Command::CraftExpeditionGear(Gear::Arrows),
        )
        .unwrap();
        bow.advance(10).unwrap();
        bow.command(
            bow.command_id + 1,
            Command::LaunchExpedition(Loadout {
                primary: Some(1),
                secondary: Some(2),
                food: 5,
                arrows: 24,
            }),
        )
        .unwrap();
        bow.advance(600).unwrap();
        let b = bow.expedition.active.as_ref().unwrap();
        assert_eq!(b.cleared, 5, "{b:?}");
        assert!(b.cleared > first);
        assert!(bow.expedition.path_reopened);
        assert!(b.trace.iter().any(|t| t.action == Action::BowRelease));
        assert!(b.trace.iter().any(|t| t.action == Action::Slash));
        assert!(b.trace.iter().any(|t| t.enemy == Enemy::Wolf));
        assert!(b.trace.iter().any(|t| t.enemy == Enemy::Boar));
        let knife = run(2, None, 5, 0);
        assert!(knife.expedition.active.unwrap().cleared < 5);
        let hungry = run(1, Some(2), 0, 12);
        assert!(hungry.expedition.active.unwrap().cleared < 5);
    }
    #[test]
    fn journal_restart_every_second_matches_offline_and_view_independence() {
        let mut s = qa_fixture();
        s.command(
            s.command_id + 1,
            Command::LaunchExpedition(Loadout {
                primary: Some(1),
                secondary: Some(2),
                food: 5,
                arrows: 24,
            }),
        )
        .unwrap();
        let dir = std::env::temp_dir().join(format!("forge-combat-{}", std::process::id()));
        let mut live = Save::new(s, 1000);
        let mut offline = live.clone();
        offline.catch_up(1600).unwrap();
        for t in 1001..=1100 {
            live.catch_up(t).unwrap();
            save::store(&dir, &mut live).unwrap();
            live = save::load(&dir).unwrap().unwrap();
            let before = live.clone();
            live.catch_up(t).unwrap();
            assert_eq!(before, live);
        }
        live.catch_up(1600).unwrap();
        assert_eq!(live.state, offline.state);
        let ended = live.state.expedition.clone();
        let gold = live.state.gold;
        let timber = live.state.commerce.timber;
        live.catch_up(2000).unwrap();
        assert_eq!(ended, live.state.expedition);
        assert_eq!(gold, live.state.gold);
        assert_eq!(timber, live.state.commerce.timber);
        assert_eq!(live.state.expedition.weapons.len(), 2);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn invalid_loadout_and_duplicate_commands_are_atomic_and_recall_is_safe() {
        let mut s = qa_fixture();
        let before = s.clone();
        let id = s.command_id + 1;
        assert!(
            s.command(
                id,
                Command::LaunchExpedition(Loadout {
                    primary: Some(1),
                    secondary: Some(1),
                    food: 5,
                    arrows: 12
                })
            )
            .is_err()
        );
        assert_eq!(s, before);
        let c = Command::LaunchExpedition(Loadout {
            primary: Some(1),
            secondary: Some(2),
            food: 5,
            arrows: 24,
        });
        s.command(id, c).unwrap();
        let before = s.clone();
        assert_eq!(s.command(id, c), Err(Error::DuplicateCommand));
        assert_eq!(s, before);
        assert_eq!(s.command(id + 1, c), Err(Error::Busy));
        s.command(id + 1, Command::RecallExpedition).unwrap();
        let before = s.clone();
        assert!(s.command(id + 2, Command::RecallExpedition).is_err());
        assert_eq!(s, before);
        assert_eq!(s.expedition.arrows, 36);
        assert_eq!(s.expedition.weapons.len(), 2);
    }
}
