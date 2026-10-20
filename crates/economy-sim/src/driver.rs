//! Drivers decide which commands to try. They read the state and call `exec`, which sends the
//! command through `State::command` exactly like a UI button; a refused command changes nothing.
use crate::scenario::{Driver, Player, parse_command};
use forge_core::{Command, State, commission::OrderPhase, items::Id, life::WorkKind};

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

/// Whether a job making `item` is queued or at the bench.
fn in_progress(s: &State, item: Id) -> bool {
    let book = forge_core::crafting::Book::bundled();
    s.life
        .queue
        .iter()
        .chain(s.life.active_job.iter())
        .any(|j| matches!(j, WorkKind::Recipe(r) if book.recipe(*r).is_some_and(|r| r.output.item == item)))
}

/// Gets `qty` of `item` on hand the way a player would, walking the recipe data: buy it if a
/// vendor sells it and the policy allows, otherwise make it (first obtaining its inputs). Returns
/// true when the stock is already there; false while purchases or jobs are still pending.
fn obtain(p: &Player, s: &mut State, item: Id, qty: u32, exec: &mut Exec, depth: u8) -> bool {
    if s.stock(item) >= qty {
        return true;
    }
    if depth > 6 || in_progress(s, item) {
        return false;
    }
    let book = forge_core::crafting::Book::bundled();
    if let Some(recipe) = book.recipes.iter().find(|r| r.output.item == item) {
        let mut ready = true;
        for input in &recipe.inputs {
            ready &= obtain(p, s, input.item, input.qty, exec, depth + 1);
        }
        if ready {
            exec(s, Command::QueueRecipe(recipe.id));
        }
        return false;
    }
    let missing = qty - s.stock(item);
    let offer = book.vendors.iter().find_map(|v| {
        v.offers
            .iter()
            .find(|o| o.item == item)
            .map(|o| (v.location, o.price))
    });
    if let Some((vendor, price)) = offer
        && p.buy_materials
        && affordable(p, s, price * u64::from(missing))
    {
        exec(
            s,
            Command::Buy {
                vendor,
                item,
                quantity: missing,
            },
        );
    }
    s.stock(item) >= qty
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
            // Schema 8: the Chef's knife is assembled from a blade and a handle.
            let book = forge_core::crafting::Book::bundled();
            let recipe = book.recipe(book.chef_order_recipe).expect("chef recipe");
            let mut ready = true;
            for input in &recipe.inputs {
                ready &= obtain(p, s, input.item, input.qty, exec, 0);
            }
            if ready {
                exec(s, Command::QueueChef);
            }
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
    for good in &p.make {
        // Keep one of each listed good in stock (kitchen knives, clubs, pans...).
        if let Ok(id) = Id::parse(good)
            && s.stock(id) < p.make_target
        {
            obtain(p, s, id, s.stock(id) + 1, exec, 0);
        }
    }
    if p.craft && s.progression.dagger_unlocked() {
        // Bounded: at most `max_queue` attempts per pass.
        for _ in 0..p.max_queue {
            if s.life.queue.len() >= p.max_queue {
                break;
            }
            if s.ore() < s.dagger_ore()
                && !(p.buy_ore && affordable(p, s, ORE_PRICE) && exec(s, Command::BuyOre))
            {
                break;
            }
            if s.ore() < s.dagger_ore() || !exec(s, Command::QueueCraft) {
                break;
            }
        }
    }
}
