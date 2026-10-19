//! Static cinematic plate and six-pose embers. Slot management belongs to #37/#39.
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, Button};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    #[default]
    Title,
    New,
    /// Pick the smith's appearance. Reached from New Game, or once from Continue when an
    /// older save has no choice yet. Backing out writes nothing.
    Choose,
    Load,
    Settings,
    InGame,
    QuitConfirm,
}
#[derive(Resource, Default)]
pub struct TitleScreen {
    pub active: bool,
    pub has_save: bool,
    pub route: Route,
    pub selected: usize,
    pub reduced_motion: bool,
    pub error: String,
}
#[derive(Component)]
pub(crate) struct TitleRoot;
#[derive(Component)]
pub(crate) struct Menu;
#[derive(Component)]
pub(crate) struct Stage;
#[derive(Component)]
pub(crate) struct Ember(usize);
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HeatMaterial {
    #[uniform(0)]
    pub strength: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub plate: Handle<Image>,
}
impl UiMaterial for HeatMaterial {
    fn fragment_shader() -> ShaderRef {
        "title/heat.wesl".into()
    }
}
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitleAction {
    New,
    Continue,
    Load,
    Settings,
    Quit,
    Back,
    ConfirmQuit,
    Begin,
    Motion,
    Pick(forge_core::Smith),
}
fn entries(route: Route) -> &'static [(&'static str, TitleAction)] {
    use TitleAction::*;
    match route {
        Route::Title => &[
            ("New Game", New),
            ("Continue", Continue),
            ("Load Game", Load),
            ("Settings", Settings),
            ("Quit", Quit),
        ],
        Route::New => &[("Start forging", Begin), ("Back", Back)],
        Route::Choose => &[
            ("Eira", Pick(forge_core::Smith::Female)),
            ("Bram", Pick(forge_core::Smith::Male)),
            ("Back", Back),
        ],
        Route::Load => &[("Continue current journey", Continue), ("Back", Back)],
        Route::Settings => &[("Toggle animation", Motion), ("Back", Back)],
        Route::QuitConfirm => &[("Save and Quit", ConfirmQuit), ("Cancel", Back)],
        Route::InGame => &[],
    }
}
fn transition(route: Route, action: TitleAction) -> Route {
    if !entries(route).iter().any(|(_, a)| *a == action) {
        return route;
    }
    match action {
        TitleAction::New => Route::New,
        TitleAction::Begin => Route::Choose,
        TitleAction::Continue | TitleAction::Pick(_) => Route::InGame,
        TitleAction::Load => Route::Load,
        TitleAction::Settings => Route::Settings,
        TitleAction::Quit => Route::QuitConfirm,
        TitleAction::Back => Route::Title,
        _ => route,
    }
}
pub fn ember_pose(index: usize, seconds: f32) -> (f32, f32, f32) {
    let frame = ((seconds * 8.) as usize + index * 2) % 6;
    let t = frame as f32 / 5.;
    let direction = (index as f32 * 2.39996).sin();
    (
        42. + direction * (2. + t * 18.),
        63. - t * (10. + (index % 5) as f32 * 4.) + t * t * 5.,
        (1. - t).max(0.),
    )
}
pub fn animate_heat(
    time: Res<Time>,
    opts: Res<crate::Options>,
    title: Res<TitleScreen>,
    mut clock: Local<f32>,
    mut embers: Query<(&Ember, &mut Node, &mut BackgroundColor)>,
    mut heat: ResMut<Assets<HeatMaterial>>,
) {
    if !title.active {
        return;
    }
    *clock = (*clock
        + if opts.qa.as_deref() == Some("title") {
            1. / 30.
        } else {
            time.delta_secs()
        })
    .rem_euclid(60.);
    let pulse = if title.reduced_motion {
        0.10
    } else {
        0.10 + 0.07 * (*clock * 2.1).sin()
    };
    for (_, material) in heat.iter_mut() {
        material.strength.x = pulse;
    }
    for (ember, mut node, mut color) in &mut embers {
        let (x, y, a) = ember_pose(ember.0, *clock);
        node.left = percent(x);
        node.top = percent(y);
        color.0 = Color::srgba(
            1.,
            0.48 + (ember.0 % 3) as f32 * 0.15,
            0.09,
            if title.reduced_motion { 0. } else { a },
        );
    }
}
pub fn build(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<HeatMaterial>>,
) {
    commands
        .spawn((
            TitleRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(100),
        ))
        .with_children(|outer| {
            outer
                .spawn((
                    Stage,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(960),
                        height: px(540),
                        ..default()
                    },
                    UiTransform::default(),
                ))
                .with_children(|root| {
                    root.spawn((
                        ImageNode::new(server.load("title/forge-static.png")),
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                    ));
                    root.spawn((
                        MaterialNode(materials.add(HeatMaterial {
                            strength: Vec4::ZERO,
                            plate: server.load("title/forge-static.png"),
                        })),
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                    ));
                    for i in 0..26 {
                        root.spawn((
                            Ember(i),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(if i % 4 == 0 { 6. } else { 3. }),
                                height: px(if i % 3 == 0 { 9. } else { 3. }),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ));
                    }
                    root.spawn((
                        Menu,
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(76),
                            top: percent(32),
                            width: percent(21),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        },
                    ));
                    root.spawn((
                        Text::new(concat!("v", env!("CARGO_PKG_VERSION"))),
                        TextFont {
                            font_size: 11.0.into(),
                            ..default()
                        },
                        TextColor(Color::srgb(0.42, 0.39, 0.36)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(18),
                            bottom: px(12),
                            ..default()
                        },
                    ));
                });
        });
}
pub fn refresh(
    mut commands: Commands,
    title: Res<TitleScreen>,
    server: Res<AssetServer>,
    menus: Query<Entity, With<Menu>>,
    mut roots: Query<&mut Visibility, With<TitleRoot>>,
) {
    if !title.is_changed() {
        return;
    }
    for mut visible in &mut roots {
        *visible = if title.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for menu in &menus {
        commands.entity(menu).despawn_children().with_children(|m| {
            m.spawn((
                Text::new("Little Forge"),
                TextFont {
                    font: server.load::<Font>("title/DejaVuSerif.ttf").into(),
                    font_size: 31.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.78, 0.54)),
            ));
            m.spawn((
                Node {
                    height: px(2),
                    width: percent(100),
                    margin: UiRect::vertical(px(8)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.61, 0.32, 0.12)),
            ));
            let help = match title.route {
                Route::New => {
                    if title.has_save { "A journey already exists. Continue it from the title. Separate new journeys arrive with save slots." } else { "Light the forge and begin your journey?" }
                }
                Route::Choose => {
                    if title.has_save { "Choose who tends this forge. Asked once; your progress is kept." } else { "Who tends the forge? Looks only; both smiths work alike." }
                }
                Route::Load => {
                    "Current recovery journal. Separate save slots are not available yet."
                }
                Route::Settings => {
                    if title.reduced_motion {
                        "Effects: Still"
                    } else {
                        "Effects: Gentle"
                    }
                }
                Route::QuitConfirm => "Save your progress before leaving?",
                _ => "",
            };
            if !help.is_empty() {
                m.spawn((
                    Text::new(help),
                    TextFont {
                        font_size: 12.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.7, 0.58)),
                ));
            }
            for (i, &(label, action)) in entries(title.route).iter().enumerate() {
                let selected = i == title.selected;
                let enabled = available(&title, action);
                let mut button = m.spawn((
                    Button,
                    forge_ui::SkinButton,
                    bevy::picking::hover::Hovered::default(),
                    action,
                    Node {
                        width: percent(100),
                        height: px(29),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(if selected && enabled {
                        Color::srgb(1., 0.65, 0.26)
                    } else {
                        Color::srgb(0.4, 0.29, 0.2)
                    }),
                    BackgroundColor(if selected && enabled {
                        Color::srgba(0.28, 0.13, 0.055, 0.95)
                    } else {
                        Color::srgba(0.085, 0.071, 0.06, 0.93)
                    }),
                ));
                if selected { button.insert(forge_ui::Selected); }
                if !enabled { button.insert((InteractionDisabled, forge_ui::Unavailable("Not available yet".into()))); }
                button.with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont {
                            font: server.load::<Font>("title/DejaVuSerif.ttf").into(),
                            font_size: 14.0.into(),
                            ..default()
                        },
                        TextColor(if enabled { Color::srgb(0.93, 0.82, 0.66) } else { Color::srgb(0.40, 0.36, 0.31) }),
                    ));
                });
            }
            m.spawn((
                Text::new(if title.error.is_empty() {
                    "A Small Forge,\nA Bigger Tomorrow."
                } else {
                    &title.error
                }),
                TextFont {
                    font_size: 12.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.36, 0.17)),
                Node {
                    margin: UiRect::top(px(8)),
                    ..default()
                },
            ));
        });
    }
}
/// Applies a title action; returns true when the app should exit.
fn perform(
    action: TitleAction,
    title: &mut TitleScreen,
    desktop: &mut crate::window::Desktop,
    session: &mut crate::Session,
    opts: &crate::Options,
) -> bool {
    if !title.active
        || !available(title, action)
        || !entries(title.route).iter().any(|(_, a)| *a == action)
    {
        return false;
    }
    title.error.clear();
    if action == TitleAction::ConfirmQuit {
        if title.has_save {
            crate::persist(session, opts);
        }
        if session.disabled {
            title.error = session.status.clone();
            return false;
        }
        return true;
    }
    if let TitleAction::Pick(smith) = action {
        // The only write on this screen: record the choice, then persist the journey once.
        let id = session.save.state.command_id + 1;
        if let Err(e) = session
            .save
            .state
            .command(id, forge_core::Command::ChooseSmith(smith))
        {
            title.error = format!("{e:?}");
            return false;
        }
        crate::persist(session, opts);
        if session.disabled {
            title.error = session.status.clone();
            return false;
        }
        title.has_save = true;
        info!("Title smith: {:?}", smith);
    }
    if action == TitleAction::Continue && session.save.state.smith.is_none() {
        // Older save without a choice: ask once, keep everything else.
        title.route = Route::Choose;
        title.selected = 0;
        info!("Title route: Choose (existing save)");
        return false;
    }
    if action == TitleAction::Motion {
        title.reduced_motion = !title.reduced_motion;
        info!("Title still effects: {}", title.reduced_motion);
        return false;
    }
    title.route = transition(title.route, action);
    info!("Title route: {:?}", title.route);
    title.selected = 0;
    title.active = title.route != Route::InGame;
    if !title.active {
        desktop.shell.expanded = true;
        desktop.shell.recover();
        session.last = std::time::Instant::now();
    }
    false
}
pub fn activate(
    event: On<Activate>,
    actions: Query<&TitleAction>,
    mut title: ResMut<TitleScreen>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut session: ResMut<crate::Session>,
    opts: Res<crate::Options>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if let Ok(&action) = actions.get(event.entity)
        && perform(action, &mut title, &mut desktop, &mut session, &opts)
    {
        exit.write(bevy::app::AppExit::Success);
    }
}
pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut title: ResMut<TitleScreen>,
    mut desktop: ResMut<crate::window::Desktop>,
    mut session: ResMut<crate::Session>,
    opts: Res<crate::Options>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if !title.active {
        if keys.just_pressed(KeyCode::F9) {
            info!("Title route: Title (F9)");
            title.active = true;
            title.route = Route::Title;
            title.selected = 0;
            desktop.shell.recover();
            desktop.shell.expanded = true;
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if title.route != Route::Title {
            title.route = Route::Title;
            info!("Title route: Title (Escape)");
            title.selected = 0;
            title.error.clear();
        }
        return;
    }
    let rows = entries(title.route);
    if keys.just_pressed(KeyCode::ArrowDown)
        || (title.route == Route::Choose && keys.just_pressed(KeyCode::ArrowRight))
    {
        title.selected = next_selection(&title, 1);
    }
    if keys.just_pressed(KeyCode::ArrowUp)
        || (title.route == Route::Choose && keys.just_pressed(KeyCode::ArrowLeft))
    {
        title.selected = next_selection(&title, -1);
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        let action = rows[title.selected].1;
        if perform(action, &mut title, &mut desktop, &mut session, &opts) {
            exit.write(bevy::app::AppExit::Success);
        }
    }
}
pub fn fit_stage(
    windows: Query<&Window>,
    mut stages: Query<(&mut Node, &mut UiTransform), With<Stage>>,
) {
    let Ok(w) = windows.single() else {
        return;
    };
    let scale = (w.width() / 960.).min(w.height() / 540.);
    for (mut node, mut transform) in &mut stages {
        node.left = px((w.width() - 960.) / 2.);
        node.top = px((w.height() - 540.) / 2.);
        transform.scale = Vec2::splat(scale);
    }
}

fn available(title: &TitleScreen, action: TitleAction) -> bool {
    match action {
        TitleAction::Continue | TitleAction::Load => title.has_save,
        TitleAction::Begin => !title.has_save,
        _ => true,
    }
}
#[derive(Component)]
pub(crate) struct Portrait(forge_core::Smith);
/// Two same-size native portraits with idle blink beside the Choose menu. Built from the same
/// atlas the game renders; integer 2x only.
#[allow(clippy::too_many_arguments)]
pub fn portraits(
    mut commands: Commands,
    title: Res<TitleScreen>,
    opts: Res<crate::Options>,
    time: Res<Time>,
    catalogs: Option<Res<forge_npc::player::PlayerCatalogs>>,
    stages: Query<Entity, With<Stage>>,
    mut existing: Query<(Entity, &Portrait, &mut ImageNode, &mut BorderColor)>,
) {
    let Some(catalogs) = catalogs else {
        return;
    };
    let show = title.active && title.route == Route::Choose;
    if !show {
        for (e, _, _, _) in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    let seconds = if opts.qa.is_some() {
        opts.frames as f32 / 30.
    } else {
        time.elapsed_secs()
    };
    if existing.is_empty() {
        let Ok(stage) = stages.single() else {
            return;
        };
        commands.entity(stage).with_children(|stage| {
            for (i, smith) in forge_core::Smith::ALL.into_iter().enumerate() {
                let Some(c) = catalogs.get(smith.id()) else {
                    continue;
                };
                let Some(neutral) = c.sprite_index("neutral") else {
                    continue;
                };
                let sprite = &c.sprites[neutral];
                let mut image = ImageNode::new(catalogs.images[smith.id()].clone());
                image.texture_atlas = Some(TextureAtlas {
                    layout: catalogs.layouts[smith.id()].clone(),
                    index: neutral,
                });
                stage.spawn((
                    Portrait(smith),
                    image,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(300. + i as f32 * 200.),
                        bottom: px(120),
                        width: px(sprite.rect[2] as f32 * 2.),
                        height: px(sprite.rect[3] as f32 * 2.),
                        border: UiRect::all(px(2)),
                        ..default()
                    },
                    BorderColor::all(Color::NONE),
                ));
            }
        });
        return;
    }
    let selected = entries(title.route)
        .get(title.selected)
        .and_then(|(_, a)| match a {
            TitleAction::Pick(s) => Some(*s),
            _ => None,
        });
    for (_, portrait, mut image, mut border) in &mut existing {
        let Some(c) = catalogs.get(portrait.0.id()) else {
            continue;
        };
        // 120 ms blink every 3.5 s, offset per smith so they do not blink in unison.
        let phase = (seconds
            + if portrait.0 == forge_core::Smith::Male {
                1.7
            } else {
                0.
            })
            % 3.5;
        let frame = if phase > 3.3 && phase < 3.42 {
            "blink"
        } else {
            "neutral"
        };
        if let (Some(index), Some(atlas)) = (c.sprite_index(frame), image.texture_atlas.as_mut())
            && atlas.index != index
        {
            atlas.index = index;
        }
        let next = BorderColor::all(if selected == Some(portrait.0) {
            Color::srgb(1., 0.65, 0.26)
        } else {
            Color::NONE
        });
        if *border != next {
            *border = next;
        }
    }
}

fn next_selection(title: &TitleScreen, direction: isize) -> usize {
    let rows = entries(title.route);
    for step in 1..=rows.len() {
        let i = (title.selected as isize + step as isize * direction)
            .rem_euclid(rows.len() as isize) as usize;
        if available(title, rows[i].1) {
            return i;
        }
    }
    title.selected
}
pub fn hover(
    event: On<bevy::picking::events::PointerOver>,
    actions: Query<&TitleAction>,
    mut colors: Query<&mut BorderColor>,
    title: Res<TitleScreen>,
) {
    if let Ok(&action) = actions.get(event.entity)
        && title.active
        && available(&title, action)
        && let Ok(mut border) = colors.get_mut(event.entity)
    {
        *border = BorderColor::all(Color::srgb(1., 0.65, 0.26));
    }
}
pub fn unhover(
    event: On<bevy::picking::events::PointerOut>,
    actions: Query<&TitleAction>,
    mut colors: Query<&mut BorderColor>,
    title: Res<TitleScreen>,
) {
    if let Ok(&action) = actions.get(event.entity)
        && let Ok(mut border) = colors.get_mut(event.entity)
    {
        let selected = entries(title.route)
            .get(title.selected)
            .is_some_and(|(_, a)| *a == action)
            && available(&title, action);
        *border = BorderColor::all(if selected {
            Color::srgb(1., 0.65, 0.26)
        } else {
            Color::srgb(0.4, 0.29, 0.2)
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_and_existing_journeys_have_safe_actions() {
        let mut title = TitleScreen {
            active: true,
            ..default()
        };
        assert!(!available(&title, TitleAction::Continue));
        assert!(!available(&title, TitleAction::Load));
        assert!(available(&title, TitleAction::Begin));
        assert_eq!(next_selection(&title, 1), 3);
        assert_eq!(next_selection(&title, -1), 4);
        title.has_save = true;
        assert!(available(&title, TitleAction::Continue));
        assert!(!available(&title, TitleAction::Begin));
    }
    #[test]
    fn invalid_transitions_do_not_enter_or_quit() {
        assert_eq!(
            transition(Route::Title, TitleAction::ConfirmQuit),
            Route::Title
        );
        assert_eq!(
            transition(Route::Settings, TitleAction::Continue),
            Route::Settings
        );
        assert_eq!(transition(Route::InGame, TitleAction::New), Route::InGame);
    }
    #[test]
    fn new_game_chooses_before_entering_and_choose_is_reversible() {
        assert_eq!(transition(Route::New, TitleAction::Begin), Route::Choose);
        assert_eq!(
            transition(Route::Choose, TitleAction::Pick(forge_core::Smith::Female)),
            Route::InGame
        );
        assert_eq!(transition(Route::Choose, TitleAction::Back), Route::Title);
        assert_eq!(
            transition(Route::Title, TitleAction::Pick(forge_core::Smith::Male)),
            Route::Title
        );
    }
    #[test]
    fn picking_records_the_smith_once_and_backing_out_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("forge-title-{}", std::process::id()));
        let mut session = crate::Session {
            save: forge_core::save::Save::new(forge_core::State::new(5), 100),
            dir: dir.clone(),
            _lock: forge_core::save::SaveLock::acquire(&dir).unwrap(),
            last: std::time::Instant::now(),
            fraction: 0.,
            status: String::new(),
            disabled: false,
            trace: vec![],
        };
        let opts = crate::Options::default();
        let mut desktop = crate::window::Desktop::new(false, true);
        let mut title = TitleScreen {
            active: true,
            ..default()
        };
        let mut go = |a: TitleAction, title: &mut TitleScreen, session: &mut crate::Session| {
            assert!(!perform(a, title, &mut desktop, session, &opts));
        };
        go(TitleAction::New, &mut title, &mut session);
        go(TitleAction::Begin, &mut title, &mut session);
        assert_eq!(title.route, Route::Choose);
        go(TitleAction::Back, &mut title, &mut session);
        assert_eq!(title.route, Route::Title);
        assert!(!title.has_save);
        assert!(std::fs::read_dir(&dir).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("save-")
        }));
        go(TitleAction::New, &mut title, &mut session);
        go(TitleAction::Begin, &mut title, &mut session);
        go(
            TitleAction::Pick(forge_core::Smith::Female),
            &mut title,
            &mut session,
        );
        assert_eq!(title.route, Route::InGame);
        assert!(title.has_save);
        assert_eq!(session.save.state.smith, Some(forge_core::Smith::Female));
        assert!(std::fs::read_dir(&dir).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("save-")
        }));
        // Continue with a chosen smith never asks again; an unchosen save asks once.
        title.active = true;
        title.route = Route::Title;
        go(TitleAction::Continue, &mut title, &mut session);
        assert_eq!(title.route, Route::InGame);
        session.save.state.smith = None;
        title.active = true;
        title.route = Route::Title;
        go(TitleAction::Continue, &mut title, &mut session);
        assert_eq!(title.route, Route::Choose);
        drop(session);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn all_submenus_are_reversible() {
        for action in [
            TitleAction::New,
            TitleAction::Load,
            TitleAction::Settings,
            TitleAction::Quit,
        ] {
            let route = transition(Route::Title, action);
            assert_ne!(route, Route::Title);
            assert_eq!(transition(route, TitleAction::Back), Route::Title);
        }
    }
    #[test]
    fn embers_have_six_stable_poses_and_repeat() {
        for i in 0..26 {
            for f in 0..6 {
                let t = f as f32 / 8.;
                let p = ember_pose(i, t);
                assert_eq!(p, ember_pose(i, t + 0.75));
                assert!((20. ..65.).contains(&p.0));
                assert!((0. ..=1.).contains(&p.2));
            }
        }
    }
}
