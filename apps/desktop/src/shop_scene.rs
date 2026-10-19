//! Read-only projection of existing showroom slots and already-settled sale events.
use crate::{Session, spaces::Spaces, title::TitleScreen};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use forge_core::{craft::Good, showroom::Slot, spaces::Space};

/// UI icons are 256px sources; in the 1:1 scene a held or displayed good is this many pixels.
const ICON: f32 = 20.;
/// Empty pegs on the wall display shelf (scene pixels, item centres): three on the top shelf,
/// one on the lower-left hook. Matches assets/forge/atlas-v2/display-shelf.png at its layout.
const SHELF_SLOTS: [(f32, f32); 4] = [(183., 41.), (201., 41.), (219., 41.), (121., 71.)];
const SHELF_ICON: f32 = 14.;

/// Stable preview indices; storage capacity is exclusively owned by forge-core.
#[derive(Component)]
pub struct DisplaySlot(pub usize);

fn displayed(slots: &[Slot], index: usize) -> Option<Good> {
    match slots.get(index) {
        Some(Slot::Displayed(good)) => Some(*good),
        _ => None,
    }
}
fn setup(mut c: Commands, server: Res<AssetServer>) {
    for (index, (x, y)) in SHELF_SLOTS.into_iter().enumerate() {
        c.spawn((
            DisplaySlot(index),
            Sprite {
                image: server.load("ui/runtime/icons/dagger.png"),
                custom_size: Some(Vec2::splat(SHELF_ICON)),
                ..default()
            },
            // Just above the wall shelf (floor y=101) and behind anyone walking past it.
            Transform::from_xyz(x - 480., 90. - y, crate::interior::depth(101.) + 0.001),
            Visibility::Hidden,
            RenderLayers::layer(1),
        ));
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn project(
    mut c: Commands,
    session: Res<Session>,
    title: Res<TitleScreen>,
    spaces: Res<Spaces>,
    sale: Res<crate::Presentation>,
    art: Res<crate::Art>,
    server: Res<AssetServer>,
    mut slots: Query<
        (&DisplaySlot, &mut Visibility),
        (Without<crate::Visitor>, Without<crate::Weapon>),
    >,
    mut customers: Query<
        (
            Entity,
            &mut Transform,
            &mut Sprite,
            &mut Visibility,
            Has<crate::Weapon>,
        ),
        (
            Or<(With<crate::Visitor>, With<crate::Weapon>)>,
            Without<DisplaySlot>,
        ),
    >,
) {
    let visible =
        !title.active && session.save.state.smith.is_some() && spaces.space != Space::Exterior;
    for (slot, mut v) in &mut slots {
        *v = if visible && displayed(&session.save.state.showroom.slots, slot.0).is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    // Reuse the existing customer and hand registration, not a second NPC instance.
    // A sale has already committed before this reenactment. No gold or slots are written here.
    let x = if sale.elapsed < 3. {
        -430. + (sale.elapsed / 3.).clamp(0., 1.) * 110.
    } else if sale.elapsed < 4.5 {
        -320.
    } else {
        -320. - ((sale.elapsed - 4.5) / 3.5).clamp(0., 1.) * 110.
    };
    let frame = ((sale.elapsed * 6.) as usize) % 4;
    for (entity, mut t, mut sprite, mut v, weapon) in &mut customers {
        c.entity(entity)
            .insert(RenderLayers::layer(if spaces.space == Space::Exterior {
                0
            } else {
                1
            }));
        if spaces.space == Space::Exterior {
            continue;
        }
        let supported = sale.item.as_deref() == Some(Good::IronDagger.id());
        *v = if visible && sale.active && (!weapon || sale.has_item && supported) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if weapon {
            sprite.image = server.load("ui/runtime/icons/dagger.png");
            sprite.custom_size = Some(Vec2::splat(ICON));
            t.translation = Vec3::new(
                x + art.visitor_hands[frame].x * if sale.elapsed >= 4.5 { -1. } else { 1. },
                -78. + art.visitor_hands[frame].y + 8.,
                5.,
            );
        } else {
            sprite.flip_x = sale.elapsed >= 4.5;
            // Foot y=168: the customer stands in front of the counter (floor y=160).
            t.translation = Vec3::new(x, -78., 4.5);
        }
    }
}
pub fn install(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
        Update,
        project.after(crate::animate).after(crate::spaces::route),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_keeps_slot_identity_and_never_displays_withheld_goods() {
        let slots = [
            Slot::Held(Good::IronDagger),
            Slot::Reserved,
            Slot::Displayed(Good::IronDagger),
            Slot::Empty,
        ];
        assert_eq!(displayed(&slots, 0), None);
        assert_eq!(displayed(&slots, 1), None);
        assert_eq!(displayed(&slots, 2), Some(Good::IronDagger));
        assert_eq!(displayed(&slots, 3), None);
        assert_eq!(displayed(&slots, 4), None);
    }
}
