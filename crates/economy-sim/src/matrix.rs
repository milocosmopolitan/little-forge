//! Scenario matrix: a base scenario, value lists at JSON-pointer paths, and seeds. Every
//! combination × seed is one run. Runs are independent and executed in parallel; results are
//! returned in case order, so output does not depend on thread timing.
use crate::runner::{Report, run};
use crate::scenario::Scenario;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    pub base: Value,
    pub seeds: Seeds,
    /// JSON pointer (e.g. "/start/gold") → values to try.
    #[serde(default)]
    pub vary: BTreeMap<String, Vec<Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Seeds {
    List(Vec<u64>),
    Range { from: u64, count: u64 },
}
impl Seeds {
    pub fn list(&self) -> Vec<u64> {
        match self {
            Seeds::List(v) => v.clone(),
            Seeds::Range { from, count } => (*from..from + count).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub index: usize,
    pub params: Vec<(String, Value)>,
    pub scenario: Scenario,
}

/// Sets `value` at a JSON pointer, creating objects on the way.
fn set(root: &mut Value, pointer: &str, value: Value) -> Result<(), String> {
    let parts: Vec<_> = pointer
        .strip_prefix('/')
        .ok_or_else(|| format!("pointer must start with '/': {pointer}"))?
        .split('/')
        .map(|p| p.replace("~1", "/").replace("~0", "~"))
        .collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        let obj = node
            .as_object_mut()
            .ok_or_else(|| format!("{pointer}: not an object at {part}"))?;
        if i + 1 == parts.len() {
            obj.insert(part.clone(), value);
            return Ok(());
        }
        node = obj
            .entry(part.clone())
            .or_insert_with(|| Value::Object(Default::default()));
    }
    Ok(())
}

pub fn expand(m: &Matrix) -> Result<Vec<Case>, String> {
    let axes: Vec<(&String, &Vec<Value>)> = m.vary.iter().collect();
    let mut combos: Vec<Vec<(String, Value)>> = vec![vec![]];
    for (path, values) in &axes {
        if values.is_empty() {
            return Err(format!("{path}: empty value list"));
        }
        combos = combos
            .into_iter()
            .flat_map(|c| {
                values.iter().map(move |v| {
                    let mut c = c.clone();
                    c.push(((*path).clone(), v.clone()));
                    c
                })
            })
            .collect();
    }
    let mut cases = vec![];
    for combo in combos {
        for seed in m.seeds.list() {
            let mut v = m.base.clone();
            for (path, value) in &combo {
                set(&mut v, path, value.clone())?;
            }
            set(&mut v, "/seed", seed.into())?;
            let mut scenario: Scenario =
                serde_json::from_value(v).map_err(|e| format!("case {combo:?}: {e}"))?;
            scenario.check()?;
            let label: Vec<String> = combo.iter().map(|(k, v)| format!("{k}={v}")).collect();
            scenario.name = format!("{}[{}]", scenario.name, label.join(","));
            cases.push(Case {
                index: cases.len(),
                params: combo.clone(),
                scenario,
            });
        }
    }
    Ok(cases)
}

pub fn run_all(cases: &[Case], threads: usize) -> Vec<Result<Report, String>> {
    let next = AtomicUsize::new(0);
    let mut out: Vec<Option<Result<Report, String>>> = vec![None; cases.len()];
    let results = std::sync::Mutex::new(&mut out);
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(case) = cases.get(i) else { break };
                    let r = run(&case.scenario);
                    results.lock().unwrap()[i] = Some(r);
                }
            });
        }
    });
    out.into_iter()
        .map(|r| r.expect("every case ran"))
        .collect()
}

const COLUMNS: &str = "case,seed,params,wall_secs,gold_start,gold_end,gold_delta,sale_gold,ore_bought_cmds,material_spend,produced,crafted,sold,chef,farmer,queue_waiting_secs,longest_queue_wait_secs,offline_uncredited_secs,failures,error";

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn summary_csv(cases: &[Case], results: &[Result<Report, String>]) -> String {
    let mut out = String::from(COLUMNS);
    out.push('\n');
    for (c, r) in cases.iter().zip(results) {
        let params: Vec<String> = c.params.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let params = csv_field(&params.join(";"));
        let row = match r {
            Ok(r) => format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},",
                c.index,
                r.seed,
                params,
                r.wall_secs,
                r.start.gold,
                r.end.gold,
                r.end.gold as i64 - r.start.gold as i64,
                r.ledger.get("sale").copied().unwrap_or(0),
                r.commands.get("BuyOre").map_or(0, |t| t.ok),
                -r.ledger.get("cmd:Buy").copied().unwrap_or(0),
                r.produced.values().sum::<u64>(),
                r.end.crafted,
                r.end.sold,
                r.end.chef,
                r.end.farmer,
                r.queue_waiting_secs,
                r.longest_queue_wait_secs,
                r.offline.iter().map(|o| o.uncredited).sum::<u64>(),
                r.failures.len(),
            ),
            Err(e) => format!(
                "{},{},{},,,,,,,,,,,,,,,,,{}",
                c.index,
                c.scenario.seed,
                params,
                csv_field(e)
            ),
        };
        out.push_str(&row);
        out.push('\n');
    }
    out
}
