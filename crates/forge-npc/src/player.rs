//! Player smith playback: consumes the production `atlas-manifest.json` (trimmed sprites, pivots)
//! and `animation.json` (clip stations, facing, loop ranges, event markers) of each character.
//! Presentation only: event markers are sound/effect hooks and never grant results.
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;

pub const CHARACTERS: [&str; 2] = ["female", "male"];
/// Clips every player character must provide, in the order the preview lists them.
pub const PLAYER_CLIPS: [&str; 17] = [
    "idle",
    "blink",
    "talk",
    "talk_blink",
    "happy",
    "surprised",
    "relaxed",
    "nod",
    "shake_head",
    "give",
    "walk",
    "front_blink",
    "hammering",
    "quenching",
    "forging",
    "researching",
    "salvaging",
];
pub const WORK_CLIPS: [&str; 5] = [
    "forging",
    "hammering",
    "quenching",
    "researching",
    "salvaging",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub enum Facing {
    #[default]
    #[serde(rename = "right_3_4")]
    Right34,
    #[serde(rename = "front")]
    Front,
    #[serde(rename = "back")]
    Back,
}
impl Facing {
    /// Clip to rest in when a one-shot clip ends.
    pub fn rest_clip(self) -> &'static str {
        match self {
            Self::Right34 => "idle",
            Self::Front => "front_idle",
            Self::Back => "back_idle",
        }
    }
    pub fn blink_clip(self) -> Option<&'static str> {
        match self {
            Self::Right34 => Some("blink"),
            Self::Front => Some("front_blink"),
            Self::Back => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSprite {
    pub id: String,
    /// x, y, w, h in the atlas page (or in the standalone file when `file` is set).
    pub rect: [u32; 4],
    pub trim: [u32; 2],
    pub source_size: [u32; 2],
    pub pivot: [u32; 2],
    /// Standalone PNG relative to the asset root, for authored scene derivatives.
    pub file: Option<String>,
}
impl PlayerSprite {
    /// Bevy anchor placing the foot pivot at the entity origin.
    pub fn anchor(&self, flipped: bool) -> Vec2 {
        let [_, _, w, h] = self.rect;
        let x = (self.pivot[0] as f32 - self.trim[0] as f32) / w as f32 - 0.5;
        let y = 0.5 - (self.pivot[1] as f32 - self.trim[1] as f32) / h as f32;
        Vec2::new(if flipped { -x } else { x }, y)
    }
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.rect[2] as f32, self.rect[3] as f32)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClipFrame {
    pub sprite: usize,
    pub duration: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClipEvent {
    pub frame: usize,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerClip {
    pub frames: Vec<ClipFrame>,
    pub looping: bool,
    /// Inclusive frame range held while the station work continues.
    pub loop_range: Option<[usize; 2]>,
    pub events: Vec<ClipEvent>,
    pub station: Option<String>,
    pub facing: Facing,
}
impl PlayerClip {
    pub fn total(&self) -> f32 {
        self.frames.iter().map(|f| f.duration).sum()
    }
}
/// Station placement authored in animation.json: where the station sits relative to the
/// character canvas origin, which layers draw in front of the smith and which markers fire.
#[derive(Debug, Clone, PartialEq)]
pub struct StationPlacement {
    pub id: String,
    /// Station origin minus the character foot pivot, in native pixels (x right, y down).
    pub offset: IVec2,
    pub occlusion: Vec<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerCatalog {
    pub id: String,
    pub image: String,
    pub page_size: [u32; 2],
    pub canvas: [u32; 2],
    pub pivot: [u32; 2],
    pub sprites: Vec<PlayerSprite>,
    pub clips: BTreeMap<String, PlayerClip>,
    pub stations: BTreeMap<String, StationPlacement>,
}
#[derive(Deserialize)]
struct AtlasManifest {
    version: u32,
    pages: Vec<AtlasPage>,
    sprites: Vec<AtlasSprite>,
    clips: BTreeMap<String, RawClip>,
}
#[derive(Deserialize)]
struct AtlasPage {
    path: String,
    size: [u32; 2],
}
#[derive(Deserialize)]
struct AtlasSprite {
    id: String,
    rect: [u32; 4],
    source_size: [u32; 2],
    trim_offset: [u32; 2],
    pivot: [u32; 2],
    #[serde(default)]
    rotated: bool,
}
#[derive(Deserialize)]
struct RawClip {
    #[serde(rename = "loop")]
    looping: bool,
    frames: Vec<RawFrame>,
    #[serde(default)]
    loop_range: Option<[usize; 2]>,
    #[serde(default)]
    events: Vec<RawEvent>,
    #[serde(default)]
    station_id: Option<String>,
    #[serde(default)]
    facing: Facing,
}
#[derive(Deserialize)]
struct RawFrame {
    sprite: String,
    duration_ms: u32,
}
#[derive(Deserialize)]
struct RawEvent {
    frame: usize,
    event: String,
}
#[derive(Deserialize)]
struct Animation {
    version: u32,
    canvas: [u32; 2],
    pivot: [u32; 2],
    clips: BTreeMap<String, RawClip>,
    workshop: Workshop,
}
#[derive(Deserialize)]
struct Workshop {
    character_origin: [i32; 2],
    stations: BTreeMap<String, RawStation>,
}
#[derive(Deserialize)]
struct RawStation {
    station_id: String,
    origin: [i32; 2],
    occlusion_order: Vec<String>,
}
impl PlayerCatalog {
    pub fn bundled(id: &str) -> Result<Self, String> {
        match id {
            "female" => Self::parse(
                "female",
                include_str!("../../../assets/characters/female/atlas-manifest.json"),
                include_str!("../../../assets/characters/female/animation.json"),
            ),
            "male" => Self::parse(
                "male",
                include_str!("../../../assets/characters/male/atlas-manifest.json"),
                include_str!("../../../assets/characters/male/animation.json"),
            ),
            _ => Err(format!("unknown player character {id}")),
        }
    }
    pub fn parse(id: &str, atlas_json: &str, animation_json: &str) -> Result<Self, String> {
        let atlas: AtlasManifest = serde_json::from_str(atlas_json).map_err(|e| e.to_string())?;
        let anim: Animation = serde_json::from_str(animation_json).map_err(|e| e.to_string())?;
        if atlas.version != 1 || anim.version != 1 || atlas.pages.len() != 1 {
            return Err("unsupported player manifest".into());
        }
        let page = &atlas.pages[0];
        if page.path.contains("..") || page.path.starts_with('/') {
            return Err("invalid atlas path".into());
        }
        // "base/" entries are the unequipped variants; they are kept for the preview's gear toggle.
        let mut sprites: Vec<PlayerSprite> = atlas
            .sprites
            .iter()
            .map(|s| PlayerSprite {
                id: s.id.clone(),
                rect: s.rect,
                trim: s.trim_offset,
                source_size: s.source_size,
                pivot: s.pivot,
                file: None,
            })
            .collect();
        for s in &atlas.sprites {
            if s.rotated {
                return Err(format!("rotated sprite {} unsupported", s.id));
            }
        }
        let mut clips = BTreeMap::new();
        let find = |sprites: &[PlayerSprite], name: &str| sprites.iter().position(|s| s.id == name);
        for (name, raw) in &atlas.clips {
            // animation.json is authoritative for station, facing, loop range and events.
            let meta = anim
                .clips
                .get(name.trim_start_matches("base/"))
                .unwrap_or(raw);
            let mut frames = vec![];
            for f in &raw.frames {
                let sprite = find(&sprites, &f.sprite)
                    .ok_or_else(|| format!("clip {name} uses unknown sprite {}", f.sprite))?;
                if f.duration_ms == 0 {
                    return Err(format!("clip {name} has a zero-length frame"));
                }
                frames.push(ClipFrame {
                    sprite,
                    duration: f.duration_ms as f32 / 1000.,
                });
            }
            if frames.is_empty() {
                return Err(format!("empty clip {name}"));
            }
            if let Some([a, b]) = meta.loop_range
                && (a > b || b >= frames.len())
            {
                return Err(format!("clip {name} loop range out of bounds"));
            }
            if meta.events.iter().any(|e| e.frame >= frames.len()) {
                return Err(format!("clip {name} event frame out of bounds"));
            }
            clips.insert(
                name.clone(),
                PlayerClip {
                    frames,
                    looping: raw.looping,
                    loop_range: meta.loop_range,
                    events: meta
                        .events
                        .iter()
                        .map(|e| ClipEvent {
                            frame: e.frame,
                            name: e.event.clone(),
                        })
                        .collect(),
                    station: meta.station_id.clone(),
                    facing: meta.facing,
                },
            );
        }
        for required in PLAYER_CLIPS {
            if !clips.contains_key(required) {
                return Err(format!("missing player clip {required}"));
            }
        }
        // Quenching in a scene uses the authored derivatives with submerged tool pixels clipped.
        for (clip, folder) in [
            ("quenching", "frames"),
            ("base/quenching", "frames/scene-base"),
        ] {
            if let Some(q) = clips.get_mut(clip) {
                for (i, frame) in q.frames.iter_mut().enumerate() {
                    let base = sprites[frame.sprite].clone();
                    let file = if folder == "frames" {
                        format!("characters/{id}/frames/scene-quenching-{i}.png")
                    } else {
                        format!("characters/{id}/{folder}/quenching-{i}.png")
                    };
                    sprites.push(PlayerSprite {
                        id: format!("{clip}/scene-{i}"),
                        rect: [0, 0, base.source_size[0], base.source_size[1]],
                        trim: [0, 0],
                        source_size: base.source_size,
                        pivot: base.pivot,
                        file: Some(file),
                    });
                    frame.sprite = sprites.len() - 1;
                }
            }
        }
        // Synthetic rest clips for the front/back baselines so one-shot clips have somewhere to end.
        for prefix in ["", "base/"] {
            for (clip, sprite) in [
                ("front_idle", "front-neutral"),
                ("back_idle", "back-neutral"),
            ] {
                let name = format!("{prefix}{sprite}");
                let index = find(&sprites, &name).ok_or_else(|| format!("missing {name}"))?;
                clips.insert(
                    format!("{prefix}{clip}"),
                    PlayerClip {
                        frames: vec![ClipFrame {
                            sprite: index,
                            duration: 1.,
                        }],
                        looping: true,
                        loop_range: None,
                        events: vec![],
                        station: None,
                        facing: if clip == "front_idle" {
                            Facing::Front
                        } else {
                            Facing::Back
                        },
                    },
                );
            }
        }
        for s in &sprites {
            let [x, y, w, h] = s.rect;
            let within = if s.file.is_some() {
                w == s.source_size[0] && h == s.source_size[1]
            } else {
                x + w <= page.size[0] && y + h <= page.size[1]
            };
            if w == 0
                || h == 0
                || !within
                || s.pivot[0] > s.source_size[0]
                || s.pivot[1] > s.source_size[1]
                || s.trim[0] + w > s.source_size[0]
                || s.trim[1] + h > s.source_size[1]
            {
                return Err(format!("invalid sprite geometry {}", s.id));
            }
        }
        let foot = IVec2::new(
            anim.workshop.character_origin[0] + anim.pivot[0] as i32,
            anim.workshop.character_origin[1] + anim.pivot[1] as i32,
        );
        let stations = anim
            .workshop
            .stations
            .values()
            .map(|s| {
                (
                    s.station_id.clone(),
                    StationPlacement {
                        id: s.station_id.clone(),
                        offset: IVec2::new(s.origin[0], s.origin[1]) - foot,
                        occlusion: s.occlusion_order.clone(),
                    },
                )
            })
            .collect();
        Ok(Self {
            id: id.into(),
            image: format!("characters/{id}/{}", page.path),
            page_size: page.size,
            canvas: anim.canvas,
            pivot: anim.pivot,
            sprites,
            clips,
            stations,
        })
    }
    pub fn layout(&self) -> TextureAtlasLayout {
        let mut a = TextureAtlasLayout::new_empty(UVec2::from_array(self.page_size));
        for s in &self.sprites {
            let [x, y, w, h] = s.rect;
            // Standalone files get a placeholder cell; the renderer swaps the image handle.
            a.add_texture(if s.file.is_some() {
                URect::new(0, 0, 1, 1)
            } else {
                URect::new(x, y, x + w, y + h)
            });
        }
        a
    }
    pub fn sprite_index(&self, id: &str) -> Option<usize> {
        self.sprites.iter().position(|s| s.id == id)
    }
}

/// Deterministic clip cursor. `hold` keeps a station clip inside its loop range; releasing it
/// finishes the pass, plays the outro and rests. Seeking re-samples from the clip start.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Actor {
    pub clip: String,
    pub elapsed: f32,
    pub paused: bool,
    pub speed: f32,
    pub facing_left: bool,
    pub hold: bool,
    pub released_at: Option<f32>,
    pub blink: bool,
    pub frame: usize,
    /// Index of the current frame inside the clip, for event detection.
    pub clip_frame: Option<usize>,
    pub finished: bool,
    /// Marker names crossed since the consumer last drained them.
    pub events: Vec<String>,
    pub rest: String,
    /// Clip-name prefix for a variant set ("" equipped, "base/" unequipped).
    pub prefix: String,
}
impl Default for Actor {
    fn default() -> Self {
        Self {
            clip: "idle".into(),
            elapsed: 0.,
            paused: false,
            speed: 1.,
            facing_left: false,
            hold: false,
            released_at: None,
            blink: true,
            frame: 0,
            clip_frame: None,
            finished: false,
            events: vec![],
            rest: "idle".into(),
            prefix: String::new(),
        }
    }
}
impl Actor {
    /// Resolve a clip name through the variant prefix, falling back to the plain name, then idle.
    pub fn resolve(&self, clip: &str, c: &PlayerCatalog) -> String {
        let prefixed = format!("{}{}", self.prefix, clip.trim_start_matches(&self.prefix));
        if c.clips.contains_key(&prefixed) {
            prefixed
        } else if c.clips.contains_key(clip) {
            clip.to_string()
        } else {
            self.resolve_or_idle(c)
        }
    }
    fn resolve_or_idle(&self, c: &PlayerCatalog) -> String {
        let prefixed = format!("{}idle", self.prefix);
        if c.clips.contains_key(&prefixed) {
            prefixed
        } else {
            "idle".into()
        }
    }
    pub fn play(&mut self, clip: &str, c: &PlayerCatalog) {
        let name = self.resolve(clip, c);
        let facing = c.clips[&name].facing;
        self.clip = name.clone();
        self.rest = self.resolve(facing.rest_clip(), c);
        self.elapsed = 0.;
        self.released_at = None;
        self.finished = false;
        self.clip_frame = None;
        self.frame = c.clips[&name].frames[0].sprite;
        self.sample(c);
    }
    /// Start a station clip and keep it looping until `release`.
    pub fn work(&mut self, clip: &str, c: &PlayerCatalog) {
        if self.clip != self.resolve(clip, c) || self.finished || !self.hold {
            self.hold = true;
            self.play(clip, c);
        }
    }
    pub fn release(&mut self) {
        if self.hold {
            self.hold = false;
            self.released_at = Some(self.elapsed);
        }
    }
    pub fn seek(&mut self, t: f32, c: &PlayerCatalog) {
        self.elapsed = if t.is_finite() { t.max(0.) } else { 0. };
        self.clip_frame = None;
        self.events.clear();
        self.finished = false;
        self.sample(c);
    }
    pub fn advance(&mut self, dt: f32, c: &PlayerCatalog) {
        if self.paused || !dt.is_finite() || dt <= 0. || !self.speed.is_finite() {
            return;
        }
        self.elapsed += dt * self.speed.max(0.);
        self.sample(c);
        if self.finished {
            let rest = self.rest.clone();
            let keep_hold = self.hold;
            self.hold = false;
            if keep_hold {
                // A held clip without a loop range repeats while the work continues.
                let clip = self.clip.clone();
                self.hold = true;
                self.play(&clip, c);
            } else {
                self.play(&rest, c);
            }
        }
    }
    /// Frame position for `elapsed`. Returns None when the clip has completed.
    fn position(&self, clip: &PlayerClip) -> Option<usize> {
        let durations: Vec<f32> = clip.frames.iter().map(|f| f.duration).collect();
        let at = |start: usize, end: usize, t: f32| -> usize {
            let mut t = t;
            for (i, d) in durations.iter().enumerate().take(end).skip(start) {
                if t < *d {
                    return i;
                }
                t -= d;
            }
            end.saturating_sub(1).max(start)
        };
        let n = clip.frames.len();
        let total = clip.total();
        let Some([a, b]) = clip
            .loop_range
            .filter(|_| self.hold || self.released_at.is_some())
        else {
            if clip.looping {
                return Some(at(0, n, self.elapsed % total));
            }
            return (self.elapsed < total).then(|| at(0, n, self.elapsed));
        };
        let intro: f32 = durations[..a].iter().sum();
        let loop_len: f32 = durations[a..=b].iter().sum();
        let outro: f32 = durations[b + 1..].iter().sum();
        if self.elapsed < intro {
            return Some(at(0, a, self.elapsed));
        }
        match self.released_at {
            None => Some(at(a, b + 1, (self.elapsed - intro) % loop_len)),
            Some(released) => {
                let released = released.max(intro);
                let pass = (released - intro) % loop_len;
                let remaining = loop_len - pass;
                let since = self.elapsed - released;
                if since < remaining {
                    Some(at(a, b + 1, pass + since))
                } else if since - remaining < outro {
                    Some(at(b + 1, n, since - remaining))
                } else {
                    None
                }
            }
        }
    }
    fn sample(&mut self, c: &PlayerCatalog) {
        let clip = c.clips.get(&self.clip).unwrap_or(&c.clips["idle"]);
        // Clip frames reference the catalog; nothing here is mutated except the cursor.
        match self.position(clip) {
            Some(i) => {
                if self.clip_frame != Some(i) {
                    for e in clip.events.iter().filter(|e| e.frame == i) {
                        self.events.push(e.name.clone());
                    }
                    self.clip_frame = Some(i);
                }
                self.frame = clip.frames[i].sprite;
            }
            None => {
                // Markers on skipped frames still register once when a large delta ends the clip.
                for e in &clip.events {
                    if self.clip_frame.is_none_or(|f| e.frame > f) {
                        self.events.push(e.name.clone());
                    }
                }
                self.finished = true;
                self.clip_frame = Some(clip.frames.len());
                self.frame = clip.frames.last().map_or(0, |f| f.sprite);
            }
        }
    }
    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.events)
    }
    /// Current native sprite.
    pub fn sprite<'a>(&self, c: &'a PlayerCatalog) -> &'a PlayerSprite {
        &c.sprites[self.frame.min(c.sprites.len() - 1)]
    }
    pub fn facing(&self, c: &PlayerCatalog) -> Facing {
        c.clips
            .get(&self.clip)
            .map_or(Facing::Right34, |k| k.facing)
    }
}

/// Which bundled character an actor entity renders.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct CharacterId(pub String);
/// Straight authored path between fixed standing points; no general pathfinding.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Walker {
    pub pos: Vec2,
    pub target: Vec2,
    /// Native pixels per second.
    pub speed: f32,
    /// Clip to play (held) once the target is reached.
    pub arrival: Option<String>,
    pub arrival_hold: bool,
}
impl Default for Walker {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            target: Vec2::ZERO,
            speed: 48.,
            arrival: None,
            arrival_hold: false,
        }
    }
}
impl Walker {
    pub fn arrived(&self) -> bool {
        self.pos.distance(self.target) < 0.5
    }
    /// Go to `target`; on arrival play `clip` (held when it is station work).
    pub fn go(&mut self, target: Vec2, clip: &str, hold: bool) {
        self.target = target;
        self.arrival = Some(clip.into());
        self.arrival_hold = hold;
    }
    /// Advance movement; returns true while walking.
    pub fn step(&mut self, dt: f32) -> bool {
        if self.arrived() {
            self.pos = self.target;
            return false;
        }
        let delta = self.target - self.pos;
        let travel = self.speed * dt.max(0.);
        if delta.length() <= travel {
            self.pos = self.target;
        } else {
            self.pos += delta.normalize() * travel;
        }
        true
    }
}
#[derive(Resource)]
pub struct PlayerCatalogs {
    pub catalogs: BTreeMap<String, PlayerCatalog>,
    pub layouts: BTreeMap<String, Handle<TextureAtlasLayout>>,
    pub images: BTreeMap<String, Handle<Image>>,
    pub files: BTreeMap<String, Handle<Image>>,
}
impl PlayerCatalogs {
    pub fn get(&self, id: &str) -> Option<&PlayerCatalog> {
        self.catalogs.get(id)
    }
}
fn load_catalogs(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut catalogs = BTreeMap::new();
    let mut layout_handles = BTreeMap::new();
    let mut images = BTreeMap::new();
    let mut files = BTreeMap::new();
    for id in CHARACTERS {
        let c = PlayerCatalog::bundled(id).expect("validated bundled player catalog");
        layout_handles.insert(id.to_string(), layouts.add(c.layout()));
        images.insert(id.to_string(), server.load(c.image.clone()));
        for s in &c.sprites {
            if let Some(path) = &s.file {
                files.insert(path.clone(), server.load(path.clone()));
            }
        }
        catalogs.insert(id.to_string(), c);
    }
    commands.insert_resource(PlayerCatalogs {
        catalogs,
        layouts: layout_handles,
        images,
        files,
    });
}
/// Shared Bevy playback for the player smith; the preview and the game add this plugin.
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::PlaybackClock>()
            .add_systems(PreStartup, load_catalogs)
            .add_systems(Update, (walk_actors, animate_actors).chain());
    }
}
pub fn walk_actors(
    time: Res<Time>,
    clock: Res<crate::PlaybackClock>,
    catalogs: Option<Res<PlayerCatalogs>>,
    mut q: Query<(&mut Actor, &mut Walker, &CharacterId)>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    let dt = clock.fixed_delta.unwrap_or_else(|| time.delta_secs());
    for (mut actor, mut walker, id) in &mut q {
        let Some(c) = catalogs.get(&id.0) else {
            continue;
        };
        if actor.paused {
            continue;
        }
        let dx = walker.target.x - walker.pos.x;
        if walker.step(dt) {
            if dx.abs() > 0.01 {
                actor.facing_left = dx < 0.;
            }
            if actor.clip != actor.resolve("walk", c) {
                actor.hold = false;
                actor.play("walk", c);
            }
        } else if let Some(clip) = walker.arrival.take() {
            if walker.arrival_hold {
                actor.work(&clip, c);
            } else {
                actor.hold = false;
                actor.play(&clip, c);
            }
            actor.facing_left = false;
        }
    }
}
#[allow(clippy::type_complexity)]
pub fn animate_actors(
    time: Res<Time>,
    clock: Res<crate::PlaybackClock>,
    server: Res<AssetServer>,
    catalogs: Option<Res<PlayerCatalogs>>,
    mut q: Query<(
        &mut Actor,
        &CharacterId,
        &mut Sprite,
        &mut bevy::sprite::Anchor,
    )>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    let dt = clock.fixed_delta.unwrap_or_else(|| time.delta_secs());
    for (mut actor, id, mut sprite, mut anchor) in &mut q {
        let Some(c) = catalogs.get(&id.0) else {
            sprite.color = Color::NONE;
            continue;
        };
        actor.advance(dt, c);
        // Timed blink while resting; never during work, walking or a gesture.
        let facing = actor.facing(c);
        if actor.blink
            && !actor.paused
            && actor.clip == actor.resolve(facing.rest_clip(), c)
            && actor.elapsed >= 3.4
            && let Some(blink) = facing.blink_clip()
        {
            actor.play(blink, c);
        }
        let frame = actor.sprite(c).clone();
        let flipped = actor.facing_left;
        if let Some(path) = &frame.file {
            if let Some(h) = catalogs.files.get(path)
                && sprite.image != *h
            {
                sprite.image = h.clone();
            }
            sprite.texture_atlas = None;
        } else {
            let image = &catalogs.images[&id.0];
            if sprite.image != *image {
                sprite.image = image.clone();
            }
            let index = actor.frame;
            match &mut sprite.texture_atlas {
                Some(a) => {
                    if a.index != index {
                        a.index = index;
                    }
                }
                None => {
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: catalogs.layouts[&id.0].clone(),
                        index,
                    })
                }
            }
        }
        let size = Some(frame.size());
        if sprite.custom_size != size {
            sprite.custom_size = size;
        }
        if sprite.flip_x != flipped {
            sprite.flip_x = flipped;
        }
        let next = frame.anchor(flipped);
        if anchor.0 != next {
            anchor.0 = next;
        }
        // Art failure hides the sprite; the owning adapter keeps tint and placement.
        let failed = matches!(
            server.get_load_state(sprite.image.id()),
            Some(bevy::asset::LoadState::Failed(_))
        );
        if failed && sprite.color != Color::NONE {
            sprite.color = Color::NONE;
        } else if !failed && sprite.color == Color::NONE {
            sprite.color = Color::WHITE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_characters_parse_with_seventeen_clips_and_five_stations() {
        for id in CHARACTERS {
            let c = PlayerCatalog::bundled(id).unwrap();
            for clip in PLAYER_CLIPS {
                assert!(c.clips.contains_key(clip), "{id} missing {clip}");
            }
            assert_eq!(c.stations.len(), 5);
            assert_eq!(c.canvas, [128, 160]);
            assert_eq!(c.pivot, [64, 140]);
            // Authored anvil contact: male station origin (123,104) against foot (96,156).
            if id == "male" {
                assert_eq!(c.stations["anvil"].offset, IVec2::new(27, -52));
            }
            assert_eq!(c.clips["hammering"].station.as_deref(), Some("anvil"));
            assert_eq!(c.clips["researching"].facing, Facing::Front);
            assert_eq!(c.clips["salvaging"].facing, Facing::Back);
            assert_eq!(c.clips["hammering"].loop_range, Some([1, 5]));
            assert!(
                c.clips["quenching"]
                    .frames
                    .iter()
                    .all(|f| c.sprites[f.sprite].file.is_some())
            );
            let neutral = &c.sprites[c.sprite_index("neutral").unwrap()];
            let a = neutral.anchor(false);
            // Foot pivot sits inside the trimmed rect and mirrors exactly.
            assert!(a.x.abs() <= 0.5 && a.y.abs() <= 0.5);
            assert_eq!(neutral.anchor(true), Vec2::new(-a.x, a.y));
        }
    }
    #[test]
    fn base_variant_resolves_through_the_prefix_and_falls_back() {
        let c = PlayerCatalog::bundled("female").unwrap();
        let mut a = Actor {
            prefix: "base/".into(),
            ..default()
        };
        a.work("hammering", &c);
        assert_eq!(a.clip, "base/hammering");
        a.release();
        a.advance(5., &c);
        assert_eq!(a.clip, "base/idle");
        a.play("quenching", &c);
        assert!(
            c.sprites[a.frame]
                .file
                .as_deref()
                .unwrap()
                .contains("scene-base")
        );
        a.play("researching", &c);
        a.advance(10., &c);
        assert_eq!(a.clip, "base/front_idle");
        a.prefix.clear();
        a.play("walk", &c);
        assert_eq!(a.clip, "walk");
    }
    #[test]
    fn invalid_manifests_are_rejected() {
        let atlas = include_str!("../../../assets/characters/male/atlas-manifest.json");
        let anim = include_str!("../../../assets/characters/male/animation.json");
        assert!(PlayerCatalog::parse("male", atlas, anim).is_ok());
        let broken = atlas.replacen("\"duration_ms\": 1000", "\"duration_ms\": 0", 1);
        assert!(PlayerCatalog::parse("male", &broken, anim).is_err());
        let mut value: serde_json::Value = serde_json::from_str(anim).unwrap();
        value["clips"]["hammering"]["loop_range"] = serde_json::json!([1, 9]);
        assert!(PlayerCatalog::parse("male", atlas, &value.to_string()).is_err());
        let mut value: serde_json::Value = serde_json::from_str(anim).unwrap();
        value["clips"]["give"]["events"][0]["frame"] = serde_json::json!(40);
        assert!(PlayerCatalog::parse("male", atlas, &value.to_string()).is_err());
        assert!(PlayerCatalog::parse("male", atlas, "{}").is_err());
    }
    #[test]
    fn held_station_clips_loop_then_finish_and_markers_fire_once_per_pass() {
        let c = PlayerCatalog::bundled("female").unwrap();
        let mut a = Actor::default();
        a.work("hammering", &c);
        // Intro frame 0 (140 ms), loop 1..=5 (620 ms).
        a.advance(0.1, &c);
        assert_eq!(a.clip_frame, Some(0));
        a.advance(0.1, &c);
        assert_eq!(a.clip_frame, Some(1));
        let mut events = 0;
        let mut b = a.clone();
        for _ in 0..200 {
            a.advance(0.01, &c);
            events += a.drain_events().len();
        }
        b.advance(2., &c);
        assert_eq!(a.clip_frame, b.clip_frame);
        // 2 s covers three loop passes; contact fires once per pass.
        assert_eq!(events, 3);
        assert_eq!(a.clip, "hammering");
        a.release();
        a.advance(5., &c);
        assert_eq!(a.clip, "idle");
        assert!(!a.hold);
        let mut f = Actor::default();
        f.work("forging", &c);
        f.advance(3., &c);
        assert!(matches!(f.clip_frame, Some(2 | 3)));
        f.release();
        f.advance(0.7, &c);
        assert!(matches!(f.clip_frame, Some(4 | 5)) || f.clip == "idle");
        f.advance(2., &c);
        assert_eq!(f.clip, "idle");
        let mut q = Actor::default();
        q.work("quenching", &c);
        q.advance(1.3, &c);
        // Without a loop range a held one-shot repeats; markers stay sound hooks only.
        assert_eq!(q.clip, "quenching");
        assert!(q.drain_events().iter().any(|e| e == "water_enter"));
    }
    #[test]
    fn front_and_back_clips_rest_in_their_own_baseline_and_seek_is_deterministic() {
        let c = PlayerCatalog::bundled("male").unwrap();
        let mut r = Actor::default();
        r.play("researching", &c);
        r.advance(10., &c);
        assert_eq!(r.clip, "front_idle");
        assert_eq!(r.facing(&c), Facing::Front);
        let mut s = Actor::default();
        s.play("salvaging", &c);
        s.advance(10., &c);
        assert_eq!(s.clip, "back_idle");
        let mut w = Actor::default();
        w.play("walk", &c);
        w.seek(0.25, &c);
        let frame = w.frame;
        w.seek(0.25, &c);
        assert_eq!(w.frame, frame);
        w.paused = true;
        w.advance(5., &c);
        assert_eq!(w.elapsed, 0.25);
        let mut g = Actor::default();
        g.play("give", &c);
        g.advance(100., &c);
        assert!(g.drain_events().contains(&"transfer".to_string()));
        g.play("missing", &c);
        assert_eq!(g.clip, "idle");
    }
    #[test]
    fn walker_moves_straight_and_hands_over_to_the_station_clip() {
        let c = PlayerCatalog::bundled("male").unwrap();
        let mut w = Walker {
            pos: Vec2::new(0., 156.),
            ..default()
        };
        w.go(Vec2::new(96., 156.), "hammering", true);
        let mut a = Actor::default();
        let mut walked = 0;
        for _ in 0..200 {
            if w.step(0.05) {
                walked += 1;
                if a.clip != "walk" {
                    a.play("walk", &c);
                }
            } else if let Some(clip) = w.arrival.take() {
                a.work(&clip, &c);
                break;
            }
        }
        assert_eq!(walked, 40);
        assert_eq!(a.clip, "hammering");
        assert!(a.hold);
        assert_eq!(w.pos, Vec2::new(96., 156.));
    }
}
