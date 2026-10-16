//! First-run title entry. This intentionally does not implement save-slot management (#37/#39).
//! The live Bevy forge scene remains loaded behind a dark overlay; all production
//! simulation is gated until the player explicitly enters the game.
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

#[derive(Resource, Default)]
pub struct TitleScreen {
    pub active: bool,
}

#[derive(Component)]
pub(crate) struct TitleRoot;

#[derive(Component)]
pub(crate) struct TitleHammer;
#[derive(Component)]
pub(crate) struct TitleImpact;
pub fn strike_y(t: f32) -> f32 {
    let t = t.rem_euclid(1.5);
    if t < 0.53 {
        -190.
    } else if t < 0.62 {
        -190. + ((t - 0.53) / 0.09).powi(2) * 190.
    } else if t < 0.74 {
        0.
    } else if t < 1.09 {
        -190. * ((t - 0.74) / 0.35).min(1.)
    } else {
        -190.
    }
}
pub fn animate_strike(
    time: Res<Time>,
    title: Res<TitleScreen>,
    mut clock: Local<f32>,
    mut hammer: Query<&mut Node, With<TitleHammer>>,
    mut impact: Query<&mut Visibility, With<TitleImpact>>,
) {
    if !title.active {
        return;
    }
    *clock = (*clock + time.delta_secs()).rem_euclid(1.5);
    for mut node in &mut hammer {
        node.top = px(strike_y(*clock));
    }
    for mut visible in &mut impact {
        *visible = if (0.61..0.73).contains(&*clock) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitleAction {
    Enter,
    Settings,
    Load,
    Quit,
}

pub fn build(mut commands: Commands, title: Res<TitleScreen>, server: Res<AssetServer>) {
    if !title.active {
        return;
    }
    commands.spawn((
        TitleRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(20)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.026, 0.022)),
        GlobalZIndex(100),
    )).with_children(|root| {
        root.spawn((
            Node {
                width: px(512),
                height: px(320),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
        )).with_children(|left| {
            // All child layers share one 512x320 canvas and fixed target at (265,208).
            left.spawn(Node { width: px(512), height: px(320), overflow: Overflow::clip(), ..default() })
                .with_children(|canvas| {
                    for name in ["background", "anvil", "hot_metal"] {
                        canvas.spawn((
                            ImageNode::new(server.load(format!("title/{name}.png"))),
                            Node { position_type: PositionType::Absolute, width: px(512), height: px(320), left: px(0), top: px(0), ..default() }
                        ));
                    }
                    canvas.spawn((
                        ImageNode::new(server.load("title/hammer_hand.png")),
                        Node { position_type: PositionType::Absolute, width: px(512), height: px(320), top: px(-190), left: px(0), ..default() },
                        TitleHammer,
                    ));
                    canvas.spawn((
                        ImageNode::new(server.load("title/impact.png")),
                        Node { position_type: PositionType::Absolute, width: px(512), height: px(320), left: px(0), top: px(0), ..default() },
                        Visibility::Hidden,
                        TitleImpact,
                    ));
                });
        });
        root.spawn((
            Node {
                width: px(330),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                row_gap: px(9),
                ..default()
            },
        )).with_children(|menu| {
            menu.spawn((
                Text::new("Little Forge"),
                TextFont { font_size: 40.0.into(), ..default() },
                TextColor(Color::srgb(0.96, 0.80, 0.58)),
                Node { margin: UiRect::bottom(px(22)), ..default() },
            ));
            for (text, action) in [
                ("Enter the Forge", TitleAction::Enter),
                ("Load Game (coming soon)", TitleAction::Load),
                ("Settings (coming soon)", TitleAction::Settings),
                ("Quit", TitleAction::Quit),
            ] {
                let disabled = matches!(action, TitleAction::Load | TitleAction::Settings);
                menu.spawn((
                    Button,
                    action,
                    Node {
                        width: percent(100),
                        height: px(44),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.46, 0.29, 0.17)),
                    BackgroundColor(if disabled {
                        Color::srgb(0.12, 0.10, 0.09)
                    } else {
                        Color::srgb(0.21, 0.13, 0.095)
                    }),
                )).with_children(|button| {
                    button.spawn((
                        Text::new(text),
                        TextFont { font_size: 17.0.into(), ..default() },
                        TextColor(if disabled {
                            Color::srgb(0.45, 0.41, 0.38)
                        } else {
                            Color::srgb(0.93, 0.83, 0.70)
                        }),
                    ));
                });
            }
            menu.spawn((
                Text::new("A Small Forge, A Bigger Tomorrow."),
                TextFont { font_size: 13.0.into(), ..default() },
                TextColor(Color::srgb(0.77, 0.66, 0.57)),
                Node { margin: UiRect::top(px(14)), ..default() },
            ));
        });
    });
}

pub fn activate(
    event: On<Activate>,
    actions: Query<&TitleAction>,
    mut title: ResMut<TitleScreen>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut roots: Query<&mut Visibility, With<TitleRoot>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if !title.active {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        match action {
            TitleAction::Enter => {
                title.active = false;
                desktop.shell.expanded = false;
                desktop.shell.recover();
                for mut visible in &mut roots {
                    *visible = Visibility::Hidden;
                }
            }
            TitleAction::Quit => {
                exit.write(bevy::app::AppExit::Success);
            }
            TitleAction::Load | TitleAction::Settings => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn impact_uses_registered_anchor() {
        assert_eq!(strike_y(0.), -190.);
        assert_eq!(strike_y(0.62), 0.);
    }
    #[test]
    fn title_state_defaults_inactive_for_qa_and_is_explicitly_activated() {
        let mut title = TitleScreen::default();
        assert!(!title.active);
        title.active = true;
        assert!(title.active);
    }
}
