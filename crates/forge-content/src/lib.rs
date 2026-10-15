use serde::Deserialize;
use std::collections::HashSet;
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub assets: Vec<Asset>,
}
#[derive(Debug, Deserialize)]
pub struct Asset {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub columns: u32,
    pub rows: u32,
    pub alpha: bool,
    pub anchor: [f32; 2],
    pub sha256: String,
}
impl Manifest {
    pub fn bundled() -> Result<Self, String> {
        let m: Self = serde_json::from_str(include_str!("../../../assets/manifest.json"))
            .map_err(|e| e.to_string())?;
        m.validate()?;
        Ok(m)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported manifest".into());
        }
        let mut ids = HashSet::new();
        for a in &self.assets {
            if !ids.insert(&a.id)
                || a.width == 0
                || a.height == 0
                || a.columns == 0
                || a.rows == 0
                || a.width % a.columns != 0
                || a.height % a.rows != 0
                || a.path.contains("..")
                || a.path.starts_with('/')
                || a.anchor
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err(format!("invalid asset {}", a.id));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_is_valid() {
        Manifest::bundled().unwrap();
    }
    #[test]
    fn duplicates_rejected() {
        let mut m = Manifest::bundled().unwrap();
        m.assets[1].id = m.assets[0].id.clone();
        assert!(m.validate().is_err());
    }
}
