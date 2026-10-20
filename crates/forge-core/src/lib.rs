//! Deterministic domain: no renderer, OS clock, or Bevy dependency.
pub mod commission;
pub mod craft;
pub mod crafting;
pub mod expedition;
pub mod items;
pub mod life;
pub mod progression;
pub mod save;
pub mod shell;
pub mod showroom;
pub mod spaces;
pub mod village;
pub mod weapon;
use serde::{Deserialize, Serialize};

pub const CONTENT_VERSION: u32 = 1;
pub const OFFLINE_CAP_SECS: u64 = 8 * 60 * 60;
pub const CRAFT_SECS: u64 = 5;
pub const SALE_PRICE: u64 = 12;

/// Player appearance. Both smiths share identical rules and economy; this only selects art.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Smith {
    Female,
    Male,
}
impl Smith {
    pub const ALL: [Self; 2] = [Self::Female, Self::Male];
    pub fn id(self) -> &'static str {
        match self {
            Self::Female => "female",
            Self::Male => "male",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Female => "Eira",
            Self::Male => "Bram",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub expedition: expedition::Expedition,
    pub progression: progression::Progression,
    pub tick: u64,
    pub gold: u64,
    /// Stackable holdings by catalog ID: materials, parts, ordinary finished goods. Replaced the
    /// old `ore` count (schema 8); iron ore is `iron_ore`.
    pub inventory: items::Inventory,
    pub daggers: u32,
    pub crafted: u32,
    pub sold: u32,
    pub job_due: Option<u64>,
    pub next_visitor: u64,
    pub rng: u64,
    pub event_id: u64,
    pub command_id: u64,
    #[serde(default)]
    pub life: life::Life,
    #[serde(default)]
    pub commerce: village::Commerce,
    pub farmer_order: commission::FarmerOrder,
    /// None until the player chooses; older saves ask once and keep their progress.
    #[serde(default)]
    pub smith: Option<Smith>,
    /// Finite finished-goods capacity; `daggers` always equals `showroom.goods()`.
    #[serde(default)]
    pub showroom: showroom::Showroom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    CraftExpeditionGear(expedition::Gear),
    ForgeWeapon(weapon::Plan),
    SellWeapon(u64),
    LaunchExpedition(expedition::Loadout),
    RecallExpedition,
    Craft,
    /// Queue a data-driven recipe (`assets/crafting.json`); inputs are debited now.
    QueueRecipe(items::Id),
    /// Buy a catalog item from a vendor's offer.
    Buy {
        vendor: village::Location,
        item: items::Id,
        quantity: u32,
    },
    AcceptChef,
    QueueChef,
    DeliverChef,
    StudyDagger,
    CompleteDaggerResearch,
    BuyOre,
    QueueCraft,
    AcceptFarmerRepair,
    QueueFarmerRepair,
    DeliverFarmerRepair,
    BuyFood,
    Eat,
    EatMeal,
    Drink,
    Rest,
    Purchase {
        vendor: village::Location,
        item: village::Item,
        quantity: u32,
    },
    /// One-time appearance choice; never resets progress.
    ChooseSmith(Smith),
    /// Toggle a finished good between display and keeping. Reservations are untouchable.
    SetForSale {
        slot: usize,
        for_sale: bool,
    },
    /// Carpenter furniture: one more display rack at the installation limit's expense.
    BuyRack,
    /// Carpenter furniture: raise one rack's tier for more slots on the same floor space.
    UpgradeRack {
        rack: usize,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Crafted,
    ProgressionChanged,
    ChefReady,
    FarmerRepairReady,
    ActionChanged(life::SmithAction),
    Ate,
    Drank,
    PantryEmpty,
    Sold {
        item: String,
        gold: u64,
    },
    VisitorWithoutStock,
    /// A queued job could not start because no result slot was free.
    CraftBlocked(spaces::BlockReason),
    /// Capacity grew (purchase, upgrade); waiting jobs are re-evaluated on the next second.
    CapacityChanged {
        slots: u32,
    },
    /// A recipe job finished and its output entered the inventory.
    Produced {
        item: String,
        qty: u32,
    },
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
    ResearchLocked,
    Busy,
    InsufficientResources,
    Overflow,
    InvalidState,
    PantryFull,
    InvalidPurchase,
    OutOfStock,
    /// No free showroom slot for a new job's result.
    DisplayFull,
    /// Rack count or tier is at its data-defined limit.
    RackLimit,
    InvalidOrderState,
}
impl State {
    #[cfg(test)]
    pub(crate) fn foundation(seed: u64) -> Self {
        let mut state = Self::new(seed);
        state.progression.legacy_dagger = true;
        state
    }
    pub fn new(seed: u64) -> Self {
        Self {
            expedition: expedition::Expedition::default(),
            progression: progression::Progression::default(),
            tick: 0,
            gold: 0,
            inventory: crafting::Book::bundled().start_inventory(),
            daggers: 0,
            crafted: 0,
            sold: 0,
            job_due: None,
            next_visitor: 12,
            rng: seed.max(1),
            event_id: 0,
            command_id: 0,
            life: life::Life::default(),
            commerce: village::Commerce::default(),
            farmer_order: commission::FarmerOrder::default(),
            smith: None,
            showroom: showroom::Showroom::default(),
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.life.validate()?;
        self.expedition.validate()?;
        self.validate_progression()?;
        self.showroom.validate()?;
        self.validate_farmer_order()?;
        self.inventory.validate(items::Registry::bundled())?;
        self.commerce.validate()?;
        self.validate_recipe_jobs()?;
        if self.rng == 0
            || self.next_visitor <= self.tick
            || self.job_due.is_some_and(|t| t <= self.tick)
            || self.job_due.is_some() != (self.life.work > 0)
            || self.sold > self.crafted
            || self.daggers != self.crafted - self.sold
            || self.daggers != self.showroom.goods()
            || self.showroom.reserved()
                != usize::from(self.life.active_job == Some(life::WorkKind::IronDagger))
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    /// Goods a visitor may buy right now.
    pub fn sellable(&self) -> u32 {
        self.showroom.sellable()
    }
    /// IDs are a monotonic command stream persisted with state. Replays cannot debit twice.
    pub fn command(&mut self, id: u64, command: Command) -> Result<(), Error> {
        if id <= self.command_id {
            return Err(Error::DuplicateCommand);
        }
        let mut next = self.clone();
        match command {
            Command::ForgeWeapon(plan) => next.queue_weapon(plan)?,
            Command::SellWeapon(id) => next.sell_weapon(id)?,
            Command::CraftExpeditionGear(g) => next.queue_expedition_gear(g)?,
            Command::LaunchExpedition(l) => next.launch_expedition(l)?,
            Command::RecallExpedition => next.recall_expedition()?,
            Command::AcceptChef
            | Command::QueueChef
            | Command::DeliverChef
            | Command::StudyDagger
            | Command::CompleteDaggerResearch => next.progression_command(command)?,
            Command::AcceptFarmerRepair => next.accept_farmer_repair()?,
            Command::QueueFarmerRepair => next.queue_farmer_repair()?,
            Command::DeliverFarmerRepair => next.deliver_farmer_repair()?,
            Command::Purchase {
                vendor,
                item,
                quantity,
            } => next.purchase(vendor, item, quantity)?,
            Command::QueueRecipe(recipe) => next.queue_recipe(recipe)?,
            Command::Buy {
                vendor,
                item,
                quantity,
            } => next.buy(vendor, item, quantity)?,
            Command::ChooseSmith(smith) => {
                if next.smith.is_some() {
                    return Err(Error::InvalidState);
                }
                next.smith = Some(smith);
            }
            Command::SetForSale { slot, for_sale } => next.showroom.set_for_sale(slot, for_sale)?,
            Command::BuyRack => {
                let (gold, timber) = next.showroom.rack_cost().ok_or(Error::RackLimit)?;
                if next.gold < gold || next.stock(crate::items::ASH_LOG) < timber {
                    return Err(Error::InsufficientResources);
                }
                next.showroom.install_rack()?;
                next.gold -= gold;
                next.inventory.take(crate::items::ASH_LOG, timber)?;
            }
            Command::UpgradeRack { rack } => {
                let (gold, timber) = next.showroom.upgrade_cost(rack).ok_or(Error::RackLimit)?;
                if next.gold < gold || next.stock(crate::items::ASH_LOG) < timber {
                    return Err(Error::InsufficientResources);
                }
                next.showroom.upgrade(rack)?;
                next.gold -= gold;
                next.inventory.take(crate::items::ASH_LOG, timber)?;
            }
            Command::Craft => {
                if !next.progression.dagger_unlocked() {
                    return Err(Error::ResearchLocked);
                }
                if next.job_due.is_some() || !next.life.queue.is_empty() {
                    return Err(Error::Busy);
                }
                if next.ore() < next.dagger_ore() {
                    return Err(Error::InsufficientResources);
                }
                // Reserve the result slot before any material is consumed.
                next.showroom.reserve()?;
                next.job_due = Some(
                    next.tick
                        .checked_add(next.work_seconds(life::WorkKind::IronDagger))
                        .ok_or(Error::Overflow)?,
                );
                {
                    let n = next.dagger_ore();
                    next.inventory.take(crate::items::IRON_ORE, n)?;
                }
                next.life.work = next.work_seconds(life::WorkKind::IronDagger) * 1000;
                next.life.active_job = Some(life::WorkKind::IronDagger);
                if next.life.action == life::SmithAction::Idle {
                    next.life.action = life::SmithAction::Working;
                }
            }
            Command::QueueCraft => {
                if !next.progression.dagger_unlocked() {
                    return Err(Error::ResearchLocked);
                }
                if next.ore() < next.dagger_ore() {
                    return Err(Error::InsufficientResources);
                }
                if next.life.queue.len() >= 32 {
                    return Err(Error::Busy);
                }
                {
                    let n = next.dagger_ore();
                    next.inventory.take(crate::items::IRON_ORE, n)?;
                }
                next.life.queue.push_back(life::WorkKind::IronDagger);
            }
            Command::BuyFood => {
                next.purchase(village::Location::Farm, village::Item::Staple, 1)?;
            }
            Command::Eat | Command::EatMeal => {
                if next.life.action != life::SmithAction::Working
                    && next.life.action != life::SmithAction::Idle
                {
                    return Err(Error::Busy);
                }
                if !next.life.pantry.take(if command == Command::EatMeal {
                    life::Food::Premium
                } else {
                    life::Food::Staple
                }) {
                    return Err(Error::InsufficientResources);
                }
                next.life.hunger = next
                    .life
                    .hunger
                    .saturating_sub(next.life.config.food_restore);
                next.life.action = life::SmithAction::Eating;
                next.life.remaining = next.life.config.consume_seconds;
            }
            Command::Drink => {
                if next.life.action != life::SmithAction::Working
                    && next.life.action != life::SmithAction::Idle
                {
                    return Err(Error::Busy);
                }
                next.life.thirst = 0;
                next.life.action = life::SmithAction::Drinking;
                next.life.remaining = next.life.config.consume_seconds;
            }
            Command::Rest => {
                if next.life.action != life::SmithAction::Working
                    && next.life.action != life::SmithAction::Idle
                {
                    return Err(Error::Busy);
                }
                next.life.action = life::SmithAction::Sleeping;
                next.life.remaining = next.life.config.sleep_seconds;
            }
            // The legacy "Buy 2 ore · 4 gold" button: the forge supply channel's iron ore offer.
            Command::BuyOre => next.buy(village::Location::Blacksmith, items::IRON_ORE, 2)?,
        }
        next.command_id = id;
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Events are processed at absolute integer timestamps; partitioning does not change RNG.
    pub fn advance(&mut self, seconds: u64) -> Result<Vec<Event>, Error> {
        self.validate()?;
        let mut next = self.clone();
        let target = next.tick.checked_add(seconds).ok_or(Error::Overflow)?;
        let mut events = vec![];
        while next.tick < target {
            next.tick += 1;
            let due = next.tick;
            next.life_step(&mut events)?;
            next.expedition_step()?;
            if next.next_visitor == due {
                if let Some(good) = next.showroom.sell() {
                    let price = good.price();
                    next.gold = next.gold.checked_add(price).ok_or(Error::Overflow)?;
                    next.sold = next.sold.checked_add(1).ok_or(Error::Overflow)?;
                    next.daggers -= 1;
                    next.emit(
                        EventKind::Sold {
                            item: good.id().into(),
                            gold: price,
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
            let mut a = State::foundation(seed);
            a.command(1, Command::Craft).unwrap();
            let mut b = a.clone();
            let expected = a.advance(500).unwrap();
            let actual: Vec<_> = (0..100).flat_map(|_| b.advance(5).unwrap()).collect();
            assert_eq!(a, b);
            assert_eq!(expected, actual);
            assert_eq!(a.gold, 12);
            assert_eq!(a.ore(), 8);
            assert_eq!(a.sold, 1);
        }
    }
    #[test]
    fn commands_are_atomic_and_idempotent() {
        let mut s = State::foundation(1);
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
        let mut s = State::foundation(1);
        s.command(1, Command::Craft).unwrap();
        s.advance(5).unwrap();
        s.gold = u64::MAX;
        let old = s.clone();
        assert_eq!(s.advance(20), Err(Error::Overflow));
        assert_eq!(s, old);
    }
}
