//! The simulation. Pure functions over plain structs — no I/O anywhere in this
//! file, so it compiles to wasm unchanged.
//!
//! Reading order: `simulate` is the whole story top-to-bottom; the helpers below
//! it are deliberately small and boring.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::metrics::{self, SimResult};
use crate::policy::{Policy, DEPTH_BAND};
use crate::signal::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExitKind {
    Resolution,
    TakeProfit,
}

/// One simulated trade, carrying everything the metrics need.
#[derive(Debug, Clone)]
pub struct Trade {
    pub token_id: String,
    pub market_slug: String,
    pub condition_id: String,
    pub asset: String,
    pub variant: String,
    pub model: String,
    pub side: Side,
    /// Time the position was opened (already includes any entry delay).
    pub entry_t: i64,
    pub exit_t: i64,
    /// The price we transacted the YES token at, after spread and slippage.
    pub fill: f64,
    /// 1.0 / 0.0 at resolution, or the executable price at a take-profit exit.
    pub exit_price: f64,
    pub shares: f64,
    /// DESIGN.md §3: `ask × shares` for a buy, `(1 − bid) × shares` for a sell.
    pub capital_locked: f64,
    pub pnl: f64,
    pub fees: f64,
    pub synthetic_fill: bool,
    pub exit_kind: ExitKind,
}

impl Trade {
    pub fn hold_days(&self) -> f64 {
        (self.exit_t - self.entry_t) as f64 / 86_400.0
    }
    /// Size-independent per-share result, in cents — the unit DESIGN.md §3
    /// argues about and the unit ladder-rv's own backtests report.
    pub fn cents_per_share(&self) -> f64 {
        if self.shares <= 0.0 {
            return 0.0;
        }
        100.0 * self.pnl / self.shares
    }
    pub fn won(&self) -> bool {
        self.pnl > 0.0
    }
}

/// Every signal ends in exactly one of these buckets — nothing is ever silently
/// dropped (DESIGN.md §4.2, §7). The first block are terminal outcomes and sum
/// to `signals`; the second block are *reported, non-gating* observations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Counts {
    pub signals: usize,
    pub traded: usize,
    pub no_quote: usize,
    pub entry_after_resolution: usize,
    pub delay_unavailable: usize,
    pub no_executable_edge: usize,
    pub side_excluded: usize,
    pub below_min_edge: usize,
    pub below_edge_percentile: usize,
    pub spread_too_wide: usize,
    pub depth_too_thin: usize,
    pub unfundable: usize,
    pub market_cap_full: usize,
    pub stake_too_small: usize,

    /// Of the `delay_unavailable` signals, how many sit on a market that
    /// resolved in favour of the token. Not random attrition: a leg that
    /// touches stops producing later observations, so this is the direction the
    /// delayed policy is biased in.
    pub delay_unavailable_token_won: usize,
    /// Rows where the depth gate and the depth cap could not be evaluated.
    pub depth_unknown: usize,
    /// Sell candidates the venue-epsilon screen could not be applied to,
    /// because no signal set carries the running-extreme distance yet.
    pub epsilon_unavailable: usize,
    /// Rows whose book had to be synthesized from the midpoint.
    pub book_synthetic: usize,
    /// Trades whose asset had no `costs.asset_category` entry and were charged
    /// the policy's `fee_rate_default`. Counted, never silently defaulted.
    pub fee_rate_unmapped: usize,
}

/// A two-sided quote, real or synthesized from the midpoint.
struct Quote {
    bid: f64,
    ask: f64,
    depth_bid: Option<f64>,
    depth_ask: Option<f64>,
    synthetic: bool,
}

/// Run one policy over one signal set. Pure: `(signals, policy) -> results`.
pub fn simulate(signals: &[Signal], policy: &Policy) -> Result<SimResult, String> {
    let combined = combine(signals, &policy.combine.method)?;

    // Price path per token: every observation we have of that token, in time
    // order. Used for delayed entry and for take-profit exits — the engine
    // never invents a price that is not in the signal set.
    let mut paths: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, s) in combined.iter().enumerate() {
        paths.entry(s.token_id.as_str()).or_default().push(i);
    }
    for v in paths.values_mut() {
        v.sort_by_key(|&i| combined[i].t);
    }

    let edge_floor = policy
        .entry
        .edge_percentile
        .map(|q| quantile(&combined.iter().map(|s| s.claimed_edge()).collect::<Vec<_>>(), q));

    let sides = policy.sides_normalized();
    let spread = policy.costs.assumed_spread;
    let min_stake = policy.min_stake();
    let bankroll = policy.bankroll();
    let delay_secs = (policy.entry.delay_hours * 3600.0).round() as i64;
    let delay_tol = (policy.delay_tolerance_hours() * 3600.0).round() as i64;

    let mut counts = Counts { signals: combined.len(), ..Default::default() };
    let mut trades: Vec<Trade> = Vec::new();
    // Open exposure ledger: (exit_t, market key, capital) — pruned as time moves.
    let mut open: Vec<(i64, String, f64)> = Vec::new();

    for (i, sig) in combined.iter().enumerate() {
        if sig.synthetic_book || sig.bid.is_none() || sig.ask.is_none() {
            counts.book_synthetic += 1;
        }

        // ---- entry observation (possibly delayed, model inputs frozen at `sig`)
        let entry_idx = if delay_secs > 0 {
            let want = sig.t + delay_secs;
            let found = paths[sig.token_id.as_str()]
                .iter()
                .copied()
                .find(|&j| combined[j].t >= want && combined[j].t <= want + delay_tol);
            match found.filter(|&j| combined[j].t < sig.resolved_at) {
                Some(j) => j,
                None => {
                    counts.delay_unavailable += 1;
                    if sig.token_won() {
                        counts.delay_unavailable_token_won += 1;
                    }
                    continue;
                }
            }
        } else {
            i
        };
        let entry = &combined[entry_idx];
        if entry.t >= sig.resolved_at {
            counts.entry_after_resolution += 1;
            continue;
        }

        let q = match quote_of(entry, spread) {
            Some(q) => q,
            None => {
                counts.no_quote += 1;
                continue;
            }
        };

        // ---- side and executable edge. Buys lift the ask, sells hit the bid;
        // a probability inside the spread is not tradeable at any size.
        let p = sig.p_model;
        let (side, edge) = if p > q.ask {
            (Side::Buy, p - q.ask)
        } else if p < q.bid {
            (Side::Sell, q.bid - p)
        } else {
            counts.no_executable_edge += 1;
            continue;
        };

        // ---- gates
        if !sides.iter().any(|s| s == side.as_str()) {
            counts.side_excluded += 1;
            continue;
        }
        if edge < policy.entry.min_edge {
            counts.below_min_edge += 1;
            continue;
        }
        if let Some(floor) = edge_floor {
            if sig.claimed_edge() < floor {
                counts.below_edge_percentile += 1;
                continue;
            }
        }
        if q.ask - q.bid > policy.entry.min_spread_ok {
            counts.spread_too_wide += 1;
            continue;
        }
        let depth = match side {
            Side::Buy => q.depth_ask,
            Side::Sell => q.depth_bid,
        };
        match depth {
            Some(d) => {
                if d < policy.entry.min_depth_usd {
                    counts.depth_too_thin += 1;
                    continue;
                }
                if policy.costs.max_book_fraction * d < min_stake {
                    counts.unfundable += 1;
                    continue;
                }
            }
            None => {}
        }

        // ---- sizing
        let market = market_key(sig);
        open.retain(|(exit_t, _, _)| *exit_t > entry.t);
        let exposure: f64 = open.iter().filter(|(_, m, _)| *m == market).map(|(_, _, c)| c).sum();
        let room = policy.sizing.max_per_market_usd - exposure;
        if room < min_stake {
            counts.market_cap_full += 1;
            continue;
        }

        let touch = match side {
            Side::Buy => q.ask,
            Side::Sell => q.bid,
        };
        let mut stake = match policy.sizing.method.as_str() {
            "flat" => policy.sizing.stake_usd.unwrap_or(0.0),
            _ => {
                let f = kelly_fraction(side, p, touch);
                f * policy.sizing.kelly_fraction.unwrap_or(0.0) * bankroll
            }
        };
        if let Some(maxf) = policy.sizing.max_bankroll_fraction {
            stake = stake.min(maxf * bankroll);
        }
        stake = stake.min(room);
        if let Some(d) = depth {
            stake = stake.min(policy.costs.max_book_fraction * d);
        }
        if !(stake >= min_stake) {
            counts.stake_too_small += 1;
            continue;
        }

        // ---- fill: never at mid, and worse than touch once we eat depth
        let fill = apply_slippage(side, touch, stake, depth);
        let per_share = capital_per_share(side, fill);
        if per_share <= 0.0 {
            counts.no_quote += 1;
            continue;
        }
        let shares = stake / per_share;

        // ---- exit
        let (exit_t, exit_price, exit_kind) = resolve_exit(
            &combined,
            &paths,
            sig,
            entry_idx,
            side,
            fill,
            stake,
            policy,
            spread,
        );

        let gross = match side {
            Side::Buy => shares * (exit_price - fill),
            Side::Sell => shares * (fill - exit_price),
        };
        // Fees are charged per *taker fill*, at that fill's price. Entering is
        // always one fill. Exiting is a second fill only when we exit in the
        // market: settling at resolution is a redemption, not a match, and the
        // venue charges nothing for it. So a held position pays once and a
        // round trip pays twice — which is the whole reason `harvest`'s ranking
        // was suspect.
        let (taker_rate, rate_mapped) = policy.costs.fee_rate_for(sig.asset_key());
        let legacy = policy.costs.fee_bps / 10_000.0;
        let mut fees = legacy * shares * fill + taker_fee(shares, taker_rate, fill);
        if exit_kind == ExitKind::TakeProfit {
            fees += legacy * shares * exit_price + taker_fee(shares, taker_rate, exit_price);
        }
        if !rate_mapped {
            counts.fee_rate_unmapped += 1;
        }

        open.push((exit_t, market, stake));
        counts.traded += 1;
        // Diagnostics about the trades we actually took: which of them the data
        // could not screen. Reported, never silently passed (DESIGN.md §4.5).
        if depth.is_none() {
            counts.depth_unknown += 1;
        }
        if policy.entry.respect_venue_epsilon && side == Side::Sell {
            counts.epsilon_unavailable += 1;
        }
        trades.push(Trade {
            token_id: sig.token_id.clone(),
            market_slug: sig.market_slug.clone(),
            condition_id: sig.condition_id.clone(),
            asset: sig.asset_key().to_string(),
            variant: sig.variant_key(),
            model: sig.model.clone(),
            side,
            entry_t: entry.t,
            exit_t,
            fill,
            exit_price,
            shares,
            capital_locked: stake,
            pnl: gross - fees,
            fees,
            synthetic_fill: q.synthetic,
            exit_kind,
        });
    }

    Ok(metrics::build(signals, policy, counts, trades))
}

/// Combination is execution's first step (DESIGN.md §1): collapse the signals
/// that speak about the same token at the same instant into one.
fn combine<'a>(signals: &'a [Signal], method: &str) -> Result<Vec<&'a Signal>, String> {
    let mut kept: Vec<&Signal> = match method {
        "best-improvement" => {
            let mut best: HashMap<(&str, i64), &Signal> = HashMap::new();
            for s in signals {
                let k = (s.token_id.as_str(), s.t);
                match best.get(&k) {
                    Some(prev) if prev.claimed_edge() >= s.claimed_edge() => {}
                    _ => {
                        best.insert(k, s);
                    }
                }
            }
            best.into_values().collect()
        }
        m if m.starts_with("single:") => {
            let want = m.trim_start_matches("single:").to_string();
            signals.iter().filter(|s| s.variant_key() == want).collect()
        }
        m => return Err(format!("unsupported combine.method '{m}'")),
    };
    kept.sort_by(|a, b| {
        a.t.cmp(&b.t)
            .then_with(|| a.token_id.cmp(&b.token_id))
            .then_with(|| a.market_slug.cmp(&b.market_slug))
    });
    Ok(kept)
}

/// A real two-sided book if the row has one, otherwise `assumed_spread` applied
/// symmetrically around the midpoint (DESIGN.md §4.1).
fn quote_of(s: &Signal, assumed_spread: f64) -> Option<Quote> {
    if !s.synthetic_book {
        if let (Some(bid), Some(ask)) = (s.bid, s.ask) {
            if ask > bid && bid >= 0.0 && ask <= 1.0 {
                return Some(Quote {
                    bid,
                    ask,
                    depth_bid: s.depth_bid_usd,
                    depth_ask: s.depth_ask_usd,
                    synthetic: false,
                });
            }
        }
    }
    let half = assumed_spread / 2.0;
    let bid = (s.p_market - half).max(0.0);
    let ask = (s.p_market + half).min(1.0);
    if ask <= bid {
        return None;
    }
    Some(Quote { bid, ask, depth_bid: None, depth_ask: None, synthetic: true })
}

/// Full-Kelly fraction of bankroll on the *executable* price.
///
/// Buy YES at `a`: `f = (p − a) / (1 − a)`.
/// Sell YES at `b` = buy NO at `1 − b` with win probability `1 − p`:
/// `f = ((1 − p) − (1 − b)) / (1 − (1 − b)) = (b − p) / b`.
pub fn kelly_fraction(side: Side, p: f64, price: f64) -> f64 {
    let f = match side {
        Side::Buy => {
            if price >= 1.0 {
                return 0.0;
            }
            (p - price) / (1.0 - price)
        }
        Side::Sell => {
            if price <= 0.0 {
                return 0.0;
            }
            (price - p) / price
        }
    };
    f.clamp(0.0, 1.0)
}

/// Polymarket's published taker fee for **one fill**, in USDC:
///
/// ```text
/// fee = shares × rate × p × (1 − p)
/// ```
///
/// Verified 2026-07-25 against `docs.polymarket.com/trading/fees` ("fee =
/// C × feeRate × p × (1 − p)", C = shares traded, p = price), against each
/// market's own `feeSchedule` on the Gamma API, and by fitting real executed
/// fills from the Data API. Makers pay nothing; every fill the engine simulates
/// crosses the spread and is therefore a taker fill.
///
/// The fee peaks at p = 0.50 (1.25c/share at rate 0.05) and vanishes at both
/// extremes. Because `p(1−p)` is invariant under `p → 1−p`, selling YES at `p`
/// and buying NO at `1 − p` — the same trade, DESIGN.md §3 — cost the same, so
/// it is correct to charge on the YES price whatever the side.
///
/// Venue rounding, from the same page: fees are rounded to 5 decimals and the
/// smallest fee charged is 0.00001 USDC; anything below that rounds to zero.
pub fn taker_fee(shares: f64, rate: f64, price: f64) -> f64 {
    if !(shares > 0.0) || !(rate > 0.0) || !(0.0..=1.0).contains(&price) {
        return 0.0;
    }
    let raw = shares * rate * price * (1.0 - price);
    let rounded = (raw * 1e5).round() / 1e5;
    if rounded < 1e-5 {
        0.0
    } else {
        rounded
    }
}

/// Capital committed per share (DESIGN.md §3).
pub fn capital_per_share(side: Side, fill: f64) -> f64 {
    match side {
        Side::Buy => fill,
        Side::Sell => 1.0 - fill,
    }
}

/// Linear slippage: consuming the whole depth measured within `DEPTH_BAND` of
/// the touch walks the price by that band. The band is bounded by the room the
/// price actually has — a 1c bid cannot slip 5c, it can only slip to zero — so
/// depth measured on a wing is not punished with a move the book cannot make.
/// With no depth data there is nothing to walk and the assumed spread is the
/// only cost. (We cannot walk individual levels: the canonical signal schema
/// carries aggregate depth, not the ladder.)
fn apply_slippage(side: Side, touch: f64, stake: f64, depth: Option<f64>) -> f64 {
    let room = match side {
        Side::Buy => 1.0 - touch,
        Side::Sell => touch,
    };
    let band = DEPTH_BAND.min(room.max(0.0));
    let slip = match depth {
        Some(d) if d > 0.0 => band * (stake / d).min(1.0),
        _ => 0.0,
    };
    match side {
        Side::Buy => (touch + slip).min(1.0),
        Side::Sell => (touch - slip).max(0.0),
    }
}

/// Where the position ends: at resolution, or early if the policy takes profit
/// once the price has closed `close_fraction` of the way from the fill to our
/// probability. The exit price must also be executable — we cross the spread
/// again on the way out.
#[allow(clippy::too_many_arguments)]
fn resolve_exit(
    combined: &[&Signal],
    paths: &HashMap<&str, Vec<usize>>,
    sig: &Signal,
    entry_idx: usize,
    side: Side,
    fill: f64,
    stake: f64,
    policy: &Policy,
    spread: f64,
) -> (i64, f64, ExitKind) {
    let at_resolution = (
        sig.resolved_at,
        if sig.token_won() { 1.0 } else { 0.0 },
        ExitKind::Resolution,
    );
    if policy.exit.rule != "take-profit" {
        return at_resolution;
    }
    let cf = policy.exit.close_fraction.unwrap_or(0.0);
    let target = match side {
        Side::Buy => fill + cf * (sig.p_model - fill),
        Side::Sell => fill - cf * (fill - sig.p_model),
    };
    let entry_t = combined[entry_idx].t;
    for &j in &paths[sig.token_id.as_str()] {
        let row = combined[j];
        if row.t <= entry_t || row.t >= sig.resolved_at {
            continue;
        }
        let Some(q) = quote_of(row, spread) else { continue };
        // Closing a short YES means buying it back (lift the ask); closing a
        // long means selling it (hit the bid).
        let (touch, depth) = match side {
            Side::Buy => (q.bid, q.depth_bid),
            Side::Sell => (q.ask, q.depth_ask),
        };
        let exit_side = match side {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        };
        let px = apply_slippage(exit_side, touch, stake, depth);
        let hit = match side {
            Side::Buy => px >= target,
            Side::Sell => px <= target,
        };
        if hit {
            return (row.t, px, ExitKind::TakeProfit);
        }
    }
    if policy.exit.else_hold_to_resolution {
        at_resolution
    } else {
        at_resolution
    }
}

fn market_key(s: &Signal) -> String {
    if s.condition_id.is_empty() {
        s.market_slug.clone()
    } else {
        s.condition_id.clone()
    }
}

/// Linear-interpolated quantile of a sample. `q` in [0,1].
pub fn quantile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pos = q.clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        v[lo]
    } else {
        v[lo] + (pos - lo as f64) * (v[hi] - v[lo])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kelly_matches_the_closed_form() {
        // Buy YES at 0.40 believing 0.60: f = (0.6-0.4)/(1-0.4) = 1/3.
        assert!((kelly_fraction(Side::Buy, 0.60, 0.40) - 1.0 / 3.0).abs() < 1e-12);
        // Sell YES at 0.20 believing 0.05: f = (0.20-0.05)/0.20 = 0.75.
        assert!((kelly_fraction(Side::Sell, 0.05, 0.20) - 0.75).abs() < 1e-12);
        // No edge, or the wrong way round, is never a bet.
        assert_eq!(kelly_fraction(Side::Buy, 0.30, 0.40), 0.0);
        assert_eq!(kelly_fraction(Side::Sell, 0.30, 0.20), 0.0);
        assert_eq!(kelly_fraction(Side::Buy, 1.0, 1.0), 0.0);
    }

    #[test]
    fn quantiles_interpolate() {
        let xs: Vec<f64> = (0..=10).map(|i| i as f64 / 10.0).collect();
        assert!((quantile(&xs, 0.9) - 0.9).abs() < 1e-12);
        assert!((quantile(&xs, 0.0) - 0.0).abs() < 1e-12);
        assert!((quantile(&xs, 1.0) - 1.0).abs() < 1e-12);
        assert_eq!(quantile(&[], 0.5), 0.0);
    }

    #[test]
    fn slippage_is_linear_in_depth_consumed_and_only_ever_hurts() {
        // Eating 50% of the depth within 5c walks the price 2.5c.
        assert!((apply_slippage(Side::Sell, 0.20, 50.0, Some(100.0)) - 0.175).abs() < 1e-12);
        assert!((apply_slippage(Side::Buy, 0.20, 50.0, Some(100.0)) - 0.225).abs() < 1e-12);
        // No depth data => nothing to walk.
        assert_eq!(apply_slippage(Side::Sell, 0.20, 50.0, None), 0.20);
    }
}
