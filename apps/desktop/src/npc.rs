//! Thin game adapter: only accepted live commands request presentation.
use crate::Options;
use bevy::prelude::*;
use forge_npc::{Catalog, NpcPlugin, Player};
#[derive(Resource, Default)]
pub struct FarmerPresentation {
    pub last: u64,
    pub pending: bool,
    pub elapsed: f32,
    pub active: bool,
    pub reaction: Option<forge_core::Command>,
}
impl FarmerPresentation {
    pub fn accepted(&mut self, id: u64, command: forge_core::Command) {
        if id > self.last {
            self.last = id;
            if matches!(
                command,
                forge_core::Command::AcceptFarmerRepair | forge_core::Command::DeliverFarmerRepair
            ) {
                self.pending = true;
                self.reaction = Some(command);
            }
            if matches!(
                command,
                forge_core::Command::BuyFood
                    | forge_core::Command::Purchase {
                        vendor: forge_core::village::Location::Farm,
                        item: forge_core::village::Item::Staple,
                        ..
                    }
            ) {
                self.pending = true;
                self.reaction = Some(command);
            }
        }
    }
}
#[derive(Component)]
pub(crate) struct Farmer;
#[derive(Component)]
pub(crate) struct Produce;
pub struct FarmerPlugin;
impl Plugin for FarmerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NpcPlugin)
            .init_resource::<FarmerPresentation>()
            .add_systems(Startup, setup)
            .add_systems(Update, update.before(forge_npc::animate));
    }
}
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    c: Res<Catalog>,
) {
    // Keep the authored pixel scale when the animation canvas gains padding.
    let height = 95. * c.cell[1] as f32 / 128.;
    let owner = commands
        .spawn((
            Sprite {
                image: server.load(&c.image),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.add(c.layout()),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(
                    c.cell[0] as f32 / c.cell[1] as f32 * height,
                    height,
                )),
                ..default()
            },
            Transform::from_xyz(270., -85., 5.),
            c.anchor(),
            Visibility::Hidden,
            Player::default(),
            Farmer,
        ))
        .id();
    commands.spawn((
        Sprite {
            image: server.load("npc/farmer/carrot.png"),
            custom_size: Some(Vec2::new(14., 22.)),
            ..default()
        },
        Transform::default(),
        Visibility::Hidden,
        Produce,
        forge_npc::ItemAttachment {
            owner,
            enabled: true,
        },
    ));
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update(
    time: Res<Time>,
    c: Res<Catalog>,
    opts: Res<Options>,
    mut clock: ResMut<forge_npc::PlaybackClock>,
    session: Res<crate::Session>,
    title: Res<crate::title::TitleScreen>,
    village: Res<crate::village::Village>,

    spaces: Option<Res<crate::spaces::Spaces>>,
    mut state: ResMut<FarmerPresentation>,
    mut farmer: Query<
        (&mut Player, &mut Transform, &mut Visibility),
        (With<Farmer>, Without<Produce>),
    >,
) {
    let Ok((mut p, mut t, mut v)) = farmer.single_mut() else {
        return;
    };
    clock.fixed_delta = opts.qa.as_ref().map(|_| 1. / 30.);
    if state.pending {
        state.pending = false;
        state.active = true;
        state.elapsed = 0.;
        let action = match state.reaction {
            Some(forge_core::Command::AcceptFarmerRepair) => "nod",
            Some(forge_core::Command::DeliverFarmerRepair) => "happy",
            _ => "give",
        };
        p.play(action, &c);
        p.item = action == "give";
    }
    if state.active {
        state.elapsed += if opts.qa.is_some() {
            1. / 30.
        } else {
            time.delta_secs()
        };
        if state.elapsed > 2. {
            state.active = false;
            p.play("idle", &c);
            p.item = false;
        }
    }
    let outside = spaces
        .as_ref()
        .is_some_and(|s| s.space == forge_core::spaces::Space::Exterior);
    t.translation.x = if outside { 270. } else { -250. };
    t.translation.y = if outside { -85. } else { -66. };
    let waiting = matches!(
        session.save.state.farmer_order.phase,
        forge_core::commission::OrderPhase::Offered | forge_core::commission::OrderPhase::Ready
    );
    let order_scene =
        opts.qa.is_none() || opts.qa.as_deref().is_some_and(|q| q.starts_with("repair-"));
    *v = if !title.active
        && (state.active
            || (order_scene && waiting && village.nav.screen == forge_core::village::Screen::Home))
    {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}
#[allow(clippy::type_complexity)]
pub fn render_layers(
    mut commands: Commands,
    spaces: Res<crate::spaces::Spaces>,
    q: Query<
        (Entity, Option<&bevy::camera::visibility::RenderLayers>),
        Or<(With<Farmer>, With<Produce>, With<forge_npc::NpcVisual>)>,
    >,
) {
    let layer = bevy::camera::visibility::RenderLayers::layer(
        if spaces.space == forge_core::spaces::Space::Exterior {
            0
        } else {
            1
        },
    );
    for (e, current) in &q {
        if current != Some(&layer) {
            commands.entity(e).insert(layer.clone());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_scene_animation_does_not_move_npc_layers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(crate::Options::default())
            .insert_resource(crate::Art {
                images: vec![],
                visitor_sizes: vec![],
                visitor_hands: vec![],
                building_active: Handle::default(),
                building_idle: Handle::default(),
            })
            .insert_resource(crate::presentation::Presentation::default())
            .insert_resource(crate::window::Desktop::new(false, false))
            .add_systems(Update, crate::animate);
        let body = app
            .world_mut()
            .spawn((
                Sprite::default(),
                Transform::from_xyz(270., -85., 5.),
                Visibility::Visible,
                Player::default(),
            ))
            .id();
        let layer = app
            .world_mut()
            .spawn((
                Sprite::default(),
                Transform::from_xyz(270., -85., 5.1),
                Visibility::Visible,
                forge_npc::NpcVisual,
            ))
            .id();
        app.update();
        for e in [body, layer] {
            assert_eq!(app.world().get::<Transform>(e).unwrap().translation.y, -85.);
        }
    }
    #[test]
    fn rejected_and_replayed_commands_cannot_trigger_or_change_inventory() {
        let mut domain = forge_core::State::new(42);
        let mut view = FarmerPresentation::default();
        let before = domain.life.pantry.staples();
        if domain.command(1, forge_core::Command::BuyFood).is_ok() {
            view.accepted(1, forge_core::Command::BuyFood);
        }
        assert!(!view.pending);
        assert_eq!(domain.life.pantry.staples(), before);
        domain.gold = 100;
        domain.command(1, forge_core::Command::BuyFood).unwrap();
        view.accepted(1, forge_core::Command::BuyFood);
        assert!(view.pending);
        view.pending = false;
        view.accepted(1, forge_core::Command::BuyFood);
        assert!(!view.pending);
        let gold = domain.gold;
        let count = domain.life.pantry.staples();
        let c = Catalog::farmer();
        let mut p = Player::default();
        p.play("give", &c);
        p.advance(100., &c);
        p.play("idle", &c);
        assert_eq!(domain.gold, gold);
        assert_eq!(domain.life.pantry.staples(), count);
    }
}
