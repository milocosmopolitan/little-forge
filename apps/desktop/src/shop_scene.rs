//! Read-only projection of existing showroom slots and already-settled sale events.
use crate::{Session, spaces::Spaces, title::TitleScreen};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use forge_core::{craft::Good, showroom::Slot, spaces::Space};

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
    for index in 0..4 {
        c.spawn((
            DisplaySlot(index),
            Sprite::from_image(server.load("ui/runtime/icons/dagger.png")),
            Transform::from_xyz(-296. + index as f32 * 18., -30., 1.3),
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
        -430. + (sale.elapsed / 3.).clamp(0., 1.) * 155.
    } else if sale.elapsed < 4.5 {
        -275.
    } else {
        -275. - ((sale.elapsed - 4.5) / 3.5).clamp(0., 1.) * 155.
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
            t.translation = Vec3::new(
                x + art.visitor_hands[frame].x * if sale.elapsed >= 4.5 { -1. } else { 1. },
                -66. + art.visitor_hands[frame].y + 8.,
                5.,
            );
        } else {
            sprite.flip_x = sale.elapsed >= 4.5;
            t.translation = Vec3::new(x, -66., 4.5);
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
