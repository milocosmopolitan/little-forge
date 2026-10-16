//! Canonical image-space registry; all picking uses the same letterboxed rectangle.
use forge_core::village::Location;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Registry {
    pub width: u32,
    pub height: u32,
    pub locations: Vec<Region>,
}
#[derive(Debug, Deserialize)]
pub struct Region {
    pub id: Location,
    pub label: String,
    pub polygon: Vec<[f32; 2]>,
    pub overlay: String,
}
impl Region {
    pub fn contains(&self, p: [f32; 2]) -> bool {
        let mut inside = false;
        for i in 0..self.polygon.len() {
            let a = self.polygon[i];
            let b = self.polygon[(i + 1) % self.polygon.len()];
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
        }
        inside
    }
    pub fn center(&self) -> [f32; 2] {
        let n = self.polygon.len() as f32;
        [
            self.polygon.iter().map(|p| p[0]).sum::<f32>() / n,
            self.polygon.iter().map(|p| p[1]).sum::<f32>() / n,
        ]
    }
}
impl Registry {
    pub fn bundled() -> Result<Self, String> {
        let r: Self =
            serde_json::from_str(include_str!("../../../assets/map/ashbrook/locations.json"))
                .map_err(|e| e.to_string())?;
        r.validate()?;
        Ok(r)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 || self.locations.len() != 8 {
            return Err("map dimensions / region count".into());
        }
        for id in Location::ALL {
            if self.locations.iter().filter(|r| r.id == id).count() != 1 {
                return Err("missing/duplicate destination".into());
            }
        }
        for r in &self.locations {
            if r.polygon.len() < 3
                || r.polygon
                    .iter()
                    .flatten()
                    .any(|n| !n.is_finite() || !(0.0..=1.0).contains(n))
                || !r.contains(r.center())
                || !r.overlay.starts_with("map/ashbrook/overlays/")
                || r.overlay.contains("..")
            {
                return Err("invalid region".into());
            }
        }
        // Exhaustive sampled overlap gate: ambiguous hit targets are not accepted.
        for y in 0..300 {
            for x in 0..300 {
                if self
                    .locations
                    .iter()
                    .filter(|r| r.contains([x as f32 / 300., y as f32 / 300.]))
                    .count()
                    > 1
                {
                    return Err("overlapping regions".into());
                }
            }
        }
        Ok(())
    }
    pub fn pick(&self, p: [f32; 2]) -> Option<usize> {
        if p.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
            return None;
        }
        self.locations.iter().position(|r| r.contains(p))
    }
    /// Logical panel point to normalized image point. DPI cancels when inputs share units.
    pub fn image_point(&self, p: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]> {
        if size.iter().any(|s| !s.is_finite() || *s <= 0.) {
            return None;
        }
        let scale = (size[0] / self.width as f32).min(size[1] / self.height as f32);
        let draw = [self.width as f32 * scale, self.height as f32 * scale];
        let origin = [(size[0] - draw[0]) / 2., (size[1] - draw[1]) / 2.];
        let n = [(p[0] - origin[0]) / draw[0], (p[1] - origin[1]) / draw[1]];
        if n.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
            Some(n)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picking_matches_scaled_and_letterboxed_art() {
        let r = Registry::bundled().unwrap();
        for size in [
            [400., 280.],
            [600., 420.],
            [800., 560.],
            [200., 140.],
            [160., 350.],
        ] {
            let scale = (size[0] / r.width as f32).min(size[1] / r.height as f32);
            let d = [r.width as f32 * scale, r.height as f32 * scale];
            for (i, l) in r.locations.iter().enumerate() {
                let c = l.center();
                let p = [
                    (size[0] - d[0]) / 2. + c[0] * d[0],
                    (size[1] - d[1]) / 2. + c[1] * d[1],
                ];
                assert_eq!(r.pick(r.image_point(p, size).unwrap()), Some(i));
            }
        }
        assert!(r.image_point([0., 0.], [400., 280.]).is_none());
        assert_eq!(r.pick([0.50, 0.50]), None);
        assert_eq!(r.pick([f32::NAN, 0.]), None);
    }
    #[test]
    fn broken_registry_is_rejected() {
        let mut r = Registry::bundled().unwrap();
        r.locations[1].id = r.locations[0].id;
        assert!(r.validate().is_err());
    }
}
