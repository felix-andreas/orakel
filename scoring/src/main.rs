//! scoring — score predictions against market resolutions.
//!
//! Reads `predictions.csv` and `resolutions.csv` from a directory (default:
//! the `predictions/` dir next to this crate), joins them, and writes
//! `scores_detail.csv` (one row per scored prediction) and `scores.csv`
//! (aggregates per variant / family / model / status / horizon / overall)
//! back into the same directory. Prints a summary table to stdout.
//!
//! Usage: scoring [--dir <predictions-dir>]
//!
//! The primary metric is the paired improvement
//! `improvement = market_brier - brier` — positive means we beat the market
//! on that row, time-matched and therefore fair across horizons.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};

const DETAIL_HEADER: [&str; 15] = [
    "timestamp",
    "market_slug",
    "outcome",
    "family",
    "variant",
    "model",
    "status",
    "horizon",
    "prediction",
    "market_price",
    "actual",
    "brier",
    "market_brier",
    "improvement",
    "logloss",
];

const SCORES_HEADER: [&str; 7] = [
    "level",
    "key",
    "n",
    "mean_improvement",
    "mean_brier",
    "mean_market_brier",
    "mean_logloss",
];

/// Output order of aggregate levels in scores.csv.
const LEVEL_ORDER: [&str; 6] = ["variant", "family", "model", "status", "horizon", "overall"];

struct Prediction {
    timestamp_raw: String,
    timestamp: DateTime<Utc>,
    market_slug: String,
    condition_id: String,
    outcome: String,
    family: String,
    variant: String,
    model: String,
    prediction: f64,
    market_price: f64,
    status: String,
}

struct Resolution {
    winning_outcome: String,
    resolved_at: DateTime<Utc>,
}

struct ScoredRow {
    timestamp: String,
    market_slug: String,
    outcome: String,
    family: String,
    variant: String,
    model: String,
    status: String,
    horizon: String,
    prediction: f64,
    market_price: f64,
    actual: f64,
    brier: f64,
    market_brier: f64,
    improvement: f64,
    logloss: f64,
}

struct AggRow {
    level: String,
    key: String,
    n: u64,
    mean_improvement: f64,
    mean_brier: f64,
    mean_market_brier: f64,
    mean_logloss: f64,
}

struct RunStats {
    scored: usize,
    unresolved: usize,
    malformed: usize,
    aggregates: Vec<AggRow>,
}

fn main() {
    let mut dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--dir" {
            match args.next() {
                Some(v) => dir = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--dir needs a value");
                    std::process::exit(2);
                }
            }
        } else if let Some(v) = a.strip_prefix("--dir=") {
            dir = Some(PathBuf::from(v));
        } else if a == "--help" || a == "-h" {
            println!("usage: scoring [--dir <predictions-dir>]");
            println!("default dir: the predictions/ directory next to this crate");
            return;
        } else {
            eprintln!("unknown argument: {a} (try --help)");
            std::process::exit(2);
        }
    }
    let dir = dir.unwrap_or_else(default_dir);
    if !dir.is_dir() {
        eprintln!("predictions dir not found: {}", dir.display());
        std::process::exit(1);
    }

    match run(&dir) {
        Ok(stats) => print_summary(&stats, &dir),
        Err(e) => {
            eprintln!("scoring failed: {e}");
            std::process::exit(1);
        }
    }
}

/// Default input/output directory: `../predictions` relative to this crate's
/// manifest (works from any cwd when built in-repo), falling back to
/// cwd-relative locations.
fn default_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../predictions");
    if manifest.is_dir() {
        return manifest.canonicalize().unwrap_or(manifest);
    }
    for cand in ["../predictions", "predictions"] {
        let p = PathBuf::from(cand);
        if p.is_dir() {
            return p.canonicalize().unwrap_or(p);
        }
    }
    manifest // does not exist; main reports a clear error
}

/// The whole pipeline: load, join, score, aggregate, write.
fn run(dir: &Path) -> Result<RunStats, String> {
    let mut malformed = 0usize;
    let resolutions = load_resolutions(&dir.join("resolutions.csv"), &mut malformed)?;
    let predictions = load_predictions(&dir.join("predictions.csv"), &mut malformed)?;

    let mut scored: Vec<ScoredRow> = Vec::new();
    let mut unresolved = 0usize;
    for p in &predictions {
        // Look up the resolution by condition_id first, then market_slug.
        let res = resolutions
            .get(&p.condition_id)
            .filter(|_| !p.condition_id.is_empty())
            .or_else(|| resolutions.get(&p.market_slug));
        let res = match res {
            Some(r) => r,
            None => {
                unresolved += 1;
                continue;
            }
        };

        let actual = if p.outcome == res.winning_outcome { 1.0 } else { 0.0 };
        let brier = (p.prediction - actual).powi(2);
        let market_brier = (p.market_price - actual).powi(2);
        scored.push(ScoredRow {
            timestamp: p.timestamp_raw.clone(),
            market_slug: p.market_slug.clone(),
            outcome: p.outcome.clone(),
            family: p.family.clone(),
            variant: p.variant.clone(),
            model: p.model.clone(),
            status: p.status.clone(),
            horizon: horizon_bucket(p.timestamp, res.resolved_at).to_string(),
            prediction: p.prediction,
            market_price: p.market_price,
            actual,
            brier,
            market_brier,
            improvement: market_brier - brier,
            logloss: logloss(p.prediction, actual),
        });
    }

    let aggregates = aggregate(&scored);
    write_detail(&dir.join("scores_detail.csv"), &scored)?;
    write_scores(&dir.join("scores.csv"), &aggregates)?;

    Ok(RunStats {
        scored: scored.len(),
        unresolved,
        malformed,
        aggregates,
    })
}

/// Binary log loss with p clamped away from 0 and 1.
fn logloss(p: f64, actual: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    -(actual * p.ln() + (1.0 - actual) * (1.0 - p).ln())
}

/// Horizon bucket from prediction time to resolution time. `resolved_at` is
/// the resolution date at 12:00 UTC. Negative horizons (prediction logged
/// after resolution time-of-day) count as 0.
fn horizon_bucket(prediction_ts: DateTime<Utc>, resolved_at: DateTime<Utc>) -> &'static str {
    let days = (resolved_at - prediction_ts).num_seconds() as f64 / 86_400.0;
    let d = days.max(0.0);
    if d < 1.0 {
        "0-1d"
    } else if d < 3.0 {
        "1-3d"
    } else if d < 7.0 {
        "3-7d"
    } else if d < 30.0 {
        "7-30d"
    } else {
        ">30d"
    }
}

/// Resolutions keyed by condition_id AND by market_slug (both point at the
/// same resolution), so predictions can join on either.
fn load_resolutions(
    path: &Path,
    malformed: &mut usize,
) -> Result<HashMap<String, Resolution>, String> {
    let content = read_file(path)?;
    let mut map = HashMap::new();
    if content.trim().is_empty() {
        return Ok(map);
    }
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| format!("{}: {e}", path.display()))?
        .clone();
    let i_slug = col(&headers, "market_slug", path)?;
    let i_cond = col(&headers, "condition_id", path)?;
    let i_win = col(&headers, "winning_outcome", path)?;
    let i_date = col(&headers, "resolved_date", path)?;
    let ncols = headers.len();

    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                warn_skip(path, None, &format!("unreadable row: {e}"), malformed);
                continue;
            }
        };
        let line = rec.position().map(|p| p.line());
        if rec.len() != ncols {
            warn_skip(path, line, &format!("expected {ncols} fields, got {}", rec.len()), malformed);
            continue;
        }
        let field = |i: usize| rec.get(i).unwrap_or("").trim().to_string();
        let date_str = field(i_date);
        let resolved_at = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            // Resolution date is treated as 12:00 UTC that day.
            Ok(d) => d.and_hms_opt(12, 0, 0).unwrap().and_utc(),
            Err(e) => {
                warn_skip(path, line, &format!("bad resolved_date '{date_str}': {e}"), malformed);
                continue;
            }
        };
        let res = Resolution {
            winning_outcome: field(i_win),
            resolved_at,
        };
        let slug = field(i_slug);
        let cond = field(i_cond);
        if !cond.is_empty() {
            map.insert(cond, Resolution { winning_outcome: res.winning_outcome.clone(), resolved_at });
        }
        if !slug.is_empty() {
            map.insert(slug, res);
        }
    }
    Ok(map)
}

fn load_predictions(path: &Path, malformed: &mut usize) -> Result<Vec<Prediction>, String> {
    let content = read_file(path)?;
    let mut out = Vec::new();
    if content.trim().is_empty() {
        return Ok(out);
    }
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| format!("{}: {e}", path.display()))?
        .clone();
    let i_ts = col(&headers, "timestamp", path)?;
    let i_slug = col(&headers, "market_slug", path)?;
    let i_cond = col(&headers, "condition_id", path)?;
    let i_outcome = col(&headers, "outcome", path)?;
    let i_family = col(&headers, "family", path)?;
    let i_variant = col(&headers, "variant", path)?;
    let i_model = col(&headers, "model", path)?;
    let i_pred = col(&headers, "prediction", path)?;
    let i_price = col(&headers, "market_price", path)?;
    let i_status = col(&headers, "status", path)?;
    let ncols = headers.len();

    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                warn_skip(path, None, &format!("unreadable row: {e}"), malformed);
                continue;
            }
        };
        let line = rec.position().map(|p| p.line());
        if rec.len() != ncols {
            warn_skip(path, line, &format!("expected {ncols} fields, got {}", rec.len()), malformed);
            continue;
        }
        let field = |i: usize| rec.get(i).unwrap_or("").trim().to_string();

        let ts_raw = field(i_ts);
        let timestamp = match DateTime::parse_from_rfc3339(&ts_raw) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                warn_skip(path, line, &format!("bad timestamp '{ts_raw}': {e}"), malformed);
                continue;
            }
        };
        let prediction = match parse_prob(&field(i_pred)) {
            Ok(v) => v,
            Err(e) => {
                warn_skip(path, line, &format!("bad prediction: {e}"), malformed);
                continue;
            }
        };
        let market_price = match parse_prob(&field(i_price)) {
            Ok(v) => v,
            Err(e) => {
                warn_skip(path, line, &format!("bad market_price: {e}"), malformed);
                continue;
            }
        };

        out.push(Prediction {
            timestamp_raw: ts_raw,
            timestamp,
            market_slug: field(i_slug),
            condition_id: field(i_cond),
            outcome: field(i_outcome),
            family: field(i_family),
            variant: field(i_variant),
            model: field(i_model),
            prediction,
            market_price,
            status: field(i_status),
        });
    }
    Ok(out)
}

fn parse_prob(s: &str) -> Result<f64, String> {
    let v: f64 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a number"))?;
    if !v.is_finite() || !(0.0..=1.0).contains(&v) {
        return Err(format!("'{s}' is not a probability in [0,1]"));
    }
    Ok(v)
}

fn read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn col(headers: &csv::StringRecord, name: &str, path: &Path) -> Result<usize, String> {
    headers
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| format!("{}: missing column '{name}'", path.display()))
}

fn warn_skip(path: &Path, line: Option<u64>, msg: &str, malformed: &mut usize) {
    *malformed += 1;
    let file = path.file_name().and_then(|f| f.to_str()).unwrap_or("?");
    match line {
        Some(l) => eprintln!("warning: {file} line {l}: {msg} — row skipped"),
        None => eprintln!("warning: {file}: {msg} — row skipped"),
    }
}

/// Aggregate scored rows at every level. Sorted by level (variant, family,
/// model, status, horizon, overall), then mean_improvement descending.
fn aggregate(rows: &[ScoredRow]) -> Vec<AggRow> {
    // (level index, key) -> (n, sum_improvement, sum_brier, sum_market_brier, sum_logloss)
    let mut acc: HashMap<(usize, String), (u64, f64, f64, f64, f64)> = HashMap::new();
    for r in rows {
        let keys = [
            (0usize, format!("{}/{}", r.family, r.variant)),
            (1, r.family.clone()),
            (2, r.model.clone()),
            (3, r.status.clone()),
            (4, r.horizon.clone()),
            (5, "overall".to_string()),
        ];
        for k in keys {
            let e = acc.entry(k).or_insert((0, 0.0, 0.0, 0.0, 0.0));
            e.0 += 1;
            e.1 += r.improvement;
            e.2 += r.brier;
            e.3 += r.market_brier;
            e.4 += r.logloss;
        }
    }
    let mut out: Vec<(usize, AggRow)> = acc
        .into_iter()
        .map(|((lvl, key), (n, imp, b, mb, ll))| {
            let nf = n as f64;
            (
                lvl,
                AggRow {
                    level: LEVEL_ORDER[lvl].to_string(),
                    key,
                    n,
                    mean_improvement: imp / nf,
                    mean_brier: b / nf,
                    mean_market_brier: mb / nf,
                    mean_logloss: ll / nf,
                },
            )
        })
        .collect();
    out.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(
                b.1.mean_improvement
                    .partial_cmp(&a.1.mean_improvement)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then_with(|| a.1.key.cmp(&b.1.key))
    });
    out.into_iter().map(|(_, r)| r).collect()
}

/// Format a float with at most 6 decimal places, trailing zeros trimmed —
/// keeps the committed CSVs free of round-trip noise like 0.049999999999999996.
fn fmt_f(v: f64) -> String {
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".to_string() } else { s.to_string() }
}

fn write_detail(path: &Path, rows: &[ScoredRow]) -> Result<(), String> {
    let mut w = csv::Writer::from_path(path)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    w.write_record(DETAIL_HEADER)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    for r in rows {
        w.write_record([
            r.timestamp.as_str(),
            r.market_slug.as_str(),
            r.outcome.as_str(),
            r.family.as_str(),
            r.variant.as_str(),
            r.model.as_str(),
            r.status.as_str(),
            r.horizon.as_str(),
            &fmt_f(r.prediction),
            &fmt_f(r.market_price),
            &fmt_f(r.actual),
            &fmt_f(r.brier),
            &fmt_f(r.market_brier),
            &fmt_f(r.improvement),
            &fmt_f(r.logloss),
        ])
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    w.flush().map_err(|e| format!("{}: {e}", path.display()))
}

fn write_scores(path: &Path, rows: &[AggRow]) -> Result<(), String> {
    let mut w = csv::Writer::from_path(path)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    w.write_record(SCORES_HEADER)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    for r in rows {
        w.write_record([
            r.level.as_str(),
            r.key.as_str(),
            &r.n.to_string(),
            &fmt_f(r.mean_improvement),
            &fmt_f(r.mean_brier),
            &fmt_f(r.mean_market_brier),
            &fmt_f(r.mean_logloss),
        ])
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    w.flush().map_err(|e| format!("{}: {e}", path.display()))
}

fn print_summary(stats: &RunStats, dir: &Path) {
    if stats.scored == 0 {
        println!(
            "0 predictions scored ({} unresolved, {} malformed) — wrote empty outputs to {}",
            stats.unresolved,
            stats.malformed,
            dir.display()
        );
        return;
    }
    println!(
        "{} predictions scored ({} unresolved skipped, {} malformed skipped)",
        stats.scored, stats.unresolved, stats.malformed
    );
    println!();
    println!(
        "{:<8} {:<32} {:>5} {:>10} {:>11} {:>11} {:>10}",
        "level", "key", "n", "mean_imp", "mean_brier", "mkt_brier", "logloss"
    );
    println!("{}", "-".repeat(93));
    for a in &stats.aggregates {
        println!(
            "{:<8} {:<32} {:>5} {:>+10.4} {:>11.4} {:>11.4} {:>10.4}",
            a.level, a.key, a.n, a.mean_improvement, a.mean_brier, a.mean_market_brier, a.mean_logloss
        );
    }
    println!();
    println!(
        "wrote {} and {}",
        dir.join("scores_detail.csv").display(),
        dir.join("scores.csv").display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("scoring-test-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    /// Parse scores.csv into (level, key) -> AggRow-like tuple.
    fn read_scores(dir: &Path) -> (Vec<(String, String)>, HashMap<(String, String), (u64, f64, f64, f64, f64)>) {
        let content = std::fs::read_to_string(dir.join("scores.csv")).unwrap();
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        assert_eq!(
            rdr.headers().unwrap().iter().collect::<Vec<_>>(),
            SCORES_HEADER.to_vec()
        );
        let mut order = Vec::new();
        let mut map = HashMap::new();
        for rec in rdr.records() {
            let r = rec.unwrap();
            let key = (r[0].to_string(), r[1].to_string());
            order.push(key.clone());
            map.insert(
                key,
                (
                    r[2].parse::<u64>().unwrap(),
                    r[3].parse::<f64>().unwrap(),
                    r[4].parse::<f64>().unwrap(),
                    r[5].parse::<f64>().unwrap(),
                    r[6].parse::<f64>().unwrap(),
                ),
            );
        }
        (order, map)
    }

    // Output floats are rounded to 6 decimals, so parse-back error is <= 5e-7.
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn float_formatting() {
        assert_eq!(fmt_f(0.049999999999999996), "0.05");
        assert_eq!(fmt_f(0.0475), "0.0475");
        assert_eq!(fmt_f(-0.005), "-0.005");
        assert_eq!(fmt_f(1.0), "1");
        assert_eq!(fmt_f(0.0), "0");
        assert_eq!(fmt_f(-0.0000001), "0");
        assert_eq!(fmt_f(0.10536051565782628), "0.105361");
    }

    #[test]
    fn horizon_buckets() {
        let res = NaiveDate::from_ymd_opt(2026, 1, 10)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();
        let ts = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&Utc)
        };
        assert_eq!(horizon_bucket(ts("2026-01-10T12:00:00Z"), res), "0-1d"); // 0
        assert_eq!(horizon_bucket(ts("2026-01-10T13:00:00Z"), res), "0-1d"); // negative -> 0
        assert_eq!(horizon_bucket(ts("2026-01-09T12:00:01Z"), res), "0-1d"); // just under 1d
        assert_eq!(horizon_bucket(ts("2026-01-09T12:00:00Z"), res), "1-3d"); // exactly 1d
        assert_eq!(horizon_bucket(ts("2026-01-07T12:00:01Z"), res), "1-3d"); // just under 3d
        assert_eq!(horizon_bucket(ts("2026-01-07T12:00:00Z"), res), "3-7d"); // exactly 3d
        assert_eq!(horizon_bucket(ts("2026-01-03T12:00:00Z"), res), "7-30d"); // exactly 7d
        assert_eq!(horizon_bucket(ts("2025-12-12T12:00:01Z"), res), "7-30d"); // just under 30d
        assert_eq!(horizon_bucket(ts("2025-12-11T12:00:00Z"), res), ">30d"); // exactly 30d
    }

    #[test]
    fn empty_inputs_write_empty_outputs() {
        let dir = tempdir("empty");
        write(
            &dir,
            "predictions.csv",
            "timestamp,market_slug,condition_id,outcome,token_id,family,variant,model,prediction,market_price,run_id,status\n",
        );
        write(
            &dir,
            "resolutions.csv",
            "market_slug,condition_id,winning_outcome,resolved_date,note\n",
        );
        let stats = run(&dir).unwrap();
        assert_eq!(stats.scored, 0);
        assert_eq!(stats.malformed, 0);
        let detail = std::fs::read_to_string(dir.join("scores_detail.csv")).unwrap();
        assert_eq!(detail.trim(), DETAIL_HEADER.join(","));
        let scores = std::fs::read_to_string(dir.join("scores.csv")).unwrap();
        assert_eq!(scores.trim(), SCORES_HEADER.join(","));
    }

    #[test]
    fn fixture_aggregates_are_correct() {
        let dir = tempdir("fixture");
        // Two markets: market-a resolves Yes on 2026-01-10 (12:00 UTC),
        // market-b resolves No on 2026-01-12. market-c is unresolved.
        write(
            &dir,
            "resolutions.csv",
            "market_slug,condition_id,winning_outcome,resolved_date,note\n\
             market-a,0xa,Yes,2026-01-10,\n\
             market-b,0xb,No,2026-01-12,test note\n",
        );
        // 8 scoreable rows across two variants (f1/v1 live model-x, f2/v2
        // trial model-y) and three horizon buckets, plus one unresolved row
        // and one malformed row.
        write(
            &dir,
            "predictions.csv",
            "timestamp,market_slug,condition_id,outcome,token_id,family,variant,model,prediction,market_price,run_id,status\n\
             2026-01-10T00:00:00Z,market-a,0xa,Yes,tokA1,f1,v1,model-x,0.9,0.8,2026-01-10/t,live\n\
             2026-01-08T12:00:00Z,market-a,0xa,Yes,tokA1,f1,v1,model-x,0.7,0.6,2026-01-08/t,live\n\
             2026-01-10T12:00:00Z,market-b,0xb,Yes,tokB1,f1,v1,model-x,0.2,0.3,2026-01-10/t,live\n\
             2026-01-12T00:00:00Z,market-b,0xb,No,tokB2,f1,v1,model-x,0.85,0.75,2026-01-12/t,live\n\
             2026-01-10T00:00:00Z,market-a,0xa,Yes,tokA1,f2,v2,model-y,0.6,0.8,2026-01-10/t,trial\n\
             2026-01-08T12:00:00Z,market-a,0xa,No,tokA2,f2,v2,model-y,0.5,0.4,2026-01-08/t,trial\n\
             2026-01-05T12:00:00Z,market-b,0xb,No,tokB2,f2,v2,model-y,0.9,0.5,2026-01-05/t,trial\n\
             2026-01-12T06:00:00Z,market-b,0xb,Yes,tokB1,f2,v2,model-y,0.1,0.2,2026-01-12/t,trial\n\
             2026-01-10T00:00:00Z,market-c,0xc,Yes,tokC1,f1,v1,model-x,0.5,0.5,2026-01-10/t,live\n\
             2026-01-10T00:00:00Z,market-a,0xa,Yes,tokA1,f1,v1,model-x,oops,0.5,2026-01-10/t,live\n",
        );

        let stats = run(&dir).unwrap();
        assert_eq!(stats.scored, 8);
        assert_eq!(stats.unresolved, 1);
        assert_eq!(stats.malformed, 1);

        // Detail file: header + 8 rows, exact header order.
        let detail = std::fs::read_to_string(dir.join("scores_detail.csv")).unwrap();
        let lines: Vec<&str> = detail.trim().lines().collect();
        assert_eq!(lines.len(), 9);
        assert_eq!(lines[0], DETAIL_HEADER.join(","));

        let (order, scores) = read_scores(&dir);

        // Paired improvement per row (market_brier - brier):
        // f1/v1: 0.03, 0.07, 0.05, 0.04           -> mean 0.0475
        // f2/v2: -0.12, -0.09, 0.24, 0.03         -> mean 0.015
        let v1 = &scores[&("variant".into(), "f1/v1".into())];
        assert_eq!(v1.0, 4);
        assert!(approx(v1.1, 0.0475), "v1 mean_improvement {}", v1.1);
        assert!(approx(v1.2, 0.040625), "v1 mean_brier {}", v1.2);
        assert!(approx(v1.3, 0.088125), "v1 mean_market_brier {}", v1.3);
        let v1_ll = (-(0.9f64.ln()) - 0.7f64.ln() - 0.8f64.ln() - 0.85f64.ln()) / 4.0;
        assert!(approx(v1.4, v1_ll), "v1 mean_logloss {} vs {}", v1.4, v1_ll);

        let v2 = &scores[&("variant".into(), "f2/v2".into())];
        assert_eq!(v2.0, 4);
        assert!(approx(v2.1, 0.015), "v2 mean_improvement {}", v2.1);

        // Family / model / status mirror the variants in this fixture.
        assert!(approx(scores[&("family".into(), "f1".into())].1, 0.0475));
        assert!(approx(scores[&("family".into(), "f2".into())].1, 0.015));
        assert!(approx(scores[&("model".into(), "model-x".into())].1, 0.0475));
        assert!(approx(scores[&("model".into(), "model-y".into())].1, 0.015));
        assert!(approx(scores[&("status".into(), "live".into())].1, 0.0475));
        assert!(approx(scores[&("status".into(), "trial".into())].1, 0.015));

        // Horizon bucketing: rows at 0.25-0.5d, exactly 2d, and exactly 7d.
        let h01 = &scores[&("horizon".into(), "0-1d".into())];
        assert_eq!(h01.0, 4);
        assert!(approx(h01.1, -0.005), "0-1d mean_improvement {}", h01.1);
        let h13 = &scores[&("horizon".into(), "1-3d".into())];
        assert_eq!(h13.0, 3);
        assert!(approx(h13.1, 0.01), "1-3d mean_improvement {}", h13.1);
        let h730 = &scores[&("horizon".into(), "7-30d".into())];
        assert_eq!(h730.0, 1);
        assert!(approx(h730.1, 0.24), "7-30d mean_improvement {}", h730.1);
        assert!(!scores.contains_key(&("horizon".into(), "3-7d".into())));
        assert!(!scores.contains_key(&("horizon".into(), ">30d".into())));

        // Overall: n=8, mean improvement 0.25/8.
        let overall = &scores[&("overall".into(), "overall".into())];
        assert_eq!(overall.0, 8);
        assert!(approx(overall.1, 0.03125), "overall mean_improvement {}", overall.1);

        // Sort order: variants first (best improvement first), overall last.
        assert_eq!(order.first().unwrap(), &("variant".to_string(), "f1/v1".to_string()));
        assert_eq!(order.last().unwrap(), &("overall".to_string(), "overall".to_string()));
        let levels: Vec<&str> = order.iter().map(|(l, _)| l.as_str()).collect();
        let mut sorted = levels.clone();
        sorted.sort_by_key(|l| LEVEL_ORDER.iter().position(|x| x == l).unwrap());
        assert_eq!(levels, sorted, "rows not grouped by level order");
    }

    #[test]
    fn logloss_is_clamped() {
        assert!(logloss(0.0, 1.0).is_finite());
        assert!(logloss(1.0, 0.0).is_finite());
        assert!(approx(logloss(0.0, 1.0), -(1e-6f64.ln())));
        assert!(approx(logloss(0.5, 1.0), -(0.5f64.ln())));
        assert!(approx(logloss(0.5, 0.0), -(0.5f64.ln())));
    }
}
