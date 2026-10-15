//! Recoverable two-slot journal. Each slot is atomically published by rename to an absent path.
//! A crash during replacement leaves the other committed generation intact, including on Windows.
use crate::{CONTENT_VERSION, OFFLINE_CAP_SECS, State};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};
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
            schema_version: 1,
            content_version: CONTENT_VERSION,
            generation: 0,
            saved_at: now,
            state,
        }
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1 || self.content_version != CONTENT_VERSION {
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
                if let Ok(s) = serde_json::from_slice::<Save>(&bytes) {
                    // Future formats must not be silently downgraded through an older slot.
                    if s.schema_version > 1 || s.content_version > CONTENT_VERSION {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "newer save version",
                        ));
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
    fn dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "forge-save-{}-{}",
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
}
