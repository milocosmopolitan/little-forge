//! Finite finished-goods capacity. Storage and display of general sale goods share these slots;
//! there is no hidden unlimited stockroom. A job reserves its result slot before it starts.
use crate::{Error, craft::Good, spaces::BlockReason};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// Free: a new job can reserve it.
    Empty,
    /// The active job's result will be placed here exactly once.
    Reserved,
    /// Finished and offered to visitors.
    Displayed(Good),
    /// Finished but withheld from sale (kept for research, salvage or later use). Still occupies capacity.
    Held(Good),
}
impl Slot {
    pub fn good(self) -> Option<Good> {
        match self {
            Self::Displayed(g) | Self::Held(g) => Some(g),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Reserved => "Reserved for the current job",
            Self::Displayed(_) => "For sale",
            Self::Held(_) => "Kept, not for sale",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rack {
    /// 1-based upgrade tier; slot count per tier comes from the config.
    pub tier: u8,
}
/// Content-owned draft values; persisted per save like the needs balance. Prices and counts are
/// starting proposals, not confirmed balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowroomConfig {
    /// Slots a rack provides at tier 1, 2, ... .
    pub rack_slots: Vec<u32>,
    pub max_racks: u32,
    pub rack_gold: u64,
    pub rack_timber: u32,
    /// Cost to upgrade a rack from tier n to n+1 (length = rack_slots.len() - 1).
    pub upgrade_gold: Vec<u64>,
    pub upgrade_timber: Vec<u32>,
}
impl Default for ShowroomConfig {
    fn default() -> Self {
        Self {
            rack_slots: vec![4, 6, 8],
            max_racks: 3,
            rack_gold: 30,
            rack_timber: 4,
            upgrade_gold: vec![20, 40],
            upgrade_timber: vec![2, 4],
        }
    }
}
impl ShowroomConfig {
    pub fn max_tier(&self) -> u8 {
        self.rack_slots.len() as u8
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.rack_slots.is_empty()
            || self.rack_slots.len() > 8
            || self.rack_slots.iter().any(|n| *n == 0 || *n > 64)
            || self.rack_slots.windows(2).any(|w| w[1] < w[0])
            || self.max_racks == 0
            || self.max_racks > 16
            || self.upgrade_gold.len() != self.rack_slots.len() - 1
            || self.upgrade_timber.len() != self.rack_slots.len() - 1
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Showroom {
    pub racks: Vec<Rack>,
    pub slots: Vec<Slot>,
    /// Migration-only: goods from an older save that exceeded the first capacity. They remain the
    /// player's, sell first, and block new general crafting until drained. Never grows at runtime.
    pub overflow: u32,
    pub config: ShowroomConfig,
}
impl Default for Showroom {
    fn default() -> Self {
        Self::with_config(ShowroomConfig::default())
    }
}
impl Showroom {
    pub fn with_config(config: ShowroomConfig) -> Self {
        let slots = vec![Slot::Empty; config.rack_slots.first().copied().unwrap_or(0) as usize];
        Self {
            racks: vec![Rack { tier: 1 }],
            slots,
            overflow: 0,
            config,
        }
    }
    pub fn rack_capacity(&self, rack: &Rack) -> usize {
        self.config
            .rack_slots
            .get(rack.tier.saturating_sub(1) as usize)
            .copied()
            .unwrap_or(0) as usize
    }
    pub fn capacity(&self) -> usize {
        self.racks.iter().map(|r| self.rack_capacity(r)).sum()
    }
    /// Contiguous slot range owned by a rack, for furniture presentation.
    pub fn rack_range(&self, index: usize) -> std::ops::Range<usize> {
        let start: usize = self.racks[..index.min(self.racks.len())]
            .iter()
            .map(|r| self.rack_capacity(r))
            .sum();
        let len = self.racks.get(index).map_or(0, |r| self.rack_capacity(r));
        start..start + len
    }
    fn count(&self, f: impl Fn(&Slot) -> bool) -> usize {
        self.slots.iter().filter(|s| f(s)).count()
    }
    pub fn free(&self) -> usize {
        self.count(|s| *s == Slot::Empty)
    }
    pub fn reserved(&self) -> usize {
        self.count(|s| *s == Slot::Reserved)
    }
    pub fn displayed(&self) -> usize {
        self.count(|s| matches!(s, Slot::Displayed(_)))
    }
    pub fn held(&self) -> usize {
        self.count(|s| matches!(s, Slot::Held(_)))
    }
    /// Every finished good the player owns in general stock.
    pub fn goods(&self) -> u32 {
        (self.displayed() + self.held()) as u32 + self.overflow
    }
    /// Goods a visitor may buy: held goods are excluded.
    pub fn sellable(&self) -> u32 {
        self.displayed() as u32 + self.overflow
    }
    pub fn block_reason(&self) -> Option<BlockReason> {
        if self.overflow > 0 {
            Some(BlockReason::LegacyOverflow)
        } else if self.free() == 0 {
            Some(BlockReason::ShowroomFull)
        } else {
            None
        }
    }
    /// Atomically claim the result slot of a job that is about to start.
    pub fn reserve(&mut self) -> Result<usize, Error> {
        if self.reserved() > 0 {
            return Err(Error::Busy);
        }
        if self.block_reason().is_some() {
            return Err(Error::DisplayFull);
        }
        let i = self
            .slots
            .iter()
            .position(|s| *s == Slot::Empty)
            .ok_or(Error::DisplayFull)?;
        self.slots[i] = Slot::Reserved;
        Ok(i)
    }
    /// Place a finished good in its reserved slot, once. Default: displayed for sale.
    pub fn place(&mut self, good: Good) -> Result<usize, Error> {
        let i = self
            .slots
            .iter()
            .position(|s| *s == Slot::Reserved)
            .ok_or(Error::InvalidState)?;
        self.slots[i] = Slot::Displayed(good);
        Ok(i)
    }
    /// Remove one sellable good for a visitor purchase. Legacy overflow sells first.
    pub fn sell(&mut self) -> Option<Good> {
        if self.overflow > 0 {
            self.overflow -= 1;
            return Some(Good::IronDagger);
        }
        let i = self
            .slots
            .iter()
            .position(|s| matches!(s, Slot::Displayed(_)))?;
        let good = self.slots[i].good();
        self.slots[i] = Slot::Empty;
        good
    }
    /// Player choice between display and keeping. Reservations and empty slots cannot be changed.
    pub fn set_for_sale(&mut self, slot: usize, for_sale: bool) -> Result<(), Error> {
        let current = *self.slots.get(slot).ok_or(Error::InvalidState)?;
        self.slots[slot] = match (current, for_sale) {
            (Slot::Displayed(g) | Slot::Held(g), true) => Slot::Displayed(g),
            (Slot::Displayed(g) | Slot::Held(g), false) => Slot::Held(g),
            _ => return Err(Error::InvalidState),
        };
        Ok(())
    }
    /// Gold and timber for one more rack, or None at the installation limit.
    pub fn rack_cost(&self) -> Option<(u64, u32)> {
        (self.racks.len() < self.config.max_racks as usize)
            .then_some((self.config.rack_gold, self.config.rack_timber))
    }
    /// Gold and timber to raise rack `index` one tier, or None at the top tier.
    pub fn upgrade_cost(&self, index: usize) -> Option<(u64, u32)> {
        let rack = self.racks.get(index)?;
        let step = rack.tier.checked_sub(1)? as usize;
        Some((
            *self.config.upgrade_gold.get(step)?,
            *self.config.upgrade_timber.get(step)?,
        ))
    }
    pub(crate) fn install_rack(&mut self) -> Result<(), Error> {
        self.rack_cost().ok_or(Error::RackLimit)?;
        self.racks.push(Rack { tier: 1 });
        let n = self.rack_capacity(&Rack { tier: 1 });
        self.slots.extend(std::iter::repeat_n(Slot::Empty, n));
        Ok(())
    }
    pub(crate) fn upgrade(&mut self, index: usize) -> Result<(), Error> {
        self.upgrade_cost(index).ok_or(Error::RackLimit)?;
        let before = self.rack_capacity(&self.racks[index]);
        self.racks[index].tier += 1;
        let after = self.rack_capacity(&self.racks[index]);
        // New slots belong to this rack: insert after its existing range so item positions hold.
        let at = self.rack_range(index).start + before;
        for _ in before..after {
            self.slots.insert(at, Slot::Empty);
        }
        Ok(())
    }
    /// Lossless import of pre-showroom stock. A job in flight keeps its reservation; a displaced
    /// legacy item goes to the overflow queue rather than being discarded or auto-sold.
    pub fn migrate_legacy(&mut self, goods: u32, inflight: bool) {
        let mut left = goods;
        for slot in &mut self.slots {
            if left > 0 && *slot == Slot::Empty {
                *slot = Slot::Displayed(Good::IronDagger);
                left -= 1;
            }
        }
        if inflight && self.reserved() == 0 {
            if let Some(i) = self.slots.iter().position(|s| *s == Slot::Empty) {
                self.slots[i] = Slot::Reserved;
            } else if let Some(i) = self
                .slots
                .iter()
                .position(|s| matches!(s, Slot::Displayed(_)))
            {
                self.slots[i] = Slot::Reserved;
                left += 1;
            }
        }
        self.overflow += left;
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.config.validate()?;
        if self.racks.is_empty()
            || self.racks.len() > self.config.max_racks as usize
            || self
                .racks
                .iter()
                .any(|r| r.tier == 0 || r.tier > self.config.max_tier())
            || self.slots.len() != self.capacity()
            || self.reserved() > 1
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserve_place_sell_and_hold_are_single_shot() {
        let mut s = Showroom::default();
        assert_eq!(s.capacity(), 4);
        let i = s.reserve().unwrap();
        assert_eq!(s.reserve(), Err(Error::Busy));
        assert_eq!(s.place(Good::IronDagger).unwrap(), i);
        assert_eq!(s.place(Good::IronDagger), Err(Error::InvalidState));
        assert_eq!(s.sellable(), 1);
        s.set_for_sale(i, false).unwrap();
        assert_eq!(s.sellable(), 0);
        assert_eq!(s.sell(), None);
        assert_eq!(s.goods(), 1);
        s.set_for_sale(i, true).unwrap();
        assert_eq!(s.sell(), Some(Good::IronDagger));
        assert_eq!(s.goods(), 0);
        assert_eq!(s.set_for_sale(i, true), Err(Error::InvalidState));
        assert_eq!(s.set_for_sale(99, true), Err(Error::InvalidState));
        for _ in 0..4 {
            s.reserve().unwrap();
            s.place(Good::IronDagger).unwrap();
        }
        assert_eq!(s.reserve(), Err(Error::DisplayFull));
        assert_eq!(s.block_reason(), Some(BlockReason::ShowroomFull));
        s.validate().unwrap();
    }
    #[test]
    fn racks_grow_capacity_without_moving_goods() {
        let mut s = Showroom::default();
        s.reserve().unwrap();
        s.place(Good::IronDagger).unwrap();
        s.install_rack().unwrap();
        assert_eq!(s.capacity(), 8);
        assert_eq!(s.rack_range(1), 4..8);
        s.upgrade(0).unwrap();
        assert_eq!(s.rack_range(0), 0..6);
        assert_eq!(s.rack_range(1), 6..10);
        assert_eq!(s.slots[0], Slot::Displayed(Good::IronDagger));
        s.upgrade(0).unwrap();
        assert_eq!(s.upgrade(0), Err(Error::RackLimit));
        s.install_rack().unwrap();
        assert_eq!(s.install_rack(), Err(Error::RackLimit));
        assert_eq!(s.rack_cost(), None);
        s.validate().unwrap();
    }
    #[test]
    fn legacy_migration_preserves_every_item_and_the_inflight_reservation() {
        let mut s = Showroom::default();
        s.migrate_legacy(6, true);
        assert_eq!(s.goods(), 6);
        assert_eq!(s.reserved(), 1);
        assert_eq!(s.overflow, 3);
        assert_eq!(s.block_reason(), Some(BlockReason::LegacyOverflow));
        s.validate().unwrap();
        assert_eq!(s.sell(), Some(Good::IronDagger));
        assert_eq!(s.overflow, 2);
        let mut small = Showroom::default();
        small.migrate_legacy(2, false);
        assert_eq!((small.displayed(), small.overflow, small.free()), (2, 0, 2));
        assert_eq!(small.block_reason(), None);
    }
    #[test]
    fn config_bounds() {
        let mut c = ShowroomConfig::default();
        c.validate().unwrap();
        c.upgrade_gold.pop();
        assert!(c.validate().is_err());
        let c = ShowroomConfig {
            rack_slots: vec![6, 4, 8],
            ..Default::default()
        };
        assert!(c.validate().is_err());
        let c = ShowroomConfig {
            max_racks: 0,
            ..Default::default()
        };
        assert!(c.validate().is_err());
    }
}
