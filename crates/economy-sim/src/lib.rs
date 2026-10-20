//! Headless economy simulator over `forge-core` (#131).
//!
//! Runs scenarios with no Bevy and no window. Every state change goes through
//! `State::command`, `State::advance` or `Save::catch_up`: the same transitions as the game.
//! There are no simulator-only rules. Drivers stand in for the player; future automation
//! policies (#133–#137) will be drivers that issue the same commands.
pub mod driver;
pub mod matrix;
pub mod runner;
pub mod scenario;

pub use matrix::{Matrix, expand, run_all, summary_csv};
pub use runner::{Report, run};
pub use scenario::Scenario;

#[cfg(test)]
mod tests;
