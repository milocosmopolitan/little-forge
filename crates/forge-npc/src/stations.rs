//! Native workshop station art (`assets/characters/stations/catalog.json`) and the fixed workshop
//! floor plan shared by the game and the isolated preview. No free placement, no pathfinding.
use crate::player::{PlayerCatalog, StationPlacement};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct StationArt {
    pub size: [u32; 2],
    pub path: String,
    pub back: String,
    pub front: String,
    pub water: String,
    pub fire: String,
    pub contact: [u32; 2],
}
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct StationCatalog {
    pub version: u32,
    pub world_canvas: [u32; 2],
    pub stations: BTreeMap<String, StationArt>,
}
impl StationCatalog {
    pub fn bundled() -> Result<Self, String> {
        Self::parse(include_str!(
            "../../../assets/characters/stations/catalog.json"
        ))
    }
    pub fn parse(json: &str) -> Result<Self, String> {
        let c: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if c.version != 1 {
            return Err("unsupported station catalog".into());
        }
        for (id, s) in &c.stations {
            for p in [&s.path, &s.back, &s.front, &s.water, &s.fire] {
                if p.contains("..") || p.starts_with('/') {
                    return Err(format!("invalid station path {id}"));
                }
            }
            if s.size.contains(&0) || s.contact[0] > s.size[0] || s.contact[1] > s.size[1] {
                return Err(format!("invalid station geometry {id}"));
            }
        }
        Ok(c)
    }
    pub fn asset_path(path: &str) -> String {
        format!("characters/{path}")
    }
}
/// Fixed floor plan in native scene pixels (x right, y down). Two depth rows keep research in
/// front of the desk and salvage behind the bench, as authored in ANIMATION.md.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Foot position of the smith while working here.
    pub foot: Vec2,
    /// Draw order row: 0 back row, 1 front row.
    pub row: u8,
}
pub const SCENE_SIZE: Vec2 = Vec2::new(480., 270.);
pub const BACK_FLOOR: f32 = 160.;
pub const FRONT_FLOOR: f32 = 245.;
/// Where the smith rests when idle, waiting or between jobs.
pub const REST_SPOT: Spot = Spot {
    foot: Vec2::new(232., FRONT_FLOOR),
    row: 1,
};
pub fn spot(station: &str) -> Option<Spot> {
    Some(match station {
        "salvage-table" => Spot {
            foot: Vec2::new(96., BACK_FLOOR),
            row: 0,
        },
        "forge" => Spot {
            foot: Vec2::new(300., BACK_FLOOR),
            row: 0,
        },
        "research-table" => Spot {
            foot: Vec2::new(120., FRONT_FLOOR),
            row: 1,
        },
        "anvil" => Spot {
            foot: Vec2::new(316., FRONT_FLOOR),
            row: 1,
        },
        "quenching-tub" => Spot {
            foot: Vec2::new(400., FRONT_FLOOR),
            row: 1,
        },
        _ => return None,
    })
}
/// Top-left of a station in scene pixels, derived from the authored contact offset.
pub fn station_origin(placement: &StationPlacement) -> Option<Vec2> {
    let s = spot(&placement.id)?;
    Some(s.foot + placement.offset.as_vec2())
}
/// Z layer for a scene element: rows separate depth, `layer` orders within a row.
pub fn z(row: u8, layer: f32) -> f32 {
    1. + row as f32 * 2. + layer
}
#[derive(Component)]
pub struct StationLayer {
    pub station: String,
    pub kind: &'static str,
    /// Native scene-pixel origin (top-left) and draw row/layer, for re-placement at any scale.
    pub origin: Vec2,
    pub row: u8,
    pub layer: f32,
}
impl StationLayer {
    /// World transform for a scene drawn at `scale` with its top-left at `world_origin`.
    pub fn transform(&self, world_origin: Vec2, scale: f32) -> Transform {
        Transform::from_xyz(
            world_origin.x + self.origin.x * scale,
            world_origin.y - self.origin.y * scale,
            z(self.row, self.layer),
        )
        .with_scale(Vec3::splat(scale))
    }
}
/// Spawn the layered art of one station at its floor-plan position.
pub fn spawn_station(
    commands: &mut Commands,
    server: &AssetServer,
    art: &StationCatalog,
    placement: &StationPlacement,
    scale: f32,
    parent_offset: Vec2,
) {
    let (Some(s), Some(origin), Some(sp)) = (
        art.stations.get(&placement.id),
        station_origin(placement),
        spot(&placement.id),
    ) else {
        return;
    };
    let behind = placement
        .occlusion
        .iter()
        .position(|o| o == "character")
        .unwrap_or(0);
    let order = |name: &str| -> f32 {
        // Entries before "character" draw behind the smith; later entries in front.
        let i = placement
            .occlusion
            .iter()
            .position(|o| o == name || (name == "back" && o == "station"))
            .unwrap_or(0);
        if i <= behind {
            0.2 + i as f32 * 0.05
        } else {
            0.8 + i as f32 * 0.05
        }
    };
    let layers: [(&'static str, &str, f32); 4] = [
        ("back", &s.back, order("station_back")),
        ("water", &s.water, order("station_water_fire")),
        ("fire", &s.fire, order("station_water_fire") + 0.01),
        ("front", &s.front, order("station_front")),
    ];
    for (kind, path, layer) in layers {
        commands.spawn((
            Sprite {
                image: server.load(StationCatalog::asset_path(path)),
                custom_size: Some(Vec2::new(s.size[0] as f32, s.size[1] as f32)),
                ..default()
            },
            bevy::sprite::Anchor::TOP_LEFT,
            Transform::from_xyz(
                parent_offset.x + origin.x * scale,
                parent_offset.y - origin.y * scale,
                z(sp.row, layer),
            )
            .with_scale(Vec3::splat(scale)),
            StationLayer {
                station: placement.id.clone(),
                kind,
                origin,
                row: sp.row,
                layer,
            },
        ));
    }
}
/// Z for the smith standing at a given floor row.
pub fn actor_z(row: u8) -> f32 {
    z(row, 0.5)
}
pub fn placements(c: &PlayerCatalog) -> impl Iterator<Item = &StationPlacement> {
    c.stations.values()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_plan_keeps_every_station_inside_the_scene() {
        let art = StationCatalog::bundled().unwrap();
        for id in crate::player::CHARACTERS {
            let c = PlayerCatalog::bundled(id).unwrap();
            for p in placements(&c) {
                let o = station_origin(p).unwrap();
                let s = &art.stations[&p.id];
                assert!(o.x >= 0. && o.y >= 0., "{} origin {o:?}", p.id);
                assert!(
                    o.x + s.size[0] as f32 <= SCENE_SIZE.x
                        && o.y + s.size[1] as f32 <= SCENE_SIZE.y,
                    "{} overflows {o:?}",
                    p.id
                );
            }
        }
        // The research desk is drawn in front of the smith; the salvage bench behind.
        let c = PlayerCatalog::bundled("male").unwrap();
        let research = &c.stations["research-table"];
        assert!(
            research.occlusion.iter().position(|o| o == "station")
                > research.occlusion.iter().position(|o| o == "character")
        );
        assert!(z(1, 0.) > z(0, 0.9));
        assert!(StationCatalog::parse("{}").is_err());
    }
}
