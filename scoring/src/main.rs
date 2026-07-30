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
//!
//! **Calibration is not tradeability.** `market_price` is a CLOB midpoint, and
//! a midpoint is the average of a bid and an ask: on a thin wing leg quoted
//! 0.001 / 0.08 it reads 4c, and 4c is a number no counterparty ever offered.
//! Beating such a price is a real forecasting result and is worth exactly
//! nothing in cash. So if `fills.csv` is present (written by `tools/fillcheck`,
//! which replays Polymarket's public trade feed), every scored row also carries
//! the best price at which somebody demonstrably traded the side we wanted,
//! and the aggregates report how many rows were reachable at all. Rows without
//! fills data get `fillable = ""` — unknown, not false.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};

const DETAIL_HEADER: [&str; 19] = [
    "timestamp",
    "market_slug",
    "outcome",
    "family",
    "variant",
    "model",
    "status",
    "horizon",
    "pricer_version",
    "prediction",
    "market_price",
    "actual",
    "brier",
    "market_brier",
    "improvement",
    "logloss",
    "best_price",
    "fillable",
    "exec_edge",
];

const SCORES_HEADER: [&str; 14] = [
    "level",
    "key",
    "n",
    // Distinct markets, and the cluster-robust interval taken across them.
    // `n` counts mornings; `n_markets` counts events, and gate 1 of the
    // pre-registered trial rule turns on whether [ci_lo, ci_hi] excludes zero.
    "n_markets",
    // `mean_improvement_market` is the point estimate the interval belongs to.
    // Emitted separately because `mean_improvement` below is the mean over
    // ROWS, and printing an interval next to a mean it is not an interval for
    // is exactly how a headline gets misread.
    "mean_improvement_market",
    "ci_lo",
    "ci_hi",
    "mean_improvement",
    "mean_brier",
    "mean_market_brier",
    "mean_logloss",
    "n_known_fill",
    "n_fillable",
    "mean_exec_edge",
];

/// Output order of aggregate levels in scores.csv.
const LEVEL_ORDER: [&str; 8] =
    ["variant", "family", "model", "status", "horizon", "market", "pricer", "overall"];

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
    /// Which build of the variant's pricer produced this number. Empty for
    /// rows written before the column existed (2026-07-28).
    pricer_version: String,
}

struct Resolution {
    winning_outcome: String,
    resolved_at: DateTime<Utc>,
}

/// What `tools/fillcheck` observed for one prediction row: the best price a
/// counterparty was demonstrably reachable at on each side, in this row's
/// outcome units. `None` means nobody traded that side at all — no fill was
/// available at any price, which is a different statement from "a bad price".
struct Fill {
    best_bid: Option<f64>,
    best_ask: Option<f64>,
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
    /// Which build of the variant's pricer produced this number; empty for
    /// rows predating the column. Aggregated as its own level so a mid-trial
    /// model change is scored separately instead of averaged away.
    pricer_version: String,
    prediction: f64,
    market_price: f64,
    actual: f64,
    brier: f64,
    market_brier: f64,
    improvement: f64,
    logloss: f64,
    /// Did `fillcheck` look at this row at all? False means unaudited, which
    /// is not the same as "no counterparty".
    audited: bool,
    /// Best price a counterparty was observed at on the side we wanted.
    /// `None` on an audited row means nobody traded that side at any price.
    best_price: Option<f64>,
    /// What the trade was worth per share at that price: the gap between the
    /// best reachable price and our own probability, signed so positive always
    /// means "the price available paid us".
    exec_edge: Option<f64>,
}

struct AggRow {
    level: String,
    key: String,
    n: u64,
    mean_improvement: f64,
    mean_brier: f64,
    mean_market_brier: f64,
    mean_logloss: f64,
    /// Rows in this bucket that `fillcheck` audited at all.
    n_known_fill: u64,
    /// …of those, the ones where a counterparty was actually reachable at a
    /// price at least as good as the one we were scored against.
    n_fillable: u64,
    /// Mean per-share edge over the fillable rows only — what the trade paid
    /// when the trade existed. `NaN` when none did.
    mean_exec_edge: f64,
    /// Distinct markets in this bucket — the number of independent-ish
    /// observations, as opposed to `n`, which counts mornings.
    n_markets: u64,
    /// Cluster-robust 95% interval for the mean improvement, **clustering on
    /// market**. Each market contributes one observation (its own mean), so a
    /// market predicted four times cannot count as four.
    ///
    /// Gate 1 of the pre-registered trial rule (`ops/decisions.md`, 2026-07-30)
    /// asks whether this interval excludes zero. Without it the reviewer has to
    /// hand-compute the number that judges the variant, which is the reviewer
    /// judging themselves. `NaN` when fewer than two markets.
    ci_lo: f64,
    ci_hi: f64,
    /// Mean improvement over MARKETS — each market collapsed to its own mean
    /// first. This is the quantity `ci_lo`/`ci_hi` bound, and the one the
    /// pre-registered trial rule judges. `mean_improvement` is the row mean and
    /// is reported beside it, never instead of it.
    mean_improvement_market: f64,
}

/// Two-sided 95% Student-t quantiles by degrees of freedom, 1..=30, then the
/// normal limit. A table rather than a dependency: it is eleven lines, exact
/// where it matters (small samples), and auditable by eye.
fn t_crit_95(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228,
        2.201, 2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086,
        2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::NAN,
        d if d <= 30 => T[d - 1],
        _ => 1.960,
    }
}

/// Mean and cluster-robust 95% interval of per-cluster means.
///
/// Deliberately the simple thing: collapse each cluster to its own mean, then a
/// t-interval across clusters. With one prediction per market per morning that
/// is the honest treatment — four rows on one barrier touch are one event, and
/// on 2026-07-27 four such rows moved the firm's headline from +0.0009 to
/// −0.0172 while being a single draw.
fn cluster_ci(per_cluster: &[f64]) -> (f64, f64) {
    let k = per_cluster.len();
    if k < 2 {
        return (f64::NAN, f64::NAN);
    }
    let mean = per_cluster.iter().sum::<f64>() / k as f64;
    let var = per_cluster.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (k as f64 - 1.0);
    let se = (var / k as f64).sqrt();
    let t = t_crit_95(k - 1);
    (mean - t * se, mean + t * se)
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
    let fills = load_fills(&dir.join("fills.csv"))?;

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

        // We take the side the market is wrong about: our probability below
        // the quote means we sell the outcome, above it means we buy. Whether
        // that trade was worth anything is measured against the price a
        // counterparty was actually observed at, not against the midpoint.
        let selling = p.prediction <= p.market_price;
        let fill = fills.get(&fill_key(&p.timestamp_raw, &p.market_slug, &p.outcome));
        let best_price = fill.and_then(|f| if selling { f.best_bid } else { f.best_ask });
        let exec_edge = best_price.map(|b| if selling { b - p.prediction } else { p.prediction - b });
        let audited = fill.is_some();

        scored.push(ScoredRow {
            timestamp: p.timestamp_raw.clone(),
            market_slug: p.market_slug.clone(),
            outcome: p.outcome.clone(),
            family: p.family.clone(),
            variant: p.variant.clone(),
            model: p.model.clone(),
            status: p.status.clone(),
            horizon: horizon_bucket(p.timestamp, res.resolved_at).to_string(),
            pricer_version: p.pricer_version.clone(),
            prediction: p.prediction,
            market_price: p.market_price,
            actual,
            brier,
            market_brier,
            improvement: market_brier - brier,
            logloss: logloss(p.prediction, actual),
            audited,
            best_price,
            exec_edge,
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
            // HARD ERROR, unlike a malformed prediction row, which is warned and
            // skipped. A resolution is a join key: dropping one silently removes
            // EVERY prediction on that market from the score, and the headline
            // then looks complete while being computed on less evidence.
            //
            // This happened. On 2026-07-27 a note containing an unquoted comma
            // ("re-added leg, window from Jul 25") made a row 6 fields wide; it
            // was warned and skipped, and the trial reported 25 scored rows for
            // two days when it had 26. The warning was printed the whole time
            // and read by nobody. resolutions.csv is small and hand-appended, so
            // a malformed line there is always a mistake worth stopping for.
            return Err(format!(
                "{}: line {} has {} fields, expected {ncols}. A malformed resolution silently \
                 drops every prediction on that market from the score — fix the row (quote any \
                 field containing a comma) and re-run.",
                path.display(),
                line.map(|l| l.to_string()).unwrap_or_else(|| "?".into()),
                rec.len(),
            ));
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

/// Join key for fills.csv. A prediction row is identified by when it was made,
/// which market, and which outcome token — the same triple fillcheck emits.
fn fill_key(timestamp: &str, slug: &str, outcome: &str) -> String {
    format!("{timestamp}|{slug}|{}", outcome.to_ascii_lowercase())
}

/// Load `fills.csv` if `tools/fillcheck` has been run. Absent file is not an
/// error: scoring still reports calibration, it just cannot say whether any of
/// it was reachable.
fn load_fills(path: &Path) -> Result<HashMap<String, Fill>, String> {
    let mut map = HashMap::new();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(map),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    if content.trim().is_empty() {
        return Ok(map);
    }
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(content.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| format!("{}: {e}", path.display()))?
        .clone();
    let i_ts = col(&headers, "timestamp", path)?;
    let i_slug = col(&headers, "market_slug", path)?;
    let i_outcome = col(&headers, "outcome", path)?;
    let i_bid = col(&headers, "bid_life", path)?;
    let i_ask = col(&headers, "ask_life", path)?;

    for rec in rdr.records().flatten() {
        let field = |i: usize| rec.get(i).unwrap_or("").trim();
        // An empty price means no counterparty was ever observed on that side.
        let price = |i: usize| field(i).parse::<f64>().ok().filter(|v| v.is_finite());
        map.insert(
            fill_key(field(i_ts), field(i_slug), field(i_outcome)),
            Fill { best_bid: price(i_bid), best_ask: price(i_ask) },
        );
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
    // Optional: added 2026-07-28, so archived CSVs and fixtures without it stay readable.
    let i_pricer = col(&headers, "pricer_version", path).ok();
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
            pricer_version: i_pricer.map(field).unwrap_or_default(),
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
    /// Running sums for one bucket. Fill statistics count only the rows
    /// fillcheck had an answer for, so an unaudited row never dilutes them.
    #[derive(Default)]
    struct Acc {
        n: u64,
        improvement: f64,
        brier: f64,
        market_brier: f64,
        logloss: f64,
        n_known_fill: u64,
        n_fillable: u64,
        exec_edge: f64,
        /// Per-market sums, so each market can be collapsed to its own mean
        /// before the interval is taken across markets.
        by_market: HashMap<String, (u64, f64)>,
    }

    let mut acc: HashMap<(usize, String), Acc> = HashMap::new();
    for r in rows {
        let keys = [
            (0usize, format!("{}/{}", r.family, r.variant)),
            (1, r.family.clone()),
            (2, r.model.clone()),
            (3, r.status.clone()),
            (4, r.horizon.clone()),
            // Per MARKET, because rows are not independent observations. We
            // predict the same market every morning, so one barrier touch is
            // scored once per day it was open — inflating a win and a loss
            // alike. On 2026-07-27 four rows on `will-wti-dip-to-85` swung the
            // firm's whole headline from +0.0009 to -0.0172; they are one
            // event. Read the per-market level to see how many EVENTS a
            // conclusion rests on, not how many rows.
            (5, r.market_slug.clone()),
            // Per PRICER VERSION, so a model change is scored as the change it
            // is rather than absorbed into the variant's running average. A
            // variant that revises its pricer mid-trial is two experiments
            // sharing a name; without this level the revision is only visible
            // to whoever remembers the date it shipped. Rows written before
            // the column existed aggregate under "unversioned" — an honest
            // label, not a bucket to compare against.
            (6, if r.pricer_version.is_empty() { "unversioned".into() } else { r.pricer_version.clone() }),
            (7, "overall".to_string()),
        ];
        for k in keys {
            let e = acc.entry(k).or_default();
            e.n += 1;
            let m = e.by_market.entry(r.market_slug.clone()).or_insert((0, 0.0));
            m.0 += 1;
            m.1 += r.improvement;
            e.improvement += r.improvement;
            e.brier += r.brier;
            e.market_brier += r.market_brier;
            e.logloss += r.logloss;
            if r.audited {
                e.n_known_fill += 1;
                // "Fillable" is the strict test: a counterparty existed at a
                // price at least as good as the one we were scored against.
                if r.best_price.is_some_and(|b| reached(b, r.market_price, r.prediction)) {
                    e.n_fillable += 1;
                    e.exec_edge += r.exec_edge.unwrap_or(0.0);
                }
            }
        }
    }
    let mut out: Vec<(usize, AggRow)> = acc
        .into_iter()
        .map(|((lvl, key), a)| {
            let nf = a.n as f64;
            // Collapse each market to its own mean, then take the interval
            // across markets — never across rows.
            let per_market: Vec<f64> =
                a.by_market.values().map(|(c, s)| s / *c as f64).collect();
            let (ci_lo, ci_hi) = cluster_ci(&per_market);
            let mean_improvement_market = if per_market.is_empty() {
                f64::NAN
            } else {
                per_market.iter().sum::<f64>() / per_market.len() as f64
            };
            (
                lvl,
                AggRow {
                    level: LEVEL_ORDER[lvl].to_string(),
                    key,
                    n: a.n,
                    mean_improvement: a.improvement / nf,
                    mean_brier: a.brier / nf,
                    mean_market_brier: a.market_brier / nf,
                    mean_logloss: a.logloss / nf,
                    n_known_fill: a.n_known_fill,
                    n_fillable: a.n_fillable,
                    mean_exec_edge: if a.n_fillable > 0 {
                        a.exec_edge / a.n_fillable as f64
                    } else {
                        f64::NAN
                    },
                    n_markets: a.by_market.len() as u64,
                    ci_lo,
                    ci_hi,
                    mean_improvement_market,
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

/// Did the observed price reach the one we were scored against? A seller
/// needs a bid at or above the midpoint, a buyer an ask at or below it.
fn reached(best: f64, market_price: f64, prediction: f64) -> bool {
    if prediction <= market_price {
        best >= market_price - 1e-9
    } else {
        best <= market_price + 1e-9
    }
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
            r.pricer_version.as_str(),
            &fmt_f(r.prediction),
            &fmt_f(r.market_price),
            &fmt_f(r.actual),
            &fmt_f(r.brier),
            &fmt_f(r.market_brier),
            &fmt_f(r.improvement),
            &fmt_f(r.logloss),
            &r.best_price.map(fmt_f).unwrap_or_default(),
            // Blank, not "false", when nobody audited this row.
            &if r.audited {
                r.best_price
                    .is_some_and(|b| reached(b, r.market_price, r.prediction))
                    .to_string()
            } else {
                String::new()
            },
            &r.exec_edge.map(fmt_f).unwrap_or_default(),
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
            &r.n_markets.to_string(),
            &if r.mean_improvement_market.is_finite() { fmt_f(r.mean_improvement_market) } else { String::new() },
            &if r.ci_lo.is_finite() { fmt_f(r.ci_lo) } else { String::new() },
            &if r.ci_hi.is_finite() { fmt_f(r.ci_hi) } else { String::new() },
            &fmt_f(r.mean_improvement),
            &fmt_f(r.mean_brier),
            &fmt_f(r.mean_market_brier),
            &fmt_f(r.mean_logloss),
            &r.n_known_fill.to_string(),
            &r.n_fillable.to_string(),
            &if r.mean_exec_edge.is_finite() { fmt_f(r.mean_exec_edge) } else { String::new() },
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
        "{:<8} {:<32} {:>5} {:>10} {:>11} {:>11} {:>10} {:>10} {:>10}",
        "level", "key", "n", "mean_imp", "mean_brier", "mkt_brier", "logloss", "fillable", "exec_edge"
    );
    println!("{}", "-".repeat(115));
    for a in &stats.aggregates {
        let fillable = if a.n_known_fill > 0 {
            format!("{}/{}", a.n_fillable, a.n_known_fill)
        } else {
            "—".to_string()
        };
        let edge = if a.mean_exec_edge.is_finite() {
            format!("{:+.4}", a.mean_exec_edge)
        } else {
            "—".to_string()
        };
        println!(
            "{:<8} {:<32} {:>5} {:>+10.4} {:>11.4} {:>11.4} {:>10.4} {:>10} {:>10}",
            a.level,
            a.key,
            a.n,
            a.mean_improvement,
            a.mean_brier,
            a.mean_market_brier,
            a.mean_logloss,
            fillable,
            edge
        );
    }
    println!();

    // Beating the quote and being able to trade on it are separate claims, and
    // the second one is the one that pays. Say it out loud whenever we know.
    if let Some(o) = stats
        .aggregates
        .iter()
        .find(|a| a.level == "overall" && a.n_known_fill > 0)
    {
        let pct = 100.0 * o.n_fillable as f64 / o.n_known_fill as f64;
        println!(
            "tradeability: {}/{} audited rows ({pct:.0}%) had a counterparty at or better than \
             the price they were scored against.",
            o.n_fillable, o.n_known_fill
        );
        if o.n_fillable == 0 {
            println!("             none of this improvement was reachable. It is calibration, not money.");
        } else if pct < 50.0 {
            println!(
                "             the rest was scored against a midpoint nobody offered — \
                 see wiki/reference/midpoint-is-not-a-fill.md"
            );
        }
        println!();
    } else if stats.scored > 0 {
        println!("tradeability: unknown — run tools/fillcheck to write predictions/fills.csv.");
        println!();
    }
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
        // By NAME, not by position. This test used to index r[3..7] and broke
        // the moment three columns were inserted ahead of them — the same
        // schema-shift bug the rest of the codebase addresses columns by name
        // to avoid. A fixture that has to be edited whenever a column is added
        // is a fixture that will eventually be edited wrongly.
        let at = |name: &str| {
            SCORES_HEADER.iter().position(|h| *h == name).expect("column in SCORES_HEADER")
        };
        let (i_lvl, i_key, i_n) = (at("level"), at("key"), at("n"));
        let (i_imp, i_br, i_mbr, i_ll) = (
            at("mean_improvement"),
            at("mean_brier"),
            at("mean_market_brier"),
            at("mean_logloss"),
        );
        let mut order = Vec::new();
        let mut map = HashMap::new();
        for rec in rdr.records() {
            let r = rec.unwrap();
            let key = (r[i_lvl].to_string(), r[i_key].to_string());
            order.push(key.clone());
            map.insert(
                key,
                (
                    r[i_n].parse::<u64>().unwrap(),
                    r[i_imp].parse::<f64>().unwrap(),
                    r[i_br].parse::<f64>().unwrap(),
                    r[i_mbr].parse::<f64>().unwrap(),
                    r[i_ll].parse::<f64>().unwrap(),
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
