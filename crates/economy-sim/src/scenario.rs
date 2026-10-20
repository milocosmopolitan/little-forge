//! Scenario files: where a run starts, how long it lasts, who acts, and when the player is away.
use forge_core::{Command, Smith, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub seed: u64,
    /// Wall-clock seconds simulated, including time spent offline.
    pub duration_secs: u64,
    /// Online step: the driver acts, then the state advances this many seconds.
    #[serde(default = "one")]
    pub step_secs: u64,
    /// Metrics sample interval (wall clock).
    #[serde(default = "ten_minutes")]
    pub sample_every_secs: u64,
    #[serde(default)]
    pub start: Start,
    #[serde(default)]
    pub driver: Driver,
    /// The player closes the game at `at` (wall clock) for `secs`; the game settles it on return
    /// with the same capped `Save::catch_up` the desktop uses.
    #[serde(default)]
    pub offline: Vec<Offline>,
}
fn one() -> u64 {
    1
}
fn ten_minutes() -> u64 {
    600
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Offline {
    pub at: u64,
    pub secs: u64,
}

/// Starting state. Presets are built with the bundled content and real commands only; the
/// optional overrides set the starting purse and stock for sensitivity runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Start {
    #[serde(default)]
    pub preset: Preset,
    pub gold: Option<u64>,
    pub ore: Option<u32>,
    pub staples: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    /// A new game as the desktop creates it: bundled progression and needs, male smith chosen.
    #[default]
    Fresh,
    /// The legacy dagger loop (recipe known from the start), as in forge-qa's craft-and-sale.
    LegacyDagger,
    /// `forge_core::weapon::qa_smith()`: Chef delivered, dagger research done, bow and arrows made.
    QaSmith,
}

/// Who issues commands. None of these is game automation: they stand in for a player so the
/// economy can be measured. Automation policies (#133–#137) will be added as their own drivers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Driver {
    /// Nobody touches the game.
    #[default]
    Idle,
    /// Fixed commands at fixed wall-clock seconds.
    Script { commands: Vec<Timed> },
    /// A diligent scripted player with the toggles below.
    Player(Player),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Timed {
    pub at: u64,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Player {
    /// Accept, queue and deliver the Chef and Farmer orders.
    #[serde(default = "yes")]
    pub orders: bool,
    /// Study and complete dagger research when it becomes available.
    #[serde(default = "yes")]
    pub research: bool,
    /// Queue iron daggers while ore allows.
    #[serde(default = "yes")]
    pub craft: bool,
    /// Buy ore when it runs short, keeping `gold_reserve`.
    #[serde(default = "yes")]
    pub buy_ore: bool,
    /// Buy staples when the pantry is empty.
    #[serde(default = "yes")]
    pub buy_food: bool,
    #[serde(default = "two")]
    pub max_queue: usize,
    #[serde(default)]
    pub gold_reserve: u64,
    /// The player only acts every N wall seconds (attention span); 1 = every step.
    #[serde(default = "one")]
    pub act_every_secs: u64,
}
fn yes() -> bool {
    true
}
fn two() -> usize {
    2
}
impl Default for Player {
    fn default() -> Self {
        Self {
            orders: true,
            research: true,
            craft: true,
            buy_ore: true,
            buy_food: true,
            max_queue: 2,
            gold_reserve: 0,
            act_every_secs: 1,
        }
    }
}

/// Unit commands a script may name. Parameterised commands are not scriptable yet.
pub fn parse_command(name: &str) -> Result<Command, String> {
    Ok(match name {
        "Craft" => Command::Craft,
        "QueueCraft" => Command::QueueCraft,
        "BuyOre" => Command::BuyOre,
        "AcceptChef" => Command::AcceptChef,
        "QueueChef" => Command::QueueChef,
        "DeliverChef" => Command::DeliverChef,
        "StudyDagger" => Command::StudyDagger,
        "CompleteDaggerResearch" => Command::CompleteDaggerResearch,
        "AcceptFarmerRepair" => Command::AcceptFarmerRepair,
        "QueueFarmerRepair" => Command::QueueFarmerRepair,
        "DeliverFarmerRepair" => Command::DeliverFarmerRepair,
        "BuyFood" => Command::BuyFood,
        "Eat" => Command::Eat,
        "EatMeal" => Command::EatMeal,
        "Drink" => Command::Drink,
        "Rest" => Command::Rest,
        "RecallExpedition" => Command::RecallExpedition,
        "BuyRack" => Command::BuyRack,
        other => return Err(format!("unknown or non-scriptable command {other:?}")),
    })
}

/// Stable short name for a command, used as a ledger key.
pub fn command_name(c: &Command) -> String {
    let s = format!("{c:?}");
    s.split(['(', ' ', '{']).next().unwrap_or(&s).to_string()
}

impl Start {
    pub fn build(&self, seed: u64) -> Result<State, String> {
        let mut s = match self.preset {
            Preset::QaSmith => forge_core::weapon::qa_smith(),
            Preset::Fresh | Preset::LegacyDagger => {
                let mut s = State::with_progression(seed, forge_content::progression_rules()?)
                    .map_err(|e| format!("{e:?}"))?;
                s.life.config = forge_content::needs_config()?;
                s.progression.legacy_dagger = self.preset == Preset::LegacyDagger;
                s.command(1, Command::ChooseSmith(Smith::Male))
                    .map_err(|e| format!("{e:?}"))?;
                s
            }
        };
        if self.preset == Preset::QaSmith {
            s.rng = seed.max(1);
        }
        if let Some(g) = self.gold {
            s.gold = g;
        }
        if let Some(o) = self.ore {
            s.ore = o;
        }
        if let Some(n) = self.staples {
            let have = s.life.pantry.staples();
            if n > have {
                let limit = s.life.config.stack_limit;
                s.life
                    .pantry
                    .add(forge_core::life::Food::Staple, n - have, limit)
                    .map_err(|e| format!("{e:?}"))?;
            }
        }
        s.validate()
            .map_err(|e| format!("start state invalid: {e:?}"))?;
        Ok(s)
    }
}

impl Scenario {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let s: Scenario = serde_json::from_str(text).map_err(|e| e.to_string())?;
        s.check()?;
        Ok(s)
    }
    pub fn check(&self) -> Result<(), String> {
        if self.step_secs == 0 || self.sample_every_secs == 0 {
            return Err("step_secs and sample_every_secs must be > 0".into());
        }
        if let Driver::Script { commands } = &self.driver {
            for c in commands {
                parse_command(&c.command)?;
            }
        }
        if let Driver::Player(p) = &self.driver
            && p.act_every_secs == 0
        {
            return Err("act_every_secs must be > 0".into());
        }
        Ok(())
    }
}
