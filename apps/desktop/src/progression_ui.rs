//! Views of core progression. Eligibility is checked by the same command used on activation.
use crate::lifestyle::{Action, HomeItem, HomeTab};
use bevy::prelude::*;
use forge_core::{Command, State, commission::OrderPhase};
use forge_ui as ui;

pub fn command_button(p: &mut ChildSpawnerCommands, s: &State, label: &str, command: Command) {
    let result = s.clone().command(s.command_id.saturating_add(1), command);
    let e = ui::button(p, label, Action(command));
    if let Err(error) = result {
        p.commands().entity(e).insert((
            bevy::ui::InteractionDisabled,
            ui::Unavailable(crate::lifestyle::command_error(error)),
        ));
    }
}
pub fn draw(
    p: &mut ChildSpawnerCommands,
    server: &AssetServer,
    s: &State,
    tab: usize,
    selected: usize,
) {
    let progress = &s.progression;
    let Some(rules) = &progress.rules else {
        p.spawn(ui::text("Progression content unavailable.", 16.));
        return;
    };
    match tab {
        3 => {
            p.spawn(ui::text("Chef / A blade for the kitchen", 19.));
            p.spawn(Node {
                column_gap: px(12),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    ImageNode::new(server.load("npc/chef/poses/neutral/composed.png")),
                    Node {
                        width: px(96),
                        height: px(128),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|text| {
                    text.spawn(ui::text(&rules.chef.request, 14.));
                    text.spawn(ui::text(
                        format!(
                            "Blade {} / 1 · Wooden handle {} / 1\nMake them in Orders (smelt, forge, handle)\nNo deadline · Reward: {} gold",
                            s.stock(forge_core::items::Id::new("straight_blade")),
                            s.stock(forge_core::items::Id::new("wooden_handle")),
                            rules.chef.reward_gold
                        ),
                        14.,
                    ));
                });
            });
            p.spawn(ui::text(
                format!(
                    "On creation: {} crafting XP\nBlade forging +{} · Handle fitting +{}",
                    rules.recipes[0].general_xp,
                    rules.recipes[0].technique_xp[0],
                    rules.recipes[0].technique_xp[1]
                ),
                14.,
            ));
            match progress.chef_phase {
                OrderPhase::Offered => {
                    command_button(p, s, "Accept Chef commission", Command::AcceptChef)
                }
                OrderPhase::Accepted => {
                    command_button(p, s, "Queue Kitchen Knife", Command::QueueChef)
                }
                OrderPhase::Queued | OrderPhase::Working => {
                    p.spawn(ui::text(
                        "Your smith is making the knife. It stays reserved for Chef.",
                        14.,
                    ));
                }
                OrderPhase::Ready => {
                    command_button(p, s, "Deliver Kitchen Knife", Command::DeliverChef)
                }
                OrderPhase::Delivered => {
                    p.spawn(ui::text(
                        "Thank you! This will serve the village kitchen well.",
                        16.,
                    ));
                    ui::button(p, "Follow the discovery", HomeTab(4));
                }
            }
        }
        4 => {
            p.spawn(ui::text("Research Tree / Blade craft", 20.));
            p.spawn(Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                ui::button(row, "Kitchen Knife", HomeItem(0));
                row.spawn(ui::text("→", 18.));
                let e = ui::button(
                    row,
                    if progress.discovered() {
                        "Dagger"
                    } else {
                        "???"
                    },
                    HomeItem(1),
                );
                if selected == 1 {
                    row.commands().entity(e).insert(ui::Selected);
                }
            });
            if !progress.discovered() {
                p.spawn(ui::text(
                    "An undiscovered design. Complete Chef's commission to find a clue.",
                    16.,
                ));
                ui::button(p, "Visit Chef", HomeTab(3));
                return;
            }
            p.spawn(ui::text(
                if selected == 0 {
                    "Source: a balanced Kitchen Knife"
                } else {
                    "Dagger / first weapon"
                },
                17.,
            ));
            p.spawn(ui::text(
                format!(
                    "Research XP {} / {}",
                    progress.research_xp, rules.dagger.goal_xp
                ),
                16.,
            ));
            ui::progress_bar(
                p,
                server,
                progress.research_xp as f32 / rules.dagger.goal_xp as f32,
                ui::Fill::Blue,
            );
            for (i, t) in rules.techniques.iter().enumerate() {
                p.spawn(ui::text(
                    format!(
                        "{} Lv.{} / requires Lv.{}",
                        t.label,
                        progress.level(i),
                        rules.dagger.technique_levels[i]
                    ),
                    14.,
                ));
            }
            p.spawn(ui::text(
                "Prior research: none\nClue: Chef's balanced blade\nUnlock: Iron Dagger recipe",
                14.,
            ));
            if progress.research_completed {
                ui::button(p, "Completed / Forge a Dagger", HomeTab(0));
            } else if progress.can_complete() {
                command_button(
                    p,
                    s,
                    "Complete Dagger research",
                    Command::CompleteDaggerResearch,
                );
            } else if progress.study_pending {
                p.spawn(ui::text(
                    "Study queued / in progress at the research table.",
                    14.,
                ));
            } else {
                command_button(
                    p,
                    s,
                    &format!(
                        "Study · {} gold · {}s · +{} XP",
                        rules.dagger.study_gold, rules.dagger.study_seconds, rules.dagger.study_xp
                    ),
                    Command::StudyDagger,
                );
            }
        }
        5 => {
            p.spawn(ui::text("Smith's Codex", 21.));
            ui::button(
                p,
                "Crafted weapons / field records",
                crate::expedition::Action::FieldCodex,
            );
            for (i, r) in rules.recipes.iter().enumerate() {
                ui::button(
                    p,
                    if i == 1 && !progress.dagger_unlocked() {
                        "Unrecorded weapon"
                    } else {
                        &r.label
                    },
                    HomeItem(i),
                );
            }
            let r = &rules.recipes[selected.min(1)];
            if let Some(record) = progress.codex.get(&r.id) {
                p.spawn(ui::text(&r.label, 19.));
                p.spawn(ui::text(format!("First creation: forge second {}\nFirst work: #{}\nCreated: {}\nBest quality: not measured", record.first_tick, record.first_item_id, record.count), 15.));
                p.spawn(ui::text("Techniques: blade forging, handle fitting\nYour record remains after delivery or sale.", 14.));
                ui::button(p, "Related research", HomeTab(4));
            } else {
                p.spawn(ui::text(
                    "Not yet created. Your first work will be recorded here.",
                    16.,
                ));
            }
        }
        _ => {
            p.spawn(ui::text("Skill Journal", 21.));
            p.spawn(ui::text(
                format!("General crafting XP: {}", progress.general_xp),
                16.,
            ));
            for (i, t) in rules.techniques.iter().enumerate() {
                p.spawn(ui::text(
                    format!(
                        "{} · Lv.{} / 2 · {} XP\n{} XP per level; excess XP is retained.",
                        t.label,
                        progress.level(i),
                        progress.technique_xp[i],
                        t.level_xp
                    ),
                    13.,
                ));
            }
            let pages = progress.journal.len().max(1).div_ceil(2);
            let page = selected.min(pages - 1);
            p.spawn(ui::text(
                format!("Accomplishments · {} / {}", page + 1, pages),
                15.,
            ));
            for m in progress.journal.iter().rev().skip(page * 2).take(2) {
                p.spawn(ui::text(format!("{}s · {}", m.tick, m.text), 13.));
            }
            if progress.journal.is_empty() {
                p.spawn(ui::text("Your first commission starts the story.", 14.));
            }
            p.spawn(Node {
                column_gap: px(8),
                ..default()
            })
            .with_children(|row| {
                ui::button(row, "Newer", HomeItem(page.saturating_sub(1)));
                ui::button(row, "Older", HomeItem((page + 1).min(pages - 1)));
            });
        }
    }
}
