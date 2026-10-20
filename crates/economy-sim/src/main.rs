//! economy-sim CLI.
//!
//!   economy-sim run <scenario.json> [--out report.json]
//!   economy-sim matrix <matrix.json> --out <dir> [--threads N] [--full]
//!
//! Exits with status 1 when any run reports an invariant failure or error, so CI can gate on it.
use economy_sim::{Matrix, Scenario, expand, run, run_all, summary_csv};
use std::path::PathBuf;

fn usage() -> ! {
    eprintln!(
        "usage:\n  economy-sim run <scenario.json> [--out report.json]\n  economy-sim matrix <matrix.json> --out <dir> [--threads N] [--full]"
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let (Some(mode), Some(file)) = (args.first(), args.get(1)) else {
        usage()
    };
    let text = std::fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("{file}: {e}");
        std::process::exit(2)
    });
    let failed = match mode.as_str() {
        "run" => {
            let sc = Scenario::from_json(&text).unwrap_or_else(|e| {
                eprintln!("{file}: {e}");
                std::process::exit(2)
            });
            let report = run(&sc).unwrap_or_else(|e| {
                eprintln!("{}: {e}", sc.name);
                std::process::exit(1)
            });
            let json = serde_json::to_string_pretty(&report).unwrap();
            match flag("--out") {
                Some(path) => std::fs::write(&path, json).expect("write report"),
                None => println!("{json}"),
            }
            eprintln!(
                "{}: gold {} -> {}, crafted {}, sold {}, failures {}",
                report.scenario,
                report.start.gold,
                report.end.gold,
                report.end.crafted,
                report.end.sold,
                report.failures.len()
            );
            !report.failures.is_empty()
        }
        "matrix" => {
            let m: Matrix = serde_json::from_str(&text).unwrap_or_else(|e| {
                eprintln!("{file}: {e}");
                std::process::exit(2)
            });
            let Some(out) = flag("--out").map(PathBuf::from) else {
                usage()
            };
            let threads = flag("--threads")
                .and_then(|t| t.parse().ok())
                .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
            let cases = expand(&m).unwrap_or_else(|e| {
                eprintln!("{file}: {e}");
                std::process::exit(2)
            });
            let started = std::time::Instant::now();
            let results = run_all(&cases, threads);
            std::fs::create_dir_all(&out).expect("create output directory");
            std::fs::write(out.join("summary.csv"), summary_csv(&cases, &results))
                .expect("write summary");
            let failures: Vec<_> = cases
                .iter()
                .zip(&results)
                .filter_map(|(c, r)| match r {
                    Ok(r) if r.failures.is_empty() => None,
                    Ok(r) => Some(serde_json::json!({"case": c.index, "scenario": r.scenario, "seed": r.seed, "failures": r.failures})),
                    Err(e) => Some(serde_json::json!({"case": c.index, "scenario": c.scenario.name, "seed": c.scenario.seed, "error": e})),
                })
                .collect();
            std::fs::write(
                out.join("failures.json"),
                serde_json::to_string_pretty(&failures).unwrap(),
            )
            .expect("write failures");
            if args.iter().any(|a| a == "--full") {
                let dir = out.join("runs");
                std::fs::create_dir_all(&dir).expect("create runs directory");
                for (c, r) in cases.iter().zip(&results) {
                    if let Ok(r) = r {
                        std::fs::write(
                            dir.join(format!("{:05}.json", c.index)),
                            serde_json::to_string(r).unwrap(),
                        )
                        .expect("write run");
                    }
                }
            }
            eprintln!(
                "{} runs on {threads} threads in {:.1}s; {} with failures -> {}",
                cases.len(),
                started.elapsed().as_secs_f64(),
                failures.len(),
                out.display()
            );
            !failures.is_empty()
        }
        _ => usage(),
    };
    std::process::exit(i32::from(failed))
}
