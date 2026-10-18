mod lifestyle;
mod native;
mod npc;
mod presentation;
mod rendering;
mod spaces;
mod title;
mod village;
mod window;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::PresentMode,
    winit::{UpdateMode, WinitSettings},
};
use forge_core::{
    Command, State,
    save::{self, Save, SaveLock},
};
use presentation::Presentation;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Resource)]
struct Session {
    save: Save,
    dir: PathBuf,
    _lock: SaveLock,
    last: std::time::Instant,
    fraction: f64,
    status: String,
    disabled: bool,
    trace: Vec<forge_core::Event>,
}
#[derive(Resource, Default)]
struct Options {
    safe: bool,
    qa: Option<String>,
    frames: u32,
    capture: Option<String>,
}
impl Options {
    /// Forge-space scenarios: `forge-<scene>[-female]`. The suffix picks the fixture smith.
    fn forge_scene(&self) -> Option<(&str, forge_core::Smith)> {
        let q = self.qa.as_deref()?.strip_prefix("forge-")?;
        Some(match q.strip_suffix("-female") {
            Some(scene) => (scene, forge_core::Smith::Female),
            None => (q, forge_core::Smith::Male),
        })
    }
    /// Scenarios that start with the management overlay expanded.
    fn qa_expanded(&self) -> bool {
        matches!(
            self.qa.as_deref(),
            Some(
                "expanded"
                    | "life-loop"
                    | "village-map"
                    | "village-home"
                    | "village-materials"
                    | "village-provisions"
                    | "village-carpenter"
                    | "village-leatherworker"
                    | "village-farm"
                    | "village-tavern"
                    | "village-chief"
                    | "village-guild"
                    | "village-outside"
                    | "village-input"
            )
        ) || self
            .forge_scene()
            .is_some_and(|(scene, _)| matches!(scene, "workshop" | "showroom" | "full" | "input" | "interior"))
    }
}
#[derive(Component)]
struct Visitor;
#[derive(Component)]
struct Weapon;
#[derive(Component)]
struct Building;
#[derive(Component)]
struct DisplayGood;
#[derive(Component)]
struct Smoke;
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Panel;
#[derive(Resource)]
struct Art {
    images: Vec<Handle<Image>>,
    visitor_sizes: Vec<Vec2>,
    visitor_hands: Vec<Vec2>,
    building_active: Handle<Image>,
    building_idle: Handle<Image>,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn main() {
    // Must precede Bevy, hotkeys, logging and every worker-thread initializer.
    rendering::prepare_process();
    if let Err(e) = run() {
        eprintln!(
            "Little Forge: {e}\nRecovery: launch with --safe-window. Save files have not been reset."
        );
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut opts = Options::default();
    let mut dir = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--safe-window" => opts.safe = true,
            "--qa" => {
                opts.qa = Some(
                    args.next()
                        .ok_or("--qa needs ambient, sale, expanded, idle, title, life-*, village-* or forge-*")?,
                );
            }
            "--save-dir" => {
                dir = Some(PathBuf::from(args.next().ok_or("--save-dir needs a path")?))
            }
            "--screenshot" => {
                opts.capture = Some(args.next().ok_or("--screenshot needs a PNG path")?)
            }
            "--help" => {
                println!(
                    "Little Forge\n--safe-window --save-dir PATH --qa ambient|sale|expanded|idle|life-loop|life-ambient|title --screenshot PATH\nC: craft, Q: queue, B: buy ore, F: farmer food, E: eat, W: free water, R: rest, Tab: expand, F8: 10s click-through, Ctrl+Alt+F8: global recovery, Esc: recover, T: topmost, D: decorations, F12: diagnostics\nQA uses an isolated save directory and a fixed simulation clock."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument {a}").into()),
        }
    }
    if opts.qa.as_ref().is_some_and(|s| {
        !matches!(
            s.as_str(),
            "ambient"
                | "sale"
                | "expanded"
                | "idle"
                | "life-loop"
                | "life-ambient"
                | "title"
                | "npc"
                | "village-map"
                | "village-home"
                | "village-materials"
                | "village-provisions"
                | "village-carpenter"
                | "village-leatherworker"
                | "village-farm"
                | "village-tavern"
                | "village-chief"
                | "village-guild"
                | "village-outside"
                | "village-input"
        ) && !matches!(
            Options {
                qa: Some(s.clone()),
                ..default()
            }
            .forge_scene(),
            Some(("exterior" | "workshop" | "showroom" | "full" | "select" | "input" | "interior", _))
        )
    }) {
        return Err("unknown QA scenario".into());
    }
    let qa = opts.qa.is_some();
    let dir = dir.unwrap_or_else(|| {
        if qa {
            PathBuf::from("qa-output/session")
        } else {
            directories::ProjectDirs::from("com", "Omiologic", "LittleForge")
                .expect("user data directory")
                .data_local_dir()
                .into()
        }
    });
    let lock = SaveLock::acquire(&dir)?;
    let loaded = if qa { None } else { save::load(&dir)? };
    let has_save = loaded.is_some();
    let mut saved = loaded.unwrap_or_else(|| Save::new(State::new(42), now()));
    if saved.state.tick == 0 {
        saved.state.life.config = forge_content::needs_config()?;
    }
    if opts.qa.as_deref().is_some_and(|q| q.starts_with("life-")) {
        saved.state.life.fatigue = 8990;
        saved.state.life.hunger = 8000;
        saved.state.life.thirst = 8000;
        saved.state.life.config.sleep_seconds = 8;
        saved
            .state
            .command(1, Command::Craft)
            .map_err(|e| format!("{e:?}"))?;
        saved
            .state
            .command(2, Command::QueueCraft)
            .map_err(|e| format!("{e:?}"))?;
    }
    if opts
        .qa
        .as_deref()
        .is_some_and(|q| q.starts_with("village-"))
    {
        saved.state.gold = 24;
    }
    if let Some((scene, smith)) = opts.forge_scene() {
        // Isolated fixtures for the forge spaces; never a user save.
        if scene != "select" {
            saved.state.smith = Some(smith);
        }
        saved.state.gold = 60;
        saved.state.commerce.timber = 6;
        if matches!(scene, "showroom" | "full") {
            // Two finished goods on display, one kept; "full" fills every slot and queues two jobs.
            let target = if scene == "full" { saved.state.showroom.capacity() } else { 3 };
            for i in 0..target {
                saved.state.showroom.reserve().map_err(|e| format!("{e:?}"))?;
                saved.state.showroom.place(forge_core::craft::Good::IronDagger).map_err(|e| format!("{e:?}"))?;
                if i == 1 {
                    saved.state.showroom.set_for_sale(i, false).map_err(|e| format!("{e:?}"))?;
                }
            }
            saved.state.crafted = target as u32;
            saved.state.daggers = target as u32;
        }
    }
    let elapsed = if qa { 0 } else { saved.catch_up(now())? };
    if saved.state.command_id == 0 && saved.state.showroom.block_reason().is_none() {
        saved
            .state
            .command(1, Command::Craft)
            .map_err(|e| format!("{e:?}"))?;
    }
    if opts.forge_scene().is_some_and(|(scene, _)| scene == "full") {
        for _ in 0..2 {
            let id = saved.state.command_id + 1;
            saved
                .state
                .command(id, Command::QueueCraft)
                .map_err(|e| format!("{e:?}"))?;
        }
    }
    if !qa && has_save {
        save::store(&dir, &mut saved)?;
    }
    let status = format!(
        "Welcome back | caught up {elapsed}s. C craft / B buy ore / Tab manage / F8 idle 10s / Ctrl+Alt+F8 recover"
    );
    let mut presentation = Presentation::default();
    if opts.qa.as_deref() == Some("sale") {
        for event in saved.state.advance(12).map_err(|e| format!("{e:?}"))? {
            presentation.accept(event);
        }
    }
    let mut app = App::new();
    let title_active = !qa
        || opts.qa.as_deref() == Some("title")
        || opts.forge_scene().is_some_and(|(scene, _)| scene == "select");
    let native = native::NativeRuntime::new();
    eprintln!("{}", native.status);
    app.insert_non_send(native);
    let asset_dir = std::env::current_exe()?.parent().unwrap().join("assets");
    let asset_dir = if asset_dir.is_dir() {
        asset_dir
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    };
    if !asset_dir.is_dir() {
        return Err(
            "assets folder missing: keep the assets directory next to the executable".into(),
        );
    }
    for asset in forge_content::Manifest::bundled()
        .map_err(|e| format!("content: {e}"))?
        .assets
    {
        if !asset_dir.join(&asset.path).is_file()
            && !asset.id.starts_with("village_")
            && !asset.id.starts_with("wash_")
            && !asset.id.starts_with("farmer_")
        {
            return Err(format!("missing bundled asset {}", asset.path).into());
        }
    }
    let win = Window {
        title: "Little Forge".into(),
        resolution: (960, 180).into(),
        transparent: !opts.safe,
        decorations: opts.safe,
        composite_alpha_mode: if opts.safe {
            bevy::window::CompositeAlphaMode::Auto
        } else {
            rendering::transparent_alpha()
        },
        resizable: false,
        present_mode: PresentMode::AutoVsync,
        ..default()
    };
    app.insert_resource(ClearColor(if opts.safe {
        Color::srgb(0.065, 0.075, 0.085)
    } else {
        Color::NONE
    }))
    .insert_resource(Session {
        save: saved,
        dir,
        _lock: lock,
        last: std::time::Instant::now(),
        fraction: 0.,
        status,
        disabled: false,
        trace: vec![],
    })
    .insert_resource(window::Desktop::new(opts.safe, title_active || opts.qa_expanded()))
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1. / 30.)),
        unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1. / 15.)),
    })
    .insert_resource(village::AssetRoot(asset_dir.clone()))
    .insert_resource(presentation)
    .insert_resource(title::TitleScreen {
        active: title_active,
        has_save,
        route: if opts.forge_scene().is_some_and(|(scene, _)| scene == "select") {
            title::Route::Choose
        } else {
            title::Route::Title
        },
        ..default()
    })
    .add_plugins(
        DefaultPlugins
            .set(rendering::plugin(opts.safe))
            .set(AssetPlugin {
                file_path: asset_dir.to_string_lossy().into_owned(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(win),
                ..default()
            }),
    )
    .add_plugins(npc::FarmerPlugin)
    .add_plugins(forge_npc::player::PlayerPlugin)
    .add_plugins(forge_ui::ForgeUiPlugin)
    .add_plugins(spaces::plugin)
    .init_resource::<village::Village>()
    .add_observer(village::activate)
    .add_observer(lifestyle::buttons)
    .init_resource::<lifestyle::HomeUi>()
    .add_observer(lifestyle::tab)
    .add_plugins(UiMaterialPlugin::<title::HeatMaterial>::default())
    .add_observer(title::activate)
    .add_observer(title::hover)
    .add_observer(title::unhover)
    .add_systems(Update, title::portraits)
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .insert_resource(opts)
    .add_systems(
        Startup,
        (setup, lifestyle::setup, village::setup, title::build),
    )
    .add_systems(
        Update,
        (
            qa_capture,
            title::animate_heat,
            title::fit_stage,
            title::keyboard,
            title::refresh,
            input,
            simulate,
            presentation::tick,
            window::update,
            village::fit_ui,
            animate,
            hud,
            lifestyle::update,
            village::input,
            village::draw,
            lifestyle::draw,
            lifestyle::live,
            village::update,
        )
            .chain(),
    );
    // QA-only A/B control; normal launches always receive the renderer correction.
    let qa_unpatched = app.world().resource::<Options>().qa.is_some()
        && std::env::var_os("LITTLE_FORGE_QA_UNPATCHED_SHADER").is_some();
    if !qa_unpatched {
        rendering::install_sprite_shader(&mut app);
    }
    app.run();
    Ok(())
}
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            image: server.load("forge/forge.png"),
            custom_size: Some(Vec2::splat(270.)),
            ..default()
        },
        Transform::from_xyz(-170., -3., 0.),
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
        Building,
    ));
    commands.spawn((
        Sprite {
            image: server.load("forge/workbench.png"),
            custom_size: Some(Vec2::splat(155.)),
            ..default()
        },
        Transform::from_xyz(90., -27., 0.),
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
    ));
    let manifest = forge_content::Manifest::bundled().expect("validated bundled manifest");
    let visitor_meta = manifest.assets.iter().find(|a| a.id == "visitor").unwrap();
    let frame_layout = |asset: &forge_content::Asset| {
        let mut layout = TextureAtlasLayout::new_empty(UVec2::new(asset.width, asset.height));
        for r in &asset.frame_rects {
            layout.add_texture(URect::new(r[0], r[1], r[2], r[3]));
        }
        layout
    };
    let sizes = |asset: &forge_content::Asset| {
        asset
            .frame_rects
            .iter()
            .map(|r| Vec2::new((r[2] - r[0]) as f32 / (r[3] - r[1]) as f32 * 75., 75.))
            .collect::<Vec<_>>()
    };
    let visitor_sizes = sizes(visitor_meta);
    let visitor_hands = visitor_meta
        .walking_hand_points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let r = visitor_meta.frame_rects[i + 4];
            let scale = 75. / (r[3] - r[1]) as f32;
            Vec2::new(
                (p[0] as f32 - (r[0] + r[2]) as f32 / 2.) * scale,
                (r[3] as f32 - p[1] as f32) * scale,
            )
        })
        .collect();
    let visitor_layout = layouts.add(frame_layout(visitor_meta));
    // The legacy smith sheet is no longer drawn: the chosen player character owns that role
    // (spaces.rs). The visitor sheet stays a visitor and is never used for the player.
    commands.spawn((
        Sprite {
            image: server.load("forge/visitor.png"),
            texture_atlas: Some(TextureAtlas {
                layout: visitor_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::splat(85.)),
            ..default()
        },
        Transform::from_xyz(-430., -85., 3.),
        bevy::sprite::Anchor::BOTTOM_CENTER,
        Visibility::Hidden,
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
        Visitor,
    ));
    let smoke = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(362, 724),
        6,
        1,
        None,
        None,
    ));
    commands.spawn((
        Sprite {
            image: server.load("forge/smoke.png"),
            texture_atlas: Some(TextureAtlas {
                layout: smoke,
                index: 0,
            }),
            custom_size: Some(Vec2::new(10., 20.)),
            ..default()
        },
        Transform::from_xyz(-134., 47., 1.),
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
        Smoke,
    ));
    // One shared placeholder image is used for the display and the carried item.
    let dagger = images.add(forge_npc::dagger_image());
    commands.spawn((
        Sprite::from_image(dagger.clone()),
        Transform::from_xyz(115., -8., 3.),
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
        DisplayGood,
    ));
    commands.spawn((
        Sprite::from_image(dagger.clone()),
        Transform::from_xyz(0., 0., 4.),
        Visibility::Hidden,
        spaces::SceneOf(forge_core::spaces::Space::Exterior),
        Weapon,
    ));
    commands.insert_resource(Art {
        visitor_sizes,
        visitor_hands,
        building_active: server.load("forge/forge.png"),
        building_idle: server.load("forge/forge_idle.png"),
        images: [
            "forge/forge.png",
            "forge/forge_idle.png",
            "forge/workbench.png",
            "forge/visitor.png",
            "forge/smoke.png",
            "forge/spaces/workshop-backdrop.png",
            "forge/spaces/showroom-backdrop.png",
            "characters/female/atlas.png",
            "characters/male/atlas.png",
        ]
        .map(|path| server.load(path))
        .to_vec(),
    });
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 14.0.into(),
            ..default()
        },
        TextColor(Color::srgb(0.96, 0.88, 0.68)),
        Node {
            position_type: PositionType::Absolute,
            top: px(6),
            left: px(12),
            ..default()
        },
        Hud,
    ));
    commands.spawn((Text::new("FORGE LEDGER\n\nC   Craft iron dagger | 2 ore | 5 seconds\nB   Buy 2 ore | 4 gold\n\nTab   Return to the small forge\nF8    Click through for 10 seconds (automatic recovery)\nCtrl+Alt+F8   Global recovery\nEsc   Restore interaction\nT      Toggle always-on-top (off by default)\nD     Toggle window decorations / drag window\nF12  Write diagnostic report\n\nFoundation preview: village orders, research and expeditions follow later."),TextFont{font_size:18.0.into(),..default()},Node{position_type:PositionType::Absolute,top:px(60),left:px(30),..default()},Visibility::Hidden,Panel));
}
#[allow(clippy::too_many_arguments)]
fn input(
    diagnostics: Res<DiagnosticsStore>,
    adapter: Option<Res<bevy::render::renderer::RenderAdapterInfo>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut desktop: ResMut<window::Desktop>,
    opts: Res<Options>,
    title: Res<title::TitleScreen>,
    mut farmer: ResMut<npc::FarmerPresentation>,
    mut village: ResMut<village::Village>,
    mut spaces: ResMut<spaces::Spaces>,
    dialog: Res<forge_ui::DialogState>,
    mut commands: Commands,
) {
    if title.active || dialog.is_open() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        desktop.shell.recover();
    }
    if keys.just_pressed(KeyCode::F8) {
        desktop.shell.idle();
    }
    if keys.just_pressed(KeyCode::Tab)
        && !desktop.shell.expanded
        && village.nav.screen == forge_core::village::Screen::Home
    {
        spaces::apply(
            spaces::Action::Enter,
            &mut spaces,
            &mut desktop,
            &mut village,
            &mut session,
            &opts,
        );
    }
    if keys.just_pressed(KeyCode::KeyT) {
        desktop.shell.always_on_top = !desktop.shell.always_on_top;
        desktop.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        desktop.decorations = !desktop.decorations;
        desktop.dirty = true;
    }
    for (key, cmd) in [
        (KeyCode::KeyC, Command::Craft),
        (KeyCode::KeyB, Command::BuyOre),
        (KeyCode::KeyQ, Command::QueueCraft),
        (KeyCode::KeyF, Command::BuyFood),
        (KeyCode::KeyE, Command::Eat),
        (KeyCode::KeyW, Command::Drink),
        (KeyCode::KeyR, Command::Rest),
    ] {
        if keys.just_pressed(key)
            && !session.disabled
            && village.nav.screen == forge_core::village::Screen::Home
        {
            let id = session.save.state.command_id + 1;
            session.status = match session.save.state.command(id, cmd) {
                Ok(()) => {
                    farmer.accepted(id, cmd);
                    format!("{cmd:?} accepted")
                }
                Err(e) => format!("{e:?}"),
            };
            persist(&mut session, &opts);
        }
    }
    if keys.just_pressed(KeyCode::F10)
        && let Some(path) = &opts.capture
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(path.clone()));
    }
    if keys.just_pressed(KeyCode::F12) {
        let path = session.dir.join("diagnostics.json");
        let data = serde_json::json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"version":env!("CARGO_PKG_VERSION"),"safe":opts.safe,"qa":opts.qa,"tick":session.save.state.tick,"life":session.save.state.life,"state":session.save.state,"village_screen":format!("{:?}",village.nav.screen),"space":format!("{:?}",spaces.space),"station":format!("{:?}",spaces.station),"activity":format!("{:?}",session.save.state.activity()),"showroom":{"capacity":session.save.state.showroom.capacity(),"free":session.save.state.showroom.free(),"displayed":session.save.state.showroom.displayed(),"held":session.save.state.showroom.held(),"reserved":session.save.state.showroom.reserved(),"overflow":session.save.state.showroom.overflow,"racks":session.save.state.showroom.racks.len()},"shell":format!("{:?}",desktop.shell),"native_display":desktop.diagnostics,"fps":diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d|d.average()),"adapter":adapter.map(|a|format!("{} | {:?} | {} {}",a.name,a.backend,a.driver,a.driver_info)),"note":"Native transparency requires manual verification"});
        session.status = match std::fs::write(&path, serde_json::to_vec_pretty(&data).unwrap()) {
            Ok(_) => format!("Diagnostics: {}", path.display()),
            Err(e) => e.to_string(),
        };
    }
}
fn persist(s: &mut Session, opts: &Options) {
    if opts.qa.is_some() {
        return;
    }
    s.save.saved_at = s.save.saved_at.max(now());
    if let Err(e) = save::store(&s.dir.clone(), &mut s.save) {
        s.disabled = true;
        s.status = format!("SAVE ERROR: {e}. Simulation paused; preserve save directory.");
    }
}
fn simulate(
    mut s: ResMut<Session>,
    mut p: ResMut<Presentation>,
    opts: Res<Options>,
    title: Res<title::TitleScreen>,
) {
    if title.active || s.disabled || (opts.qa.is_some() && opts.frames == 0) {
        return;
    }
    let seconds = if opts.qa.as_deref().is_some_and(|q| q.starts_with("life-")) {
        if opts.frames.is_multiple_of(30) {
            1.
        } else {
            0.
        }
    } else if opts.qa.is_some() {
        1. / 30.
    } else {
        let n = std::time::Instant::now();
        let d = n
            .duration_since(s.last)
            .as_secs_f64()
            .min(forge_core::OFFLINE_CAP_SECS as f64);
        s.last = n;
        d
    };
    s.fraction += seconds;
    let whole = s.fraction.floor() as u64;
    s.fraction -= whole as f64;
    if whole > 0 {
        match s.save.state.advance(whole) {
            Ok(events) => {
                for e in events {
                    if opts.qa.as_deref().is_some_and(|q| q.starts_with("life-")) {
                        s.trace.push(e.clone());
                    }
                    p.accept(e);
                }
                if opts.qa.as_deref().is_some_and(|q| q.starts_with("life-"))
                    && s.save.state.sold > 0
                    && s.save.state.command_id == 2
                {
                    s.save
                        .state
                        .command(3, Command::BuyFood)
                        .expect("sale funds staple");
                    s.status = "Farmer: bought staple food after sale (scripted QA action)".into();
                }
                persist(&mut s, &opts);
            }
            Err(e) => {
                s.disabled = true;
                s.status = format!("Simulation paused: {e:?}");
            }
        }
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)] // One sprite query keeps mutually exclusive presentation markers together.
fn animate(
    art: Res<Art>,
    session: Option<Res<Session>>,
    time: Res<Time>,
    opts: Res<Options>,
    p: Res<Presentation>,
    desktop: Res<window::Desktop>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut q: Query<
        (
            &mut Transform,
            &mut Sprite,
            Option<&Visitor>,
            Option<&Weapon>,
            Option<&Building>,
            Option<&Smoke>,
            Option<&DisplayGood>,
            &mut Visibility,
        ),
        (
            Without<forge_npc::Player>,
            Without<forge_npc::ItemAttachment>,
            Without<forge_npc::NpcVisual>,
            Without<forge_npc::player::Actor>,
            Without<spaces::WindowGlow>,
            Without<spaces::Backdrop>,
            Without<forge_npc::stations::StationLayer>,
        ),
    >,
) {
    let animation_time = if opts.qa.is_some() {
        opts.frames as f32 / 30.
    } else {
        time.elapsed_secs()
    };
    let offset = if desktop.shell.expanded {
        -(windows.single().map_or(540., |w| w.height()) - 180.) / 2.
    } else {
        0.
    };
    let walk_frame = ((p.elapsed * 6.) as usize) % 4;
    let visitor_frame = if matches!(
        p.phase,
        presentation::Phase::Walk | presentation::Phase::ExitWithItem
    ) {
        4 + walk_frame
    } else {
        0
    };
    let legacy_qa = opts
        .qa
        .as_deref()
        .is_some_and(|q| !q.starts_with("life-") && !q.starts_with("village-") && !q.starts_with("forge-"));
    let working = legacy_qa
        || session.as_ref().is_some_and(|s| {
            matches!(s.save.state.activity(), forge_core::spaces::Activity::Working { .. })
        });
    let sellable = legacy_qa || session.as_ref().is_some_and(|s| s.save.state.sellable() > 0);
    for (mut t, mut sprite, visitor, weapon, building, smoke, display, mut visibility) in &mut q {
        if visitor.is_some() {
            t.translation = Vec3::new(p.x(), -85. + offset, 3.);
            *visibility = if p.active {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if let Some(a) = &mut sprite.texture_atlas {
                a.index = visitor_frame;
                sprite.custom_size = Some(art.visitor_sizes[visitor_frame]);
            }
        } else if weapon.is_some() {
            t.translation = Vec3::new(
                p.x() + art.visitor_hands[walk_frame].x,
                -85. + art.visitor_hands[walk_frame].y + 8. + offset,
                4.,
            );
            *visibility = if p.has_item {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else if building.is_some() {
            // The exterior plate shows a lit forge only while a job is actually running.
            t.translation.y = -3. + offset;
            let image = if working {
                &art.building_active
            } else {
                &art.building_idle
            };
            if sprite.image != *image {
                sprite.image = image.clone();
            }
        } else if smoke.is_some() {
            t.translation.y = 47. + offset;
            if let Some(a) = &mut sprite.texture_atlas {
                a.index = ((animation_time * 5.) as usize) % 6;
            }
            sprite.color = if working { Color::WHITE } else { Color::NONE };
        } else if display.is_some() {
            t.translation.y = -8. + offset;
            sprite.color = if sellable { Color::WHITE } else { Color::NONE };
        } else {
            t.translation.y = if t.translation.x == -170. {
                -3. + offset
            } else if t.translation.x == 90. {
                -27. + offset
            } else {
                -8. + offset
            };
        }
    }
}
#[allow(clippy::type_complexity)]
fn hud(
    s: Res<Session>,
    desktop: Res<window::Desktop>,
    mut q: Query<(&mut Text, &mut Visibility), (With<Hud>, Without<Panel>)>,
    mut panel: Query<&mut Visibility, With<Panel>>,
) {
    for (mut t, mut visibility) in &mut q {
        *visibility = if desktop.shell.expanded {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        **t = format!(
            "LITTLE FORGE   Gold {}   Ore {}   Finished {}/{}   {:?}\n{}",
            s.save.state.gold,
            s.save.state.ore,
            s.save.state.showroom.goods(),
            s.save.state.showroom.capacity(),
            desktop.shell.mode,
            s.status
        );
    }
    for mut v in &mut panel {
        *v = Visibility::Hidden;
    }
}
// Publish screenshot files only after the encoder has closed and synced them.
// This also prevents capture consumers from observing a half-written PNG.
fn save_capture(path: impl AsRef<std::path::Path>) -> impl FnMut(On<ScreenshotCaptured>) {
    let path = path.as_ref().to_owned();
    let temporary = path.with_extension("capture.png");
    let mut save = save_to_disk(temporary.clone());
    move |event| {
        save(event);
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&temporary)
            .and_then(|f| f.sync_all())
            .expect("sync screenshot");
        if path.exists() {
            std::fs::remove_file(&path).expect("replace screenshot");
        }
        std::fs::rename(&temporary, &path).expect("publish screenshot");
    }
}
#[allow(clippy::too_many_arguments)]
fn qa_capture(
    mut commands: Commands,
    mut opts: ResMut<Options>,
    server: Res<AssetServer>,
    art: Res<Art>,
    mut desktop: ResMut<window::Desktop>,
    mut exit: MessageWriter<AppExit>,
    mut session: ResMut<Session>,
    mut farmer: ResMut<npc::FarmerPresentation>,
) {
    if opts.qa.is_none() {
        return;
    }
    if art
        .images
        .iter()
        .any(|h| !server.is_loaded_with_dependencies(h.id()))
    {
        return;
    }
    opts.frames += 1;
    if opts.qa.as_deref() == Some("npc") && opts.frames == 170 {
        session.save.state.gold = 100; // isolated QA fixture, never a user save
        let id = session.save.state.command_id + 1;
        let command = Command::Purchase {
            vendor: forge_core::village::Location::Farm,
            item: forge_core::village::Item::Staple,
            quantity: 1,
        };
        session
            .save
            .state
            .command(id, command)
            .expect("QA farmer purchase");
        farmer.accepted(id, command);
    }
    if opts.qa.as_deref() == Some("title")
        && [30, 34, 38, 42, 46, 50].contains(&opts.frames)
        && let Some(path) = &opts.capture
    {
        let extra = format!("{}-{:03}.png", path.trim_end_matches(".png"), opts.frames);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(extra));
    }
    if opts.frames == 1 && opts.qa.as_deref() == Some("idle") {
        desktop.shell.idle();
    }
    if opts.forge_scene().is_some_and(|(scene, _)| scene == "workshop")
        && [45, 100, 140].contains(&opts.frames)
        && let Some(path) = &opts.capture
    {
        // Heating, hammering and quenching stages of the single 5-second job.
        let extra = format!("{}-{:03}.png", path.trim_end_matches(".png"), opts.frames);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(extra));
    }
    let life = opts.qa.as_deref().is_some_and(|q| q.starts_with("life-"));
    if life
        && [150, 330, 390, 1200].contains(&opts.frames)
        && let Some(path) = &opts.capture
    {
        let extra = format!("{}-{}.png", path.trim_end_matches(".png"), opts.frames);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(extra));
    }
    let long = life
        || opts.qa.as_deref() == Some("village-input")
        || opts.forge_scene().is_some_and(|(scene, _)| scene == "input");
    if opts.frames == if long { 1800 } else { 180 }
        && let Some(path) = &opts.capture
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(path.clone()));
    }
    if opts.frames == if long { 1830 } else { 210 } {
        if life && let Some(path) = &opts.capture {
            let report = serde_json::json!({"scenario":opts.qa,"clock":"QA scripted 1 eligible second per 30 rendered frames; production uses monotonic elapsed time", "state":session.save.state,"events":session.trace});
            std::fs::write(
                format!("{path}.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .expect("write QA trace");
            assert!(
                session.save.state.sold >= 2 && session.save.state.command_id == 3,
                "life loop did not complete"
            );
        }
        exit.write(AppExit::Success);
    }
}
