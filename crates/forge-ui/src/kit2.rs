//! Kit v2 (timber / parchment / gold) widgets for the management screens (#118).
//!
//! Art is `assets/ui/kit-v2/*.png`, drawn by `scripts/draw-ui-kit-v2.py`. One art pixel is one
//! logical pixel, so nine-slice corners never scale (`max_corner_scale = 1`). Text is drawn by
//! Bevy on top of the art and never baked into it.
use crate::{FocusKey, Selected, UiFont};
use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, TextureSlicer};
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;

pub const DIR: &str = "ui/kit-v2";

/// Slice insets (left, top, right, bottom) in art pixels. Mirrors `assets/ui/kit-v2/manifest.json`;
/// a test keeps the two in step.
pub const SLICES: &[(&str, [f32; 4])] = &[
    ("window-frame", [16., 16., 16., 16.]),
    ("panel-parchment", [12., 12., 12., 12.]),
    ("card", [4., 4., 4., 4.]),
    ("card-selected", [6., 6., 6., 6.]),
    ("tab-normal", [8., 8., 8., 8.]),
    ("tab-hover", [8., 8., 8., 8.]),
    ("tab-selected", [8., 8., 8., 8.]),
    ("button-normal", [6., 6., 6., 6.]),
    ("button-hover", [6., 6., 6., 6.]),
    ("button-pressed", [6., 6., 6., 6.]),
    ("button-disabled", [6., 6., 6., 6.]),
    ("button-primary-normal", [6., 6., 6., 6.]),
    ("button-primary-hover", [6., 6., 6., 6.]),
    ("button-primary-pressed", [6., 6., 6., 6.]),
    ("slot-normal", [5., 5., 5., 5.]),
    ("slot-selected", [5., 5., 5., 5.]),
    ("slot-locked", [5., 5., 5., 5.]),
    ("tag-daily", [4., 4., 4., 4.]),
    ("tag-special", [4., 4., 4., 4.]),
    ("tag-rank", [4., 4., 4., 4.]),
    ("tag-gold", [4., 4., 4., 4.]),
    ("progress-track", [3., 3., 3., 3.]),
    ("progress-fill", [3., 3., 3., 3.]),
    ("scrollbar-track", [3., 3., 3., 3.]),
    ("scrollbar-thumb", [3., 3., 3., 3.]),
    ("divider", [6., 0., 6., 0.]),
];

/// Text colours from the kit manifest (`text_colors`).
pub const INK: Color = Color::srgb_u8(0x5a, 0x3a, 0x20);
pub const INK_MUTED: Color = Color::srgb_u8(0x7a, 0x5e, 0x40);
pub const INK_WOOD: Color = Color::srgb_u8(0xec, 0xd1, 0xa6);
pub const INK_WOOD_MUTED: Color = Color::srgb_u8(0xb8, 0x9a, 0x74);
pub const DEADLINE: Color = Color::srgb_u8(0x8a, 0x3a, 0x28);
pub const GOLD: Color = Color::srgb_u8(0xf2, 0xc3, 0x7f);
/// Backing behind the kit while the frame art loads (dark timber).
pub const BACKING: Color = Color::srgb_u8(0x2e, 0x1d, 0x14);

fn insets(name: &str) -> [f32; 4] {
    SLICES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, i)| *i)
        .unwrap_or_else(|| panic!("kit-v2 has no slice named {name}"))
}

/// Nine-slice image for a kit component; corners stay at 1:1.
pub fn sliced(server: &AssetServer, name: &str) -> ImageNode {
    let [l, t, r, b] = insets(name);
    let mut image = ImageNode::new(server.load(format!("{DIR}/{name}.png")));
    image.visual_box = bevy::ui::VisualBox::BorderBox;
    image.image_mode = NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(l, t),
            max_inset: Vec2::new(r, b),
        },
        max_corner_scale: 1.,
        ..default()
    });
    image
}

/// A framed node: `frame(server, "panel-parchment", node)`.
pub fn frame(server: &AssetServer, name: &str, node: Node) -> impl Bundle {
    (sliced(server, name), node)
}

/// Label in the shared UI font.
pub fn label(s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font_size: size.into(),
            ..crate::theme_font()
        },
        TextColor(color),
        UiFont,
    )
}

/// Bold label (titles, tab names); Noto Sans KR is a variable font.
pub fn strong(s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font_size: size.into(),
            ..crate::theme_font()
        }
        .with_font_weight(bevy::text::FontWeight::BOLD),
        TextColor(color),
        UiFont,
    )
}

/// Small inline glyph at its native size (coin, hourglass, star, sparkle).
pub fn glyph(server: &AssetServer, name: &str, w: f32, h: f32) -> impl Bundle {
    (
        ImageNode::new(server.load(format!("{DIR}/{name}.png"))),
        Node {
            width: px(w),
            height: px(h),
            flex_shrink: 0.,
            ..default()
        },
    )
}

/// Which art a kit control draws for its states.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Plain,
    Primary,
    Tab,
    Card,
}
impl Kind {
    pub fn art(self, selected: bool, hovered: bool, pressed: bool, disabled: bool) -> &'static str {
        match self {
            Kind::Plain if disabled => "button-disabled",
            Kind::Plain if pressed => "button-pressed",
            Kind::Plain if hovered => "button-hover",
            Kind::Plain => "button-normal",
            Kind::Primary if disabled => "button-disabled",
            Kind::Primary if pressed => "button-primary-pressed",
            Kind::Primary if hovered => "button-primary-hover",
            Kind::Primary => "button-primary-normal",
            Kind::Tab if selected => "tab-selected",
            Kind::Tab if hovered => "tab-hover",
            Kind::Tab => "tab-normal",
            Kind::Card if selected => "card-selected",
            Kind::Card => "card",
        }
    }
}

/// An activatable kit control. Children (labels, icons) are added by the caller.
pub fn control(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    kind: Kind,
    selected: bool,
    focus_key: impl Into<String>,
    node: Node,
    action: impl Bundle,
) -> Entity {
    let mut e = parent.spawn((
        Button,
        kind,
        FocusKey(focus_key.into()),
        TabIndex(0),
        Hovered::default(),
        sliced(server, kind.art(selected, false, false, false)),
        Outline::new(px(1), px(1), Color::NONE),
        node,
        action,
    ));
    if selected {
        e.insert(Selected);
    }
    e.id()
}

/// Plain or primary button with a centred label.
pub fn button(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    kind: Kind,
    text: impl Into<String>,
    width: f32,
    action: impl Bundle,
) -> Entity {
    let text = text.into();
    let color = if kind == Kind::Primary {
        GOLD
    } else {
        INK_WOOD
    };
    let e = control(
        parent,
        server,
        kind,
        false,
        text.clone(),
        Node {
            width: px(width),
            height: px(28),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.,
            ..default()
        },
        action,
    );
    parent.commands().entity(e).with_children(|b| {
        b.spawn(strong(text, 13., color));
    });
    e
}

/// Swaps state art and shows a gold focus ring; runs every frame but only writes on change.
#[allow(clippy::type_complexity)]
pub fn decorate(
    server: Res<AssetServer>,
    focus: Res<InputFocus>,
    mut q: Query<(
        Entity,
        &Kind,
        &Hovered,
        Has<Pressed>,
        Has<Selected>,
        Has<InteractionDisabled>,
        &mut ImageNode,
        &mut Outline,
    )>,
) {
    for (e, kind, hovered, pressed, selected, disabled, mut image, mut outline) in &mut q {
        let name = kind.art(selected, hovered.get(), pressed, disabled);
        let handle: Handle<Image> = server.load(format!("{DIR}/{name}.png"));
        if image.image != handle {
            *image = sliced(&server, name);
        }
        let ring = if focus.get() == Some(e) {
            GOLD
        } else {
            Color::NONE
        };
        if outline.color != ring {
            outline.color = ring;
        }
    }
}

/// Recessed item slot holding a 1x icon (or nothing). `rect` addresses part of a larger sheet.
pub fn slot(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    state: &str,
    size: f32,
    icon: Option<(&str, Option<Rect>)>,
    scale: f32,
) -> Entity {
    parent
        .spawn(frame(
            server,
            &format!("slot-{state}"),
            Node {
                width: px(size),
                height: px(size),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|s| {
            if let Some((path, rect)) = icon {
                let (w, h) = rect.map_or((32., 32.), |r| (r.width(), r.height()));
                let mut image = ImageNode::new(server.load(path.to_owned()));
                image.rect = rect;
                s.spawn((
                    image,
                    Node {
                        width: px(w * scale),
                        height: px(h * scale),
                        ..default()
                    },
                ));
            }
        })
        .id()
}

/// Pill tag behind a short label.
pub fn tag(
    parent: &mut ChildSpawnerCommands,
    server: &AssetServer,
    kind: &str,
    text: impl Into<String>,
) {
    parent
        .spawn(frame(
            server,
            &format!("tag-{kind}"),
            Node {
                padding: UiRect::axes(px(7), px(1)),
                height: px(18),
                align_items: AlignItems::Center,
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|t| {
            t.spawn(label(text, 11., INK_WOOD));
        });
}

/// Horizontal rule with diamond caps.
pub fn divider(parent: &mut ChildSpawnerCommands, server: &AssetServer) {
    parent.spawn(frame(
        server,
        "divider",
        Node {
            width: percent(100),
            height: px(5),
            flex_shrink: 0.,
            ..default()
        },
    ));
}

/// Up to three stars; `filled` of them gold.
pub fn stars(parent: &mut ChildSpawnerCommands, server: &AssetServer, filled: u8) {
    parent
        .spawn(Node {
            column_gap: px(2),
            ..default()
        })
        .with_children(|r| {
            for i in 0..3 {
                r.spawn(glyph(
                    server,
                    if i < filled {
                        "glyph-star"
                    } else {
                        "glyph-star-empty"
                    },
                    9.,
                    9.,
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slice_table_matches_the_kit_manifest() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/ui/kit-v2/manifest.json"
        );
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for (name, ltrb) in SLICES {
            let asset = manifest["assets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["name"] == *name)
                .unwrap_or_else(|| panic!("{name} missing from manifest"));
            let m: Vec<f32> = asset["slice_ltrb"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect();
            assert_eq!(&m[..], &ltrb[..], "{name}");
        }
    }
    #[test]
    fn every_state_has_art() {
        for kind in [Kind::Plain, Kind::Primary, Kind::Tab, Kind::Card] {
            for bits in 0..16u8 {
                let name = kind.art(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
                let path = format!(
                    "{}/../../assets/{DIR}/{name}.png",
                    env!("CARGO_MANIFEST_DIR")
                );
                assert!(std::path::Path::new(&path).exists(), "{path}");
            }
        }
    }
}
