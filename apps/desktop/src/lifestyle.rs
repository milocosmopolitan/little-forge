//! Presentation reads authoritative life state; buttons issue atomic domain commands.
use crate::{Options, Session, window::Desktop};
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};
use forge_core::Command;
#[derive(Component)]
pub struct Management;
#[derive(Component)]
pub struct Needs;
#[derive(Component)]
pub struct Ledger;
#[derive(Component)]
pub struct Action(Command);
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 13.0.into(),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.93, 0.85)),
        BackgroundColor(Color::srgba(0.04, 0.06, 0.07, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            top: px(43),
            left: px(12),
            padding: UiRect::all(px(4)),
            ..default()
        },
        Needs,
    ));
    commands.spawn((Node{position_type:PositionType::Absolute,top:px(78),left:px(24),width:px(900),padding:UiRect::all(px(16)),flex_direction:FlexDirection::Column,row_gap:px(10),..default()},BackgroundColor(Color::srgba(0.055,0.075,0.08,0.96)),Visibility::Hidden,Management)).with_children(|parent| {
        parent.spawn((Text::new("THE DAILY FORGE  /  Work, rest & provisions"),TextFont{font_size:21.0.into(),..default()},TextColor(Color::srgb(0.96,0.88,0.68))));
        parent.spawn((Text::new(""),TextFont{font_size:16.0.into(),..default()},Ledger));
        parent.spawn(Node{flex_direction:FlexDirection::Row,column_gap:px(8),flex_wrap:FlexWrap::Wrap,row_gap:px(6),..default()}).with_children(|row| {
            for (label,command) in [("Queue dagger [Q]",Command::QueueCraft),("Buy ore [B]",Command::BuyOre),("Farmer food [F]",Command::BuyFood),("Eat staple [E]",Command::Eat),("Free well [W]",Command::Drink),("Rest [R]",Command::Rest)] {
                row.spawn((Button,Node{padding:UiRect::axes(px(12),px(9)),..default()},BackgroundColor(Color::srgb(0.18,0.26,0.25)),Action(command))).with_children(|b|{b.spawn((Text::new(label),TextFont{font_size:14.0.into(),..default()}));});
            }
        });
        parent.spawn((Text::new("Queued ore is reserved. Sleep pauses work, then staples & free water restore needs.\nEmpty pantry? Work still resumes. Premium meals are never eaten automatically.\nTab: small forge  |  F8: click through 10s  |  Ctrl+Alt+F8: recover  |  D: window border"),TextFont{font_size:13.0.into(),..default()},TextColor(Color::srgb(0.68,0.76,0.74))));
    });
}
pub fn buttons(
    event: On<Activate>,
    actions: Query<&Action>,
    mut s: ResMut<Session>,
    opts: Res<Options>,
    desktop: Res<Desktop>,
) {
    if !desktop.shell.expanded || s.disabled {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        let id = s.save.state.command_id + 1;
        s.status = match s.save.state.command(id, action.0) {
            Ok(()) => format!("{:?} accepted", action.0),
            Err(e) => format!("{e:?}"),
        };
        crate::persist(&mut s, &opts);
    }
}
#[allow(clippy::type_complexity)]
pub fn update(
    s: Res<Session>,
    opts: Res<Options>,
    desktop: Res<Desktop>,
    mut panel: Query<&mut Visibility, (With<Management>, Without<Needs>)>,
    mut needs: Query<(&mut Text, &mut Visibility), With<Needs>>,
    mut ledger: Query<&mut Text, (With<Ledger>, Without<Needs>)>,
) {
    let visible = opts.qa.as_deref().is_none_or(|q| q.starts_with("life-"));
    let l = &s.save.state.life;
    for mut v in &mut panel {
        *v = if visible && desktop.shell.expanded {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut text, mut v) in &mut needs {
        *v = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        **text = format!(
            "Fatigue {}%  /  Hunger {}%  /  Thirst {}%   |   {:?}{}",
            l.fatigue / 100,
            l.hunger / 100,
            l.thirst / 100,
            l.action,
            if l.pantry.staples() == 0 {
                "  - pantry empty"
            } else {
                ""
            }
        );
    }
    for mut text in &mut ledger {
        let slots: Vec<_> = l
            .pantry
            .slots
            .iter()
            .enumerate()
            .map(|(i, s)| match s {
                Some(s) => format!("{}:{:?} x{}", i + 1, s.food, s.count),
                None => format!("{}: --", i + 1),
            })
            .collect();
        **text = format!(
            "{:?}  |  timer {}s  |  work speed {}%  |  queued {}\nFatigue slows remaining work. Sleep restores fatigue; food / water restore hunger / thirst.\n\nPANTRY  (12 stackable slots, {} per stack)\n{}\n{}\nFarmer staple: {} gold  |  Ore: 4 gold / 2  |  Well: always free",
            l.action,
            l.remaining,
            l.speed() / 10,
            l.queue,
            l.config.stack_limit,
            slots[..6].join("    "),
            slots[6..].join("    "),
            l.config.food_price
        );
    }
}
