//! Shared production widgets. Preview and desktop use these exact builders.
use bevy::input_focus::{
    InputFocus,
    tab_navigation::{TabIndex, TabNavigationPlugin},
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, TextureSlicer};
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
#[derive(Component)]
pub struct SkinPanel;
pub const INK: Color = Color::srgb(0.94, 0.88, 0.74);
pub const SURFACE: Color = Color::srgb(0.105, 0.083, 0.068);
pub const EDGE: Color = Color::srgb(0.38, 0.25, 0.15);
pub const BLUE: Color = Color::srgb(0.14, 0.28, 0.36);
#[derive(Component)]
pub struct SkinButton;
/// Which exported control art a `SkinButton` uses; absent means a plain button.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SkinKind {
    Tab,
    Slot,
}
/// Slot with nothing in it; draws the exported locked-slot art.
#[derive(Component)]
pub struct Locked;
#[derive(Component)]
pub struct FocusKey(pub String);
#[derive(Resource, Default)]
struct FocusMemory(Option<String>);
fn remember_focus(focus: Res<InputFocus>, q: Query<&FocusKey>, mut memory: ResMut<FocusMemory>) {
    if let Some(key) = focus.get().and_then(|e| q.get(e).ok()) {
        memory.0 = Some(key.0.clone());
    }
}
fn restore_rebuilt_focus(
    mut focus: ResMut<InputFocus>,
    memory: Res<FocusMemory>,
    entities: Query<Entity>,
    q: Query<(Entity, &FocusKey, &InheritedVisibility), Without<bevy::ui::InteractionDisabled>>,
) {
    // Preserve semantic focus when a state-driven list replaces its entities.
    if focus.get().is_some_and(|e| !entities.contains(e))
        && let Some(key) = &memory.0
        && let Some((entity, _, _)) = q
            .iter()
            .find(|(_, candidate, visible)| visible.get() && &candidate.0 == key)
    {
        focus.set(entity, bevy::input_focus::FocusCause::Auto);
    }
}
#[derive(Component)]
pub struct Selected;
#[derive(Component)]
pub struct Unavailable(pub String);
#[derive(Component)]
pub struct Help(pub String);
#[derive(Component)]
pub struct UiFont;
pub struct ForgeUiPlugin;
impl Plugin for ForgeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .init_resource::<MerchantReaction>()
            .init_resource::<DialogState>()
            .init_resource::<FocusMemory>()
            .add_systems(PreUpdate, remember_focus)
            .add_systems(PostUpdate, restore_rebuilt_focus)
            .add_observer(answer)
            .add_systems(PostUpdate, (dialog_keys, dialog_draw).chain())
            .add_systems(Startup, (tooltip_setup, load_font))
            .add_systems(
                Update,
                (
                    font,
                    interaction,
                    skin,
                    decorate_buttons,
                    windows,
                    frames,
                    decorate_slots,
                    animate_merchants,
                    tooltip,
                ),
            );
    }
}
#[derive(Resource)]
struct ThemeFont(Handle<Font>);
fn load_font(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ThemeFont(server.load("ui/fonts/NotoSansKR.ttf")));
}
fn font(
    server: Res<AssetServer>,
    theme: Res<ThemeFont>,
    mut texts: Query<&mut TextFont, With<UiFont>>,
) {
    let handle = &theme.0;
    if server.is_loaded_with_dependencies(handle.id()) {
        for mut font in &mut texts {
            if font.font != handle.clone().into() {
                font.font = handle.clone().into();
            }
        }
    }
}
#[allow(clippy::type_complexity)]
fn interaction(
    focus: Res<InputFocus>,
    mut q: Query<
        (
            Entity,
            &Hovered,
            Has<Pressed>,
            Option<&Selected>,
            Option<&Unavailable>,
            Has<ImageNode>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<SkinButton>,
    >,
) {
    for (entity, state, pressed, selected, disabled, skinned, mut bg, mut border) in &mut q {
        // Exported art carries every state itself; the node colour only backs unskinned
        // (missing art) controls, so rounded art corners never sit on a square box.
        let next_bg = if skinned {
            Color::NONE
        } else if disabled.is_some() {
            Color::srgb(0.14, 0.13, 0.12)
        } else if pressed {
            Color::srgb(0.10, 0.20, 0.27)
        } else if state.get() || selected.is_some() {
            BLUE
        } else {
            Color::srgb(0.23, 0.17, 0.12)
        };
        if bg.0 != next_bg {
            bg.0 = next_bg;
        }
        // Border doubles as the focus/hover ring and is invisible at rest on skinned art.
        let ring = state.get() || focus.get() == Some(entity);
        let next_border = BorderColor::all(if ring {
            INK
        } else if skinned {
            Color::NONE
        } else {
            EDGE
        });
        if *border != next_border {
            *border = next_border;
        }
    }
}
pub fn text(s: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font_size: size.into(),
            ..default()
        },
        TextColor(INK),
        UiFont,
    )
}
pub fn panel(width: Val) -> impl Bundle {
    (
        SkinPanel,
        Node {
            width,
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            min_width: px(0),
            ..default()
        },
        BackgroundColor(SURFACE),
        BorderColor::all(EDGE),
    )
}
pub fn button(
    parent: &mut ChildSpawnerCommands,
    label: impl Into<String>,
    action: impl Bundle,
) -> Entity {
    let label = label.into();
    parent
        .spawn((
            Button,
            SkinButton,
            FocusKey(label.clone()),
            TabIndex(0),
            Hovered::default(),
            Node {
                padding: UiRect::axes(px(12), px(7)),
                border: UiRect::all(px(1)),
                min_height: px(32),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(SURFACE),
            BorderColor::all(EDGE),
            action,
        ))
        .with_children(|b| {
            b.spawn(text(label, 14.));
        })
        .id()
}
/// Source pixels are addressed explicitly; no destructive edits of art masters.
pub fn image(
    server: &AssetServer,
    path: &str,
    rect: Option<Rect>,
    width: f32,
    height: f32,
) -> impl Bundle {
    let mut image = ImageNode::new(server.load(path.to_owned()));
    image.rect = rect;
    (
        image,
        Node {
            width: px(width),
            height: px(height),
            flex_shrink: 0.,
            ..default()
        },
    )
}
pub const ICONS: [&str; 24] = [
    "craft",
    "materials",
    "provisions",
    "map",
    "commissions",
    "research",
    "codex",
    "settings",
    "close",
    "back",
    "plus",
    "minus",
    "coin",
    "lock",
    "warning",
    "ore",
    "dagger",
    "timber",
    "leather",
    "staples",
    "meal",
    "water",
    "rest",
    "queue",
];
pub fn icon(parent: &mut ChildSpawnerCommands, server: &AssetServer, name: &str, size: f32) {
    let name = if ICONS.contains(&name) {
        name
    } else {
        "warning"
    };
    parent.spawn(image(
        server,
        &format!("ui/runtime/icons/{name}.png"),
        None,
        size,
        size,
    ));
}
pub fn item_card(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    name: &str,
    label: &str,
    count: impl std::fmt::Display,
) {
    parent
        .spawn((
            Node {
                width: px(104),
                min_height: px(106),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(3),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.12, 0.09)),
            BorderColor::all(EDGE),
        ))
        .with_children(|p| {
            icon(p, server, name, 40.);
            p.spawn(text(label, 12.));
            p.spawn(text(count.to_string(), 13.));
        });
}
/// Compact selectable inventory slot. Contents and counts are live UI nodes.
pub fn slot(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    name: &str,
    count: u32,
    selected: bool,
    action: impl Bundle,
) {
    let mut slot = parent.spawn((
        Button,
        SkinButton,
        SkinKind::Slot,
        TabIndex(0),
        Hovered::default(),
        action,
        Node {
            width: px(64),
            height: px(64),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(SURFACE),
        BorderColor::all(EDGE),
    ));
    if selected {
        slot.insert(Selected);
    }
    if count == 0 {
        slot.insert(Locked);
    }
    slot.with_children(|p| {
        if count > 0 {
            icon(p, server, name, 32.);
        }
        p.spawn(text(
            if count == 0 {
                "Empty".into()
            } else {
                count.to_string()
            },
            11.,
        ));
    });
}
/// Fill colour of an exported progress bar.
#[derive(Clone, Copy)]
pub enum Fill {
    Amber,
    Blue,
}
fn capped(server: &AssetServer, path: &'static str) -> ImageNode {
    let mut image = ImageNode::new(server.load(path));
    image.image_mode = NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(12.),
        max_corner_scale: 1.,
        ..default()
    });
    image
}
/// Exported track with a clip-driven fill. Returns the fill entity so callers can retarget it.
pub fn progress_bar(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    fraction: f32,
    fill: Fill,
) -> Entity {
    let fill_path = match fill {
        Fill::Amber => "ui/runtime/controls/progress-fill-amber.png",
        Fill::Blue => "ui/runtime/controls/progress-fill-blue.png",
    };
    let mut fill_id = Entity::PLACEHOLDER;
    parent
        .spawn((
            capped(server, "ui/runtime/controls/progress-track.png"),
            Node {
                width: percent(100),
                height: px(16),
                ..default()
            },
        ))
        .with_children(|p| {
            fill_id = p
                .spawn((
                    capped(server, fill_path),
                    Node {
                        width: percent(fraction.clamp(0., 1.) * 100.),
                        height: percent(100),
                        ..default()
                    },
                ))
                .id();
        });
    fill_id
}
pub fn progress(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    fraction: f32,
    label: impl Into<String>,
) {
    parent.spawn(text(label, 13.));
    progress_bar(parent, server, fraction, Fill::Amber);
}
/// Placement of a shopkeeper in the 1250x200 header art, in header pixels.
struct Stage {
    /// Sprite scale relative to the 128x160 NPC canvas.
    scale: f32,
    /// Distance from the header's right edge to the sprite canvas' right edge.
    right: f32,
    /// Distance from the header's top to the sprite canvas' top.
    top: f32,
}
fn stage(header: &str) -> Stage {
    match header {
        // The farmer is in front of the stall: 2x (the header art grid) with the waist near the
        // counter top, legs running off the bottom edge so no feet float above the ground.
        "farm" => Stage {
            scale: 2.,
            right: 6.,
            top: -16.,
        },
        _ => Stage {
            scale: 1.,
            right: 50.,
            top: 40.,
        },
    }
}
/// Shared shop stage, with separate original NPC and title layers.
pub fn merchant_header(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    id: &str,
    title: &str,
    line: &str,
) {
    let header = match id {
        "carpenter" => "carpenter",
        "leatherworker" => "leatherworker",
        "farmer" => "farm",
        _ => "tavern",
    };
    let stage = stage(header);
    parent
        .spawn((
            Node {
                width: percent(100),
                // Header art is 1250x200; matching height keeps it undistorted.
                height: px(200),
                min_height: px(200),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(SURFACE),
        ))
        .with_children(|p| {
            let mut bg = ImageNode::new(server.load(format!("ui/runtime/headers/{header}.png")));
            bg.image_mode = NodeImageMode::Stretch;
            // Uniform scale, anchored right (NPC side); narrow windows crop the title side.
            p.spawn((
                bg,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    top: px(0),
                    height: percent(100),
                    aspect_ratio: Some(1250. / 200.),
                    ..default()
                },
            ));
            p.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(20),
                top: px(18),
                width: percent(58),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            })
            .with_children(|t| {
                t.spawn(text(title, 25.));
                t.spawn(text(line, 15.));
            });
            p.spawn((
                MerchantActor(id.to_owned()),
                ImageNode::new(server.load(format!("npc/{id}/frames/neutral.png"))),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(stage.right),
                    top: px(stage.top),
                    width: px(128. * stage.scale),
                    height: px(160. * stage.scale),
                    ..default()
                },
            ));
        });
}

fn skin(mut commands: Commands, server: Res<AssetServer>, panels: Query<Entity, Added<SkinPanel>>) {
    for entity in &panels {
        let mut image = ImageNode::new(server.load("ui/runtime/controls/inset.png"));
        image.visual_box = bevy::ui::VisualBox::BorderBox;
        image.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(20.),
            max_corner_scale: 1.,
            ..default()
        });
        commands.entity(entity).insert(image);
    }
}

#[derive(Component)]
pub struct WindowSkin;
/// Any registered nine-slice frame (e.g. the showroom window and detail panel in
/// `assets/forge/spaces`). Insets are source pixels; `corner_scale` caps corner growth.
#[derive(Component, Clone)]
pub struct SkinFrame {
    pub path: String,
    pub inset: f32,
    pub corner_scale: f32,
}
fn frames(
    mut c: Commands,
    server: Res<AssetServer>,
    q: Query<(Entity, &SkinFrame), Added<SkinFrame>>,
) {
    for (e, frame) in &q {
        let mut image = ImageNode::new(server.load(frame.path.clone()));
        image.visual_box = bevy::ui::VisualBox::BorderBox;
        image.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(frame.inset),
            max_corner_scale: frame.corner_scale,
            ..default()
        });
        c.entity(e).insert(image);
    }
}
/// Inventory slot skin (slot-normal / slot-selected / slot-locked runtime exports).
#[derive(Component)]
pub struct SkinSlot;
#[allow(clippy::type_complexity)]
fn decorate_slots(
    server: Res<AssetServer>,
    mut q: Query<
        (
            Entity,
            Has<Selected>,
            Has<bevy::ui::InteractionDisabled>,
            Option<&mut ImageNode>,
        ),
        With<SkinSlot>,
    >,
    mut c: Commands,
) {
    for (e, selected, disabled, image) in &mut q {
        let path = if disabled {
            "ui/runtime/controls/slot-locked.png"
        } else if selected {
            "ui/runtime/controls/slot-selected.png"
        } else {
            "ui/runtime/controls/slot-normal.png"
        };
        let handle: Handle<Image> = server.load(path);
        if let Some(mut image) = image {
            if image.image != handle {
                image.image = handle;
            }
        } else {
            let mut image = ImageNode::new(handle);
            image.visual_box = bevy::ui::VisualBox::BorderBox;
            image.image_mode = NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(24.),
                max_corner_scale: 0.5,
                ..default()
            });
            c.entity(e).insert(image);
        }
    }
}
/// Slot state badge colours shared by the showroom, crafting and later codex views.
pub const BADGE_SALE: Color = Color::srgb(0.16, 0.42, 0.70);
pub const BADGE_HOLD: Color = Color::srgb(0.18, 0.16, 0.14);
pub const BADGE_RESERVED: Color = Color::srgb(0.80, 0.52, 0.12);
/// Larger inventory slot with an optional state badge. One icon per item is reused everywhere;
/// weapons of any shape share this component instead of bespoke display art.
pub fn item_slot(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    icon_name: Option<&str>,
    badge: Option<(&str, Color)>,
    selected: bool,
    action: impl Bundle,
) -> Entity {
    let mut slot = parent.spawn((
        Button,
        SkinSlot,
        TabIndex(0),
        Hovered::default(),
        action,
        Node {
            width: px(74),
            height: px(78),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(px(4)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
    ));
    if selected {
        slot.insert(Selected);
    }
    slot.with_children(|p| {
        match icon_name {
            Some(name) => icon(p, server, name, 40.),
            None => {
                p.spawn((
                    Text::new("+"),
                    TextFont {
                        font_size: 22.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.40, 0.34)),
                    UiFont,
                    Node {
                        height: px(40),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));
            }
        }
        if let Some((label, color)) = badge {
            p.spawn((
                Node {
                    width: percent(100),
                    padding: UiRect::axes(px(4), px(1)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(color),
            ))
            .with_children(|b| {
                b.spawn(text(label, 10.));
            });
        } else {
            p.spawn(Node {
                height: px(14),
                ..default()
            });
        }
    });
    slot.id()
}
#[derive(Component)]
pub struct MerchantActor(pub String);
#[derive(Resource, Default)]
pub struct MerchantReaction {
    pub actor: String,
    pub seconds: f32,
}
#[derive(Component)]
pub struct Tip;
#[allow(clippy::type_complexity)]
fn decorate_buttons(
    server: Res<AssetServer>,
    mut q: Query<
        (
            Entity,
            &Hovered,
            Has<Pressed>,
            Has<Selected>,
            Has<bevy::ui::InteractionDisabled>,
            Has<Locked>,
            Option<&SkinKind>,
            Option<&mut ImageNode>,
        ),
        With<SkinButton>,
    >,
    mut c: Commands,
) {
    for (e, hover, pressed, selected, disabled, locked, kind, image) in &mut q {
        let path = match kind {
            Some(SkinKind::Tab) if selected => "ui/runtime/controls/tab-selected.png",
            Some(SkinKind::Tab) => "ui/runtime/controls/tab-normal.png",
            Some(SkinKind::Slot) if selected => "ui/runtime/controls/slot-selected.png",
            Some(SkinKind::Slot) if locked || disabled => "ui/runtime/controls/slot-locked.png",
            Some(SkinKind::Slot) => "ui/runtime/controls/slot-normal.png",
            None if disabled => "ui/runtime/controls/button-disabled.png",
            None if selected => "ui/runtime/controls/button-selected.png",
            None if pressed => "ui/runtime/controls/button-pressed.png",
            None if hover.get() => "ui/runtime/controls/button-hover.png",
            None => "ui/runtime/controls/button-normal.png",
        };
        let handle: Handle<Image> = server.load(path);
        if let Some(mut image) = image {
            if image.image != handle {
                image.image = handle;
            }
        } else {
            let (border, scale) = match kind {
                Some(SkinKind::Tab) => (22., 0.5),
                Some(SkinKind::Slot) => (24., 0.4),
                None => (20., 0.5),
            };
            let mut image = ImageNode::new(handle);
            image.visual_box = bevy::ui::VisualBox::BorderBox;
            image.image_mode = NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(border),
                max_corner_scale: scale,
                ..default()
            });
            c.entity(e).insert(image);
        }
    }
}
fn windows(mut c: Commands, server: Res<AssetServer>, q: Query<Entity, Added<WindowSkin>>) {
    for e in &q {
        let mut image = ImageNode::new(server.load("ui/runtime/controls/window.png"));
        image.visual_box = bevy::ui::VisualBox::BorderBox;
        image.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: Vec2::new(52., 88.),
                max_inset: Vec2::new(56., 40.),
            },
            max_corner_scale: 0.35,
            ..default()
        });
        c.entity(e).insert(image);
    }
}
fn animate_merchants(
    time: Res<Time>,
    server: Res<AssetServer>,
    mut reaction: ResMut<MerchantReaction>,
    mut q: Query<(&MerchantActor, &mut ImageNode)>,
    mut frames: Local<std::collections::HashMap<String, Handle<Image>>>,
) {
    reaction.seconds = (reaction.seconds - time.delta_secs()).max(0.);
    for (actor, mut image) in &mut q {
        let frame = if reaction.seconds > 0. && reaction.actor == actor.0 {
            "happy"
        } else if time.elapsed_secs() % 5. < 0.12 {
            "blink"
        } else {
            "neutral"
        };
        let path = format!("npc/{}/frames/{frame}.png", actor.0);
        let handle = frames
            .entry(path.clone())
            .or_insert_with(|| server.load(path))
            .clone();
        if server.is_loaded_with_dependencies(handle.id()) && image.image != handle {
            image.image = handle;
        }
    }
}
fn tooltip_setup(mut c: Commands) {
    c.spawn((
        Tip,
        text("", 13.),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            left: px(24),
            max_width: px(620),
            padding: UiRect::all(px(8)),
            ..default()
        },
        BackgroundColor(SURFACE),
        GlobalZIndex(200),
        Visibility::Hidden,
    ));
}
#[allow(clippy::type_complexity)]
fn tooltip(
    focus: Res<InputFocus>,
    q: Query<(
        Entity,
        &Hovered,
        Option<&Help>,
        Option<&Unavailable>,
        &InheritedVisibility,
    )>,
    mut tip: Query<(&mut Text, &mut Visibility), With<Tip>>,
) {
    let value = q.iter().find_map(|(e, h, help, disabled, visible)| {
        if visible.get() && (h.get() || focus.get() == Some(e)) {
            disabled
                .map(|d| d.0.clone())
                .or_else(|| help.map(|h| h.0.clone()))
        } else {
            None
        }
    });
    for (mut t, mut v) in &mut tip {
        let next = if value.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *v != next {
            *v = next;
        }
        if let Some(s) = &value
            && t.0 != *s
        {
            t.0.clone_from(s);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Confirmed(String),
    Cancelled(String),
}
#[derive(Resource, Default)]
pub struct DialogState {
    request: Option<(String, String, String)>,
    pub result: Option<Decision>,
    previous_focus: Option<Entity>,
}
impl DialogState {
    pub fn open(&mut self, id: &str, title: &str, message: &str, focus: Option<Entity>) {
        if self.request.is_none() {
            self.request = Some((id.into(), title.into(), message.into()));
            self.result = None;
            self.previous_focus = focus;
        }
    }
    pub fn is_open(&self) -> bool {
        self.request.is_some()
    }
    fn finish(&mut self, confirm: bool) {
        if let Some((id, _, _)) = self.request.take() {
            self.result = Some(if confirm {
                Decision::Confirmed(id)
            } else {
                Decision::Cancelled(id)
            });
        }
    }
}
#[derive(Component)]
struct DialogRoot;
#[derive(Component)]
struct Answer(bool);
fn dialog_draw(
    mut c: Commands,
    state: Res<DialogState>,
    roots: Query<Entity, With<DialogRoot>>,
    mut focus: ResMut<InputFocus>,
) {
    if !state.is_changed() {
        return;
    }
    for e in &roots {
        c.entity(e).despawn();
    }
    let Some((_, title, message)) = &state.request else {
        return;
    };
    c.spawn((
        DialogRoot,
        bevy::input_focus::tab_navigation::TabGroup::modal(),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
        GlobalZIndex(300),
    ))
    .with_children(|p| {
        p.spawn(panel(px(440))).with_children(|p| {
            p.spawn(text(title, 23.));
            p.spawn(text(message, 16.));
            button(p, "Confirm", Answer(true));
            let cancel = button(p, "Cancel [Esc]", Answer(false));
            focus.set(cancel, bevy::input_focus::FocusCause::Auto);
        });
    });
}
fn answer(
    e: On<bevy::ui_widgets::Activate>,
    q: Query<&Answer>,
    mut state: ResMut<DialogState>,
    mut focus: ResMut<InputFocus>,
    entities: Query<Entity>,
) {
    if let Ok(answer) = q.get(e.entity) {
        state.finish(answer.0);
        restore_focus(&state, &mut focus, &entities);
    }
}
fn restore_focus(state: &DialogState, focus: &mut InputFocus, entities: &Query<Entity>) {
    if let Some(e) = state.previous_focus.filter(|e| entities.contains(*e)) {
        focus.set(e, bevy::input_focus::FocusCause::Auto);
    } else {
        focus.clear();
    }
}
fn dialog_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DialogState>,
    mut focus: ResMut<InputFocus>,
    entities: Query<Entity>,
) {
    if state.is_open() && keys.just_pressed(KeyCode::Escape) {
        state.finish(false);
        restore_focus(&state, &mut focus, &entities);
    }
}

pub fn item_button(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    icon_name: &str,
    label: &str,
    price: u64,
    selected: bool,
    action: impl Bundle,
) {
    let mut row = parent.spawn((
        Button,
        SkinButton,
        FocusKey(format!("item:{icon_name}:{label}")),
        TabIndex(0),
        Hovered::default(),
        Node {
            padding: UiRect::all(px(10)),
            column_gap: px(10),
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(SURFACE),
        BorderColor::all(EDGE),
        action,
    ));
    if selected {
        row.insert(Selected);
    }
    row.with_children(|p| {
        icon(p, server, icon_name, 40.);
        p.spawn(text(format!("{label}\n{price} gold"), 14.));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rebuilding_list_restores_visible_enabled_semantic_focus() {
        let mut app = App::new();
        app.init_resource::<InputFocus>()
            .init_resource::<FocusMemory>();
        let old = app
            .world_mut()
            .spawn((FocusKey("buy".into()), InheritedVisibility::VISIBLE))
            .id();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(old, bevy::input_focus::FocusCause::Auto);
        app.add_systems(Update, remember_focus);
        app.update();
        app.world_mut().despawn(old);
        let disabled = app
            .world_mut()
            .spawn((
                FocusKey("buy".into()),
                InheritedVisibility::VISIBLE,
                bevy::ui::InteractionDisabled,
            ))
            .id();
        let new = app
            .world_mut()
            .spawn((FocusKey("buy".into()), InheritedVisibility::VISIBLE))
            .id();
        app.add_systems(PostUpdate, restore_rebuilt_focus);
        app.update();
        assert_ne!(app.world().resource::<InputFocus>().get(), Some(disabled));
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(new));
    }
    #[test]
    fn modal_does_not_replace_pending_action_and_cancel_cannot_confirm() {
        let mut s = DialogState::default();
        s.open("delete", "Delete?", "", None);
        s.open("quit", "Quit?", "", None);
        s.finish(false);
        assert_eq!(s.result, Some(Decision::Cancelled("delete".into())));
        s.finish(true);
        assert_eq!(s.result, Some(Decision::Cancelled("delete".into())));
        assert!(!s.is_open());
    }
}
