//! Thin game adapter: only accepted live commands request presentation.
use crate::{Options, window::Desktop};
use bevy::prelude::*;
use forge_npc::{Catalog, NpcPlugin, Player};
#[derive(Resource, Default)]
pub struct FarmerPresentation {
    pub last: u64,
    pub pending: bool,
    pub elapsed: f32,
    pub active: bool,
}
impl FarmerPresentation {
    pub fn accepted(&mut self, id: u64, command: forge_core::Command) {
        if id > self.last {
            self.last = id;
            if matches!(command, forge_core::Command::BuyFood) {
                self.pending = true;
            }
        }
    }
}
#[derive(Component)]
struct Farmer;
#[derive(Component)]
struct Produce;
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
    let owner = commands
        .spawn((
            Sprite {
                image: server.load(&c.image),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.add(c.layout()),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(c.cell[0] as f32 / c.cell[1] as f32 * 95., 95.)),
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
#[allow(clippy::type_complexity)]
fn update(
    time: Res<Time>,
    c: Res<Catalog>,
    opts: Res<Options>,
    mut clock: ResMut<forge_npc::PlaybackClock>,
    desktop: Res<Desktop>,
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
        p.play("give", &c);
        p.item = true;
    }
    if state.active {
        state.elapsed += if opts.qa.is_some() {
            1. / 30.
        } else {
            time.delta_secs()
        };
        if state.elapsed > 2. {
            state.active = false;
        }
    }
    t.translation.y = -85. + if desktop.shell.expanded { -180. } else { 0. };
    *v = if state.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
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
                smith_sizes: vec![],
                visitor_sizes: vec![],
                visitor_hands: vec![],
            })
            .insert_resource(crate::presentation::Presentation::default())
            .insert_resource(Desktop::new(false, false))
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
