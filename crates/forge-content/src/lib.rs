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
    #[serde(default)]
    pub frame_rects: Vec<[u32; 4]>,
    #[serde(default)]
    pub walking_hand_points: Vec<[u32; 2]>,
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
            if !a.walking_hand_points.is_empty() {
                if a.walking_hand_points.len() != 4 || a.frame_rects.len() < 8 {
                    return Err(format!("invalid hand anchors {}", a.id));
                }
                for (i, p) in a.walking_hand_points.iter().enumerate() {
                    let r = a.frame_rects[i + 4];
                    if p[0] < r[0] || p[0] >= r[2] || p[1] < r[1] || p[1] >= r[3] {
                        return Err(format!("hand outside frame {}", a.id));
                    }
                }
            }
            if !a.frame_rects.is_empty()
                && (a.frame_rects.len() != (a.columns * a.rows) as usize
                    || a.frame_rects
                        .iter()
                        .any(|r| r[0] >= r[2] || r[1] >= r[3] || r[2] > a.width || r[3] > a.height))
            {
                return Err(format!("invalid frames {}", a.id));
            }
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
/// Provisional production balance, separate from QA scenario time and fixtures.
pub fn needs_config() -> Result<forge_core::life::NeedsConfig, String> {
    let config = serde_json::from_str(include_str!("../../../assets/needs.json"))
        .map_err(|e| format!("{e}"))?;
    let life = forge_core::life::Life {
        config,
        ..Default::default()
    };
    life.validate().map_err(|e| format!("{e:?}"))?;
    Ok(life.config)
}

/// Draft showroom furniture balance (#95): slot counts, installation limit and carpenter prices.
/// Values are review candidates recorded as data, not confirmed balance.
pub fn showroom_config() -> Result<forge_core::showroom::ShowroomConfig, String> {
    let config: forge_core::showroom::ShowroomConfig =
        serde_json::from_str(include_str!("../../../assets/showroom.json"))
            .map_err(|e| format!("{e}"))?;
    config.validate().map_err(|e| format!("{e:?}"))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_is_valid() {
        Manifest::bundled().unwrap();
        expedition_content().unwrap();
    }
    #[test]
    fn showroom_balance_is_valid_and_matches_core_defaults() {
        let config = showroom_config().unwrap();
        assert_eq!(config, forge_core::showroom::ShowroomConfig::default());
        needs_config().unwrap();
    }
    #[test]
    fn duplicates_rejected() {
        let mut m = Manifest::bundled().unwrap();
        m.assets[1].id = m.assets[0].id.clone();
        assert!(m.validate().is_err());
    }
}

pub mod village;

/// Versioned first-region contract, checked against the deterministic simulation rules.
pub fn expedition_content() -> Result<serde_json::Value, String> {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/expedition/encounters.json"))
            .map_err(|e| e.to_string())?;
    let waves: Vec<Vec<forge_core::expedition::Enemy>> =
        serde_json::from_value(data["waves"].clone()).map_err(|e| e.to_string())?;
    if data["version"] != forge_core::expedition::VERSION
        || waves.iter().map(Vec::as_slice).collect::<Vec<_>>() != forge_core::expedition::WAVES
    {
        return Err("expedition content and simulation version differ".into());
    }
    Ok(data)
}

/// Alpha tuning and stable content IDs; core validates reachability before a new game.
pub fn progression_rules() -> Result<forge_core::progression::Rules, String> {
    let rules: forge_core::progression::Rules =
        serde_json::from_str(include_str!("../../../assets/progression.json"))
            .map_err(|e| e.to_string())?;
    rules.validate().map_err(|e| format!("{e:?}"))?;
    Ok(rules)
}
