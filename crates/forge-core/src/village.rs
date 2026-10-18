//! Village transactions and ephemeral navigation; independent of Bevy and the OS.
use crate::{Error, State, life::Food};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    Blacksmith,
    Carpenter,
    Leatherworker,
    Farm,
    Tavern,
    VillageChief,
    MerchantGuild,
    Outside,
}
impl Location {
    pub const ALL: [Self; 8] = [
        Self::Blacksmith,
        Self::Carpenter,
        Self::Leatherworker,
        Self::Farm,
        Self::Tavern,
        Self::VillageChief,
        Self::MerchantGuild,
        Self::Outside,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Blacksmith => "Home / Forge",
            Self::Carpenter => "Carpenter",
            Self::Leatherworker => "Leatherworker",
            Self::Farm => "Farm",
            Self::Tavern => "Tavern",
            Self::VillageChief => "Village chief",
            Self::MerchantGuild => "Merchant guild",
            Self::Outside => "Outside / Expeditions",
        }
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Home,
    Map,
    Destination(Location),
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Navigation {
    pub screen: Screen,
    pub focus: Option<usize>,
}
impl Navigation {
    pub fn map(&mut self) {
        self.screen = Screen::Map;
        self.focus = None;
    }
    pub fn visit(&mut self, l: Location) {
        self.screen = if l == Location::Blacksmith {
            Screen::Home
        } else {
            Screen::Destination(l)
        };
        self.focus = None;
    }
    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::Destination(_) => Screen::Map,
            _ => Screen::Home,
        };
        self.focus = None;
    }
    pub fn cycle(&mut self, reverse: bool) {
        self.focus = Some(match self.focus {
            None => {
                if reverse {
                    7
                } else {
                    0
                }
            }
            Some(i) => (i + if reverse { 7 } else { 1 }) % 8,
        });
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    Timber,
    Leather,
    Staple,
    Meal,
}
impl Item {
    pub const ALL: [Self; 4] = [Self::Timber, Self::Leather, Self::Staple, Self::Meal];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn vendor(self) -> Location {
        match self {
            Self::Timber => Location::Carpenter,
            Self::Leather => Location::Leatherworker,
            Self::Staple => Location::Farm,
            Self::Meal => Location::Tavern,
        }
    }
    pub fn price(self, s: &State) -> u64 {
        match self {
            Self::Timber => 3,
            Self::Leather => 5,
            Self::Staple => s.life.config.food_price,
            Self::Meal => 9,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Timber => "Seasoned timber",
            Self::Leather => "Leather straps",
            Self::Staple => "Staple provisions",
            Self::Meal => "Prepared meal (manual use)",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commerce {
    pub stock: [u32; 4],
    pub timber: u32,
    pub leather: u32,
}
impl Default for Commerce {
    fn default() -> Self {
        Self {
            stock: [40, 30, 60, 12],
            timber: 0,
            leather: 0,
        }
    }
}
impl State {
    /// Pure quote hook: later relationship pricing belongs here, never in a widget.
    pub fn quote(&self, vendor: Location, item: Item, quantity: u32) -> Result<u64, Error> {
        if vendor != item.vendor() || quantity == 0 || quantity > 99 {
            return Err(Error::InvalidPurchase);
        }
        if self.commerce.stock[item.index()] < quantity {
            return Err(Error::OutOfStock);
        }
        item.price(self)
            .checked_mul(u64::from(quantity))
            .ok_or(Error::Overflow)
    }
    pub(crate) fn purchase(
        &mut self,
        vendor: Location,
        item: Item,
        quantity: u32,
    ) -> Result<(), Error> {
        let price = self.quote(vendor, item, quantity)?;
        if self.gold < price {
            return Err(Error::InsufficientResources);
        }
        match item {
            Item::Timber => {
                self.commerce.timber = self
                    .commerce
                    .timber
                    .checked_add(quantity)
                    .ok_or(Error::Overflow)?
            }
            Item::Leather => {
                self.commerce.leather = self
                    .commerce
                    .leather
                    .checked_add(quantity)
                    .ok_or(Error::Overflow)?
            }
            Item::Staple | Item::Meal => self.life.pantry.add(
                if item == Item::Staple {
                    Food::Staple
                } else {
                    Food::Premium
                },
                quantity,
                self.life.config.stack_limit,
            )?,
        }
        self.gold -= price;
        self.commerce.stock[item.index()] -= quantity;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command,
        save::{self, Save},
    };
    #[test]
    fn atomic_validation_and_restart_dedup() {
        let mut s = State::new(42);
        s.gold = 100;
        let old = s.clone();
        for (v, i, n) in [
            (Location::Farm, Item::Timber, 1),
            (Location::Carpenter, Item::Timber, 0),
            (Location::Carpenter, Item::Timber, 41),
        ] {
            assert!(
                s.command(
                    1,
                    Command::Purchase {
                        vendor: v,
                        item: i,
                        quantity: n
                    }
                )
                .is_err()
            );
            assert_eq!(s, old);
        }
        let buy = Command::Purchase {
            vendor: Location::Carpenter,
            item: Item::Timber,
            quantity: 2,
        };
        s.command(1, buy).unwrap();
        assert_eq!(
            (s.gold, s.commerce.timber, s.commerce.stock[0]),
            (94, 2, 38)
        );
        let dir = std::env::temp_dir().join(format!("village-save-{}", std::process::id()));
        let mut saved = Save::new(s, 100);
        save::store(&dir, &mut saved).unwrap();
        let mut replay = save::load(&dir).unwrap().unwrap();
        let old = replay.clone();
        assert_eq!(replay.state.command(1, buy), Err(Error::DuplicateCommand));
        assert_eq!(replay, old);
        replay.state.command(2, Command::BuyFood).unwrap();
        assert_eq!(replay.state.life.pantry.staples(), 4);
        assert_eq!(replay.state.commerce.stock[2], 59);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn insufficient_overflow_and_pantry_roll_back() {
        let mut s = State::new(7);
        let buy = Command::Purchase {
            vendor: Location::Carpenter,
            item: Item::Timber,
            quantity: 1,
        };
        let old = s.clone();
        assert_eq!(s.command(1, buy), Err(Error::InsufficientResources));
        assert_eq!(s, old);
        s.gold = 100;
        s.commerce.timber = u32::MAX;
        let old = s.clone();
        assert_eq!(s.command(1, buy), Err(Error::Overflow));
        assert_eq!(s, old);
        s.life.pantry.slots.fill(Some(crate::life::Stack {
            food: Food::Staple,
            count: s.life.config.stack_limit,
        }));
        let old = s.clone();
        assert_eq!(s.command(1, Command::BuyFood), Err(Error::PantryFull));
        assert_eq!(s, old);
    }
    #[test]
    fn navigation_is_recoverable() {
        let mut n = Navigation::default();
        for l in Location::ALL {
            n.map();
            n.visit(l);
            if l != Location::Blacksmith {
                n.back();
                assert_eq!(n.screen, Screen::Map);
            }
            n.back();
            assert_eq!(n.screen, Screen::Home);
        }
        n.map();
        n.cycle(true);
        assert_eq!(n.focus, Some(7));
        n.cycle(false);
        assert_eq!(n.focus, Some(0));
    }
}

#[cfg(test)]
mod compatibility {
    use super::*;
    use crate::{Command, save};
    #[test]
    fn schema_two_migrates_and_premium_is_opt_in() {
        let dir = std::env::temp_dir().join(format!("village-migrate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut value = serde_json::to_value(save::Save::new(State::new(2), 100)).unwrap();
        value["schema_version"] = 2.into();
        value["state"]["life"]["queue"] = 0.into();
        value["state"].as_object_mut().unwrap().remove("commerce");
        std::fs::write(dir.join("save-0.json"), serde_json::to_vec(&value).unwrap()).unwrap();
        let mut saved = save::load(&dir).unwrap().unwrap();
        assert_eq!(saved.schema_version, 4);
        assert_eq!(saved.state.commerce, Commerce::default());
        saved.state.gold = 100;
        saved.state.life.hunger = 8000;
        saved
            .state
            .command(
                1,
                Command::Purchase {
                    vendor: Location::Tavern,
                    item: Item::Meal,
                    quantity: 1,
                },
            )
            .unwrap();
        let count = |s: &State| {
            s.life
                .pantry
                .slots
                .iter()
                .flatten()
                .filter(|s| s.food == Food::Premium)
                .map(|s| s.count)
                .sum::<u32>()
        };
        assert_eq!(count(&saved.state), 1);
        saved.state.advance(2).unwrap();
        assert_eq!(count(&saved.state), 1);
        saved.state.command(2, Command::EatMeal).unwrap();
        assert_eq!(count(&saved.state), 0);
        save::store(&dir, &mut saved).unwrap();
        assert_eq!(save::load(&dir).unwrap().unwrap(), saved);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
