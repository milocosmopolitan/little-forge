//! Presentation-only NPC playback shared by the game and the isolated preview.
//! No game state, persistence, economy or window bootstrap dependencies.
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

pub const ACTIONS: [&str; 8] = [
    "idle",
    "talk",
    "blink",
    "surprised",
    "nod",
    "shake_head",
    "walk",
    "give",
];
#[derive(Debug, Clone, Deserialize)]
pub struct Frame {
    pub rect: [u32; 4],
    pub duration: f32,
    pub hand: [f32; 2],
    #[serde(default)]
    pub transfer: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Clip {
    pub frames: Vec<usize>,
    pub looping: bool,
}
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct FaceAssets {
    pub mouths: String,
    pub eyes: String,
    pub hands: String,
}
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct Catalog {
    pub id: String,
    pub faces: Option<FaceAssets>,
    pub version: u32,
    pub image: String,
    pub size: [u32; 2],
    pub cell: [u32; 2],
    pub foot: [f32; 2],
    pub head: [f32; 2],
    pub frames: Vec<Frame>,
    pub clips: BTreeMap<String, Clip>,
}
impl Catalog {
    pub fn fixture() -> Self {
        let mut c = Self::farmer();
        c.id = "fixture-b".into();
        for f in &mut c.frames {
            f.duration *= 1.4;
        }
        c
    }
    pub fn farmer() -> Self {
        Self::parse(include_str!("../../../assets/npc/farmer/character.json"))
            .expect("validated farmer")
    }
    pub fn parse(json: &str) -> Result<Self, String> {
        let c: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.cell.contains(&0)
            || self.size.contains(&0)
            || self.image.contains("..")
            || self.image.starts_with('/')
        {
            return Err("invalid NPC header".into());
        }
        if let Some(faces) = &self.faces {
            for path in [&faces.mouths, &faces.eyes, &faces.hands] {
                if path.starts_with('/') || path.contains("..") {
                    return Err("invalid layer path".into());
                }
            }
        }
        for p in [self.foot, self.head] {
            if p.iter()
                .enumerate()
                .any(|(i, v)| !v.is_finite() || *v < 0. || *v > self.cell[i] as f32)
            {
                return Err("invalid reference socket".into());
            }
        }
        for f in &self.frames {
            let [x, y, w, h] = f.rect;
            if w != self.cell[0]
                || h != self.cell[1]
                || x.checked_add(w).is_none_or(|r| r > self.size[0])
                || y.checked_add(h).is_none_or(|r| r > self.size[1])
                || !f.duration.is_finite()
                || f.duration <= 0.
                || f.hand
                    .iter()
                    .enumerate()
                    .any(|(i, v)| !v.is_finite() || *v < 0. || *v > self.cell[i] as f32)
            {
                return Err("invalid frame/hand/duration".into());
            }
        }
        let mut used = HashSet::<usize>::new();
        for action in ACTIONS {
            let c = self
                .clips
                .get(action)
                .ok_or_else(|| format!("missing {action}"))?;
            if c.frames.is_empty() || c.frames.iter().any(|i| *i >= self.frames.len()) {
                return Err(format!("invalid clip {action}"));
            }
            used.extend(c.frames.iter().copied());
        }
        if used.is_empty() {
            return Err("empty catalog".into());
        }
        Ok(())
    }
    pub fn anchor(&self) -> bevy::sprite::Anchor {
        bevy::sprite::Anchor(Vec2::new(
            self.foot[0] / self.cell[0] as f32 - 0.5,
            0.5 - self.foot[1] / self.cell[1] as f32,
        ))
    }
    pub fn layout(&self) -> TextureAtlasLayout {
        let mut a = TextureAtlasLayout::new_empty(UVec2::from_array(self.size));
        for f in &self.frames {
            let [x, y, w, h] = f.rect;
            a.add_texture(URect::new(x, y, x + w, y + h));
        }
        a
    }
}

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub action: String,
    pub elapsed: f32,
    pub paused: bool,
    pub speed: f32,
    pub facing_left: bool,
    pub blink: bool,
    pub item: bool,
    pub frame: usize,
    pub transferred: bool,
    pub layers: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            action: "idle".into(),
            elapsed: 0.,
            paused: false,
            speed: 1.,
            facing_left: false,
            blink: true,
            item: false,
            frame: 0,
            transferred: false,
            layers: true,
        }
    }
}
impl Player {
    pub fn play(&mut self, action: &str, c: &Catalog) {
        let name = if c.clips.contains_key(action) {
            action
        } else {
            "idle"
        };
        self.action = name.into();
        self.elapsed = 0.;
        self.frame = c.clips[name].frames[0];
        self.transferred = false;
    }
    pub fn seek(&mut self, t: f32, c: &Catalog) {
        self.transferred = false;
        self.elapsed = if t.is_finite() { t.max(0.) } else { 0. };
        self.sample(c);
    }
    pub fn step(&mut self, forward: bool, c: &Catalog) {
        self.paused = true;
        let clip = c.clips.get(&self.action).unwrap_or(&c.clips["idle"]);
        let total: f32 = clip.frames.iter().map(|i| c.frames[*i].duration).sum();
        let t = if clip.looping {
            self.elapsed % total
        } else {
            self.elapsed
        };
        let mut starts = vec![0.];
        let mut sum = 0.;
        for i in &clip.frames {
            sum += c.frames[*i].duration;
            starts.push(sum);
        }
        let target = if forward {
            starts
                .iter()
                .copied()
                .find(|s| *s > t + 0.0001)
                .unwrap_or(0.)
        } else {
            starts
                .iter()
                .copied()
                .rev()
                .find(|s| *s < t - 0.0001)
                .unwrap_or(0.)
        };
        self.seek(target + 0.00001, c);
    }
    pub fn advance(&mut self, dt: f32, c: &Catalog) {
        if !self.paused && dt.is_finite() && dt > 0. && self.speed.is_finite() {
            self.elapsed += dt * self.speed.max(0.);
            self.sample(c);
        }
    }
    fn sample(&mut self, c: &Catalog) {
        let clip = c.clips.get(&self.action).unwrap_or(&c.clips["idle"]);
        let total: f32 = clip.frames.iter().map(|i| c.frames[*i].duration).sum();
        if !clip.looping && self.elapsed >= total {
            // Preserve a crossed transfer marker even when a large delta skips its frame.
            self.transferred |= clip.frames.iter().any(|i| c.frames[*i].transfer);
            self.action = "idle".into();
            self.elapsed = 0.;
            self.frame = c.clips["idle"].frames[0];
            return;
        }
        let mut t = if clip.looping {
            self.elapsed % total
        } else {
            self.elapsed
        };
        for i in &clip.frames {
            if c.frames[*i].transfer {
                self.transferred = true;
            }
            if t < c.frames[*i].duration {
                self.frame = *i;
                break;
            }
            t -= c.frames[*i].duration;
        }
    }
    pub fn hand_offset(&self, c: &Catalog, height: f32) -> Vec2 {
        let f = &c.frames[self.frame];
        let s = height / c.cell[1] as f32;
        let x = (f.hand[0] - c.foot[0]) * s;
        Vec2::new(
            if self.facing_left { -x } else { x },
            (c.foot[1] - f.hand[1]) * s,
        )
    }
}
/// Sprite player uses full registered frames. Face overlays may be supplied independently.
#[derive(Component)]
pub struct NpcVisual;
#[derive(Resource, Default)]
pub struct PlaybackClock {
    pub fixed_delta: Option<f32>,
}
pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Catalog::farmer())
            .init_resource::<PlaybackClock>()
            .add_systems(
                Update,
                (add_face_layers, animate, attach_items, face_layers).chain(),
            );
    }
}
pub fn animate(
    time: Res<Time>,
    clock: Res<PlaybackClock>,
    c: Res<Catalog>,
    mut q: Query<(&mut Player, &mut Sprite, Option<&Catalog>)>,
) {
    for (mut p, mut s, instance) in &mut q {
        let catalog = instance.unwrap_or(&c);
        p.advance(
            clock.fixed_delta.unwrap_or_else(|| time.delta_secs()),
            catalog,
        );
        if let Some(a) = &mut s.texture_atlas {
            a.index = if p.action == "talk" { 0 } else { p.frame };
        }
        s.flip_x = p.facing_left;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_content_rejected() {
        let mut c = Catalog::farmer();
        c.frames[0].duration = 0.;
        assert!(c.validate().is_err());
        let mut c = Catalog::farmer();
        c.frames[0].rect[0] = u32::MAX;
        assert!(c.validate().is_err());
        let mut c = Catalog::farmer();
        c.clips.remove("give");
        assert!(c.validate().is_err());
        let mut c = Catalog::farmer();
        c.frames[0].hand = [f32::NAN, 0.];
        assert!(c.validate().is_err());
    }
    #[test]
    fn partition_mirror_and_interruption() {
        let c = Catalog::farmer();
        let mut a = Player::default();
        a.play("walk", &c);
        let mut b = a.clone();
        a.advance(0.45, &c);
        for _ in 0..45 {
            b.advance(0.01, &c);
        }
        assert_eq!(a.frame, b.frame);
        let hand = a.hand_offset(&c, 100.);
        a.facing_left = true;
        assert_eq!(a.hand_offset(&c, 100.), Vec2::new(-hand.x, hand.y));
        a.play("give", &c);
        a.advance(10., &c);
        assert_eq!(a.action, "idle");
        assert!(a.transferred);
        a.play("nod", &c);
        assert!(!a.transferred);
        a.play("missing", &c);
        assert_eq!(a.action, "idle");
    }
    #[test]
    fn handoff_scrub_and_independent_catalogs() {
        let c = Catalog::farmer();
        let fixture = Catalog::fixture();
        let mut p = Player::default();
        p.play("give", &c);
        p.seek(0.13, &c);
        assert!(!p.transferred);
        p.seek(0.8, &c);
        assert!(p.transferred);
        p.seek(0., &c);
        assert!(!p.transferred);
        let mut other = Player::default();
        other.play("walk", &fixture);
        other.seek(0.13, &fixture);
        p.play("walk", &c);
        p.seek(0.13, &c);
        assert_ne!(p.frame, other.frame);
    }
    #[test]
    fn paused_seek_is_deterministic() {
        let c = Catalog::farmer();
        let mut p = Player::default();
        p.play("walk", &c);
        p.paused = true;
        p.advance(10., &c);
        assert_eq!(p.elapsed, 0.);
        p.seek(0.4, &c);
        assert_eq!(p.elapsed, 0.4);
    }
}

#[derive(Component)]
struct FaceLayer {
    owner: Entity,
    kind: u8,
}
fn add_face_layers(
    mut commands: Commands,
    added: Query<(Entity, Option<&Catalog>), Added<Player>>,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    c: Res<Catalog>,
) {
    for (owner, instance) in &added {
        let c = instance.unwrap_or(&c);
        let Some(face) = &c.faces else {
            continue;
        };
        for (kind, path, count, rows) in [
            (0, face.mouths.clone(), 4, 1),
            (1, face.eyes.clone(), 3, 1),
            (2, face.hands.clone(), 4, 6),
        ] {
            commands.spawn((
                Sprite {
                    image: server.load(path),
                    texture_atlas: Some(TextureAtlas {
                        layout: layouts.add(if kind == 2 {
                            c.layout()
                        } else {
                            TextureAtlasLayout::from_grid(
                                UVec2::from_array(c.cell),
                                count,
                                rows,
                                None,
                                None,
                            )
                        }),
                        index: 0,
                    }),
                    ..default()
                },
                Transform::default(),
                c.anchor(),
                Visibility::Hidden,
                FaceLayer { owner, kind },
                NpcVisual,
            ));
        }
    }
}
#[allow(clippy::type_complexity)]
fn face_layers(
    mut commands: Commands,
    actors: Query<(&Player, &Sprite, &Transform, &Visibility), With<Player>>,
    mut layers: Query<
        (
            Entity,
            &FaceLayer,
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
        ),
        Without<Player>,
    >,
) {
    for (entity, layer, mut sprite, mut transform, mut visibility) in &mut layers {
        let Ok((p, base, t, v)) = actors.get(layer.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        let blink_phase = p.elapsed % 3.5;
        let blinking = p.blink && blink_phase > 3.20 && blink_phase < 3.48;
        let compatible = matches!(p.action.as_str(), "idle" | "talk");
        let active = if layer.kind == 2 {
            true
        } else if layer.kind == 0 {
            p.action == "talk"
        } else {
            compatible && blinking
        };
        *visibility = if active && p.layers && *v != Visibility::Hidden {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *transform = *t;
        transform.translation.z += if layer.kind == 2 {
            0.6
        } else if layer.kind == 0 {
            0.02
        } else {
            0.03
        };
        sprite.custom_size = base.custom_size;
        sprite.flip_x = base.flip_x;
        sprite.color = base.color;
        if let Some(a) = &mut sprite.texture_atlas {
            a.index = if layer.kind == 2 {
                p.frame
            } else if layer.kind == 0 {
                p.frame.min(3)
            } else if blink_phase < 3.27 || blink_phase > 3.41 {
                1
            } else {
                2
            };
        }
    }
}

/// Existing Foundation dagger art shared by game and preview item tests.
pub fn dagger_image() -> Image {
    let mut bytes = vec![0u8; 8 * 24 * 4];
    for y in 0..24 {
        for x in 0..8 {
            let on = if y < 16 {
                (3..=4).contains(&x)
            } else if y < 18 {
                (1..=6).contains(&x)
            } else {
                (3..=4).contains(&x)
            };
            if on {
                let i = (y * 8 + x) * 4;
                bytes[i..i + 4].copy_from_slice(if y < 16 {
                    &[202, 217, 219, 255]
                } else {
                    &[157, 104, 50, 255]
                });
            }
        }
    }
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: 8,
            height: 24,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        bytes,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

/// Interchangeable prop attached through the same socket renderer in game and preview.
#[derive(Component)]
pub struct ItemAttachment {
    pub owner: Entity,
    pub enabled: bool,
}
#[allow(clippy::type_complexity)]
fn attach_items(
    actors: Query<(&Player, &Sprite, &Transform, &Visibility, Option<&Catalog>), With<Player>>,
    default_catalog: Res<Catalog>,
    mut items: Query<
        (
            &ItemAttachment,
            &mut Transform,
            &mut Visibility,
            &mut Sprite,
        ),
        Without<Player>,
    >,
) {
    for (attachment, mut transform, mut visible, mut sprite) in &mut items {
        sprite.flip_x = false;
        let Ok((p, s, t, v, catalog)) = actors.get(attachment.owner) else {
            *visible = Visibility::Hidden;
            continue;
        };
        sprite.flip_x = p.facing_left;
        let c = catalog.unwrap_or(&default_catalog);
        let height = s.custom_size.map_or(c.cell[1] as f32, |s| s.y) * t.scale.y;
        transform.translation =
            (t.translation.truncate() + p.hand_offset(c, height)).extend(t.translation.z + 0.5);
        transform.scale = t.scale;
        *visible = if attachment.enabled
            && *v != Visibility::Hidden
            && (p.item || p.action == "give")
            && !p.transferred
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
