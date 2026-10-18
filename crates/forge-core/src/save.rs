//! Recoverable two-slot journal. Each slot is atomically published by rename to an absent path.
//! A crash during replacement leaves the other committed generation intact, including on Windows.
use crate::{CONTENT_VERSION, OFFLINE_CAP_SECS, State};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};
/// Schema 4 adds the chosen smith and the finite showroom (#86). Schema 3 saves import their
/// dagger stock losslessly; newer schemas are refused rather than downgraded.
pub const SCHEMA_VERSION: u32 = 4;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Save {
    pub schema_version: u32,
    pub content_version: u32,
    pub generation: u64,
    pub saved_at: u64,
    pub state: State,
}
impl Save {
    pub fn new(state: State, now: u64) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            content_version: CONTENT_VERSION,
            generation: 0,
            saved_at: now,
            state,
        }
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != SCHEMA_VERSION || self.content_version != CONTENT_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported save/content version; preserve files",
            ));
        }
        self.state
            .validate()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))
    }
    pub fn catch_up(&mut self, now: u64) -> io::Result<u64> {
        let elapsed = now.saturating_sub(self.saved_at).min(OFFLINE_CAP_SECS);
        self.state
            .advance(elapsed)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        // Preserve high-water timestamp on a backwards wall clock.
        self.saved_at = self.saved_at.max(now);
        Ok(elapsed)
    }
}
pub fn load(dir: &Path) -> io::Result<Option<Save>> {
    let mut good = vec![];
    let mut present = false;
    for slot in 0..2 {
        let path = dir.join(format!("save-{slot}.json"));
        match fs::read(path) {
            Ok(bytes) => {
                present = true;
                if let Ok(header) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && (header
                        .get("schema_version")
                        .and_then(|v| v.as_u64())
                        .is_some_and(|v| v > SCHEMA_VERSION as u64)
                        || header
                            .get("content_version")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|v| v > CONTENT_VERSION as u64))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "newer save version; preserve files",
                    ));
                }
                if let Ok(header) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && header["schema_version"].as_u64().is_some_and(|v| v >= 2)
                    && header["state"].get("life").is_none()
                {
                    continue;
                }
                if let Ok(header) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && header["schema_version"].as_u64().is_some_and(|v| v >= 3)
                    && header["state"].get("commerce").is_none()
                {
                    continue;
                }
                if let Ok(header) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && header["schema_version"] == 4
                    && header["state"].get("showroom").is_none()
                {
                    continue;
                }
                if let Ok(mut s) = serde_json::from_slice::<Save>(&bytes) {
                    // Future formats must not be silently downgraded through an older slot.
                    if s.schema_version > SCHEMA_VERSION || s.content_version > CONTENT_VERSION {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "newer save version",
                        ));
                    }
                    if s.schema_version == 1 {
                        s.schema_version = 3;
                        s.state.life = crate::life::Life::default();
                        if let Some(due) = s.state.job_due {
                            s.state.life.work =
                                due.saturating_sub(s.state.tick).min(crate::CRAFT_SECS) * 1000;
                            s.state.life.action = crate::life::SmithAction::Working;
                        }
                    }
                    if s.schema_version == 2 {
                        s.schema_version = 3;
                    }
                    if s.schema_version == 3 {
                        // Pre-showroom stock: keep every dagger and the in-flight reservation.
                        s.state.showroom = crate::showroom::Showroom::default();
                        s.state
                            .showroom
                            .migrate_legacy(s.state.daggers, s.state.life.work > 0);
                        s.schema_version = 4;
                    }
                    if s.validate().is_ok() {
                        good.push(s);
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    if good.is_empty() && present {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "both save slots invalid; files preserved",
        ));
    }
    Ok(good.into_iter().max_by_key(|s| s.generation))
}
pub fn store(dir: &Path, save: &mut Save) -> io::Result<()> {
    save.validate()?;
    fs::create_dir_all(dir)?;
    let mut next = save.clone();
    next.generation = next
        .generation
        .checked_add(1)
        .ok_or_else(|| io::Error::other("generation exhausted"))?;
    let slot = dir.join(format!("save-{}.json", next.generation % 2));
    let tmp = dir.join("save.tmp");
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&tmp)?;
    f.write_all(&serde_json::to_vec_pretty(&next)?)?;
    f.sync_all()?;
    drop(f);
    if slot.exists() {
        fs::remove_file(&slot)?;
    }
    fs::rename(&tmp, slot)?;
    #[cfg(unix)]
    fs::File::open(dir)?.sync_all()?;
    *save = next;
    Ok(())
}
/// Prevent simultaneous instances from overwriting each other's journal. OS releases on crash.
pub struct SaveLock(std::fs::File);
impl SaveLock {
    pub fn acquire(dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("session.lock"))?;
        f.try_lock()
            .map_err(|e| io::Error::other(format!("another instance owns this save: {e}")))?;
        Ok(Self(f))
    }
}
impl Drop for SaveLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    static NEXT_DIR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    fn dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "forge-save-{}-{}-{}",
            NEXT_DIR.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn restart_recovery_and_no_duplicate_rewards() {
        let d = dir();
        let mut s = Save::new(State::new(42), 100);
        s.state.command(1, Command::Craft).unwrap();
        store(&d, &mut s).unwrap();
        s.catch_up(120).unwrap();
        store(&d, &mut s).unwrap();
        let mut loaded = load(&d).unwrap().unwrap();
        assert_eq!(s, loaded);
        loaded.catch_up(120).unwrap();
        assert_eq!(s, loaded);
        assert_eq!(loaded.state.gold, 12);
        fs::write(d.join("save-0.json"), b"broken").unwrap();
        let recovered = load(&d).unwrap().unwrap();
        assert_eq!(recovered.generation, 1);
        fs::write(d.join("save-1.json"), b"broken").unwrap();
        assert!(load(&d).is_err());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn clock_cap_and_regression() {
        let mut s = Save::new(State::new(2), 100);
        assert_eq!(s.catch_up(50).unwrap(), 0);
        assert_eq!(s.saved_at, 100);
        assert_eq!(s.catch_up(200_000).unwrap(), OFFLINE_CAP_SECS);
    }
    #[test]
    fn lock_excludes_second_instance() {
        let d = dir();
        let a = SaveLock::acquire(&d).unwrap();
        assert!(SaveLock::acquire(&d).is_err());
        drop(a);
        assert!(SaveLock::acquire(&d).is_ok());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn future_schema_cannot_fall_back_to_older_generation() {
        let d = dir();
        let mut s = Save::new(State::new(1), 100);
        store(&d, &mut s).unwrap();
        fs::write(
            d.join("save-0.json"),
            br#"{"schema_version":5,"content_version":1,"different_state":{}}"#,
        )
        .unwrap();
        assert!(load(&d).is_err());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn v1_migrates_inflight_job_without_losing_stock() {
        let d = dir();
        fs::create_dir_all(&d).unwrap();
        let mut state = State::new(42);
        state.command(1, Command::Craft).unwrap();
        state.advance(2).unwrap();
        let mut value = serde_json::to_value(Save::new(state, 100)).unwrap();
        value["schema_version"] = serde_json::json!(1);
        value["state"].as_object_mut().unwrap().remove("life");
        fs::write(d.join("save-1.json"), serde_json::to_vec(&value).unwrap()).unwrap();
        let mut migrated = load(&d).unwrap().unwrap();
        assert_eq!(migrated.schema_version, SCHEMA_VERSION);
        assert_eq!(migrated.state.life.fatigue, 0);
        assert_eq!(migrated.state.life.work, 3000);
        assert_eq!(migrated.state.showroom.reserved(), 1);
        migrated.catch_up(120).unwrap();
        assert_eq!(migrated.state.gold, 12);
        assert_eq!(migrated.state.ore, 8);
        store(&d, &mut migrated).unwrap();
        assert_eq!(load(&d).unwrap().unwrap(), migrated);
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn schema_three_stock_beyond_capacity_is_preserved_and_blocks_new_jobs() {
        let d = dir();
        fs::create_dir_all(&d).unwrap();
        let mut state = State::new(42);
        state.command(1, Command::Craft).unwrap();
        state.advance(2).unwrap();
        // Hand-build a pre-showroom state: seven finished daggers plus the in-flight job.
        state.crafted += 7;
        state.daggers += 7;
        let mut value = serde_json::to_value(Save::new(state, 100)).unwrap();
        value["schema_version"] = serde_json::json!(3);
        let st = value["state"].as_object_mut().unwrap();
        st.remove("showroom");
        st.remove("smith");
        fs::write(d.join("save-0.json"), serde_json::to_vec(&value).unwrap()).unwrap();
        let mut s = load(&d).unwrap().unwrap();
        assert_eq!(s.schema_version, 4);
        assert_eq!(s.state.smith, None);
        assert_eq!(s.state.daggers, 7);
        assert_eq!(s.state.showroom.goods(), 7);
        assert_eq!(s.state.showroom.reserved(), 1);
        assert_eq!(s.state.showroom.overflow, 4);
        s.state.validate().unwrap();
        // The in-flight job still completes into its reserved slot; new work waits for overflow.
        s.catch_up(103).unwrap();
        assert_eq!(s.state.crafted, 8);
        assert_eq!(s.state.showroom.free(), 0);
        assert_eq!(
            s.state.command(2, Command::Craft),
            Err(crate::Error::DisplayFull)
        );
        s.state.command(2, Command::QueueCraft).unwrap();
        assert_eq!(s.state.ore, 6);
        let before = s.state.gold;
        // Legacy overflow sells first; nothing is discarded or sold outside visitor sales.
        s.catch_up(100 + 3 + 4 * 31).unwrap();
        assert!(s.state.sold >= 4);
        assert!(s.state.gold > before);
        store(&d, &mut s).unwrap();
        assert_eq!(load(&d).unwrap().unwrap(), s);
        // A schema 4 file without the showroom field is not a valid candidate.
        let mut value = serde_json::to_value(&s).unwrap();
        value["state"].as_object_mut().unwrap().remove("showroom");
        value["generation"] = serde_json::json!(99);
        let other = d.join(format!("save-{}.json", (s.generation + 1) % 2));
        fs::write(other, serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(load(&d).unwrap().unwrap(), s);
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn abandoned_temp_does_not_replace_committed_state() {
        let d = dir();
        let mut s = Save::new(State::new(1), 100);
        store(&d, &mut s).unwrap();
        fs::write(d.join("save.tmp"), b"partial").unwrap();
        assert_eq!(load(&d).unwrap(), Some(s));
        fs::remove_dir_all(d).unwrap();
    }
}
