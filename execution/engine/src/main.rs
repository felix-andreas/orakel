//! `engine` — run the execution policies over the signal sets.
//!
//! ```text
//! engine run --set <name|all> --policy <file|name|all>
//! engine report                      # rebuild summary.csv + SUMMARY.md from results/
//! ```
//!
//! This file is the only place in the binary that touches the filesystem; the
//! simulation itself is a pure function in the library (`engine::simulate`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use engine::metrics::MIN_N_FOR_A_WINNER;
use engine::{fmt_f, parse_signals_csv, simulate, Policy, SimResult};

/// The order DESIGN.md §5 introduces the policies in: baseline first, house
/// style last. Anything not listed sorts after, alphabetically.
const POLICY_ORDER: [&str; 8] =
    ["mirror", "gate", "kelly", "anchor", "fade", "patient", "sniper", "harvest"];

const SUMMARY_HEADER: [&str; 26] = [
    "signal_set",
    "policy",
    "policy_version",
    "n_signals",
    "n_trades",
    "unfundable",
    "depth_unknown",
    "epsilon_unavailable",
    "delay_unavailable",
    "staked_usd",
    "net_pnl_usd",
    "cents_per_trade",
    "cents_per_trade_se",
    "t_stat",
    "hit_rate",
    "mean_hold_days",
    "return_on_locked_capital",
    "annualized_return_on_locked_capital",
    "capital_efficiency",
    "max_capital_efficiency",
    "max_drawdown_usd",
    "longest_losing_streak",
    "synthetic_fill_share",
    "date_start",
    "date_end",
    "underpowered",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        usage();
        return;
    }
    let root = execution_root();
    let result = match args[0].as_str() {
        "run" => run(&args[1..], &root),
        "report" => report(&root.join("results")),
        other => Err(format!("unknown command '{other}' (try --help)")),
    };
    if let Err(e) = result {
        eprintln!("engine: {e}");
        std::process::exit(1);
    }
}

fn usage() {
    println!("usage:");
    println!("  engine run --set <name|all> --policy <file|name|all> [--root <execution-dir>]");
    println!("  engine report [--root <execution-dir>]");
    println!();
    println!("signal sets:  <root>/signals/<name>.csv");
    println!("policies:     <root>/policies/<name>-v<version>.toml");
    println!("results:      <root>/results/<set>/<policy>.json, results/summary.csv, results/SUMMARY.md");
}

/// `execution/` — next to this crate when built in-repo, else cwd-relative.
fn execution_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if manifest.join("policies").is_dir() {
        return manifest.canonicalize().unwrap_or(manifest);
    }
    for cand in [".", "execution", "../execution"] {
        let p = PathBuf::from(cand);
        if p.join("policies").is_dir() {
            return p.canonicalize().unwrap_or(p);
        }
    }
    manifest
}

fn run(args: &[String], default_root: &Path) -> Result<(), String> {
    let mut set = String::new();
    let mut policy = String::new();
    let mut root = default_root.to_path_buf();
    let mut i = 0;
    while i < args.len() {
        let need = |i: usize, what: &str| -> Result<String, String> {
            args.get(i + 1).cloned().ok_or_else(|| format!("{what} needs a value"))
        };
        match args[i].as_str() {
            "--set" => {
                set = need(i, "--set")?;
                i += 2;
            }
            "--policy" => {
                policy = need(i, "--policy")?;
                i += 2;
            }
            "--root" => {
                root = PathBuf::from(need(i, "--root")?);
                i += 2;
            }
            a => return Err(format!("unknown argument '{a}'")),
        }
    }
    if set.is_empty() || policy.is_empty() {
        return Err("run needs --set and --policy".to_string());
    }

    let sets = resolve_sets(&root.join("signals"), &set)?;
    let policies = resolve_policies(&root.join("policies"), &policy)?;
    let results_dir = root.join("results");

    for (set_name, set_path) in &sets {
        let text = std::fs::read_to_string(set_path)
            .map_err(|e| format!("cannot read {}: {e}", set_path.display()))?;
        let (signals, warnings) = parse_signals_csv(&text)?;
        for w in &warnings {
            eprintln!("warning: {}: {w}", set_path.display());
        }
        if signals.is_empty() {
            eprintln!("warning: {} has no usable signals — skipped", set_path.display());
            continue;
        }
        println!("== {set_name}: {} signals ({} parse warnings)", signals.len(), warnings.len());
        let out_dir = results_dir.join(set_name);
        std::fs::create_dir_all(&out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;

        for (pol_name, pol_path) in &policies {
            let ptext = std::fs::read_to_string(pol_path)
                .map_err(|e| format!("cannot read {}: {e}", pol_path.display()))?;
            let pol = Policy::from_toml(&ptext)
                .map_err(|e| format!("{}: {e}", pol_path.display()))?;
            let res = simulate(&signals, &pol)?;
            print_one(&res);
            let json = serde_json::to_string_pretty(&res)
                .map_err(|e| format!("serializing {pol_name}: {e}"))?;
            let path = out_dir.join(format!("{}.json", res.policy));
            std::fs::write(&path, json + "\n")
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
    }
    report(&results_dir)
}

fn resolve_sets(dir: &Path, want: &str) -> Result<Vec<(String, PathBuf)>, String> {
    if want != "all" {
        let p = if Path::new(want).is_file() {
            PathBuf::from(want)
        } else {
            dir.join(format!("{want}.csv"))
        };
        if !p.is_file() {
            return Err(format!("signal set not found: {}", p.display()));
        }
        return Ok(vec![(stem(&p), p)]);
    }
    let mut out: Vec<(String, PathBuf)> = list_dir(dir, "csv")?.into_iter().map(|p| (stem(&p), p)).collect();
    out.sort();
    if out.is_empty() {
        return Err(format!("no signal sets in {}", dir.display()));
    }
    Ok(out)
}

fn resolve_policies(dir: &Path, want: &str) -> Result<Vec<(String, PathBuf)>, String> {
    let all = list_dir(dir, "toml")?;
    let mut out: Vec<(String, PathBuf)> = if want == "all" {
        all.into_iter().map(|p| (policy_name(&p), p)).collect()
    } else if Path::new(want).is_file() {
        let p = PathBuf::from(want);
        vec![(policy_name(&p), p)]
    } else {
        let hits: Vec<PathBuf> = all.into_iter().filter(|p| policy_name(p) == want).collect();
        if hits.is_empty() {
            return Err(format!("policy '{want}' not found in {}", dir.display()));
        }
        hits.into_iter().map(|p| (policy_name(&p), p)).collect()
    };
    out.sort_by_key(|(n, _)| (order_of(n), n.clone()));
    Ok(out)
}

/// `anchor-v1.toml` -> `anchor`.
fn policy_name(p: &Path) -> String {
    let s = stem(p);
    match s.rsplit_once("-v") {
        Some((base, ver)) if ver.chars().all(|c| c.is_ascii_digit()) => base.to_string(),
        _ => s,
    }
}

fn stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string()
}

fn order_of(name: &str) -> usize {
    POLICY_ORDER.iter().position(|p| *p == name).unwrap_or(POLICY_ORDER.len())
}

fn list_dir(dir: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

fn print_one(r: &SimResult) {
    let m = &r.metrics;
    println!(
        "   {:<8} n={:<5} pnl=${:<10} c/trade={:>8} +-{:<7} t={:>6} hit={:>6} annROLC={:>9} capeff={:>7}{}",
        r.policy,
        m.n,
        fmt_f(m.net_pnl_usd),
        opt(m.cents_per_trade),
        opt(m.cents_per_trade_se),
        opt(m.t_stat),
        opt(m.hit_rate),
        opt(m.annualized_return_on_locked_capital),
        opt(m.capital_efficiency),
        if m.underpowered { "  [underpowered]" } else { "" },
    );
}

fn opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.4}")).unwrap_or_else(|| "-".to_string())
}

// ---------------------------------------------------------------- reporting

fn report(results_dir: &Path) -> Result<(), String> {
    let mut by_set: BTreeMap<String, Vec<SimResult>> = BTreeMap::new();
    let rd = std::fs::read_dir(results_dir)
        .map_err(|e| format!("cannot read {}: {e}", results_dir.display()))?;
    for e in rd.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let set = e.file_name().to_string_lossy().to_string();
        for p in list_dir(&e.path(), "json")? {
            let text = std::fs::read_to_string(&p)
                .map_err(|e| format!("cannot read {}: {e}", p.display()))?;
            let r: SimResult = serde_json::from_str(&text)
                .map_err(|e| format!("cannot parse {}: {e}", p.display()))?;
            by_set.entry(set.clone()).or_default().push(r);
        }
    }
    for v in by_set.values_mut() {
        v.sort_by_key(|r| (order_of(&r.policy), r.policy.clone()));
    }

    write_summary_csv(&results_dir.join("summary.csv"), &by_set)?;
    write_summary_md(&results_dir.join("SUMMARY.md"), &by_set)?;
    println!(
        "wrote {} and {}",
        results_dir.join("summary.csv").display(),
        results_dir.join("SUMMARY.md").display()
    );
    Ok(())
}

fn write_summary_csv(
    path: &Path,
    by_set: &BTreeMap<String, Vec<SimResult>>,
) -> Result<(), String> {
    let mut w = csv::Writer::from_path(path)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    w.write_record(SUMMARY_HEADER).map_err(|e| e.to_string())?;
    for (set, rows) in by_set {
        for r in rows {
            let m = &r.metrics;
            let c = &r.counts;
            w.write_record([
                set.as_str(),
                r.policy.as_str(),
                &r.policy_version.to_string(),
                &c.signals.to_string(),
                &m.n.to_string(),
                &c.unfundable.to_string(),
                &c.depth_unknown.to_string(),
                &c.epsilon_unavailable.to_string(),
                &c.delay_unavailable.to_string(),
                &fmt_f(m.staked_usd),
                &fmt_f(m.net_pnl_usd),
                &optf(m.cents_per_trade),
                &optf(m.cents_per_trade_se),
                &optf(m.t_stat),
                &optf(m.hit_rate),
                &optf(m.mean_hold_days),
                &optf(m.return_on_locked_capital),
                &optf(m.annualized_return_on_locked_capital),
                &optf(m.capital_efficiency),
                &optf(m.max_capital_efficiency),
                &fmt_f(m.max_drawdown_usd),
                &m.longest_losing_streak.to_string(),
                &optf(m.synthetic_fill_share),
                r.date_start.as_str(),
                r.date_end.as_str(),
                if m.underpowered { "yes" } else { "no" },
            ])
            .map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())
}

fn optf(v: Option<f64>) -> String {
    v.map(fmt_f).unwrap_or_default()
}

fn md(v: Option<f64>, dp: usize) -> String {
    match v {
        Some(x) => format!("{x:.dp$}"),
        None => "—".to_string(),
    }
}

fn pct(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1}%", x * 100.0),
        None => "—".to_string(),
    }
}

fn write_summary_md(
    path: &Path,
    by_set: &BTreeMap<String, Vec<SimResult>>,
) -> Result<(), String> {
    let mut s = String::new();
    s.push_str("# Execution results — the matrix\n\n");
    s.push_str(
        "Generated by `engine report`. Every number carries its sample size; a policy with \
         n < 30 trades is labelled **underpowered** and the engine refuses to rank it \
         (DESIGN.md §7). The deciding metric is **annualized return on locked capital** \
         (annROLC), not cents per trade — see DESIGN.md §3.\n\n",
    );

    for (set, rows) in by_set {
        let (start, end) = rows
            .first()
            .map(|r| (r.set_date_start.clone(), r.set_date_end.clone()))
            .unwrap_or_default();
        s.push_str(&format!("## `{set}` — {start} .. {end}\n\n"));
        if let Some(r) = rows.first() {
            s.push_str(&format!(
                "{} signals in the set. One regime; read every row below as conditional on this window.\n\n",
                r.counts.signals
            ));
        }

        s.push_str("### Headline\n\n");
        s.push_str("| policy | n | c/trade | ± se | t | hit | hold d | ROLC | **annROLC** | cap.eff | net $ | maxDD $ | synth fills |\n");
        s.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        for r in rows {
            let m = &r.metrics;
            s.push_str(&format!(
                "| {}{} | {} | {} | {} | {} | {} | {} | {} | **{}** | {} | {} | {} | {} |\n",
                r.policy,
                if m.underpowered { " ⚠" } else { "" },
                m.n,
                md(m.cents_per_trade, 2),
                md(m.cents_per_trade_se, 2),
                md(m.t_stat, 2),
                pct(m.hit_rate),
                md(m.mean_hold_days, 2),
                pct(m.return_on_locked_capital),
                pct(m.annualized_return_on_locked_capital),
                pct(m.capital_efficiency),
                md(Some(m.net_pnl_usd), 2),
                md(Some(m.max_drawdown_usd), 2),
                pct(m.synthetic_fill_share),
            ));
        }
        s.push_str("\n⚠ = underpowered (n < 30). ROLC = return on locked capital.\n\n");

        s.push_str("### Where the signals went\n\n");
        s.push_str("| policy | signals | traded | no exec. edge | side | edge | %ile | spread | depth | unfundable | mkt cap | delay n/a | ε n/a | depth n/a |\n");
        s.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        for r in rows {
            let c = &r.counts;
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                r.policy,
                c.signals,
                c.traded,
                c.no_executable_edge,
                c.side_excluded,
                c.below_min_edge,
                c.below_edge_percentile,
                c.spread_too_wide,
                c.depth_too_thin,
                c.unfundable,
                c.market_cap_full,
                c.delay_unavailable,
                c.epsilon_unavailable,
                c.depth_unknown,
            ));
        }
        s.push_str("\nThe first nine columns are terminal and sum to `signals`; `ε n/a` and `depth n/a` are *reported, non-gating* — screens the data could not support, counted rather than silently passed.\n\n");

        s.push_str("### Side and fill quality\n\n");
        s.push_str("| policy | buys n | buys c/trade | sells n | sells c/trade | real-book n | real-book c/trade |\n");
        s.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
        for r in rows {
            let g = |v: &Vec<engine::Group>, k: &str| -> (String, String) {
                match engine::metrics::group(v, k) {
                    Some(x) => (x.n.to_string(), md(x.cents_per_trade, 2)),
                    None => ("0".to_string(), "—".to_string()),
                }
            };
            let (bn, bc) = g(&r.by_side, "buy");
            let (sn, sc) = g(&r.by_side, "sell");
            let (rn, rc) = g(&r.by_fill, "real-book");
            s.push_str(&format!("| {} | {bn} | {bc} | {sn} | {sc} | {rn} | {rc} |\n", r.policy));
        }
        s.push('\n');

        s.push_str(&contrasts(rows));
        s.push_str(&reading(rows));
        s.push_str(&cannot_tell(rows));
    }

    s.push_str("## Reading notes\n\n");
    s.push_str(
        "- **Fills are never at mid.** Buys lift the ask, sells hit the bid; where only a \
         midpoint exists the policy's `assumed_spread` is applied symmetrically and the \
         trade is marked synthetic. A policy whose result rests on synthetic fills is \
         flagged, not celebrated (DESIGN.md §4).\n",
    );
    s.push_str(
        "- **annROLC** = `Σ pnl / Σ (capital_locked × days_held) × 365`, the formula in \
         DESIGN.md §3. It is a rate, not a fund return: multiply by `cap.eff` to see what \
         the bankroll would actually have earned.\n",
    );
    s.push_str(
        "- **cap.eff > 100%** means the stated bankroll could not have funded the policy; \
         the engine does not silently shrink positions to fit.\n",
    );
    s.push_str(
        "- Equity is bankroll plus *realized* PnL — open positions are not marked to market, \
         because the signal sets carry no price for every day of every hold.\n",
    );
    std::fs::write(path, s).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// The eight policies exist to answer six specific questions. Answer them with
/// the numbers, and refuse to answer where the sample is too small.
fn contrasts(rows: &[SimResult]) -> String {
    let find = |name: &str| rows.iter().find(|r| r.policy == name);
    let pairs: [(&str, &str, &str); 6] = [
        ("gate", "mirror", "is filtering alone worth anything?"),
        ("kelly", "gate", "does conviction-sizing beat equal stakes?"),
        ("anchor", "kelly", "what does capacity realism cost?"),
        ("fade", "anchor", "are our sells really the whole edge?"),
        ("patient", "fade", "does a 24h delay improve sell fills?"),
        ("harvest", "fade", "is patience actually paid for?"),
    ];
    let mut s = String::from("### What separates the policies\n\n");
    s.push_str("| contrast | question | n (a / b) | annROLC a → b | c/trade a → b | verdict |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for (a, b, q) in pairs {
        let (ra, rb) = (find(a), find(b));
        let (Some(ra), Some(rb)) = (ra, rb) else { continue };
        let na = ra.metrics.n;
        let nb = rb.metrics.n;
        let verdict = if na < MIN_N_FOR_A_WINNER || nb < MIN_N_FOR_A_WINNER {
            "**underpowered** — no call".to_string()
        } else {
            match (
                ra.metrics.annualized_return_on_locked_capital,
                rb.metrics.annualized_return_on_locked_capital,
            ) {
                (Some(x), Some(y)) if x > y => format!("{a} ahead by {:.0} pp", (x - y) * 100.0),
                (Some(x), Some(y)) if y > x => format!("{b} ahead by {:.0} pp", (y - x) * 100.0),
                _ => "no difference measurable".to_string(),
            }
        };
        s.push_str(&format!(
            "| `{b}` → `{a}` | {q} | {nb} / {na} | {} → {} | {} → {} | {verdict} |\n",
            pct(rb.metrics.annualized_return_on_locked_capital),
            pct(ra.metrics.annualized_return_on_locked_capital),
            md(rb.metrics.cents_per_trade, 2),
            md(ra.metrics.cents_per_trade, 2),
        ));
    }
    // sniper is a fan-out of anchor, not a chain step
    if let (Some(sn), Some(an)) = (find("sniper"), find("anchor")) {
        let verdict = if sn.metrics.n < MIN_N_FOR_A_WINNER || an.metrics.n < MIN_N_FOR_A_WINNER {
            "**underpowered** — no call".to_string()
        } else {
            match (
                sn.metrics.annualized_return_on_locked_capital,
                an.metrics.annualized_return_on_locked_capital,
            ) {
                (Some(x), Some(y)) if x > y => format!("sniper ahead by {:.0} pp", (x - y) * 100.0),
                (Some(x), Some(y)) if y > x => format!("anchor ahead by {:.0} pp", (y - x) * 100.0),
                _ => "no difference measurable".to_string(),
            }
        };
        s.push_str(&format!(
            "| `anchor` → `sniper` | does concentration beat breadth? | {} / {} | {} → {} | {} → {} | {verdict} |\n",
            an.metrics.n,
            sn.metrics.n,
            pct(an.metrics.annualized_return_on_locked_capital),
            pct(sn.metrics.annualized_return_on_locked_capital),
            md(an.metrics.cents_per_trade, 2),
            md(sn.metrics.cents_per_trade, 2),
        ));
    }
    s.push('\n');

    let powered: Vec<&SimResult> =
        rows.iter().filter(|r| r.metrics.n >= MIN_N_FOR_A_WINNER).collect();
    if powered.is_empty() {
        s.push_str(&format!(
            "**No policy on this set reaches n = {MIN_N_FOR_A_WINNER}. Per DESIGN.md §7 the engine names no winner here.**\n\n"
        ));
    } else {
        let best = powered
            .iter()
            .max_by(|a, b| {
                a.metrics
                    .annualized_return_on_locked_capital
                    .unwrap_or(f64::MIN)
                    .partial_cmp(&b.metrics.annualized_return_on_locked_capital.unwrap_or(f64::MIN))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        s.push_str(&format!(
            "Highest annROLC among the {} policies that reach n = {MIN_N_FOR_A_WINNER}: **`{}`** at {} on {} trades ({} c/trade, t = {}). Policies below that sample size are listed but not ranked.\n\n",
            powered.len(),
            best.policy,
            pct(best.metrics.annualized_return_on_locked_capital),
            best.metrics.n,
            md(best.metrics.cents_per_trade, 2),
            md(best.metrics.t_stat, 2),
        ));
    }
    s
}

/// A short, mechanical reading of the matrix: the facts a human would extract
/// first, generated from the numbers so they can never drift from them.
fn reading(rows: &[SimResult]) -> String {
    let find = |name: &str| rows.iter().find(|r| r.policy == name);
    let mut s = String::from("### The short reading\n\n");
    // Facts about the *set* hold even when no policy is powered enough to rank.
    let silent: Vec<&str> =
        rows.iter().filter(|r| r.metrics.n == 0).map(|r| r.policy.as_str()).collect();
    if !silent.is_empty() {
        // Which gate stopped the most of them, and on how many signals.
        let mut tally: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
        for r in rows.iter().filter(|r| r.metrics.n == 0) {
            let (reason, n) = dominant_rejection(r);
            let e = tally.entry(reason).or_insert((0, 0));
            e.0 += 1;
            e.1 = e.1.max(n);
        }
        let modal = tally.iter().max_by_key(|(_, (k, _))| *k);
        s.push_str(&format!(
            "- **{} of {} policies took no trades at all** ({}){}\n",
            silent.len(),
            rows.len(),
            silent.join(", "),
            match modal {
                Some((reason, (k, n))) => format!(
                    ". For {k} of them the binding constraint was `{reason}`, which rejected up to {n} of the {} signals in the set.",
                    rows.first().map(|r| r.counts.signals).unwrap_or(0)
                ),
                None => ".".to_string(),
            },
        ));
    }

    let powered: Vec<&SimResult> =
        rows.iter().filter(|r| r.metrics.n >= MIN_N_FOR_A_WINNER).collect();
    if powered.is_empty() {
        s.push_str(
            "- No policy here reaches n = 30, so nothing below the counts can be read. The engine stops.\n\n",
        );
        return s;
    }

    // 1. The two metrics disagree, and that is the point of DESIGN.md §3.
    let by_cents = powered
        .iter()
        .max_by(|a, b| cmpf(a.metrics.cents_per_trade, b.metrics.cents_per_trade))
        .unwrap();
    let by_ann = powered
        .iter()
        .max_by(|a, b| {
            cmpf(
                a.metrics.annualized_return_on_locked_capital,
                b.metrics.annualized_return_on_locked_capital,
            )
        })
        .unwrap();
    if by_cents.policy == by_ann.policy {
        s.push_str(&format!(
            "- `{}` leads on both cents/trade ({}) and annROLC ({}) — the two metrics agree here, which they need not.\n",
            by_ann.policy,
            md(by_ann.metrics.cents_per_trade, 2),
            pct(by_ann.metrics.annualized_return_on_locked_capital),
        ));
    } else {
        s.push_str(&format!(
            "- **The two metrics disagree, exactly as DESIGN.md §3 predicts.** Highest cents/trade is `{}` ({} on {} trades, {} days held); highest annualized return on locked capital is `{}` ({} on {} trades, {} days held). The capital-lockup rule, not the headline cents, is what separates them.\n",
            by_cents.policy,
            md(by_cents.metrics.cents_per_trade, 2),
            by_cents.metrics.n,
            md(by_cents.metrics.mean_hold_days, 2),
            by_ann.policy,
            pct(by_ann.metrics.annualized_return_on_locked_capital),
            by_ann.metrics.n,
            md(by_ann.metrics.mean_hold_days, 2),
        ));
    }

    // 2. Buys vs sells on the widest two-sided policy that traded both.
    if let Some(r) = rows.iter().find(|r| {
        engine::metrics::group(&r.by_side, "buy").is_some()
            && engine::metrics::group(&r.by_side, "sell").is_some()
    }) {
        let b = engine::metrics::group(&r.by_side, "buy").unwrap();
        let sell = engine::metrics::group(&r.by_side, "sell").unwrap();
        s.push_str(&format!(
            "- **Sides.** On `{}`, sells earned {} c/trade (n = {}, annROLC {}) against {} c/trade for buys (n = {}, annROLC {}).\n",
            r.policy,
            md(sell.cents_per_trade, 2),
            sell.n,
            pct(sell.annualized_return_on_locked_capital),
            md(b.cents_per_trade, 2),
            b.n,
            pct(b.annualized_return_on_locked_capital),
        ));
    }

    // 3. The delayed policy's sample is not the same sample.
    if let Some(p) = find("patient") {
        let c = &p.counts;
        if c.delay_unavailable > 0 {
            let frac = c.delay_unavailable_token_won as f64 / c.delay_unavailable as f64;
            s.push_str(&format!(
                "- **`patient` is not measured on the same sample as `fade`.** {} signals had no observation {}h later and were dropped; {:.0}% of those sit on markets that resolved in the token's favour — i.e. the dropped rows are enriched in the sell side's losses. Its {} number is therefore an upper bound, and the honest comparison to `fade` does not exist in this data.\n",
                c.delay_unavailable,
                p.entry_delay_hours,
                frac * 100.0,
                pct(p.metrics.annualized_return_on_locked_capital),
            ));
        }
    }

    // 4. Capacity: which policies the stated bankroll could not fund.
    let over: Vec<String> = rows
        .iter()
        .filter(|r| r.metrics.max_capital_efficiency.unwrap_or(0.0) > 1.0)
        .map(|r| {
            format!("{} ({:.0}%)", r.policy, r.metrics.max_capital_efficiency.unwrap() * 100.0)
        })
        .collect();
    if !over.is_empty() {
        s.push_str(&format!(
            "- **Capacity.** Peak deployment exceeded the stated bankroll for {}. Dollar PnL across policies is therefore not comparable; the rates are.\n",
            over.join(", ")
        ));
    }

    // 5. The fill quality caveat, with a number.
    let synth: Vec<&SimResult> = rows
        .iter()
        .filter(|r| r.metrics.synthetic_fill_share.unwrap_or(0.0) >= 0.999 && r.metrics.n > 0)
        .collect();
    if synth.len() == powered.len() && !synth.is_empty() {
        s.push_str(
            "- **Every fill here is synthetic.** No row in this set carries a real book, so all of the above is priced at `mid ± assumed_spread/2`. Per DESIGN.md §4 these results are flagged, not celebrated: the ranking is a hypothesis about execution, not a measurement of it.\n",
        );
    }
    s.push('\n');
    s
}

/// The rejection bucket that swallowed the most signals for one policy.
fn dominant_rejection(r: &SimResult) -> (&'static str, usize) {
    let c = &r.counts;
    let buckets: [(&'static str, usize); 10] = [
        ("no executable edge (our p sits inside the spread)", c.no_executable_edge),
        ("min_edge", c.below_min_edge),
        ("edge_percentile", c.below_edge_percentile),
        ("sides", c.side_excluded),
        ("min_spread_ok", c.spread_too_wide),
        ("min_depth_usd", c.depth_too_thin),
        ("unfundable depth", c.unfundable),
        ("max_per_market_usd", c.market_cap_full),
        ("no observation delay_hours later", c.delay_unavailable),
        ("stake below the minimum ticket", c.stake_too_small),
    ];
    buckets.into_iter().max_by_key(|(_, n)| *n).unwrap_or(("—", 0))
}

fn cmpf(a: Option<f64>, b: Option<f64>) -> std::cmp::Ordering {
    a.unwrap_or(f64::MIN).partial_cmp(&b.unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal)
}

/// Collapse notes that say the same thing with different numbers into one
/// bullet, so the caveat list stays readable as policies multiply.
fn cannot_tell(rows: &[SimResult]) -> String {
    let mut s = String::from("### What this sample cannot tell us\n\n");
    // Group by the note with every digit run masked, so "1161 of the trades ..."
    // and "793 of the trades ..." land in the same bucket.
    let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for r in rows {
        for n in &r.notes {
            let key = mask_digits(n);
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push((r.policy.clone(), n.clone()));
        }
    }
    for key in order {
        let entries = &groups[&key];
        let identical = entries.iter().all(|(_, txt)| *txt == entries[0].1);
        if identical {
            let names = if entries.len() == rows.len() {
                "all policies".to_string()
            } else {
                entries.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>().join(", ")
            };
            s.push_str(&format!("- **{names}** — {}\n", entries[0].1));
        } else {
            // Show one policy's note in full, then the others by their number.
            s.push_str(&format!("- **{}** — {}\n", entries[0].0, entries[0].1));
            let rest: Vec<String> = entries[1..]
                .iter()
                .map(|(p, txt)| match first_number(txt) {
                    Some(v) => format!("{p} {v}"),
                    None => p.clone(),
                })
                .collect();
            if !rest.is_empty() {
                s.push_str(&format!("  - same for: {}\n", rest.join(", ")));
            }
        }
    }
    s.push('\n');
    s
}

fn mask_digits(s: &str) -> String {
    let mut out = String::new();
    let mut in_num = false;
    for c in s.chars() {
        if c.is_ascii_digit() || (in_num && (c == '.' || c == ',')) {
            if !in_num {
                out.push('#');
                in_num = true;
            }
        } else {
            in_num = false;
            out.push(c);
        }
    }
    out
}

fn first_number(s: &str) -> Option<String> {
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || (!cur.is_empty() && (c == '.' || c == '%')) {
            cur.push(c);
            if c == '%' {
                return Some(cur);
            }
        } else if !cur.is_empty() {
            return Some(cur);
        }
    }
    (!cur.is_empty()).then_some(cur)
}

