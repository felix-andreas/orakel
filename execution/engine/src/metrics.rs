//! Turning trades into the numbers DESIGN.md §3 and §6 demand — and into the
//! honesty notes §7 demands. Pure.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::policy::Policy;
use crate::signal::Signal;
use crate::sim::{Counts, ExitKind, Trade};
use crate::{fmt_date, fmt_ts, r6, ENGINE_VERSION};

/// DESIGN.md §7: below this many trades a policy may not be called a winner.
pub const MIN_N_FOR_A_WINNER: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimResult {
    pub signal_set: String,
    pub policy: String,
    pub policy_version: u32,
    pub policy_character: String,
    pub engine_version: String,
    /// Set-level date span, so a result can never be quoted without its regime.
    pub set_date_start: String,
    pub set_date_end: String,
    /// Trade-level date span (first entry to last exit).
    pub date_start: String,
    pub date_end: String,
    pub span_days: f64,
    pub bankroll_usd: f64,
    pub assumed_spread: f64,
    pub entry_delay_hours: f64,
    pub counts: Counts,
    pub metrics: Metrics,
    pub by_variant: Vec<Group>,
    pub by_asset: Vec<Group>,
    pub by_side: Vec<Group>,
    pub by_fill: Vec<Group>,
    pub equity_curve: Vec<EquityPoint>,
    /// Machine-generated caveats. Every one of them is a thing the reader would
    /// otherwise have to know to ask about.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Sample size. DESIGN.md §7: it is printed beside every number.
    pub n: usize,
    pub underpowered: bool,
    pub staked_usd: f64,
    pub net_pnl_usd: f64,
    pub cents_per_trade: Option<f64>,
    pub cents_per_trade_se: Option<f64>,
    pub t_stat: Option<f64>,
    pub hit_rate: Option<f64>,
    pub mean_hold_days: Option<f64>,
    /// `Σ pnl / Σ capital_locked`.
    pub return_on_locked_capital: Option<f64>,
    /// `Σ pnl / Σ (capital_locked × days) × 365` — DESIGN.md §3's deciding number.
    pub annualized_return_on_locked_capital: Option<f64>,
    /// Time-weighted mean of deployed capital / bankroll.
    pub capital_efficiency: Option<f64>,
    /// Peak deployed / bankroll. Above 1.0 the stated bankroll could not have
    /// funded the policy.
    pub max_capital_efficiency: Option<f64>,
    pub max_drawdown_usd: f64,
    pub max_drawdown_frac: f64,
    pub longest_losing_streak: usize,
    pub synthetic_fill_share: Option<f64>,
    pub take_profit_exits: usize,
    pub mean_fill_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub key: String,
    pub n: usize,
    pub net_pnl_usd: f64,
    pub cents_per_trade: Option<f64>,
    pub cents_per_trade_se: Option<f64>,
    pub hit_rate: Option<f64>,
    pub return_on_locked_capital: Option<f64>,
    pub annualized_return_on_locked_capital: Option<f64>,
    pub mean_hold_days: Option<f64>,
    pub underpowered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub t: String,
    pub equity: f64,
    pub deployed: f64,
}

/// Assemble the result. `signals` is the raw set (for the set-level date span),
/// `trades` the simulated positions.
pub fn build(signals: &[Signal], policy: &Policy, counts: Counts, trades: Vec<Trade>) -> SimResult {
    let bankroll = policy.bankroll();
    let (set_start, set_end) = span(signals.iter().map(|s| s.t), signals.iter().map(|s| s.resolved_at));
    let (t0, t1) = span_i(trades.iter().map(|t| t.entry_t), trades.iter().map(|t| t.exit_t));
    let (curve, cap_eff, max_cap_eff, dd_usd, dd_frac) = equity(&trades, bankroll);

    let metrics = Metrics {
        n: trades.len(),
        underpowered: trades.len() < MIN_N_FOR_A_WINNER,
        staked_usd: r6(trades.iter().map(|t| t.capital_locked).sum()),
        net_pnl_usd: r6(trades.iter().map(|t| t.pnl).sum()),
        cents_per_trade: mean(&trades.iter().map(|t| t.cents_per_share()).collect::<Vec<_>>()),
        cents_per_trade_se: stderr(&trades.iter().map(|t| t.cents_per_share()).collect::<Vec<_>>()),
        t_stat: t_stat(&trades.iter().map(|t| t.cents_per_share()).collect::<Vec<_>>()),
        hit_rate: mean(
            &trades.iter().map(|t| if t.won() { 1.0 } else { 0.0 }).collect::<Vec<_>>(),
        ),
        mean_hold_days: mean(&trades.iter().map(|t| t.hold_days()).collect::<Vec<_>>()),
        return_on_locked_capital: ratio(
            trades.iter().map(|t| t.pnl).sum(),
            trades.iter().map(|t| t.capital_locked).sum(),
        ),
        annualized_return_on_locked_capital: ratio(
            trades.iter().map(|t| t.pnl).sum::<f64>() * 365.0,
            trades.iter().map(|t| t.capital_locked * t.hold_days()).sum(),
        ),
        capital_efficiency: cap_eff,
        max_capital_efficiency: max_cap_eff,
        max_drawdown_usd: r6(dd_usd),
        max_drawdown_frac: r6(dd_frac),
        longest_losing_streak: longest_losing_streak(&trades),
        synthetic_fill_share: mean(
            &trades
                .iter()
                .map(|t| if t.synthetic_fill { 1.0 } else { 0.0 })
                .collect::<Vec<_>>(),
        ),
        take_profit_exits: trades.iter().filter(|t| t.exit_kind == ExitKind::TakeProfit).count(),
        mean_fill_price: mean(&trades.iter().map(|t| t.fill).collect::<Vec<_>>()),
    };

    let notes = notes(policy, &counts, &metrics, bankroll, &set_start, &set_end);

    SimResult {
        signal_set: signals.first().map(|s| s.signal_set.clone()).unwrap_or_default(),
        policy: policy.name.clone(),
        policy_version: policy.version,
        policy_character: policy.character.clone(),
        engine_version: ENGINE_VERSION.to_string(),
        set_date_start: set_start,
        set_date_end: set_end,
        date_start: if trades.is_empty() { String::new() } else { fmt_date(t0) },
        date_end: if trades.is_empty() { String::new() } else { fmt_date(t1) },
        span_days: if trades.is_empty() { 0.0 } else { r6((t1 - t0) as f64 / 86_400.0) },
        bankroll_usd: bankroll,
        assumed_spread: policy.costs.assumed_spread,
        entry_delay_hours: policy.entry.delay_hours,
        counts,
        metrics,
        by_variant: group_by(&trades, |t| t.variant.clone()),
        by_asset: group_by(&trades, |t| t.asset.clone()),
        by_side: group_by(&trades, |t| t.side.as_str().to_string()),
        by_fill: group_by(&trades, |t| {
            if t.synthetic_fill { "synthetic".to_string() } else { "real-book".to_string() }
        }),
        equity_curve: curve,
        notes,
    }
}

fn span(starts: impl Iterator<Item = i64>, ends: impl Iterator<Item = i64>) -> (String, String) {
    let lo = starts.min();
    let hi = ends.max();
    match (lo, hi) {
        (Some(a), Some(b)) => (fmt_date(a), fmt_date(b)),
        _ => (String::new(), String::new()),
    }
}

fn span_i(starts: impl Iterator<Item = i64>, ends: impl Iterator<Item = i64>) -> (i64, i64) {
    (starts.min().unwrap_or(0), ends.max().unwrap_or(0))
}

/// Equity curve, capital efficiency and drawdown in one sweep. Equity is
/// bankroll plus *realized* PnL — open positions are not marked to market,
/// because the signal sets do not carry a price for every day of every hold.
fn equity(
    trades: &[Trade],
    bankroll: f64,
) -> (Vec<EquityPoint>, Option<f64>, Option<f64>, f64, f64) {
    if trades.is_empty() {
        return (Vec::new(), None, None, 0.0, 0.0);
    }
    let mut events: Vec<(i64, f64, f64)> = Vec::with_capacity(trades.len() * 2);
    for t in trades {
        events.push((t.entry_t, t.capital_locked, 0.0));
        events.push((t.exit_t, -t.capital_locked, t.pnl));
    }
    events.sort_by_key(|e| e.0);

    let mut curve: Vec<EquityPoint> = Vec::new();
    let mut deployed = 0.0f64;
    let mut realized = 0.0f64;
    let mut peak = bankroll;
    let mut dd_usd = 0.0f64;
    let mut dd_frac = 0.0f64;
    let mut area = 0.0f64;
    let mut max_dep = 0.0f64;
    let mut i = 0;
    let mut prev_t = events[0].0;
    while i < events.len() {
        let t = events[i].0;
        area += deployed * (t - prev_t) as f64;
        prev_t = t;
        while i < events.len() && events[i].0 == t {
            deployed += events[i].1;
            realized += events[i].2;
            i += 1;
        }
        deployed = deployed.max(0.0);
        max_dep = max_dep.max(deployed);
        let eq = bankroll + realized;
        peak = peak.max(eq);
        dd_usd = dd_usd.max(peak - eq);
        if peak > 0.0 {
            dd_frac = dd_frac.max((peak - eq) / peak);
        }
        curve.push(EquityPoint { t: fmt_ts(t), equity: r6(eq), deployed: r6(deployed) });
    }

    let (t0, t1) = span_i(trades.iter().map(|t| t.entry_t), trades.iter().map(|t| t.exit_t));
    let dur = (t1 - t0) as f64;
    let cap_eff = if dur > 0.0 && bankroll > 0.0 { Some(r6(area / dur / bankroll)) } else { None };
    let max_eff = if bankroll > 0.0 { Some(r6(max_dep / bankroll)) } else { None };
    (curve, cap_eff, max_eff, dd_usd, dd_frac)
}

fn longest_losing_streak(trades: &[Trade]) -> usize {
    let mut order: Vec<&Trade> = trades.iter().collect();
    order.sort_by(|a, b| a.exit_t.cmp(&b.exit_t).then_with(|| a.entry_t.cmp(&b.entry_t)));
    let mut best = 0usize;
    let mut cur = 0usize;
    for t in order {
        if t.pnl < 0.0 {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 0;
        }
    }
    best
}

fn group_by(trades: &[Trade], key: impl Fn(&Trade) -> String) -> Vec<Group> {
    let mut buckets: BTreeMap<String, Vec<&Trade>> = BTreeMap::new();
    for t in trades {
        buckets.entry(key(t)).or_default().push(t);
    }
    let mut out: Vec<Group> = buckets
        .into_iter()
        .map(|(k, ts)| {
            let cents: Vec<f64> = ts.iter().map(|t| t.cents_per_share()).collect();
            Group {
                key: k,
                n: ts.len(),
                net_pnl_usd: r6(ts.iter().map(|t| t.pnl).sum()),
                cents_per_trade: mean(&cents),
                cents_per_trade_se: stderr(&cents),
                hit_rate: mean(
                    &ts.iter().map(|t| if t.won() { 1.0 } else { 0.0 }).collect::<Vec<_>>(),
                ),
                return_on_locked_capital: ratio(
                    ts.iter().map(|t| t.pnl).sum(),
                    ts.iter().map(|t| t.capital_locked).sum(),
                ),
                annualized_return_on_locked_capital: ratio(
                    ts.iter().map(|t| t.pnl).sum::<f64>() * 365.0,
                    ts.iter().map(|t| t.capital_locked * t.hold_days()).sum(),
                ),
                mean_hold_days: mean(&ts.iter().map(|t| t.hold_days()).collect::<Vec<_>>()),
                underpowered: ts.len() < MIN_N_FOR_A_WINNER,
            }
        })
        .collect();
    out.sort_by(|a, b| b.n.cmp(&a.n).then_with(|| a.key.cmp(&b.key)));
    out
}

fn notes(
    policy: &Policy,
    c: &Counts,
    m: &Metrics,
    bankroll: f64,
    set_start: &str,
    set_end: &str,
) -> Vec<String> {
    let mut n = Vec::new();
    n.push(format!(
        "signal set spans {set_start} .. {set_end} — one regime; every number below is conditional on it."
    ));
    if m.n == 0 {
        n.push("no trades: this policy did not fire on this signal set at all.".to_string());
    } else if m.underpowered {
        n.push(format!(
            "UNDERPOWERED: n = {} < {MIN_N_FOR_A_WINNER}. DESIGN.md §7 forbids calling this policy a winner or a loser.",
            m.n
        ));
    }
    if let Some(s) = m.synthetic_fill_share {
        if s > 0.0 {
            n.push(format!(
                "{:.0}% of fills are synthetic ({}c assumed spread, no real book) — DESIGN.md §4 flags such a result, it does not celebrate it.",
                s * 100.0,
                policy.costs.assumed_spread * 100.0
            ));
        }
    }
    if policy.entry.require_book && m.synthetic_fill_share.unwrap_or(0.0) > 0.0 {
        n.push(format!(
            "require_book = true was asked for, but {:.0}% of the trades taken filled against a book the engine had to synthesize. The engine reads require_book as \"price it honestly and flag it\", not \"refuse it\" — under the strict reading this policy would have taken 0 trades on the synthetic rows. This is the single most consequential interpretive choice in the engine; see engine/README.md.",
            m.synthetic_fill_share.unwrap_or(0.0) * 100.0
        ));
    }
    if c.epsilon_unavailable > 0 {
        n.push(format!(
            "respect_venue_epsilon is on but UNAPPLIED to {} of the sells taken: no signal set carries the barrier-to-running-extreme distance, so the screen could not run. Counted, not silently passed.",
            c.epsilon_unavailable
        ));
    }
    if c.depth_unknown > 0 {
        n.push(format!(
            "{} of the trades taken carried no depth data: the depth gate and the depth cap could not be evaluated for them.",
            c.depth_unknown
        ));
    }
    if c.unfundable > 0 {
        n.push(format!(
            "{} signals were UNFUNDABLE: max_book_fraction x visible depth is below the ${:.2} minimum ticket.",
            c.unfundable,
            policy.min_stake()
        ));
    }
    if c.delay_unavailable > 0 {
        n.push(format!(
            "{} signals had no observation {}h later and were EXCLUDED from this policy ({} of them on markets that resolved in the token's favour). This attrition is not random: a barrier that touches stops producing later observations, so a delayed policy is measured on a sample biased away from its losses.",
            c.delay_unavailable, policy.entry.delay_hours, c.delay_unavailable_token_won
        ));
    }
    if let Some(e) = m.max_capital_efficiency {
        if e > 1.0 {
            n.push(format!(
                "peak deployment was {:.0}% of the stated ${bankroll:.0} bankroll — that bankroll could not actually have funded this policy; treat the dollar PnL as a scale-free rate, not a fund result.",
                e * 100.0
            ));
        }
    }
    if let Some(h) = m.mean_hold_days {
        if h < 1.0 {
            n.push(format!(
                "mean hold is {h:.2} days — the annualized figure extrapolates a sub-daily holding period by a factor of {:.0}; read it as an order of magnitude at best.",
                365.0 / h.max(1e-6)
            ));
        }
    }
    if m.take_profit_exits > 0 && m.synthetic_fill_share.unwrap_or(0.0) > 0.0 {
        n.push(format!(
            "{} of the exits were taken in the market rather than at settlement, and they were priced off a synthetic quote. An early-exit policy pays the spread twice, so this is the most spread-sensitive result in the matrix — it is the first number a real book would revise.",
            m.take_profit_exits
        ));
    }
    if let Some(t) = m.t_stat {
        if t.abs() < 2.0 {
            n.push(format!(
                "cents/trade t-stat is {t:.2} — this policy's per-trade result is not distinguishable from zero."
            ));
        }
    }
    n.push(
        "repeated signals on one market share a single outcome, so the per-trade standard error above is optimistic: collapse the trades on a leg to one observation before believing the t-stat.".to_string(),
    );
    n
}

// ---- small statistics helpers (kept local; duplication beats indirection) ----

pub fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    Some(r6(xs.iter().sum::<f64>() / xs.len() as f64))
}

pub fn stderr(xs: &[f64]) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let m = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0);
    Some(r6((var / n).sqrt()))
}

pub fn t_stat(xs: &[f64]) -> Option<f64> {
    let m = mean(xs)?;
    let se = stderr(xs)?;
    if se <= 0.0 {
        return None;
    }
    Some(r6(m / se))
}

fn ratio(num: f64, den: f64) -> Option<f64> {
    if den.abs() < 1e-12 {
        None
    } else {
        Some(r6(num / den))
    }
}

/// Convenience for the side/fill breakdown lookups in reports.
pub fn group<'a>(groups: &'a [Group], key: &str) -> Option<&'a Group> {
    groups.iter().find(|g| g.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_helpers() {
        assert_eq!(mean(&[]), None);
        assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(stderr(&[1.0]), None);
        // sd = 1, n = 4 => se = 0.5
        assert_eq!(stderr(&[1.0, 2.0, 3.0, 2.0]).map(|v| (v * 1e6).round()), Some(408248.0));
        assert_eq!(t_stat(&[2.0, 2.0, 2.0]), None); // zero variance
    }

    #[test]
    fn ratios_guard_against_zero_denominators() {
        assert_eq!(ratio(1.0, 0.0), None);
        assert_eq!(ratio(1.0, 4.0), Some(0.25));
    }
}
