//! Canonical item identity and stackable inventory (#119 stage 1, #74).
//!
//! Item IDs are the catalog IDs of the approved asset catalogs (40 materials, 23 parts, 22
//! finished goods). Stackable things (materials, parts, ordinary finished goods) are counted
//! here, in one store. Individual crafted weapons with quality and history (expedition armory)
//! and anonymous showroom goods keep their current stores until #119's instance stage (D05).
use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// A catalog ID, stored inline so jobs and commands stay `Copy`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    len: u8,
    bytes: [u8; 31],
}
impl Id {
    /// Builds an ID from lowercase ASCII `[a-z0-9_]`, at most 31 bytes. Panics on misuse; use
    /// `parse` for untrusted input.
    pub const fn new(s: &str) -> Self {
        let b = s.as_bytes();
        assert!(!b.is_empty() && b.len() <= 31, "item id length");
        let mut bytes = [0u8; 31];
        let mut i = 0;
        while i < b.len() {
            assert!(
                b[i].is_ascii_lowercase() || b[i].is_ascii_digit() || b[i] == b'_',
                "item id characters"
            );
            bytes[i] = b[i];
            i += 1;
        }
        Self {
            len: b.len() as u8,
            bytes,
        }
    }
    pub fn parse(s: &str) -> Result<Self, Error> {
        let ok = !s.is_empty()
            && s.len() <= 31
            && s.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
        if ok {
            Ok(Self::new(s))
        } else {
            Err(Error::InvalidState)
        }
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len as usize]).expect("ascii id")
    }
}
impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for Id {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Id::parse(&s).map_err(|_| serde::de::Error::custom(format!("invalid item id {s:?}")))
    }
}

pub const IRON_ORE: Id = Id::new("iron_ore");
pub const ASH_LOG: Id = Id::new("ash_log");
pub const DEER_LEATHER: Id = Id::new("deer_leather");

/// Which approved catalog an item comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Material,
    Part,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub id: Id,
    pub kind: Kind,
    pub family: String,
    pub name_ko: String,
    pub name_en: String,
    /// Icon path relative to `assets/`.
    pub icon: String,
}

/// The item registry: every ID the game may hold. Built from the asset catalogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    pub items: BTreeMap<Id, Definition>,
}

#[derive(Deserialize)]
struct CatalogFile {
    items: Vec<CatalogItem>,
}
#[derive(Deserialize)]
struct CatalogItem {
    id: String,
    #[serde(default)]
    family: Option<String>,
    #[serde(default)]
    category: Option<String>,
    name_ko: String,
    name_en: String,
    /// Catalog entries "on hold" (e.g. `shield_core`) have no icon and are not items yet.
    #[serde(default)]
    status: Option<String>,
}

impl Registry {
    pub fn from_catalogs(materials: &str, parts: &str, finished: &str) -> Result<Self, String> {
        let mut items = BTreeMap::new();
        for (text, kind, dir) in [
            (materials, Kind::Material, "items/materials/icons"),
            (parts, Kind::Part, "items/parts/icons"),
            (finished, Kind::Finished, "items/finished/icons"),
        ] {
            let file: CatalogFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
            for i in file
                .items
                .into_iter()
                .filter(|i| i.status.as_deref() != Some("on hold"))
            {
                let id = Id::parse(&i.id).map_err(|_| format!("bad item id {:?}", i.id))?;
                let def = Definition {
                    id,
                    kind,
                    family: i.family.or(i.category).unwrap_or_default(),
                    name_ko: i.name_ko,
                    name_en: i.name_en,
                    icon: format!("{dir}/{}.png", i.id),
                };
                if items.insert(id, def).is_some() {
                    return Err(format!("duplicate item id {id}"));
                }
            }
        }
        Ok(Self { items })
    }
    /// The catalogs shipped in `assets/items/`.
    pub fn bundled() -> &'static Self {
        static R: OnceLock<Registry> = OnceLock::new();
        R.get_or_init(|| {
            Self::from_catalogs(
                include_str!("../../../assets/items/materials/material-catalog.json"),
                include_str!("../../../assets/items/parts/parts-catalog.json"),
                include_str!("../../../assets/items/finished/finished-goods-catalog.json"),
            )
            .expect("bundled item catalogs are valid")
        })
    }
    pub fn get(&self, id: Id) -> Option<&Definition> {
        self.items.get(&id)
    }
    pub fn contains(&self, id: Id) -> bool {
        self.items.contains_key(&id)
    }
}

/// Stackable holdings by catalog ID. Zero counts are never stored, so equal holdings serialize
/// identically.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    stacks: BTreeMap<Id, u32>,
}
/// Most of one item the forge can hold: generous for migrated saves, far from u32 overflow.
pub const STACK_CAP: u32 = 1_000_000;

impl Inventory {
    pub fn count(&self, id: Id) -> u32 {
        self.stacks.get(&id).copied().unwrap_or(0)
    }
    pub fn add(&mut self, id: Id, n: u32) -> Result<(), Error> {
        if n == 0 {
            return Ok(());
        }
        let next = self.count(id).checked_add(n).ok_or(Error::Overflow)?;
        if next > STACK_CAP {
            return Err(Error::Overflow);
        }
        self.stacks.insert(id, next);
        Ok(())
    }
    pub fn take(&mut self, id: Id, n: u32) -> Result<(), Error> {
        let have = self.count(id);
        if have < n {
            return Err(Error::InsufficientResources);
        }
        if have == n {
            self.stacks.remove(&id);
        } else {
            self.stacks.insert(id, have - n);
        }
        Ok(())
    }
    pub fn has(&self, needs: &[(Id, u32)]) -> bool {
        let mut total: BTreeMap<Id, u32> = BTreeMap::new();
        for (id, n) in needs {
            *total.entry(*id).or_default() += n;
        }
        total.iter().all(|(id, n)| self.count(*id) >= *n)
    }
    pub fn iter(&self) -> impl Iterator<Item = (Id, u32)> + '_ {
        self.stacks.iter().map(|(k, v)| (*k, *v))
    }
    pub fn ids(&self) -> BTreeSet<Id> {
        self.stacks.keys().copied().collect()
    }
    pub fn validate(&self, registry: &Registry) -> Result<(), Error> {
        if self
            .stacks
            .iter()
            .any(|(id, n)| *n == 0 || *n > STACK_CAP || !registry.contains(*id))
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
    fn registry_has_every_catalog_item_once() {
        let r = Registry::bundled();
        let count = |k| r.items.values().filter(|d| d.kind == k).count();
        assert_eq!(
            (
                count(Kind::Material),
                count(Kind::Part),
                count(Kind::Finished)
            ),
            (40, 23, 22)
        );
        for id in [IRON_ORE, ASH_LOG, DEER_LEATHER] {
            assert!(r.contains(id));
        }
        for d in r.items.values() {
            let path = format!("{}/../../assets/{}", env!("CARGO_MANIFEST_DIR"), d.icon);
            assert!(std::path::Path::new(&path).exists(), "{path}");
        }
    }
    #[test]
    fn ids_round_trip_and_reject_bad_text() {
        let id = Id::new("leather_wrapped_handle");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"leather_wrapped_handle\"");
        assert_eq!(serde_json::from_str::<Id>(&json).unwrap(), id);
        for bad in ["", "Iron", "a b", "x".repeat(32).as_str()] {
            assert!(Id::parse(bad).is_err(), "{bad:?}");
        }
    }
    #[test]
    fn inventory_is_exact_and_never_negative() {
        let mut inv = Inventory::default();
        inv.add(IRON_ORE, 3).unwrap();
        assert_eq!(inv.take(IRON_ORE, 4), Err(Error::InsufficientResources));
        assert_eq!(inv.count(IRON_ORE), 3);
        inv.take(IRON_ORE, 3).unwrap();
        assert_eq!(inv, Inventory::default(), "empty stacks are removed");
        assert_eq!(inv.add(IRON_ORE, STACK_CAP + 1), Err(Error::Overflow));
        assert!(!inv.has(&[(IRON_ORE, 1), (IRON_ORE, 1)]));
    }
}
