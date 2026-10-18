//! Integer, one-second eligible clock. Balance values are provisional and persisted per save.
use crate::{CRAFT_SECS, Error, Event, EventKind, State};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkKind {
    IronDagger,
    FarmerSickleRepair,
}
impl WorkKind {
    pub fn seconds(self) -> u64 {
        match self {
            Self::IronDagger => CRAFT_SECS,
            Self::FarmerSickleRepair => crate::commission::REPAIR_SECS,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::IronDagger => "Iron dagger",
            Self::FarmerSickleRepair => "Farmer's sickle repair",
        }
    }
}
pub const MAX_NEED: u32 = 10_000;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SmithAction {
    #[default]
    Idle,
    Working,
    Sleeping,
    Eating,
    Drinking,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Food {
    Staple,
    Premium,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stack {
    pub food: Food,
    pub count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pantry {
    pub slots: Vec<Option<Stack>>,
}
impl Default for Pantry {
    fn default() -> Self {
        let mut slots = vec![None; 12];
        slots[0] = Some(Stack {
            food: Food::Staple,
            count: 3,
        });
        Self { slots }
    }
}
impl Pantry {
    pub fn add(&mut self, food: Food, count: u32, limit: u32) -> Result<(), Error> {
        let mut next = self.clone();
        let mut left = count;
        for slot in &mut next.slots {
            if let Some(stack) = slot
                && stack.food == food
            {
                let n = left.min(limit.saturating_sub(stack.count));
                stack.count += n;
                left -= n;
            }
        }
        for slot in &mut next.slots {
            if slot.is_none() && left > 0 {
                let n = left.min(limit);
                *slot = Some(Stack { food, count: n });
                left -= n;
            }
        }
        if left > 0 {
            return Err(Error::PantryFull);
        }
        *self = next;
        Ok(())
    }
    pub fn take_staple(&mut self) -> bool {
        self.take(Food::Staple)
    }
    pub fn take(&mut self, food: Food) -> bool {
        for slot in &mut self.slots {
            if let Some(stack) = slot
                && stack.food == food
                && stack.count > 0
            {
                stack.count -= 1;
                if stack.count == 0 {
                    *slot = None;
                }
                return true;
            }
        }
        false
    }
    pub fn staples(&self) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|s| s.food == Food::Staple)
            .map(|s| s.count)
            .sum()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeedsConfig {
    pub hunger_rate: u32,
    pub thirst_rate: u32,
    pub work_fatigue: u32,
    pub sleep_threshold: u32,
    pub sleep_seconds: u32,
    pub eat_threshold: u32,
    pub drink_threshold: u32,
    pub food_restore: u32,
    pub food_price: u64,
    pub stack_limit: u32,
    pub slowdown_threshold: u32,
    pub minimum_speed: u32,
    pub consume_seconds: u32,
}
impl Default for NeedsConfig {
    fn default() -> Self {
        Self {
            hunger_rate: 1,
            thirst_rate: 2,
            work_fatigue: 4,
            sleep_threshold: 9000,
            sleep_seconds: 120,
            eat_threshold: 3000,
            drink_threshold: 3000,
            food_restore: 6000,
            food_price: 2,
            stack_limit: 20,
            slowdown_threshold: 2500,
            minimum_speed: 250,
            consume_seconds: 2,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Life {
    pub fatigue: u32,
    pub hunger: u32,
    pub thirst: u32,
    pub action: SmithAction,
    pub remaining: u32,
    /// Remaining work in thousandths of an effective second; ore reserved on enqueue.
    pub work: u64,
    pub queue: VecDeque<WorkKind>,
    pub active_job: Option<WorkKind>,
    pub pantry: Pantry,
    pub config: NeedsConfig,
}
impl Life {
    pub fn speed(&self) -> u64 {
        1000 - self.fatigue.saturating_sub(self.config.slowdown_threshold) as u64
            * (1000 - self.config.minimum_speed) as u64
            / (MAX_NEED - self.config.slowdown_threshold) as u64
    }
    pub fn validate(&self) -> Result<(), Error> {
        let c = &self.config;
        if [self.fatigue, self.hunger, self.thirst]
            .iter()
            .any(|&v| v > MAX_NEED)
            || c.sleep_threshold == 0
            || c.sleep_threshold > MAX_NEED
            || c.sleep_seconds == 0
            || c.slowdown_threshold >= MAX_NEED
            || !(1..=1000).contains(&c.minimum_speed)
            || !(1..=3600).contains(&c.consume_seconds)
            || c.stack_limit == 0
            || c.stack_limit > 1000
            || c.food_price == 0
            || [
                c.hunger_rate,
                c.thirst_rate,
                c.work_fatigue,
                c.eat_threshold,
                c.drink_threshold,
                c.food_restore,
            ]
            .iter()
            .any(|&v| v > MAX_NEED)
            || self.queue.len() > 32
            || (self.work > 0) != self.active_job.is_some()
            || self.work > self.active_job.map_or(0, |job| job.seconds() * 1000)
            || self.pantry.slots.len() != 12
            || self
                .pantry
                .slots
                .iter()
                .flatten()
                .any(|s| s.count == 0 || s.count > c.stack_limit)
            || (matches!(
                self.action,
                SmithAction::Sleeping | SmithAction::Eating | SmithAction::Drinking
            ) && self.remaining == 0)
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}
impl State {
    fn action(
        &mut self,
        action: SmithAction,
        seconds: u32,
        events: &mut Vec<Event>,
    ) -> Result<(), Error> {
        self.life.action = action;
        self.life.remaining = seconds;
        self.emit(EventKind::ActionChanged(action), events)
    }
    /// A queued job may start when its result has somewhere to go.
    pub fn can_start(&self, job: WorkKind) -> bool {
        job != WorkKind::IronDagger || self.showroom.block_reason().is_none()
    }
    fn resume(&mut self, events: &mut Vec<Event>) -> Result<(), Error> {
        let action = if self.life.work > 0 || self.life.queue.iter().any(|j| self.can_start(*j)) {
            SmithAction::Working
        } else {
            SmithAction::Idle
        };
        self.action(action, 0, events)
    }
    fn hydrate(&mut self, events: &mut Vec<Event>) -> Result<(), Error> {
        if self.life.thirst >= self.life.config.drink_threshold {
            self.life.thirst = 0;
            self.emit(EventKind::Drank, events)?;
            self.action(
                SmithAction::Drinking,
                self.life.config.consume_seconds,
                events,
            )
        } else {
            self.resume(events)
        }
    }
    pub(crate) fn life_step(&mut self, events: &mut Vec<Event>) -> Result<(), Error> {
        let l = &mut self.life;
        l.hunger = l.hunger.saturating_add(l.config.hunger_rate).min(MAX_NEED);
        l.thirst = l.thirst.saturating_add(l.config.thirst_rate).min(MAX_NEED);
        match l.action {
            SmithAction::Sleeping | SmithAction::Eating | SmithAction::Drinking => {
                l.remaining = l.remaining.saturating_sub(1);
                if l.remaining == 0 {
                    match l.action {
                        SmithAction::Sleeping => {
                            l.fatigue = 0;
                            if l.hunger >= l.config.eat_threshold {
                                if l.pantry.take_staple() {
                                    l.hunger = l.hunger.saturating_sub(l.config.food_restore);
                                    self.emit(EventKind::Ate, events)?;
                                    self.action(
                                        SmithAction::Eating,
                                        self.life.config.consume_seconds,
                                        events,
                                    )?;
                                } else {
                                    self.emit(EventKind::PantryEmpty, events)?;
                                    self.hydrate(events)?;
                                }
                            } else {
                                self.hydrate(events)?;
                            }
                        }
                        SmithAction::Eating => self.hydrate(events)?,
                        _ => self.resume(events)?,
                    }
                }
            }
            _ => {
                if l.work == 0 {
                    // Materials were reserved on enqueue. A dagger also claims its showroom slot
                    // here; an entrusted repair never needs one, so it may start ahead of daggers
                    // that are waiting for space. Daggers stay FIFO among themselves.
                    let pick = self.life.queue.iter().position(|job| self.can_start(*job));
                    match pick {
                        Some(i) => {
                            let job = self.life.queue.remove(i).ok_or(Error::InvalidState)?;
                            if job == WorkKind::IronDagger {
                                self.showroom.reserve()?;
                            }
                            self.life.active_job = Some(job);
                            self.life.work = job.seconds() * 1000;
                            if job == WorkKind::FarmerSickleRepair {
                                self.farmer_order.phase = crate::commission::OrderPhase::Working;
                            }
                        }
                        None if !self.life.queue.is_empty() => {
                            if let Some(reason) = self.showroom.block_reason()
                                && self.life.action != SmithAction::Idle
                            {
                                self.emit(EventKind::CraftBlocked(reason), events)?;
                            }
                        }
                        None => {}
                    }
                }
                let l = &mut self.life;
                if l.work > 0 {
                    l.action = SmithAction::Working;
                    if l.fatigue >= l.config.sleep_threshold {
                        let duration = l.config.sleep_seconds;
                        self.action(SmithAction::Sleeping, duration, events)?;
                    } else {
                        l.work = l.work.saturating_sub(l.speed());
                        let gain = l.config.work_fatigue * (MAX_NEED + l.hunger) / MAX_NEED;
                        let gain = gain * (MAX_NEED + l.thirst) / MAX_NEED;
                        l.fatigue = l.fatigue.saturating_add(gain).min(MAX_NEED);
                        if l.work == 0 {
                            match self.life.active_job.take().ok_or(Error::InvalidState)? {
                                WorkKind::IronDagger => {
                                    self.showroom.place(self.current_good())?;
                                    self.crafted =
                                        self.crafted.checked_add(1).ok_or(Error::Overflow)?;
                                    self.daggers =
                                        self.daggers.checked_add(1).ok_or(Error::Overflow)?;
                                    self.emit(EventKind::Crafted, events)?;
                                }
                                WorkKind::FarmerSickleRepair => {
                                    self.farmer_order.phase = crate::commission::OrderPhase::Ready;
                                    self.farmer_order.completed_at = Some(self.tick);
                                    self.emit(EventKind::FarmerRepairReady, events)?;
                                }
                            }
                            self.resume(events)?;
                        }
                    }
                } else if l.fatigue >= l.config.sleep_threshold {
                    let duration = l.config.sleep_seconds;
                    self.action(SmithAction::Sleeping, duration, events)?;
                } else {
                    l.action = SmithAction::Idle;
                }
            }
        }
        self.job_due = if self.life.work > 0 {
            Some(
                self.tick
                    .checked_add(
                        self.life.work.div_ceil(self.life.speed()) + self.life.remaining as u64,
                    )
                    .ok_or(Error::Overflow)?,
            )
        } else {
            None
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, save::Save};
    fn fixture() -> State {
        let mut s = State::new(42);
        s.life.fatigue = 8990;
        s.life.hunger = 8000;
        s.life.thirst = 8000;
        s.life.config.sleep_seconds = 3;
        s.command(1, Command::Craft).unwrap();
        s.command(2, Command::QueueCraft).unwrap();
        s
    }
    #[test]
    fn loop_partition_resume_food_water_sale() {
        for seed in 1..30 {
            let mut a = fixture();
            a.rng = seed;
            let mut b = a.clone();
            let expected = a.advance(300).unwrap();
            let mut actual = vec![];
            for step in [1, 2, 7, 11, 29, 50, 100, 100] {
                actual.extend(b.advance(step).unwrap());
            }
            assert_eq!(a, b);
            assert_eq!(expected, actual);
            assert!(
                expected
                    .iter()
                    .any(|e| matches!(e.kind, EventKind::ActionChanged(SmithAction::Sleeping)))
            );
            assert!(expected.iter().any(|e| e.kind == EventKind::Ate));
            assert!(expected.iter().any(|e| e.kind == EventKind::Drank));
            assert_eq!(a.life.pantry.staples(), 2);
            assert_eq!(a.sold, 2);
            a.command(3, Command::BuyFood).unwrap();
            assert_eq!(a.life.pantry.staples(), 3);
            assert_eq!(a.gold, 22);
        }
    }
    #[test]
    fn save_every_boundary_does_not_repeat_consumption_or_rewards() {
        let mut expected = fixture();
        expected.advance(100).unwrap();
        let mut s = Save::new(fixture(), 1000);
        for time in 1001..=1100 {
            s.catch_up(time).unwrap();
            s = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
            let before = s.clone();
            s.catch_up(time).unwrap();
            assert_eq!(s, before);
        }
        assert_eq!(expected, s.state);
    }
    #[test]
    fn empty_pantry_and_premium_are_safe() {
        let mut s = fixture();
        s.life.pantry.slots = vec![None; 12];
        s.life.pantry.add(Food::Premium, 2, 20).unwrap();
        s.advance(200).unwrap();
        assert_eq!(s.sold, 2);
        assert_eq!(s.life.pantry.slots[0].as_ref().unwrap().count, 2);
        s.gold = 0;
        let hunger = s.life.hunger;
        s.command(3, Command::Drink).unwrap();
        assert_eq!(s.life.thirst, 0);
        assert_eq!(s.gold, 0);
        assert_eq!(s.life.hunger, hunger);
        let before = s.clone();
        assert_eq!(s.command(3, Command::Drink), Err(Error::DuplicateCommand));
        assert_eq!(s, before);
    }
    #[test]
    fn full_pantry_purchase_is_atomic_and_need_bounds_hold() {
        let mut s = State::new(42);
        s.gold = 50;
        s.life.pantry.slots = vec![
            Some(Stack {
                food: Food::Premium,
                count: 20
            });
            12
        ];
        let before = s.clone();
        assert_eq!(s.command(1, Command::BuyFood), Err(Error::PantryFull));
        assert_eq!(s, before);
        s.advance(3 * 86400).unwrap();
        s.validate().unwrap();
        assert_eq!(s.life.hunger, MAX_NEED);
        assert_eq!(s.life.thirst, MAX_NEED);
    }
    #[test]
    fn final_job_crossing_threshold_sleeps_without_another_queue() {
        let mut s = State::new(42);
        s.command(1, Command::Craft).unwrap();
        s.life.work = 1;
        s.life.fatigue = 8999;
        s.advance(1).unwrap();
        assert_eq!(s.crafted, 1);
        assert!(s.life.queue.is_empty());
        s.advance(1).unwrap();
        assert_eq!(s.life.action, SmithAction::Sleeping);
        s.advance(s.life.config.sleep_seconds as u64 + 10).unwrap();
        assert_eq!(s.life.fatigue, 0);
        assert_eq!(s.life.action, SmithAction::Idle);
        assert_eq!(s.crafted, 1);
    }
    #[test]
    fn needs_slow_remaining_work_without_revoking_progress() {
        let mut healthy = State::new(1);
        healthy.command(1, Command::Craft).unwrap();
        let mut tired = healthy.clone();
        tired.life.fatigue = 8000;
        healthy.advance(5).unwrap();
        tired.advance(5).unwrap();
        assert_eq!(healthy.crafted, 1);
        assert_eq!(tired.crafted, 0);
        assert!(tired.life.work < 5000);
        tired.advance(60).unwrap();
        assert_eq!(tired.crafted, 1);
    }
}
