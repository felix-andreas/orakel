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

const SUMMARY_HEADER: [&str; 31] = [
    "signal_set",
    "policy",
    "policy_version",
    "fee_model",
    "n_signals",
    "n_trades",
    "unfundable",
    "depth_unknown",
    "epsilon_unavailable",
    "delay_unavailable",
    "fee_rate_unmapped",
    "staked_usd",
    "gross_pnl_usd",
    "fees_usd",
    "fee_share_of_gross",
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
            // Versioned filename: a policy's v1 and v2 results must coexist, or
            // "keep v1 so old results stay attributable" (DESIGN.md §5) is a
            // rule the engine itself breaks on the next run.
            let path = out_dir.join(format!("{}-v{}.json", res.policy, res.policy_version));
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
        "   {:<8} v{} n={:<5} pnl=${:<10} fees=${:<9} c/trade={:>8} +-{:<7} t={:>6} hit={:>6} annROLC={:>9} capeff={:>7}{}",
        r.policy,
        r.policy_version,
        m.n,
        fmt_f(m.net_pnl_usd),
        fmt_f(m.fees_usd),
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

/// `results/<set>/<policy>-v<version>.json`, grouped set → version → policies.
type Matrix = BTreeMap<String, BTreeMap<u32, Vec<SimResult>>>;

fn report(results_dir: &Path) -> Result<(), String> {
    let mut by_set: Matrix = BTreeMap::new();
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
            by_set.entry(set.clone()).or_default().entry(r.policy_version).or_default().push(r);
        }
    }
    for versions in by_set.values_mut() {
        for v in versions.values_mut() {
            v.sort_by_key(|r| (order_of(&r.policy), r.policy.clone()));
        }
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

fn write_summary_csv(path: &Path, by_set: &Matrix) -> Result<(), String> {
    let mut w = csv::Writer::from_path(path)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    w.write_record(SUMMARY_HEADER).map_err(|e| e.to_string())?;
    for (set, versions) in by_set {
        for rows in versions.values() {
            for r in rows {
            let m = &r.metrics;
            let c = &r.counts;
            w.write_record([
                set.as_str(),
                r.policy.as_str(),
                &r.policy_version.to_string(),
                r.fee_model.as_str(),
                &c.signals.to_string(),
                &m.n.to_string(),
                &c.unfundable.to_string(),
                &c.depth_unknown.to_string(),
                &c.epsilon_unavailable.to_string(),
                &c.delay_unavailable.to_string(),
                &c.fee_rate_unmapped.to_string(),
                &fmt_f(m.staked_usd),
                &fmt_f(m.gross_pnl_usd),
                &fmt_f(m.fees_usd),
                &optf(m.fee_share_of_gross),
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

fn write_summary_md(path: &Path, by_set: &Matrix) -> Result<(), String> {
    let mut s = String::new();
    s.push_str("# Execution results — the matrix\n\n");
    s.push_str(
        "Generated by `engine report`. Every number carries its sample size; a policy with \
         n < 30 trades is labelled **underpowered** and the engine refuses to rank it \
         (DESIGN.md §7). The deciding metric is **annualized return on locked capital** \
         (annROLC), not cents per trade — see DESIGN.md §3.\n\n",
    );
    s.push_str(
        "> **Read the version number before the numbers.** The `-v1` policies charge **no \
         venue fee**, which is wrong: Polymarket has charged a taker fee since 2026-01-05. \
         The `-v2` policies charge it. v1 rows are kept only so earlier reports stay \
         attributable — **every conclusion should be read off v2.**\n\n",
    );

    for (set, versions) in by_set {
        let first = versions.values().flat_map(|v| v.iter()).next();
        let (start, end) = first
            .map(|r| (r.set_date_start.clone(), r.set_date_end.clone()))
            .unwrap_or_default();
        s.push_str(&format!("## `{set}` — {start} .. {end}\n\n"));
        if let Some(r) = first {
            s.push_str(&format!(
                "{} signals in the set. One regime; read every row below as conditional on this window.\n\n",
                r.counts.signals
            ));
        }

        if versions.len() > 1 {
            s.push_str(&fee_comparison(versions));
        }

        // Newest version first: the most correct cost model is the one a reader
        // should hit before any superseded one.
        for (ver, rows) in versions.iter().rev() {
            let model = rows.first().map(|r| r.fee_model.clone()).unwrap_or_default();
            let short = if model.starts_with("none") {
                "**no venue fee charged — superseded, kept for attribution**"
            } else {
                "with the venue's taker fee"
            };
            s.push_str(&format!("### Policies `v{ver}` — {short}\n\n"));
            s.push_str(&format!("Cost model: `{model}`\n\n"));
            s.push_str(&version_block(rows, "####"));
        }
    }

    s.push_str("## Reading notes\n\n");
    s.push_str(
        "- **Fills are never at mid.** Buys lift the ask, sells hit the bid; where only a \
         midpoint exists the policy's `assumed_spread` is applied symmetrically and the \
         trade is marked synthetic. A policy whose result rests on synthetic fills is \
         flagged, not celebrated (DESIGN.md §4).\n",
    );
    s.push_str(
        "- **Fees** are Polymarket's published taker fee, `shares × rate × p × (1 − p)` \
         USDC per fill, charged on entry and again on an in-market exit, never at \
         resolution (redemption is not a match). Rates are per category, read off each \
         market's own `feeSchedule`: crypto 0.07, finance 0.04. See DESIGN.md §4.4.\n",
    );
    s.push_str(
        "- **annROLC** = `Σ pnl / Σ (capital_locked × days_held) × 365`, the formula in \
         DESIGN.md §3. It is a rate, not a fund return: multiply by `cap.eff` to see what \
         the bankroll would actually have earned. Fees are inside `pnl`; `capital_locked` \
         is collateral only, so v1→v2 moves the numerator and leaves the denominator \
         alone.\n",
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

/// One version's full block of tables. `h` is the heading level to emit at, so
/// the same block can sit under a set (`###`) or under a version (`####`).
fn version_block(rows: &[SimResult], h: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("{h} Headline\n\n"));
    s.push_str("| policy | n | c/trade | ± se | t | hit | hold d | ROLC | **annROLC** | cap.eff | gross $ | fees $ | net $ | maxDD $ | synth fills |\n");
    s.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for r in rows {
        let m = &r.metrics;
        s.push_str(&format!(
            "| {}{} | {} | {} | {} | {} | {} | {} | {} | **{}** | {} | {} | {} | {} | {} | {} |\n",
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
            md(Some(m.gross_pnl_usd), 2),
            md(Some(m.fees_usd), 2),
            md(Some(m.net_pnl_usd), 2),
            md(Some(m.max_drawdown_usd), 2),
            pct(m.synthetic_fill_share),
        ));
    }
    s.push_str("\n⚠ = underpowered (n < 30). ROLC = return on locked capital. `net = gross − fees`.\n\n");

    s.push_str(&format!("{h} Where the signals went\n\n"));
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

    s.push_str(&format!("{h} Side and fill quality\n\n"));
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

    s.push_str(&contrasts(rows, h));
    s.push_str(&reading(rows, h));
    s.push_str(&cannot_tell(rows, h));
    s
}

/// The policy leading on cents/trade (`by_cents`) or on annROLC.
fn lead<'a>(v: &[&'a SimResult], by_cents: bool) -> Option<&'a SimResult> {
    v.iter().copied().max_by(|a, b| {
        if by_cents {
            cmpf(a.metrics.cents_per_trade, b.metrics.cents_per_trade)
        } else {
            cmpf(
                a.metrics.annualized_return_on_locked_capital,
                b.metrics.annualized_return_on_locked_capital,
            )
        }
    })
}

/// Policies ranked by annROLC, underpowered ones excluded (DESIGN.md §7).
fn ranked(rows: &[SimResult]) -> Vec<&SimResult> {
    let mut v: Vec<&SimResult> =
        rows.iter().filter(|r| r.metrics.n >= MIN_N_FOR_A_WINNER).collect();
    v.sort_by(|a, b| {
        cmpf(
            b.metrics.annualized_return_on_locked_capital,
            a.metrics.annualized_return_on_locked_capital,
        )
    });
    v
}

/// The before/after the cost model was fixed. Same signals, same gates, same
/// sizing, same fills — the *only* difference between the two versions is that
/// one charges the venue's real taker fee and the other charges nothing. Every
/// sentence here is generated from the numbers so it cannot drift from them.
fn fee_comparison(versions: &BTreeMap<u32, Vec<SimResult>>) -> String {
    let (Some((&lo, old)), Some((&hi, new))) = (versions.iter().next(), versions.iter().last())
    else {
        return String::new();
    };
    if lo == hi {
        return String::new();
    }
    let find = |rows: &[SimResult], name: &str| -> Option<usize> {
        rows.iter().position(|r| r.policy == name)
    };
    let ann = |rows: &[SimResult], name: &str| -> Option<f64> {
        find(rows, name)
            .and_then(|i| rows[i].metrics.annualized_return_on_locked_capital)
            .filter(|_| find(rows, name).map(|i| rows[i].metrics.n).unwrap_or(0) >= MIN_N_FOR_A_WINNER)
    };

    let mut s = format!("### Before and after fees — `v{lo}` (fee-free) → `v{hi}` (real fees)\n\n");
    s.push_str(&format!(
        "Identical selection, sizing, entry, exit and fills. The single difference is the \
         cost model: `v{lo}` charged nothing, `v{hi}` charges Polymarket's published taker \
         fee `shares × rate × p × (1 − p)` on every fill — entry always, exit only when the \
         position is closed in the market. **This is the table that says which earlier \
         conclusions survive.**\n\n"
    ));

    let rank_lo = ranked(old);
    let rank_hi = ranked(new);
    let rank_of = |r: &[&SimResult], name: &str| -> String {
        match r.iter().position(|x| x.policy == name) {
            Some(i) => format!("#{}", i + 1),
            None => "—".to_string(),
        }
    };

    s.push_str(&format!("| policy | n | annROLC v{lo} | annROLC v{hi} | Δ | rank v{lo} → v{hi} | c/trade v{lo} → v{hi} | fees $ | fees % of gross |\n"));
    s.push_str("|---|---:|---:|---:|---:|:---:|---:|---:|---:|\n");
    for r in new {
        let Some(i) = find(old, &r.policy) else { continue };
        let o = &old[i];
        let (mo, mn) = (&o.metrics, &r.metrics);
        let delta = match (
            mo.annualized_return_on_locked_capital,
            mn.annualized_return_on_locked_capital,
        ) {
            (Some(a), Some(b)) => format!("{:+.0} pp", (b - a) * 100.0),
            _ => "—".to_string(),
        };
        s.push_str(&format!(
            "| {}{} | {} | {} | {} | {} | {} → {} | {} → {} | {} | {} |\n",
            r.policy,
            if mn.underpowered { " ⚠" } else { "" },
            mn.n,
            pct(mo.annualized_return_on_locked_capital),
            pct(mn.annualized_return_on_locked_capital),
            delta,
            rank_of(&rank_lo, &r.policy),
            rank_of(&rank_hi, &r.policy),
            md(mo.cents_per_trade, 2),
            md(mn.cents_per_trade, 2),
            md(Some(mn.fees_usd), 2),
            pct(mn.fee_share_of_gross),
        ));
    }
    s.push('\n');

    if rank_hi.is_empty() {
        s.push_str(&format!(
            "No policy on this set reaches n = {MIN_N_FOR_A_WINNER}, so the engine ranks nothing here and the fee change cannot be read as a re-ranking (DESIGN.md §7).\n\n"
        ));
        return s;
    }

    let names = |v: &[&SimResult]| -> String {
        v.iter().map(|r| r.policy.as_str()).collect::<Vec<_>>().join(" > ")
    };
    s.push_str(&format!("**Ranking by annROLC** (n ≥ {MIN_N_FOR_A_WINNER} only)\n\n"));
    s.push_str(&format!("- before fees: {}\n", names(&rank_lo)));
    s.push_str(&format!("- after fees: {}\n\n", names(&rank_hi)));

    let moved: Vec<String> = rank_hi
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            let was = rank_lo.iter().position(|x| x.policy == r.policy)?;
            (was != i).then(|| format!("`{}` {}→{}", r.policy, was + 1, i + 1))
        })
        .collect();
    match (rank_lo.first(), rank_hi.first()) {
        (Some(a), Some(b)) if a.policy != b.policy => s.push_str(&format!(
            "**The winner changes: `{}` loses the crown to `{}`.** {}\n\n",
            a.policy,
            b.policy,
            if moved.is_empty() { String::new() } else { format!("Positions moved: {}.", moved.join(", ")) }
        )),
        (Some(a), Some(_)) => s.push_str(&format!(
            "**`{}` keeps the top spot after fees.** {}\n\n",
            a.policy,
            if moved.is_empty() {
                "No policy changed rank.".to_string()
            } else {
                format!("Below it, positions moved: {}.", moved.join(", "))
            }
        )),
        _ => {}
    }

    // Keeping first place and keeping a lead are different things. A margin
    // that collapses is the warning the bare ranking hides.
    if let (Some(a1), Some(a2), Some(b1), Some(b2)) =
        (rank_lo.first(), rank_lo.get(1), rank_hi.first(), rank_hi.get(1))
    {
        let gap = |x: &SimResult, y: &SimResult| {
            match (
                x.metrics.annualized_return_on_locked_capital,
                y.metrics.annualized_return_on_locked_capital,
            ) {
                (Some(p), Some(q)) => Some((p - q) * 100.0),
                _ => None,
            }
        };
        if let (Some(g0), Some(g1)) = (gap(a1, a2), gap(b1, b2)) {
            let shrank = g0 > 0.0 && g1 < g0;
            s.push_str(&format!(
                "**Margin at the top: {g0:.0} pp → {g1:.0} pp.** `{}` led `{}` by {g0:.0} pp fee-free; `{}` leads `{}` by {g1:.0} pp once fees are charged{}\n\n",
                a1.policy,
                a2.policy,
                b1.policy,
                b2.policy,
                if shrank {
                    format!(
                        " — the lead shrank by {:.0}%. Ranking alone would have hidden that; the first place is no longer a comfortable one.",
                        (1.0 - g1 / g0) * 100.0
                    )
                } else {
                    ".".to_string()
                },
            ));
        }
    }

    // ---- the three conclusions the firm has already reported, re-checked.
    s.push_str("**The conclusions on record, re-checked after fees**\n\n");

    // (a) filtering is the biggest lever: mirror -> gate.
    match (ann(old, "mirror"), ann(old, "gate"), ann(new, "mirror"), ann(new, "gate")) {
        (Some(mo), Some(go), Some(mn), Some(gn)) if mo > 0.0 && mn > 0.0 => {
            let (x_old, x_new) = (go / mo, gn / mn);
            let verdict = if x_new >= 1.5 {
                "**SURVIVES**"
            } else if x_new > 1.0 {
                "**WEAKENED**"
            } else {
                "**FAILS**"
            };
            s.push_str(&format!(
                "- (a) *filtering is the biggest lever* — {verdict}. `mirror`→`gate` was {} → {} (×{x_old:.2}) fee-free; with fees it is {} → {} (×{x_new:.2}).\n",
                pct(Some(mo)), pct(Some(go)), pct(Some(mn)), pct(Some(gn)),
            ));
        }
        _ => s.push_str(
            "- (a) *filtering is the biggest lever* — cannot be re-checked: `mirror` or `gate` is underpowered on this set.\n",
        ),
    }

    // (b) sells replicate, buys do not — on the widest two-sided policy.
    if let Some(i) = find(new, "mirror") {
        let (o, n) = (&old[find(old, "mirror").unwrap_or(i)], &new[i]);
        let g = |r: &SimResult, k: &str| engine::metrics::group(&r.by_side, k).and_then(|x| x.cents_per_trade);
        match (g(o, "buy"), g(o, "sell"), g(n, "buy"), g(n, "sell")) {
            (Some(bo), Some(so), Some(bn), Some(sn)) => {
                let verdict = if sn > 0.0 && sn > bn {
                    "**SURVIVES**"
                } else if sn > bn {
                    "**DIRECTION HOLDS, LEVEL DOES NOT** (sells no longer profitable)"
                } else {
                    "**FAILS**"
                };
                s.push_str(&format!(
                    "- (b) *sells replicate, buys do not* — {verdict}. On `mirror`, sells {bo_s} → {sn_s} c/trade and buys {bb} → {bn_s} c/trade once the fee is charged.\n",
                    bo_s = md(Some(so), 2),
                    sn_s = md(Some(sn), 2),
                    bb = md(Some(bo), 2),
                    bn_s = md(Some(bn), 2),
                ));
            }
            _ => s.push_str("- (b) *sells replicate, buys do not* — `mirror` did not trade both sides on this set.\n"),
        }
    }

    // (c) the two metrics disagree: who leads each.
    if let (Some(co), Some(ao), Some(cn), Some(an)) = (
        lead(&rank_lo, true),
        lead(&rank_lo, false),
        lead(&rank_hi, true),
        lead(&rank_hi, false),
    ) {
        let verdict = if co.policy == cn.policy && ao.policy == an.policy {
            "**SURVIVES**"
        } else {
            "**CHANGES**"
        };
        s.push_str(&format!(
            "- (c) *`{}` wins annROLC while `{}` wins cents/trade* — {verdict}. After fees the annROLC leader is **`{}`** ({}) and the cents/trade leader is **`{}`** ({}).\n",
            ao.policy,
            co.policy,
            an.policy,
            pct(an.metrics.annualized_return_on_locked_capital),
            cn.policy,
            md(cn.metrics.cents_per_trade, 2),
        ));
    }
    s.push('\n');
    s
}

/// The eight policies exist to answer six specific questions. Answer them with
/// the numbers, and refuse to answer where the sample is too small.
fn contrasts(rows: &[SimResult], h: &str) -> String {
    let find = |name: &str| rows.iter().find(|r| r.policy == name);
    let pairs: [(&str, &str, &str); 6] = [
        ("gate", "mirror", "is filtering alone worth anything?"),
        ("kelly", "gate", "does conviction-sizing beat equal stakes?"),
        ("anchor", "kelly", "what does capacity realism cost?"),
        ("fade", "anchor", "are our sells really the whole edge?"),
        ("patient", "fade", "does a 24h delay improve sell fills?"),
        ("harvest", "fade", "is patience actually paid for?"),
    ];
    let mut s = format!("{h} What separates the policies\n\n");
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
fn reading(rows: &[SimResult], h: &str) -> String {
    let find = |name: &str| rows.iter().find(|r| r.policy == name);
    let mut s = format!("{h} The short reading\n\n");
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
fn cannot_tell(rows: &[SimResult], h: &str) -> String {
    let mut s = format!("{h} What this sample cannot tell us\n\n");
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

