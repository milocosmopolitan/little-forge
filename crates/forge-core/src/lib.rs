//! Deterministic domain: no renderer, OS clock, or Bevy dependency.
pub mod save;
pub mod shell;
use serde::{Deserialize, Serialize};

pub const CONTENT_VERSION: u32 = 1;
pub const OFFLINE_CAP_SECS: u64 = 8 * 60 * 60;
pub const CRAFT_SECS: u64 = 5;
pub const SALE_PRICE: u64 = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub tick: u64,
    pub gold: u64,
    pub ore: u32,
    pub daggers: u32,
    pub crafted: u32,
    pub sold: u32,
    pub job_due: Option<u64>,
    pub next_visitor: u64,
    pub rng: u64,
    pub event_id: u64,
    pub command_id: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Craft,
    BuyOre,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Crafted,
    Sold { item: String, gold: u64 },
    VisitorWithoutStock,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    pub tick: u64,
    pub kind: EventKind,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DuplicateCommand,
    Busy,
    InsufficientResources,
    Overflow,
    InvalidState,
}
impl State {
    pub fn new(seed: u64) -> Self {
        Self {
            tick: 0,
            gold: 0,
            ore: 10,
            daggers: 0,
            crafted: 0,
            sold: 0,
            job_due: None,
            next_visitor: 12,
            rng: seed.max(1),
            event_id: 0,
            command_id: 0,
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.rng == 0
            || self.next_visitor <= self.tick
            || self.job_due.is_some_and(|t| t <= self.tick)
            || self.sold > self.crafted
            || self.daggers != self.crafted - self.sold
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    /// IDs are a monotonic command stream persisted with state. Replays cannot debit twice.
    pub fn command(&mut self, id: u64, command: Command) -> Result<(), Error> {
        if id <= self.command_id {
            return Err(Error::DuplicateCommand);
        }
        let mut next = self.clone();
        match command {
            Command::Craft => {
                if next.job_due.is_some() {
                    return Err(Error::Busy);
                }
                if next.ore < 2 {
                    return Err(Error::InsufficientResources);
                }
                next.job_due = Some(next.tick.checked_add(CRAFT_SECS).ok_or(Error::Overflow)?);
                next.ore -= 2;
            }
            Command::BuyOre => {
                if next.gold < 4 {
                    return Err(Error::InsufficientResources);
                }
                next.ore = next.ore.checked_add(2).ok_or(Error::Overflow)?;
                next.gold -= 4;
            }
        }
        next.command_id = id;
        *self = next;
        Ok(())
    }
    /// Events are processed at absolute integer timestamps; partitioning does not change RNG.
    pub fn advance(&mut self, seconds: u64) -> Result<Vec<Event>, Error> {
        let mut next = self.clone();
        let target = next.tick.checked_add(seconds).ok_or(Error::Overflow)?;
        let mut events = vec![];
        loop {
            let due = next.job_due.unwrap_or(u64::MAX).min(next.next_visitor);
            if due > target {
                break;
            }
            next.tick = due;
            // Stable tie rule: crafting completes before a visitor buys.
            if next.job_due == Some(due) {
                next.job_due = None;
                next.crafted = next.crafted.checked_add(1).ok_or(Error::Overflow)?;
                next.daggers = next.daggers.checked_add(1).ok_or(Error::Overflow)?;
                next.emit(EventKind::Crafted, &mut events)?;
            }
            if next.next_visitor == due {
                if next.daggers > 0 {
                    next.gold = next.gold.checked_add(SALE_PRICE).ok_or(Error::Overflow)?;
                    next.sold = next.sold.checked_add(1).ok_or(Error::Overflow)?;
                    next.daggers -= 1;
                    next.emit(
                        EventKind::Sold {
                            item: "iron_dagger".into(),
                            gold: SALE_PRICE,
                        },
                        &mut events,
                    )?;
                } else {
                    next.emit(EventKind::VisitorWithoutStock, &mut events)?;
                }
                next.rng ^= next.rng << 13;
                next.rng ^= next.rng >> 7;
                next.rng ^= next.rng << 17;
                next.next_visitor = due.checked_add(18 + next.rng % 13).ok_or(Error::Overflow)?;
            }
        }
        next.tick = target;
        next.validate()?;
        *self = next;
        Ok(events)
    }
    fn emit(&mut self, kind: EventKind, out: &mut Vec<Event>) -> Result<(), Error> {
        self.event_id = self.event_id.checked_add(1).ok_or(Error::Overflow)?;
        out.push(Event {
            id: self.event_id,
            tick: self.tick,
            kind,
        });
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partition_and_seed_replay() {
        for seed in 1..100 {
            let mut a = State::new(seed);
            a.command(1, Command::Craft).unwrap();
            let mut b = a.clone();
            let expected = a.advance(500).unwrap();
            let actual: Vec<_> = (0..100).flat_map(|_| b.advance(5).unwrap()).collect();
            assert_eq!(a, b);
            assert_eq!(expected, actual);
            assert_eq!(a.gold, 12);
            assert_eq!(a.ore, 8);
            assert_eq!(a.sold, 1);
        }
    }
    #[test]
    fn commands_are_atomic_and_idempotent() {
        let mut s = State::new(1);
        let old = s.clone();
        assert_eq!(
            s.command(1, Command::BuyOre),
            Err(Error::InsufficientResources)
        );
        assert_eq!(s, old);
        s.command(1, Command::Craft).unwrap();
        let old = s.clone();
        assert_eq!(s.command(1, Command::Craft), Err(Error::DuplicateCommand));
        assert_eq!(s, old);
        assert_eq!(s.command(2, Command::Craft), Err(Error::Busy));
        assert_eq!(s, old);
    }
    #[test]
    fn overflow_rolls_back() {
        let mut s = State::new(1);
        s.command(1, Command::Craft).unwrap();
        s.advance(5).unwrap();
        s.gold = u64::MAX;
        let old = s.clone();
        assert_eq!(s.advance(20), Err(Error::Overflow));
        assert_eq!(s, old);
    }
}
