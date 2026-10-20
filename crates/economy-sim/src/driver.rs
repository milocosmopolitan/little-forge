//! Drivers decide which commands to try. They read the state and call `exec`, which sends the
//! command through `State::command` exactly like a UI button; a refused command changes nothing.
use crate::scenario::{Driver, Player, parse_command};
use forge_core::{Command, State, commission::OrderPhase};

pub type Exec<'a> = dyn FnMut(&mut State, Command) -> bool + 'a;

pub fn act(driver: &Driver, s: &mut State, wall: u64, exec: &mut Exec) {
    match driver {
        Driver::Idle => {}
        Driver::Script { commands } => {
            for c in commands.iter().filter(|c| c.at == wall) {
                exec(s, parse_command(&c.command).expect("checked when loaded"));
            }
        }
        Driver::Player(p) => {
            if wall.is_multiple_of(p.act_every_secs) {
                player(p, s, exec)
            }
        }
    }
}

/// Can the player afford `cost` and keep the reserve?
fn affordable(p: &Player, s: &State, cost: u64) -> bool {
    s.gold >= cost.saturating_add(p.gold_reserve)
}

/// One attention pass of a diligent player: deliver, take and queue orders, follow research,
/// keep food, then keep the bench busy with daggers. Ore is bought only to cover a real need.
fn player(p: &Player, s: &mut State, exec: &mut Exec) {
    // Ore purchases go through the same command as the UI's "Buy 2 ore · 4 gold" button.
    const ORE_PRICE: u64 = 4;
    let buy_ore_then = |s: &mut State, exec: &mut Exec, cmd: Command| {
        if exec(s, cmd) {
            return true;
        }
        if p.buy_ore && affordable(p, s, ORE_PRICE) && exec(s, Command::BuyOre) {
            return exec(s, cmd);
        }
        false
    };
    if p.orders {
        if s.progression.chef_phase == OrderPhase::Ready {
            exec(s, Command::DeliverChef);
        }
        if s.farmer_order.phase == OrderPhase::Ready {
            exec(s, Command::DeliverFarmerRepair);
        }
        if s.progression.chef_phase == OrderPhase::Offered {
            exec(s, Command::AcceptChef);
        }
        if s.progression.chef_phase == OrderPhase::Accepted {
            buy_ore_then(s, exec, Command::QueueChef);
        }
        if s.farmer_order.phase == OrderPhase::Offered {
            exec(s, Command::AcceptFarmerRepair);
        }
        if s.farmer_order.phase == OrderPhase::Accepted {
            buy_ore_then(s, exec, Command::QueueFarmerRepair);
        }
    }
    if p.research && s.progression.discovered() && !s.progression.research_completed {
        if s.progression.can_complete() {
            exec(s, Command::CompleteDaggerResearch);
        } else if !s.progression.study_pending {
            let cost = s
                .progression
                .rules
                .as_ref()
                .map_or(0, |r| r.dagger.study_gold);
            if affordable(p, s, cost) {
                exec(s, Command::StudyDagger);
            }
        }
    }
    if p.buy_food && s.life.pantry.staples() == 0 && affordable(p, s, s.life.config.food_price) {
        exec(s, Command::BuyFood);
    }
    if p.craft && s.progression.dagger_unlocked() {
        // Bounded: at most `max_queue` attempts per pass.
        for _ in 0..p.max_queue {
            if s.life.queue.len() >= p.max_queue {
                break;
            }
            if s.ore < s.dagger_ore()
                && !(p.buy_ore && affordable(p, s, ORE_PRICE) && exec(s, Command::BuyOre))
            {
                break;
            }
            if s.ore < s.dagger_ore() || !exec(s, Command::QueueCraft) {
                break;
            }
        }
    }
}
