//! Pure desktop policy; native adapters report physical work areas, never assume origin zero.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn fit(self, work: Self, height: f32) -> Self {
        let w = self.w.min(work.w).max(1.);
        let h = height.min(work.h).max(1.);
        Self {
            x: self.x.clamp(work.x, work.x + work.w - w),
            y: (self.y + self.h - h).clamp(work.y, work.y + work.h - h),
            w,
            h,
        }
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Interaction,
    Idle,
}
#[derive(Debug, Default)]
pub struct Shell {
    pub mode: Mode,
    pub expanded: bool,
    pub safe: bool,
    pub always_on_top: bool,
    pub lease_left: f32,
}
impl Shell {
    /// Timed pass-through lease guarantees recovery even when the window cannot receive keys.
    pub fn idle(&mut self) {
        if !self.safe {
            self.mode = Mode::Idle;
            self.expanded = false;
            self.lease_left = 10.;
        }
    }
    pub fn recover(&mut self) {
        self.mode = Mode::Interaction;
        self.lease_left = 0.;
    }
    pub fn tick(&mut self, seconds: f32) {
        if self.mode == Mode::Idle {
            self.lease_left -= seconds;
            if self.lease_left <= 0. {
                self.recover();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expansion_preserves_bottom_and_clamps() {
        let w = Rect {
            x: -1920.,
            y: 0.,
            w: 1920.,
            h: 1040.,
        };
        let r = Rect {
            x: -960.,
            y: 860.,
            w: 960.,
            h: 180.,
        };
        let e = r.fit(w, 540.);
        assert_eq!(e.y, 500.);
        assert_eq!(e.fit(w, 180.), r);
        assert_eq!(
            r.fit(
                Rect {
                    x: 0.,
                    y: 0.,
                    w: 800.,
                    h: 600.
                },
                540.
            )
            .x,
            0.
        );
    }
    #[test]
    fn unfocused_recovery_and_safe_mode() {
        let mut s = Shell::default();
        s.idle();
        s.tick(11.);
        assert_eq!(s.mode, Mode::Interaction);
        s.safe = true;
        s.idle();
        assert_eq!(s.mode, Mode::Interaction);
    }
}
