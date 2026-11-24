//! The command line behind the cargo aliases in `.cargo/config.toml`:
//!
//! ```text
//! cargo scaffold <day>                 new day: code, example, input and writeup files
//! cargo solve <day>                    run a day on your input (release build)
//! cargo time <day|all>                 like solve, timing the median of repeated runs
//! cargo all                            run every day so far, then print a summary
//! cargo writeup <day|all> [--watch]    build writeups/DD.typ into writeups/pdf/day-DD.pdf
//! ```
//!
//! Every day is a standalone binary, `src/bin/DD.rs`, whose `main` prints one
//! line per part: `part 1 (2314131 ns): 12345`, or `part 1 (531 ns)` with no
//! answer yet. `solve`, `time` and `all` read those lines and save answers and
//! times to `writeups/results/DD.json`, where the day's writeup reads them.

use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

const USAGE: &str = "\
usage:
  cargo scaffold <day>                 new day: code, example, input and writeup files
  cargo solve <day>                    run a day on your input (release build)
  cargo time <day|all>                 like solve, timing the median of repeated runs
  cargo all                            run every day so far, then print a summary
  cargo writeup <day|all> [--watch]    build writeups/DD.typ into writeups/pdf/day-DD.pdf";

const DAY_TEMPLATE: &str = include_str!("../templates/day.rs.tmpl");
const WRITEUP_TEMPLATE: &str = include_str!("../templates/writeup.typ.tmpl");

/// Advent of Code 2026 has 12 puzzles, 1 to 12 December.
const DAYS: u8 = 12;

/// `cargo time` reruns a day for about this long, then keeps the median.
const BENCH_BUDGET: Duration = Duration::from_secs(1);
const BENCH_MAX_RUNS: usize = 1_000;

/// An empty error means whatever failed has already said why.
type Result<T = ()> = std::result::Result<T, String>;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let result = match args.as_slice() {
        ["scaffold", day] => parse_day(day).and_then(scaffold),
        ["solve", day] => parse_day(day).and_then(|d| run_days(&[d], false)),
        ["time", "all"] => days().and_then(|ds| run_days(&ds, true)),
        ["time", day] => parse_day(day).and_then(|d| run_days(&[d], true)),
        ["all"] => days().and_then(|ds| run_days(&ds, false)),
        ["writeup", "all"] => days().and_then(|ds| ds.into_iter().try_for_each(|d| writeup(d, false))),
        ["writeup", day] => parse_day(day).and_then(|d| writeup(d, false)),
        ["writeup", day, "--watch"] | ["writeup", "--watch", day] => parse_day(day).and_then(|d| writeup(d, true)),
        ["help" | "-h" | "--help"] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => Err(USAGE.to_string()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("{e}");
            }
            ExitCode::FAILURE
        }
    }
}

fn parse_day(s: &str) -> Result<u8> {
    match s.parse::<u8>() {
        Ok(day @ 1..=DAYS) => Ok(day),
        _ => Err(format!("`{s}` isn't a day: expected a number from 1 to {DAYS}")),
    }
}

// ── Paths ────────────────────────────────────────────────────────────────────

/// The repository root, where `Cargo.toml` lives.
fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// `path` relative to the repository root, for messages.
fn relative(path: &Path) -> String {
    path.strip_prefix(root()).unwrap_or(path).display().to_string()
}

fn day_source(day: u8) -> PathBuf {
    root().join(format!("src/bin/{day:02}.rs"))
}

fn results_path(day: u8) -> PathBuf {
    root().join(format!("writeups/results/{day:02}.json"))
}

/// The days scaffolded so far: every `src/bin/DD.rs`, in order.
fn days() -> Result<Vec<u8>> {
    let mut days: Vec<u8> = fs::read_dir(root().join("src/bin"))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter_map(|name| name.strip_suffix(".rs")?.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    days.sort_unstable();
    if days.is_empty() {
        return Err("No days yet: start one with `cargo scaffold <day>`.".into());
    }
    Ok(days)
}

// ── scaffold ─────────────────────────────────────────────────────────────────

fn scaffold(day: u8) -> Result {
    let dd = format!("{day:02}");
    let fill = |template: &str| template.replace("{{DAY}}", &day.to_string()).replace("{{DD}}", &dd);

    let files = [
        (day_source(day), fill(DAY_TEMPLATE)),
        (root().join(format!("data/examples/{dd}.txt")), String::new()),
        (root().join(format!("data/inputs/{dd}.txt")), String::new()),
        (root().join(format!("writeups/{dd}.typ")), fill(WRITEUP_TEMPLATE)),
    ];

    println!("Day {day}");
    for (path, contents) in files {
        create(&path, &contents)?;
    }
    let results = results_path(day);
    if results.exists() {
        println!("  exists   {}", relative(&results));
    } else {
        write_results(day, None).map_err(|e| format!("cannot write {}: {e}", relative(&results)))?;
        println!("  created  {}", relative(&results));
    }

    println!("\nNext:");
    println!("  paste the example into data/examples/{dd}.txt (and {dd}-2.txt if part two has its own)");
    println!("  paste your input into  data/inputs/{dd}.txt");
    for (command, what) in [
        (format!("cargo test --bin {dd}"), "test both parts on the example".to_string()),
        (format!("cargo solve {day}"), "run both parts on your input".to_string()),
        (format!("cargo writeup {day}"), format!("build writeups/pdf/day-{dd}.pdf")),
    ] {
        println!("  {command:<22} {what}");
    }
    Ok(())
}

/// Writes `contents` to `path` unless the file already exists.
fn create(path: &Path, contents: &str) -> Result {
    if path.exists() {
        println!("  exists   {}", relative(path));
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", relative(dir)))?;
    }
    fs::write(path, contents).map_err(|e| format!("cannot write {}: {e}", relative(path)))?;
    println!("  created  {}", relative(path));
    Ok(())
}

// ── solve, time, all ─────────────────────────────────────────────────────────

/// One part of one day: its answer, if it gave one, and how long it took.
struct Part {
    answer: Option<String>,
    /// Nanoseconds for one run, or the median of all runs when benchmarking.
    ns: u64,
    runs: usize,
}

/// Runs each day on its input and records the results. With more than one
/// day, carries on past failures and prints a summary table at the end.
fn run_days(days: &[u8], bench: bool) -> Result {
    let mut failed = Vec::new();
    for (i, &day) in days.iter().enumerate() {
        if !day_source(day).exists() {
            return Err(format!("Day {day} isn't scaffolded yet: run `cargo scaffold {day}`."));
        }
        if i > 0 {
            println!();
        }
        if let Err(e) = run_day(day, bench) {
            if !e.is_empty() {
                eprintln!("{e}");
            }
            failed.push(day);
        }
    }

    if days.len() > 1 {
        println!();
        summary(days);
    }
    match failed.as_slice() {
        [] => Ok(()),
        _ if days.len() == 1 => Err(String::new()),
        [day] => Err(format!("Day {day} failed.")),
        _ => Err(format!("Days {failed:?} failed.")),
    }
}

fn run_day(day: u8, bench: bool) -> Result {
    let exe = build(day)?;
    println!("Day {day:02}");

    let started = Instant::now();
    let mut parts = run_once(day, &exe, true)?;
    if bench {
        let mut times = [vec![parts[0].ns], vec![parts[1].ns]];
        while started.elapsed() < BENCH_BUDGET && times[0].len() < BENCH_MAX_RUNS {
            let [one, two] = run_once(day, &exe, false)?;
            times[0].push(one.ns);
            times[1].push(two.ns);
        }
        for (part, mut times) in parts.iter_mut().zip(times) {
            times.sort_unstable();
            part.ns = times[times.len() / 2];
            part.runs = times.len();
        }
    }

    for (i, part) in parts.iter().enumerate() {
        let answer = part.answer.as_deref().unwrap_or("(no answer yet)");
        let mut time = format_duration(Duration::from_nanos(part.ns));
        if part.runs > 1 {
            time = format!("{time}, median of {} runs", part.runs);
        }
        println!("  Part {}  {answer:<20} ({time})", i + 1);
    }
    write_results(day, Some(&parts)).map_err(|e| format!("cannot save {}: {e}", relative(&results_path(day))))
}

/// Builds a day in release mode and returns the path of its executable.
fn build(day: u8) -> Result<PathBuf> {
    let name = format!("{day:02}");
    let output = cargo()
        .args(["build", "--release", "--quiet", "--bin", &name])
        .arg("--message-format=json-render-diagnostics")
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cannot run cargo: {e}"))?;
    if !output.status.success() {
        return Err(format!("Day {day} doesn't compile."));
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|msg| msg["reason"] == "compiler-artifact" && msg["target"]["name"] == name.as_str())
        .and_then(|msg| msg["executable"].as_str().map(PathBuf::from))
        .ok_or_else(|| format!("cargo didn't say where it built day {day}"))
}

/// The `cargo` that is running us, so the same toolchain builds the days.
fn cargo() -> Command {
    let mut cmd = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    cmd.current_dir(root());
    cmd
}

/// Runs a built day once and reads its two `part N (… ns): …` lines. With
/// `echo`, anything else the day prints (debug output) is passed through.
fn run_once(day: u8, exe: &Path, echo: bool) -> Result<[Part; 2]> {
    let output = Command::new(exe)
        .current_dir(root())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cannot run day {day}: {e}"))?;
    let mut parts = [None, None];
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        match parse_part_line(line) {
            Some((part @ 1..=2, part_result)) => parts[part as usize - 1] = Some(part_result),
            _ if echo => println!("{line}"),
            _ => {}
        }
    }
    if !output.status.success() {
        return Err(String::new()); // its stderr has already said why
    }
    match parts {
        [Some(one), Some(two)] => Ok([one, two]),
        _ => Err(format!("Day {day} didn't print a `part 1 (… ns): …` and a `part 2 (… ns): …` line.")),
    }
}

/// `part 1 (2314131 ns): 12345`, or `part 1 (531 ns)` when there's no answer.
fn parse_part_line(line: &str) -> Option<(u8, Part)> {
    let (part, rest) = line.strip_prefix("part ")?.split_once(" (")?;
    let (ns, rest) = rest.split_once(" ns)")?;
    let answer = rest.strip_prefix(": ").map(str::to_string);
    Some((part.parse().ok()?, Part { answer, ns: ns.parse().ok()?, runs: 1 }))
}

/// Saves a day's answers and times to `writeups/results/DD.json`. With no
/// parts, saves no answers and no times: how `cargo scaffold` starts the file.
fn write_results(day: u8, parts: Option<&[Part; 2]>) -> io::Result<()> {
    let part = |i: usize| match parts {
        Some(parts) => serde_json::json!({
            "answer": parts[i].answer,
            "time_ns": parts[i].ns,
            "runs": parts[i].runs,
        }),
        None => serde_json::json!({ "answer": null, "time_ns": null, "runs": 0 }),
    };
    let json = serde_json::json!({ "day": day, "part1": part(0), "part2": part(1) });
    let path = results_path(day);
    fs::create_dir_all(path.parent().expect("results_path has a parent"))?;
    fs::write(&path, serde_json::to_string_pretty(&json)? + "\n")
}

/// A table of every day's recorded answers and times.
fn summary(days: &[u8]) {
    println!("{:<5} {:<20} {:<20} {:>10}", "Day", "Part 1", "Part 2", "Time");
    let mut total = Duration::ZERO;
    for &day in days {
        let Some(results) = fs::read_to_string(results_path(day))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        else {
            println!("{day:02}    (no results)");
            continue;
        };
        let answer = |part: &str| results[part]["answer"].as_str().unwrap_or("-").to_string();
        let ns = |part: &str| results[part]["time_ns"].as_u64().unwrap_or(0);
        let time = Duration::from_nanos(ns("part1") + ns("part2"));
        total += time;
        println!(
            "{day:02}    {:<20} {:<20} {:>10}",
            answer("part1"),
            answer("part2"),
            format_duration(time)
        );
    }
    println!("{:<47} {:>10}", "Total", format_duration(total));
}

/// `1.23 ms`, `45.6 µs`, `789 ns`, `2.01 s`.
fn format_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    match ns {
        0..1_000 => format!("{ns} ns"),
        1_000..1_000_000 => format!("{:.1} µs", ns as f64 / 1e3),
        1_000_000..1_000_000_000 => format!("{:.2} ms", ns as f64 / 1e6),
        _ => format!("{:.2} s", ns as f64 / 1e9),
    }
}

// ── writeup ──────────────────────────────────────────────────────────────────

/// Compiles (or watches) `writeups/DD.typ` with the repository as the Typst
/// root, so the writeup can read the solution's source in `src/bin/`.
fn writeup(day: u8, watch: bool) -> Result {
    let source = root().join(format!("writeups/{day:02}.typ"));
    if !source.exists() {
        return Err(format!("{} doesn't exist: run `cargo scaffold {day}`.", relative(&source)));
    }
    let out_dir = root().join("writeups/pdf");
    fs::create_dir_all(&out_dir).map_err(|e| format!("cannot create {}: {e}", relative(&out_dir)))?;
    let pdf = out_dir.join(format!("day-{day:02}.pdf"));

    let status = Command::new("typst")
        .current_dir(root())
        .arg(if watch { "watch" } else { "compile" })
        .args(["--root", "."])
        .arg(&source)
        .arg(&pdf)
        .status()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => "typst isn't installed: see https://github.com/typst/typst#installation".to_string(),
            _ => format!("cannot run typst: {e}"),
        })?;
    if !status.success() {
        return Err(format!("typst couldn't build {}", relative(&source)));
    }
    println!("{}", relative(&pdf));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_part_lines() {
        let (part, p) = parse_part_line("part 1 (2314131 ns): 12345").unwrap();
        assert_eq!((part, p.ns, p.answer.as_deref()), (1, 2_314_131, Some("12345")));

        let (part, p) = parse_part_line("part 2 (531 ns)").unwrap();
        assert_eq!((part, p.ns, p.answer), (2, 531, None));

        assert!(parse_part_line("debug: part 1 is hard").is_none());
    }

    #[test]
    fn durations_pick_a_readable_unit() {
        assert_eq!(format_duration(Duration::from_nanos(789)), "789 ns");
        assert_eq!(format_duration(Duration::from_nanos(45_600)), "45.6 µs");
        assert_eq!(format_duration(Duration::from_micros(1_234)), "1.23 ms");
        assert_eq!(format_duration(Duration::from_millis(2_010)), "2.01 s");
    }
}
