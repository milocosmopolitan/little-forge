//! First-run title entry. This intentionally does not implement save-slot management (#37/#39).
//! The live Bevy forge scene remains loaded behind a dark overlay; all production
//! simulation is gated until the player explicitly enters the game.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct TitleScreen {
    pub active: bool,
}

#[derive(Component)]
struct TitleRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum TitleAction {
    Enter,
    Settings,
    Load,
    Quit,
}

pub fn build(
    mut commands: Commands,
    title: Res<TitleScreen>,
) {
    if !title.active {
        return;
    }
    commands.spawn((
        TitleRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(30)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.026, 0.022, 0.95)),
        GlobalZIndex(100),
    )).with_children(|root| {
        root.spawn((
            Node {
                width: percent(54),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(px(20)),
                ..default()
            },
        )).with_children(|left| {
            left.spawn((
                Text::new("LITTLE FORGE"),
                TextFont { font_size: 46.0.into(), ..default() },
                TextColor(Color::srgb(0.96, 0.80, 0.58)),
            ));
            left.spawn((
                Text::new("A Small Forge, A Bigger Tomorrow.\n\nThe forge burns quietly.\nOne strike begins a story."),
                TextFont { font_size: 18.0.into(), ..default() },
                TextColor(Color::srgb(0.77, 0.66, 0.57)),
                Node { margin: UiRect::top(px(16)), ..default() },
            ));
        });
        root.spawn((
            Node {
                width: percent(40),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                row_gap: px(9),
                ..default()
            },
        )).with_children(|menu| {
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
        });
    });
}

pub fn interact(
    mut title: ResMut<TitleScreen>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut actions: Query<(&Interaction, &TitleAction, &mut BackgroundColor), (With<Button>, Changed<Interaction>)>,
    mut roots: Query<&mut Visibility, With<TitleRoot>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    for (interaction, action, mut bg) in &mut actions {
        match *interaction {
            Interaction::Pressed => match action {
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
                TitleAction::Load | TitleAction::Settings => {},
            },
            Interaction::Hovered => {
                if matches!(action, TitleAction::Enter | TitleAction::Quit) {
                    *bg = BackgroundColor(Color::srgb(0.33, 0.20, 0.12));
                }
            }
            Interaction::None => {
                *bg = BackgroundColor(if matches!(action, TitleAction::Load | TitleAction::Settings) {
                    Color::srgb(0.12, 0.10, 0.09)
                } else {
                    Color::srgb(0.21, 0.13, 0.095)
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn title_state_defaults_inactive_for_qa_and_is_explicitly_activated() {
        let mut title = TitleScreen::default();
        assert!(!title.active);
        title.active = true;
        assert!(title.active);
    }
}
