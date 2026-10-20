//! Runs one scenario on `forge_core::State` and measures it. Every state change goes through
//! `State::command` / `State::advance` / `Save::catch_up`; the runner only observes.
use crate::driver;
use crate::scenario::{Scenario, command_name};
use forge_core::{Command, EventKind, State, save::Save};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Sample {
    pub wall: u64,
    pub tick: u64,
    pub gold: u64,
    pub ore: u32,
    pub daggers: u32,
    pub crafted: u32,
    pub sold: u32,
    pub timber: u32,
    pub leather: u32,
    pub staples: u32,
    pub queue: usize,
    pub working: bool,
    pub fatigue: u32,
    pub hunger: u32,
    pub thirst: u32,
    pub chef: String,
    pub farmer: String,
    pub research_xp: u64,
    pub weapons: usize,
}
impl Sample {
    pub fn of(s: &State, wall: u64) -> Self {
        Self {
            wall,
            tick: s.tick,
            gold: s.gold,
            ore: s.ore,
            daggers: s.daggers,
            crafted: s.crafted,
            sold: s.sold,
            timber: s.commerce.timber,
            leather: s.commerce.leather,
            staples: s.life.pantry.staples(),
            queue: s.life.queue.len(),
            working: s.life.active_job.is_some(),
            fatigue: s.life.fatigue,
            hunger: s.life.hunger,
            thirst: s.life.thirst,
            chef: format!("{:?}", s.progression.chef_phase),
            farmer: format!("{:?}", s.farmer_order.phase),
            research_xp: s.progression.research_xp,
            weapons: s.expedition.weapons.len(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Tally {
    pub ok: u64,
    pub refused: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OfflineResult {
    pub at: u64,
    pub requested: u64,
    pub settled: u64,
    /// Seconds beyond the offline cap that the game does not credit.
    pub uncredited: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Failure {
    pub wall: u64,
    pub tick: u64,
    pub check: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Report {
    pub scenario: String,
    pub seed: u64,
    pub wall_secs: u64,
    pub start: Sample,
    pub end: Sample,
    pub samples: Vec<Sample>,
    /// Gold change by source: `cmd:<Command>` for commands, `sale` for visitor sales during
    /// `advance`, `advance:other` for any other gold change inside `advance`.
    pub ledger: BTreeMap<String, i64>,
    pub commands: BTreeMap<String, Tally>,
    pub events: BTreeMap<String, u64>,
    /// Seconds with jobs queued but none at the bench (rest, meals, blocked slots...).
    pub queue_waiting_secs: u64,
    pub longest_queue_wait_secs: u64,
    pub offline: Vec<OfflineResult>,
    pub failures: Vec<Failure>,
}

fn delta(before: u64, after: u64) -> i64 {
    after as i64 - before as i64
}

fn event_name(k: &EventKind) -> String {
    let s = format!("{k:?}");
    s.split(['(', ' ', '{']).next().unwrap_or(&s).to_string()
}

struct Run {
    report: Report,
    deliveries: BTreeMap<&'static str, u64>,
    wait_streak: u64,
    prev: (u64, u32, u32, u64, u64),
}

impl Run {
    fn fail(&mut self, s: &State, wall: u64, check: &str, detail: impl Into<String>) {
        self.report.failures.push(Failure {
            wall,
            tick: s.tick,
            check: check.into(),
            detail: detail.into(),
        });
    }

    /// Invariants that must hold after every step of every run.
    fn check(&mut self, s: &State, wall: u64) {
        if let Err(e) = s.validate() {
            self.fail(s, wall, "validate", format!("{e:?}"));
        }
        let now = (s.tick, s.crafted, s.sold, s.event_id, s.command_id);
        let names = ["tick", "crafted", "sold", "event_id", "command_id"];
        let a = [
            self.prev.0,
            self.prev.1 as u64,
            self.prev.2 as u64,
            self.prev.3,
            self.prev.4,
        ];
        let b = [now.0, now.1 as u64, now.2 as u64, now.3, now.4];
        for i in 0..5 {
            if b[i] < a[i] {
                self.fail(
                    s,
                    wall,
                    "monotonic",
                    format!("{} went {} -> {}", names[i], a[i], b[i]),
                );
            }
        }
        self.prev = now;
        let repeated: Vec<String> = self
            .deliveries
            .iter()
            .filter(|(_, n)| **n > 1)
            .map(|(order, n)| format!("{order} delivered {n} times"))
            .collect();
        for detail in repeated {
            self.fail(s, wall, "single-reward", detail);
        }
    }

    fn exec(&mut self, s: &mut State, c: Command) -> bool {
        let name = command_name(&c);
        let gold = s.gold;
        let ok = s.command(s.command_id + 1, c).is_ok();
        let tally = self.report.commands.entry(name.clone()).or_default();
        if ok {
            tally.ok += 1;
            *self.report.ledger.entry(format!("cmd:{name}")).or_default() += delta(gold, s.gold);
            match c {
                Command::DeliverChef => *self.deliveries.entry("chef").or_default() += 1,
                Command::DeliverFarmerRepair => *self.deliveries.entry("farmer").or_default() += 1,
                _ => {}
            }
        } else {
            tally.refused += 1;
        }
        ok
    }

    fn advance(&mut self, s: &mut State, wall: u64, secs: u64) -> bool {
        let gold = s.gold;
        match s.advance(secs) {
            Ok(events) => {
                self.settle(gold, s, &events);
                true
            }
            Err(e) => {
                self.fail(s, wall, "advance", format!("{e:?}"));
                false
            }
        }
    }

    fn settle(&mut self, gold_before: u64, s: &State, events: &[forge_core::Event]) {
        let mut sales = 0i64;
        for e in events {
            *self.report.events.entry(event_name(&e.kind)).or_default() += 1;
            if let EventKind::Sold { gold, .. } = e.kind {
                sales += gold as i64;
            }
        }
        if sales != 0 {
            *self.report.ledger.entry("sale".into()).or_default() += sales;
        }
        let other = delta(gold_before, s.gold) - sales;
        if other != 0 {
            *self
                .report
                .ledger
                .entry("advance:other".into())
                .or_default() += other;
        }
    }

    fn watch_queue(&mut self, s: &State, secs: u64) {
        if !s.life.queue.is_empty() && s.life.active_job.is_none() {
            self.report.queue_waiting_secs += secs;
            self.wait_streak += secs;
            self.report.longest_queue_wait_secs =
                self.report.longest_queue_wait_secs.max(self.wait_streak);
        } else {
            self.wait_streak = 0;
        }
    }
}

/// Simulates `sc` and returns its report. Never panics on game errors: they become failures.
pub fn run(sc: &Scenario) -> Result<Report, String> {
    sc.check()?;
    let mut s = sc.start.build(sc.seed)?;
    let start = Sample::of(&s, 0);
    let mut r = Run {
        report: Report {
            scenario: sc.name.clone(),
            seed: sc.seed,
            wall_secs: sc.duration_secs,
            start: start.clone(),
            end: start.clone(),
            samples: vec![start],
            ledger: BTreeMap::new(),
            commands: BTreeMap::new(),
            events: BTreeMap::new(),
            queue_waiting_secs: 0,
            longest_queue_wait_secs: 0,
            offline: vec![],
            failures: vec![],
        },
        deliveries: BTreeMap::new(),
        wait_streak: 0,
        prev: (s.tick, s.crafted, s.sold, s.event_id, s.command_id),
    };
    let mut offline = sc.offline.clone();
    offline.sort_by_key(|o| o.at);
    let mut offline = offline.into_iter().peekable();
    let mut wall = 0u64;
    let mut next_sample = sc.sample_every_secs;
    let start_gold = s.gold;
    'outer: while wall < sc.duration_secs {
        if let Some(off) = offline.next_if(|o| o.at <= wall) {
            // Quit: the save must survive a round trip, then the game settles on return.
            let mut save = Save::new(s.clone(), wall);
            let bytes = serde_json::to_vec(&save).map_err(|e| e.to_string())?;
            let back: Save = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if back != save {
                r.fail(&s, wall, "save-roundtrip", "decoded save differs");
            }
            let gold = save.state.gold;
            let secs = off.secs.min(sc.duration_secs - wall);
            let before_tick = save.state.tick;
            match save.catch_up(wall + secs) {
                Ok(settled) => {
                    r.report.offline.push(OfflineResult {
                        at: wall,
                        requested: secs,
                        settled,
                        uncredited: secs - settled,
                    });
                    if save.state.tick != before_tick + settled {
                        r.fail(
                            &save.state,
                            wall,
                            "offline-clock",
                            "tick did not move by the settled time",
                        );
                    }
                }
                Err(e) => {
                    r.fail(&save.state, wall, "offline", e.to_string());
                    break 'outer;
                }
            }
            // Events are not returned by catch_up; account the gold change as offline.
            *r.report.ledger.entry("offline".into()).or_default() += delta(gold, save.state.gold);
            s = save.state;
            wall += secs;
            r.check(&s, wall);
            while next_sample <= wall {
                r.report.samples.push(Sample::of(&s, next_sample.min(wall)));
                next_sample += sc.sample_every_secs;
            }
            continue;
        }
        {
            let mut exec = |st: &mut State, c: Command| r.exec(st, c);
            driver::act(&sc.driver, &mut s, wall, &mut exec);
        }
        r.check(&s, wall);
        let until_offline = offline
            .peek()
            .map_or(u64::MAX, |o| o.at.saturating_sub(wall).max(1));
        let step = sc.step_secs.min(sc.duration_secs - wall).min(until_offline);
        if !r.advance(&mut s, wall, step) {
            break;
        }
        wall += step;
        r.watch_queue(&s, step);
        r.check(&s, wall);
        if wall >= next_sample {
            r.report.samples.push(Sample::of(&s, wall));
            next_sample += sc.sample_every_secs;
        }
    }
    let total: i64 = r.report.ledger.values().sum();
    if start_gold as i64 + total != s.gold as i64 {
        let detail = format!("start {start_gold} + ledger {total} != end {}", s.gold);
        r.fail(&s, wall, "gold-ledger", detail);
    }
    r.report.end = Sample::of(&s, wall);
    Ok(r.report)
}
