//! One entrusted tool, one settlement. Animations never own inventory or rewards.
use crate::{Command, Error, State, life::WorkKind};
use serde::{Deserialize, Serialize};

// First playable balance, shared with UI and documented separately from final economy.
pub const REPAIR_SECS: u64 = 20;
pub const REPAIR_ORE: u32 = 2;
pub const DEADLINE_SECS: u64 = 2 * 60 * 60;
pub const REWARD_GOLD: u64 = 24;
pub const LATE_REWARD_GOLD: u64 = 12;
pub const REWARD_FAVOR: u8 = 5;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderPhase {
    #[default]
    Offered,
    Accepted,
    Queued,
    Working,
    Ready,
    Delivered,
}
impl OrderPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Offered => "A farmer needs your help",
            Self::Accepted => "Sickle repair accepted",
            Self::Queued => "Sickle repair queued",
            Self::Working => "Repairing the farmer's sickle",
            Self::Ready => "Sickle ready to deliver",
            Self::Delivered => "First repair delivered",
        }
    }
    pub fn command(self) -> Option<Command> {
        match self {
            Self::Offered => Some(Command::AcceptFarmerRepair),
            Self::Accepted => Some(Command::QueueFarmerRepair),
            Self::Ready => Some(Command::DeliverFarmerRepair),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FarmerOrder {
    pub phase: OrderPhase,
    pub accepted_at: Option<u64>,
    pub deadline_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub delivered_at: Option<u64>,
    pub settled_gold: u64,
    pub favor: u8,
}
impl FarmerOrder {
    pub fn on_time(&self) -> bool {
        matches!((self.completed_at, self.deadline_at), (Some(done), Some(due)) if done <= due)
    }
    pub fn reward(&self) -> (u64, u8) {
        if self.on_time() {
            (REWARD_GOLD, REWARD_FAVOR)
        } else {
            (LATE_REWARD_GOLD, 0)
        }
    }
}
impl State {
    pub(crate) fn accept_farmer_repair(&mut self) -> Result<(), Error> {
        if self.farmer_order.phase != OrderPhase::Offered {
            return Err(Error::InvalidOrderState);
        }
        self.farmer_order.phase = OrderPhase::Accepted;
        self.farmer_order.accepted_at = Some(self.tick);
        self.farmer_order.deadline_at = Some(
            self.tick
                .checked_add(DEADLINE_SECS)
                .ok_or(Error::Overflow)?,
        );
        Ok(())
    }
    pub(crate) fn queue_farmer_repair(&mut self) -> Result<(), Error> {
        if self.farmer_order.phase != OrderPhase::Accepted {
            return Err(Error::InvalidOrderState);
        }
        if self.life.queue.len() >= 32 {
            return Err(Error::Busy);
        }
        if self.ore() < REPAIR_ORE {
            return Err(Error::InsufficientResources);
        }
        self.inventory.take(crate::items::IRON_ORE, REPAIR_ORE)?;
        self.life.queue.push_back(WorkKind::FarmerSickleRepair);
        self.farmer_order.phase = OrderPhase::Queued;
        Ok(())
    }
    pub(crate) fn deliver_farmer_repair(&mut self) -> Result<(), Error> {
        if self.farmer_order.phase != OrderPhase::Ready {
            return Err(Error::InvalidOrderState);
        }
        let (gold, favor) = self.farmer_order.reward();
        self.gold = self.gold.checked_add(gold).ok_or(Error::Overflow)?;
        self.farmer_order.settled_gold = gold;
        self.farmer_order.favor = favor;
        self.farmer_order.delivered_at = Some(self.tick);
        self.farmer_order.phase = OrderPhase::Delivered;
        Ok(())
    }
    pub(crate) fn validate_farmer_order(&self) -> Result<(), Error> {
        let o = &self.farmer_order;
        let queued = self
            .life
            .queue
            .iter()
            .filter(|&&j| j == WorkKind::FarmerSickleRepair)
            .count();
        let working = self.life.active_job == Some(WorkKind::FarmerSickleRepair);
        if queued != usize::from(o.phase == OrderPhase::Queued)
            || working != (o.phase == OrderPhase::Working)
        {
            return Err(Error::InvalidState);
        }
        if o.phase == OrderPhase::Offered {
            return if *o == FarmerOrder::default() {
                Ok(())
            } else {
                Err(Error::InvalidState)
            };
        }
        let accepted = o.accepted_at.ok_or(Error::InvalidState)?;
        if accepted > self.tick || o.deadline_at != accepted.checked_add(DEADLINE_SECS) {
            return Err(Error::InvalidState);
        }
        let complete = matches!(o.phase, OrderPhase::Ready | OrderPhase::Delivered);
        if complete != o.completed_at.is_some()
            || o.completed_at
                .is_some_and(|t| t < accepted || t > self.tick)
            || (o.phase == OrderPhase::Delivered) != o.delivered_at.is_some()
            || o.delivered_at
                .is_some_and(|t| t < o.completed_at.unwrap_or(0) || t > self.tick)
        {
            return Err(Error::InvalidState);
        }
        if o.phase == OrderPhase::Delivered {
            if (o.settled_gold, o.favor) != o.reward() {
                return Err(Error::InvalidState);
            }
        } else if o.settled_gold != 0 || o.favor != 0 {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EventKind,
        life::SmithAction,
        save::{self, Save},
    };
    fn cmd(s: &mut State, c: Command) {
        s.command(s.command_id + 1, c).unwrap();
    }
    #[test]
    fn mixed_queue_is_fifo_and_entrusted_tool_never_enters_sale_stock() {
        let mut s = State::foundation(42);
        cmd(&mut s, Command::Craft);
        cmd(&mut s, Command::QueueCraft);
        cmd(&mut s, Command::AcceptFarmerRepair);
        cmd(&mut s, Command::QueueFarmerRepair);
        cmd(&mut s, Command::QueueCraft);
        assert_eq!(s.ore(), 2);
        assert_eq!(
            s.command(s.command_id + 1, Command::Craft),
            Err(Error::Busy)
        );
        s.advance(10).unwrap();
        assert_eq!(s.crafted, 2);
        assert_eq!(s.farmer_order.phase, OrderPhase::Queued);
        s.advance(1).unwrap();
        assert_eq!(s.farmer_order.phase, OrderPhase::Working);
        s.advance(19).unwrap();
        assert_eq!(s.farmer_order.phase, OrderPhase::Ready);
        assert_eq!(s.farmer_order.completed_at, Some(30));
        assert_eq!(s.crafted, 2);
        s.advance(100).unwrap();
        assert_eq!(s.crafted, 3);
        assert_eq!(s.sold, 3);
        assert_eq!(s.daggers, 0);
        let before = s.gold;
        cmd(&mut s, Command::DeliverFarmerRepair);
        assert_eq!(s.gold, before + REWARD_GOLD);
        assert_eq!(s.farmer_order.favor, 5);
        let settled = s.clone();
        assert_eq!(
            s.command(s.command_id, Command::DeliverFarmerRepair),
            Err(Error::DuplicateCommand)
        );
        assert_eq!(
            s.command(s.command_id + 1, Command::DeliverFarmerRepair),
            Err(Error::InvalidOrderState)
        );
        assert_eq!(s, settled);
    }
    #[test]
    fn invalid_actions_and_resource_failures_are_atomic() {
        let mut s = State::foundation(1);
        for c in [Command::QueueFarmerRepair, Command::DeliverFarmerRepair] {
            let before = s.clone();
            assert_eq!(s.command(1, c), Err(Error::InvalidOrderState));
            assert_eq!(s, before);
        }
        s.advance(DEADLINE_SECS + 1).unwrap();
        assert!(s.farmer_order.deadline_at.is_none());
        cmd(&mut s, Command::AcceptFarmerRepair);
        assert_eq!(s.farmer_order.deadline_at, Some(s.tick + DEADLINE_SECS));
        s.set_stock(crate::items::IRON_ORE, 1);
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::QueueFarmerRepair),
            Err(Error::InsufficientResources)
        );
        assert_eq!(s, before);
        s.set_stock(crate::items::IRON_ORE, 100);
        for _ in 0..32 {
            cmd(&mut s, Command::QueueCraft);
        }
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::QueueFarmerRepair),
            Err(Error::Busy)
        );
        assert_eq!(s, before);
    }
    #[test]
    fn ready_time_controls_deadline_even_when_delivery_is_much_later() {
        for (start, on_time) in [
            (DEADLINE_SECS - REPAIR_SECS, true),
            (DEADLINE_SECS - REPAIR_SECS + 1, false),
        ] {
            let mut s = State::foundation(1);
            cmd(&mut s, Command::AcceptFarmerRepair);
            s.advance(start).unwrap();
            cmd(&mut s, Command::QueueFarmerRepair);
            s.advance(REPAIR_SECS).unwrap();
            assert_eq!(s.farmer_order.on_time(), on_time);
            s.advance(DEADLINE_SECS).unwrap();
            cmd(&mut s, Command::DeliverFarmerRepair);
            assert_eq!(
                s.gold,
                if on_time {
                    REWARD_GOLD
                } else {
                    LATE_REWARD_GOLD
                }
            );
            assert_eq!(s.farmer_order.favor, if on_time { REWARD_FAVOR } else { 0 });
        }
    }
    #[test]
    fn repair_respects_sleep_and_survives_journal_restart_in_every_phase() {
        let dir = std::env::temp_dir().join(format!("forge-repair-{}", std::process::id()));
        let mut s = Save::new(State::foundation(42), 1000);
        for action in [
            None,
            Some(Command::AcceptFarmerRepair),
            Some(Command::QueueFarmerRepair),
        ] {
            if let Some(c) = action {
                cmd(&mut s.state, c);
            }
            save::store(&dir, &mut s).unwrap();
            assert_eq!(save::load(&dir).unwrap().unwrap(), s);
        }
        s.state.life.fatigue = 9000;
        s.state.advance(1).unwrap();
        assert_eq!(s.state.life.action, SmithAction::Sleeping);
        save::store(&dir, &mut s).unwrap();
        s = save::load(&dir).unwrap().unwrap();
        let before = s.clone();
        s.catch_up(1000).unwrap();
        assert_eq!(s, before);
        s.catch_up(1000 + DEADLINE_SECS * 2).unwrap();
        assert_eq!(s.state.farmer_order.phase, OrderPhase::Ready);
        assert_eq!(s.state.crafted, 0);
        assert_eq!(s.state.gold, 0);
        save::store(&dir, &mut s).unwrap();
        s = save::load(&dir).unwrap().unwrap();
        cmd(&mut s.state, Command::DeliverFarmerRepair);
        save::store(&dir, &mut s).unwrap();
        let loaded = save::load(&dir).unwrap().unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.state.gold, REWARD_GOLD);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn time_partitioning_preserves_work_and_ready_event() {
        let mut a = State::foundation(1);
        cmd(&mut a, Command::AcceptFarmerRepair);
        cmd(&mut a, Command::QueueFarmerRepair);
        let mut b = a.clone();
        let events = a.advance(300).unwrap();
        let mut split = vec![];
        for _ in 0..300 {
            split.extend(b.advance(1).unwrap());
        }
        assert_eq!(a, b);
        assert_eq!(events, split);
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::FarmerRepairReady)
                .count(),
            1
        );
        let mut bad = a;
        bad.life.queue.push_back(WorkKind::FarmerSickleRepair);
        assert_eq!(bad.validate(), Err(Error::InvalidState));
    }
    #[test]
    fn overflow_does_not_consume_the_delivery() {
        let mut s = State::foundation(1);
        cmd(&mut s, Command::AcceptFarmerRepair);
        cmd(&mut s, Command::QueueFarmerRepair);
        s.advance(REPAIR_SECS).unwrap();
        s.gold = u64::MAX;
        let before = s.clone();
        assert_eq!(
            s.command(s.command_id + 1, Command::DeliverFarmerRepair),
            Err(Error::Overflow)
        );
        assert_eq!(s, before);
    }
}
