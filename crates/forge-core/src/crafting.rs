//! Generic processing and assembly (materials → parts → finished goods) and catalog vendors.
//!
//! Recipes and offers are data (`assets/crafting.json`), referencing catalog item IDs. Inputs are
//! debited when a job is queued (the existing work rule), and the output is added to the
//! inventory exactly once when the job completes. No cancellation refunds or salvage exist.
use crate::items::{Id, Registry};
use crate::life::WorkKind;
use crate::village::Location;
use crate::{Error, EventKind, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stack {
    pub item: Id,
    pub qty: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Station {
    Forge,
    Anvil,
    Workbench,
}

/// How a recipe becomes available. v1 ships starter recipes only; research gates (#75/#76)
/// will add variants here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unlock {
    Starter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: Id,
    pub inputs: Vec<Stack>,
    pub output: Stack,
    pub seconds: u64,
    pub station: Station,
    pub unlock: Unlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub item: Id,
    pub price: u64,
    /// `None` = the channel never runs out (the existing ore supply).
    pub stock: Option<u32>,
    /// Old aggregate stock this offer replaces (`timber`, `leather`), for migration.
    #[serde(default)]
    pub legacy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vendor {
    pub location: Location,
    #[serde(default)]
    pub note: Option<String>,
    pub offers: Vec<Offer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Book {
    pub version: u32,
    pub status: String,
    pub doc: String,
    pub recipes: Vec<Recipe>,
    /// The assembly the Chef's commission uses (its output is held for the Chef).
    pub chef_order_recipe: Id,
    pub start_inventory_note: String,
    /// What a new game starts with (draft). Migrated saves keep their own holdings.
    pub start_inventory: Vec<Stack>,
    pub vendors: Vec<Vendor>,
}

/// Parses a catalog recipe line such as `"log x1 + pine_resin x1"` or `"wooden_pole x1 (long)"`
/// into (name-or-family, qty) pairs; attributes in parentheses are ignored here.
fn catalog_line(line: &str) -> Result<Vec<(String, u32)>, String> {
    line.split('+')
        .map(|part| {
            let part = part.split('(').next().unwrap_or("").trim();
            let (name, qty) = part
                .rsplit_once(" x")
                .ok_or_else(|| format!("catalog recipe {line:?}: missing quantity"))?;
            let qty = qty
                .trim()
                .parse()
                .map_err(|_| format!("catalog recipe {line:?}: bad quantity"))?;
            Ok((name.trim().to_string(), qty))
        })
        .collect()
}

impl Book {
    pub fn from_json(text: &str, registry: &Registry) -> Result<Self, String> {
        let book: Book = serde_json::from_str(text).map_err(|e| e.to_string())?;
        book.validate(registry)?;
        Ok(book)
    }
    pub fn bundled() -> &'static Self {
        static B: OnceLock<Book> = OnceLock::new();
        B.get_or_init(|| {
            Self::from_json(
                include_str!("../../../assets/crafting.json"),
                Registry::bundled(),
            )
            .expect("bundled crafting content is valid")
        })
    }
    pub fn recipe(&self, id: Id) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == id)
    }
    pub fn offer(&self, vendor: Location, item: Id) -> Option<&Offer> {
        self.vendors
            .iter()
            .find(|v| v.location == vendor)
            .and_then(|v| v.offers.iter().find(|o| o.item == item))
    }
    pub fn start_inventory(&self) -> crate::items::Inventory {
        let mut inv = crate::items::Inventory::default();
        for s in &self.start_inventory {
            inv.add(s.item, s.qty).expect("start inventory within cap");
        }
        inv
    }
    /// The bundled book's starting stock, computed once (validation runs every simulated second).
    pub fn bundled_initial_stock() -> &'static BTreeMap<String, u32> {
        static S: OnceLock<BTreeMap<String, u32>> = OnceLock::new();
        S.get_or_init(|| Self::bundled().initial_stock())
    }
    /// Starting finite stock of every offer, keyed for `Commerce::vendor_stock`.
    pub fn initial_stock(&self) -> BTreeMap<String, u32> {
        self.vendors
            .iter()
            .flat_map(|v| {
                v.offers
                    .iter()
                    .filter_map(move |o| o.stock.map(|n| (stock_key(v.location, o.item), n)))
            })
            .collect()
    }

    /// Content checks: known IDs, positive quantities and time, no self-loops, unique IDs, and
    /// inputs that match the asset catalogs' own recipe lines for each part and finished good.
    pub fn validate(&self, registry: &Registry) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported crafting content version".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for r in &self.recipes {
            if !seen.insert(r.id) {
                return Err(format!("duplicate recipe {}", r.id));
            }
            let all = r.inputs.iter().chain([&r.output]);
            for s in all {
                if !registry.contains(s.item) || s.qty == 0 || s.qty > 99 {
                    return Err(format!("{}: bad stack {:?}", r.id, s));
                }
            }
            if r.inputs.is_empty() || r.seconds == 0 || r.seconds > 3600 {
                return Err(format!("{}: needs inputs and 1..=3600 seconds", r.id));
            }
            if r.inputs.iter().any(|s| s.item == r.output.item) {
                return Err(format!("{}: consumes its own output", r.id));
            }
            self.matches_catalog(r, registry)?;
        }
        if self
            .start_inventory
            .iter()
            .any(|s| !registry.contains(s.item) || s.qty == 0)
        {
            return Err("bad start_inventory".into());
        }
        if self.recipe(self.chef_order_recipe).is_none() {
            return Err("chef_order_recipe is not a recipe".into());
        }
        let mut offers = std::collections::BTreeSet::new();
        for v in &self.vendors {
            for o in &v.offers {
                if !registry.contains(o.item)
                    || o.price == 0
                    || o.stock.is_some_and(|n| n > crate::items::STACK_CAP)
                    || !offers.insert((v.location as u8, o.item))
                {
                    return Err(format!("{:?}: bad offer {}", v.location, o.item));
                }
            }
        }
        Ok(())
    }

    fn matches_catalog(&self, r: &Recipe, registry: &Registry) -> Result<(), String> {
        let lines = catalog_recipe(r.output.item);
        let Some(lines) = lines else { return Ok(()) };
        let wanted = lines
            .iter()
            .map(|l| catalog_line(l))
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        // "iron|bronze ingot" lists alternatives: iron_ingot or bronze_ingot.
        let alternatives = |name: &str| -> Vec<String> {
            match name.split_once(' ') {
                Some((alts, noun)) if alts.contains('|') => {
                    alts.split('|').map(|a| format!("{a}_{noun}")).collect()
                }
                _ => vec![name.to_string()],
            }
        };
        let fits = |name: &str, item: Id| {
            alternatives(name).iter().any(|n| n == item.as_str())
                || registry
                    .get(item)
                    .is_some_and(|d| d.family == name && d.kind == crate::items::Kind::Material)
        };
        let ok = wanted.len() == r.inputs.len()
            && wanted
                .iter()
                .all(|(name, q)| r.inputs.iter().any(|s| fits(name, s.item) && s.qty == *q));
        if ok {
            Ok(())
        } else {
            Err(format!(
                "{}: inputs differ from the catalog recipe {wanted:?}",
                r.id
            ))
        }
    }
}

/// The catalog's own recipe lines for a part or finished good, if it has one.
fn catalog_recipe(item: Id) -> Option<Vec<String>> {
    #[derive(Deserialize)]
    struct F {
        items: Vec<serde_json::Value>,
    }
    static C: OnceLock<BTreeMap<String, Vec<String>>> = OnceLock::new();
    let map = C.get_or_init(|| {
        let mut m = BTreeMap::new();
        for text in [
            include_str!("../../../assets/items/parts/parts-catalog.json"),
            include_str!("../../../assets/items/finished/finished-goods-catalog.json"),
        ] {
            let f: F = serde_json::from_str(text).expect("catalog json");
            for i in f.items {
                let id = i["id"].as_str().unwrap_or_default().to_string();
                let lines = match &i["recipe"] {
                    serde_json::Value::String(s) => vec![s.clone()],
                    serde_json::Value::Array(a) => a
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => vec![],
                };
                if !lines.is_empty() {
                    m.insert(id, lines);
                }
            }
        }
        m
    });
    map.get(item.as_str()).cloned()
}

pub fn stock_key(vendor: Location, item: Id) -> String {
    let v = match vendor {
        Location::Blacksmith => "blacksmith",
        Location::Carpenter => "carpenter",
        Location::Leatherworker => "leatherworker",
        Location::Farm => "farm",
        Location::Tavern => "tavern",
        Location::VillageChief => "village_chief",
        Location::MerchantGuild => "merchant_guild",
        Location::Outside => "outside",
    };
    format!("{v}/{item}")
}

/// Makes one Chef knife's blade and handle through real commands and simulated time: smelt and
/// handle first, then the blade once the ingot exists (inputs are debited at enqueue). For QA
/// fixtures and tests; players do the same steps from the UI.
pub fn make_chef_parts(s: &mut State) -> Result<(), Error> {
    let blade = Id::new("straight_blade");
    let handle = Id::new("wooden_handle");
    let ingot = Id::new("iron_ingot");
    let (b0, h0, i0) = (s.stock(blade), s.stock(handle), s.stock(ingot));
    for r in ["smelt_iron_ingot", "make_wooden_handle"] {
        s.command(s.command_id + 1, crate::Command::QueueRecipe(Id::new(r)))?;
    }
    let mut waited = 0;
    while (s.stock(ingot) <= i0 || s.stock(handle) <= h0) && waited < 3600 {
        s.advance(1)?;
        waited += 1;
    }
    s.command(
        s.command_id + 1,
        crate::Command::QueueRecipe(Id::new("forge_straight_blade")),
    )?;
    while s.stock(blade) <= b0 && waited < 3600 {
        s.advance(1)?;
        waited += 1;
    }
    if s.stock(blade) > b0 && s.stock(handle) > h0 {
        Ok(())
    } else {
        Err(Error::Busy)
    }
}

/// A pure purchase quote: total price, or why the purchase cannot happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quote {
    pub unit: u64,
    pub total: u64,
}

impl State {
    pub fn stock(&self, item: Id) -> u32 {
        self.inventory.count(item)
    }
    /// Iron ore on hand (the legacy `ore` value).
    pub fn ore(&self) -> u32 {
        self.inventory.count(crate::items::IRON_ORE)
    }
    /// Sets a stack directly. Isolated QA fixtures and tests only; gameplay uses commands.
    pub fn set_stock(&mut self, item: Id, n: u32) {
        let have = self.inventory.count(item);
        if n > have {
            self.inventory
                .add(item, n - have)
                .expect("fixture stock within cap");
        } else {
            self.inventory.take(item, have - n).expect("fixture stock");
        }
    }
    pub fn vendor_stock(&self, vendor: Location, item: Id) -> Option<u32> {
        let offer = Book::bundled().offer(vendor, item)?;
        match offer.stock {
            None => Some(u32::MAX),
            Some(_) => Some(
                self.commerce
                    .vendor_stock
                    .get(&stock_key(vendor, item))
                    .copied()
                    .unwrap_or(0),
            ),
        }
    }
    pub fn quote_buy(&self, vendor: Location, item: Id, quantity: u32) -> Result<Quote, Error> {
        let offer = Book::bundled()
            .offer(vendor, item)
            .ok_or(Error::InvalidPurchase)?;
        if quantity == 0 || quantity > 99 {
            return Err(Error::InvalidPurchase);
        }
        if self.vendor_stock(vendor, item).unwrap_or(0) < quantity {
            return Err(Error::OutOfStock);
        }
        let total = offer
            .price
            .checked_mul(u64::from(quantity))
            .ok_or(Error::Overflow)?;
        Ok(Quote {
            unit: offer.price,
            total,
        })
    }
    pub(crate) fn buy(&mut self, vendor: Location, item: Id, quantity: u32) -> Result<(), Error> {
        let q = self.quote_buy(vendor, item, quantity)?;
        if self.gold < q.total {
            return Err(Error::InsufficientResources);
        }
        self.inventory.add(item, quantity)?;
        self.gold -= q.total;
        if let Some(n) = self.commerce.vendor_stock.get_mut(&stock_key(vendor, item)) {
            *n -= quantity;
        }
        Ok(())
    }
    /// Whether a recipe can be queued now, and the inputs it would debit.
    pub fn quote_recipe(&self, id: Id) -> Result<&'static Recipe, Error> {
        let r = Book::bundled().recipe(id).ok_or(Error::InvalidState)?;
        match r.unlock {
            Unlock::Starter => {}
        }
        if self.life.queue.len() >= 32 {
            return Err(Error::Busy);
        }
        let needs: Vec<_> = r.inputs.iter().map(|s| (s.item, s.qty)).collect();
        if !self.inventory.has(&needs) {
            return Err(Error::InsufficientResources);
        }
        Ok(r)
    }
    pub(crate) fn queue_recipe(&mut self, id: Id) -> Result<(), Error> {
        let r = self.quote_recipe(id)?;
        for s in &r.inputs {
            self.inventory.take(s.item, s.qty)?;
        }
        self.life.queue.push_back(WorkKind::Recipe(id));
        Ok(())
    }
    pub(crate) fn validate_recipe_jobs(&self) -> Result<(), Error> {
        let book = Book::bundled();
        let known = |j: &WorkKind| match j {
            WorkKind::Recipe(id) => book.recipe(*id).is_some(),
            _ => true,
        };
        if !self.life.queue.iter().all(known) || !self.life.active_job.iter().all(known) {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    pub(crate) fn finish_recipe(
        &mut self,
        id: Id,
        events: &mut Vec<crate::Event>,
    ) -> Result<(), Error> {
        let r = Book::bundled().recipe(id).ok_or(Error::InvalidState)?;
        self.inventory.add(r.output.item, r.output.qty)?;
        self.emit(
            EventKind::Produced {
                item: r.output.item.to_string(),
                qty: r.output.qty,
            },
            events,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use crate::items::*;

    /// A new game with the draft start kit removed, so counts below are exact.
    fn state() -> State {
        let mut s = State::new(1);
        s.gold = 200;
        for item in ["charcoal", "ash_log", "pine_resin"] {
            s.set_stock(Id::new(item), 0);
        }
        s
    }
    fn cmd(s: &mut State, c: Command) -> Result<(), Error> {
        s.command(s.command_id + 1, c)
    }
    fn buy(s: &mut State, v: Location, item: &str, n: u32) {
        cmd(
            s,
            Command::Buy {
                vendor: v,
                item: Id::new(item),
                quantity: n,
            },
        )
        .unwrap();
    }
    fn queue(s: &mut State, r: &str) -> Result<(), Error> {
        cmd(s, Command::QueueRecipe(Id::new(r)))
    }

    #[test]
    fn new_game_kit_makes_exactly_one_chef_knife() {
        let mut s = State::new(1);
        make_chef_parts(&mut s).unwrap();
        for item in ["charcoal", "ash_log", "pine_resin"] {
            assert_eq!(s.stock(Id::new(item)), 0, "{item}");
        }
        assert_eq!(s.ore(), 8);
        assert_eq!(make_chef_parts(&mut s), Err(Error::InsufficientResources));
    }

    #[test]
    fn bundled_book_matches_catalog_quantities() {
        let b = Book::bundled();
        assert_eq!(b.recipes.len(), 22);
        // Every offer is unlimited for now (owner decision 2026-10-20).
        assert!(
            b.vendors
                .iter()
                .flat_map(|v| &v.offers)
                .all(|o| o.stock.is_none())
        );
        // A recipe whose inputs disagree with the catalog line is rejected.
        let mut bad = b.clone();
        bad.recipes
            .iter_mut()
            .find(|r| r.id == Id::new("forge_frying_pan"))
            .unwrap()
            .inputs[0]
            .qty = 3;
        assert!(bad.validate(Registry::bundled()).is_err());
    }

    #[test]
    fn kitchen_knife_chain_consumes_exactly_and_outputs_once() {
        let mut s = state();
        let ore = s.ore();
        buy(&mut s, Location::Blacksmith, "charcoal", 1);
        buy(&mut s, Location::Carpenter, "ash_log", 1);
        buy(&mut s, Location::Farm, "pine_resin", 1);
        queue(&mut s, "smelt_iron_ingot").unwrap();
        assert_eq!(s.ore(), ore - 2);
        assert_eq!(s.stock(Id::new("charcoal")), 0);
        s.advance(60).unwrap();
        assert_eq!(s.stock(Id::new("iron_ingot")), 1);
        queue(&mut s, "forge_straight_blade").unwrap();
        queue(&mut s, "make_wooden_handle").unwrap();
        s.advance(60).unwrap();
        queue(&mut s, "assemble_kitchen_knife").unwrap();
        let events = s.advance(60).unwrap();
        assert_eq!(s.stock(Id::new("kitchen_knife")), 1);
        for part in [
            "iron_ingot",
            "straight_blade",
            "wooden_handle",
            "ash_log",
            "pine_resin",
        ] {
            assert_eq!(s.stock(Id::new(part)), 0, "{part}");
        }
        let produced = events
            .iter()
            .filter(
                |e| matches!(&e.kind, EventKind::Produced { item, .. } if item == "kitchen_knife"),
            )
            .count();
        assert_eq!(produced, 1);
    }

    #[test]
    fn direct_recipes_work() {
        let mut s = state();
        buy(&mut s, Location::Carpenter, "ash_log", 3);
        buy(&mut s, Location::Blacksmith, "iron_ingot", 2);
        for r in ["carve_wooden_club", "make_wooden_pole", "forge_frying_pan"] {
            queue(&mut s, r).unwrap();
        }
        s.advance(120).unwrap();
        queue(&mut s, "make_wooden_staff").unwrap();
        s.advance(60).unwrap();
        for (item, n) in [
            ("wooden_club", 1),
            ("wooden_staff", 1),
            ("frying_pan", 1),
            ("ash_log", 1),
            ("wooden_pole", 0),
        ] {
            assert_eq!(s.stock(Id::new(item)), n, "{item}");
        }
    }

    #[test]
    fn refusals_are_atomic() {
        let mut s = state();
        let before = s.clone();
        assert_eq!(
            queue(&mut s, "forge_frying_pan"),
            Err(Error::InsufficientResources)
        );
        assert_eq!(queue(&mut s, "no_such_recipe"), Err(Error::InvalidState));
        assert_eq!(
            cmd(
                &mut s,
                Command::Buy {
                    vendor: Location::Farm,
                    item: Id::new("iron_ore"),
                    quantity: 1
                }
            ),
            Err(Error::InvalidPurchase),
            "the farm does not sell ore"
        );
        s.gold = 1;
        let before_poor = s.clone();
        assert_eq!(
            cmd(
                &mut s,
                Command::Buy {
                    vendor: Location::Carpenter,
                    item: Id::new("ash_log"),
                    quantity: 1
                }
            ),
            Err(Error::InsufficientResources)
        );
        assert_eq!(s, before_poor);
        s.gold = before.gold;
        assert_eq!(
            cmd(
                &mut s,
                Command::Buy {
                    vendor: Location::Carpenter,
                    item: Id::new("ash_log"),
                    quantity: 100
                }
            ),
            Err(Error::InvalidPurchase),
            "at most 99 per purchase"
        );
    }

    #[test]
    fn vendors_sell_every_wood_and_leather_material_without_limit() {
        let reg = Registry::bundled();
        let book = Book::bundled();
        let sells = |loc: Location, id: Id| book.offer(loc, id).is_some();
        for d in reg.items.values().filter(|d| d.kind == Kind::Material) {
            match d.family.as_str() {
                "log" | "branch" => assert!(sells(Location::Carpenter, d.id), "{}", d.id),
                "raw-hide" | "leather" => assert!(sells(Location::Leatherworker, d.id), "{}", d.id),
                _ => {}
            }
        }
        assert!(sells(Location::Carpenter, Id::new("pine_resin")));
        let mut s = state();
        s.gold = 100_000;
        for _ in 0..20 {
            buy(&mut s, Location::Carpenter, "wooden_pole", 99);
        }
        assert_eq!(s.stock(Id::new("wooden_pole")), 20 * 99);
        assert!(
            s.commerce.vendor_stock.is_empty(),
            "no finite stock is tracked"
        );
        for _ in 0..50 {
            cmd(&mut s, Command::BuyOre).unwrap();
        }
    }

    #[test]
    fn the_six_p0_tools_are_craftable_from_the_start() {
        let mut s = state();
        s.gold = 1_000;
        buy(&mut s, Location::Blacksmith, "iron_ingot", 7);
        buy(&mut s, Location::Carpenter, "ash_log", 5);
        buy(&mut s, Location::Carpenter, "pine_resin", 3);
        buy(&mut s, Location::Leatherworker, "deer_leather", 1);
        for r in [
            "cut_wooden_plank",
            "forge_metal_rivet",
            "cut_leather_strap",
            "forge_hoe_head",
            "forge_sickle_blade",
            "forge_axe_head",
            "forge_wide_blade",
            "forge_saw_blade",
            "make_wooden_pole",
            "make_wooden_handle",
            "make_wooden_handle",
            "make_wooden_handle",
        ] {
            queue(&mut s, r).unwrap();
        }
        s.advance(600).unwrap();
        assert_eq!(s.stock(Id::new("wooden_plank")), 2, "planks come in twos");
        assert_eq!(s.stock(Id::new("metal_rivet")), 8, "rivets come in eights");
        buy(&mut s, Location::Carpenter, "wooden_pole", 1);
        for r in [
            "assemble_wooden_shield",
            "assemble_farming_hoe",
            "assemble_sickle",
            "assemble_woodcutting_axe",
            "assemble_cleaver",
            "assemble_hand_saw",
        ] {
            queue(&mut s, r).unwrap();
        }
        s.advance(600).unwrap();
        for item in [
            "wooden_shield",
            "farming_hoe",
            "sickle",
            "woodcutting_axe",
            "cleaver",
            "hand_saw",
        ] {
            assert_eq!(s.stock(Id::new(item)), 1, "{item}");
        }
        assert_eq!(s.stock(Id::new("metal_rivet")), 7);
        assert_eq!(s.stock(Id::new("iron_ingot")), 0);
    }

    #[test]
    fn jobs_and_inventory_survive_save_and_partition() {
        let mut s = state();
        buy(&mut s, Location::Blacksmith, "charcoal", 2);
        queue(&mut s, "smelt_iron_ingot").unwrap();
        queue(&mut s, "smelt_iron_ingot").unwrap();
        let mut stepped = s.clone();
        let json = serde_json::to_string(&s).unwrap();
        let mut loaded: State = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, s);
        loaded.advance(90).unwrap();
        for _ in 0..90 {
            stepped.advance(1).unwrap();
        }
        assert_eq!(loaded, stepped);
        assert_eq!(loaded.stock(Id::new("iron_ingot")), 2);
    }

    /// A schema-7 save as the previous release wrote it: aggregate ore/timber/leather counts.
    fn v7_save(queue_legacy_knife: bool) -> serde_json::Value {
        let mut st = State::with_progression(
            5,
            serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap(),
        )
        .unwrap();
        st.gold = 50;
        if queue_legacy_knife {
            st.progression.chef_phase = crate::commission::OrderPhase::Queued;
            st.life.queue.push_back(WorkKind::KitchenKnife);
        }
        let mut v = serde_json::to_value(crate::save::Save::new(st, 100)).unwrap();
        v["schema_version"] = 7.into();
        let state = v["state"].as_object_mut().unwrap();
        state.remove("inventory");
        state.insert("ore".into(), 6.into());
        let c = state.get_mut("commerce").unwrap().as_object_mut().unwrap();
        c.remove("vendor_stock");
        c.insert("stock".into(), serde_json::json!([33, 21, 60, 12]));
        c.insert("timber".into(), 4.into());
        c.insert("leather".into(), 2.into());
        v
    }
    fn load_v(v: &serde_json::Value, tag: &str) -> crate::save::Save {
        let dir = std::env::temp_dir().join(format!("forge-v8-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("save-0.json"), serde_json::to_vec(v).unwrap()).unwrap();
        let s = crate::save::load(&dir).unwrap().expect("migrated save");
        std::fs::remove_dir_all(dir).unwrap();
        s
    }

    #[test]
    fn schema_seven_moves_aggregate_stock_to_catalog_ids() {
        let s = load_v(&v7_save(false), "stock");
        assert_eq!(s.schema_version, 8);
        let st = &s.state;
        assert_eq!(
            (st.ore(), st.stock(ASH_LOG), st.stock(DEER_LEATHER)),
            (6, 4, 2)
        );
        // No start kit is granted to an existing save.
        assert_eq!(st.stock(Id::new("charcoal")), 0);
        // Offers are unlimited now, so the old remaining shop stock is not tracked.
        assert_eq!(
            st.vendor_stock(Location::Carpenter, ASH_LOG),
            Some(u32::MAX)
        );
        assert!(st.commerce.vendor_stock.is_empty());
        assert_eq!(st.commerce.stock, [0, 0, 60, 12]);
        assert_eq!(st.gold, 50);
    }

    #[test]
    fn legacy_queued_chef_knife_finishes_under_the_old_rule() {
        let mut s = load_v(&v7_save(true), "knife").state;
        assert_eq!(
            s.progression.chef_phase,
            crate::commission::OrderPhase::Queued
        );
        s.advance(30).unwrap();
        assert_eq!(
            s.progression.chef_phase,
            crate::commission::OrderPhase::Ready
        );
        assert_eq!(
            s.stock(Id::new("kitchen_knife")),
            0,
            "held for the Chef, not stocked"
        );
        s.command(s.command_id + 1, Command::DeliverChef).unwrap();
        assert_eq!(
            s.gold,
            50 + s.progression.rules.as_ref().unwrap().chef.reward_gold
        );
    }

    #[test]
    fn unknown_recipe_jobs_and_items_are_rejected() {
        let mut s = state();
        s.life
            .queue
            .push_back(WorkKind::Recipe(Id::new("not_a_recipe")));
        assert_eq!(s.validate(), Err(Error::InvalidState));
        let mut s = state();
        let mut v = serde_json::to_value(&s.inventory).unwrap();
        v["stacks"]["unobtainium"] = 1.into();
        s.inventory = serde_json::from_value(v).unwrap();
        assert_eq!(s.validate(), Err(Error::InvalidState));
    }

    #[test]
    fn new_chef_order_holds_the_knife_and_pays_once() {
        let mut s = State::with_progression(
            9,
            serde_json::from_str(include_str!("../../../assets/progression.json")).unwrap(),
        )
        .unwrap();
        cmd(&mut s, Command::AcceptChef).unwrap();
        assert_eq!(
            cmd(&mut s, Command::QueueChef),
            Err(Error::InsufficientResources)
        );
        make_chef_parts(&mut s).unwrap();
        cmd(&mut s, Command::QueueChef).unwrap();
        assert_eq!(
            (
                s.stock(Id::new("straight_blade")),
                s.stock(Id::new("wooden_handle"))
            ),
            (0, 0)
        );
        s.advance(30).unwrap();
        assert_eq!(
            s.progression.chef_phase,
            crate::commission::OrderPhase::Ready
        );
        assert_eq!(s.stock(Id::new("kitchen_knife")), 0);
        cmd(&mut s, Command::DeliverChef).unwrap();
        assert_eq!(
            cmd(&mut s, Command::DeliverChef),
            Err(Error::InvalidOrderState)
        );
    }
}
