use bevy::prelude::*;
use forge_core::{Event, EventKind};
use std::collections::VecDeque;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Enter,
    Walk,
    Inspect,
    Purchase,
    ExitWithItem,
}
#[derive(Resource, Default)]
pub struct Presentation {
    pub queue: VecDeque<Event>,
    pub last_seen: u64,
    pub phase: Phase,
    pub elapsed: f32,
    pub active: bool,
    pub has_item: bool,
}
impl Presentation {
    pub fn accept(&mut self, event: Event) {
        if event.id > self.last_seen {
            self.last_seen = event.id;
            if matches!(event.kind, EventKind::Sold { .. }) {
                self.queue.push_back(event);
            }
        }
    }
    pub fn advance(&mut self, dt: f32) {
        if !self.active {
            if self.queue.pop_front().is_none() {
                return;
            }
            self.active = true;
            self.elapsed = 0.;
        }
        self.elapsed += dt;
        self.phase = if self.elapsed < 0.5 {
            Phase::Enter
        } else if self.elapsed < 3. {
            Phase::Walk
        } else if self.elapsed < 4. {
            Phase::Inspect
        } else if self.elapsed < 4.5 {
            Phase::Purchase
        } else {
            Phase::ExitWithItem
        };
        self.has_item = self.phase == Phase::ExitWithItem;
        if self.elapsed >= 8. {
            self.active = false;
            self.has_item = false;
        }
    }
    pub fn x(&self) -> f32 {
        if self.elapsed < 3. {
            -430. + self.elapsed / 3. * 430.
        } else if self.elapsed < 4.5 {
            0.
        } else {
            (self.elapsed - 4.5) / 3.5 * 510.
        }
    }
}
pub fn tick(time: Res<Time>, opts: Option<Res<crate::Options>>, mut p: ResMut<Presentation>) {
    let dt = match opts.as_deref() {
        Some(o) if o.qa.is_some() => {
            if o.frames > 0 {
                1. / 30.
            } else {
                0.
            }
        }
        _ => time.delta_secs(),
    };
    p.advance(dt);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn headless_sale_fsm_deduplicates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Presentation>()
            .add_systems(Update, tick);
        let event = Event {
            id: 1,
            tick: 12,
            kind: EventKind::Sold {
                item: "iron_dagger".into(),
                gold: 12,
            },
        };
        let mut p = app.world_mut().resource_mut::<Presentation>();
        p.accept(event.clone());
        p.accept(event);
        assert_eq!(p.queue.len(), 1);
        p.advance(3.5);
        assert_eq!(p.phase, Phase::Inspect);
        p.advance(1.5);
        assert!(p.has_item);
        p.advance(4.);
        assert!(!p.active);
        assert!(p.queue.is_empty());
        app.update();
    }
}
