//! Hand-computed accounting tests. DESIGN.md §3 is the rule these check:
//! a sell of YES at `p` receives `p × shares` and locks `(1 − p) × shares`; a
//! buy at `a` commits `a × shares`. Every number below is worked out by hand in
//! the comment above it — if the engine and the comment disagree, the engine is
//! wrong.

use engine::{parse_signals_csv, simulate, Policy, Signal};

const HEADER: &str = "signal_set,t,market_slug,condition_id,outcome,token_id,family,variant,model,p_model,p_market,bid,ask,depth_bid_usd,depth_ask_usd,resolved_outcome,resolved_date,synthetic_book,asset";

/// One signal row, with a real two-sided book unless `bid`/`ask` are empty.
#[allow(clippy::too_many_arguments)]
fn row(
    t: &str,
    token: &str,
    p_model: f64,
    p_market: f64,
    bid: &str,
    ask: &str,
    depth_bid: &str,
    depth_ask: &str,
    resolved_outcome: &str,
    resolved: &str,
) -> String {
    let synthetic = if bid.is_empty() || ask.is_empty() { "1" } else { "0" };
    format!(
        "s,{t},mkt-{token},0x{token},Yes,{token},fam,var,m,{p_model},{p_market},{bid},{ask},{depth_bid},{depth_ask},{resolved_outcome},{resolved},{synthetic},ast"
    )
}

fn signals(rows: &[String]) -> Vec<Signal> {
    let csv = format!("{HEADER}\n{}\n", rows.join("\n"));
    let (s, w) = parse_signals_csv(&csv).unwrap();
    assert!(w.is_empty(), "unexpected parse warnings: {w:?}");
    s
}

/// Build a policy from four blocks. Empty string = the permissive default.
/// Deliberately dumb string assembly: a test helper that needs its own parser
/// is a test helper that can lie to you.
fn pol(entry: &str, sizing: &str, exit: &str, costs: &str) -> Policy {
    let entry = if entry.is_empty() {
        "min_edge = 0.01\nsides = [\"buy\", \"sell\"]\ndelay_hours = 0\nrequire_book = true\nmin_spread_ok = 1.0\nmin_depth_usd = 0.0\nrespect_venue_epsilon = false"
    } else {
        entry
    };
    let sizing = if sizing.is_empty() {
        "method = \"flat\"\nstake_usd = 10.0\nmax_per_market_usd = 1000.0"
    } else {
        sizing
    };
    let exit = if exit.is_empty() { "rule = \"hold-to-resolution\"" } else { exit };
    let costs = if costs.is_empty() {
        "assumed_spread = 0.03\nmax_book_fraction = 1.0\nfee_bps = 0"
    } else {
        costs
    };
    let text = format!(
        "name = \"test\"\nversion = 1\n[combine]\nmethod = \"best-improvement\"\n\
         [entry]\n{entry}\n[sizing]\n{sizing}\n[exit]\n{exit}\n[costs]\n{costs}\n"
    );
    Policy::from_toml(&text).unwrap_or_else(|e| panic!("bad test policy: {e}\n{text}"))
}

/// The permissive default policy.
fn base() -> Policy {
    pol("", "", "", "")
}

// ---------------------------------------------------------------- the sell case

/// **Hand-computed sell.**
///
/// Book 0.15 / 0.17, we think the token is worth 0.02, flat $10 stake, no depth
/// limit, hold to resolution, market resolves **No** (our YES token loses, i.e.
/// the wing seller wins).
///
/// - side: p = 0.02 < bid = 0.15 → SELL at 0.15.
/// - capital per share = 1 − 0.15 = **0.85**; shares = 10 / 0.85 = **11.7647**.
/// - cash received = 0.15 × 11.7647 = 1.7647; collateral locked = **$10**.
/// - resolution: token loses ⇒ per-share profit = the whole premium 0.15
///   ⇒ pnl = 0.15 × 11.7647 = **$1.7647**.
/// - cents/trade = **15.00c**.
/// - ROLC = 1.7647 / 10 = **17.647%**; held 2026-05-01 → 2026-05-09 = **8 days**
///   ⇒ annualized = 0.17647 × 365 / 8 = **805.15%**.
#[test]
fn sell_case_is_hand_computable() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z",
        "tokA",
        0.02,
        0.16,
        "0.15",
        "0.17",
        "",
        "",
        "No",
        "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&s, &base()).unwrap();
    assert_eq!(r.metrics.n, 1);
    let m = &r.metrics;
    assert!((m.staked_usd - 10.0).abs() < 1e-6, "capital locked {}", m.staked_usd);
    assert!((m.net_pnl_usd - 1.764706).abs() < 1e-5, "pnl {}", m.net_pnl_usd);
    assert!((m.cents_per_trade.unwrap() - 15.0).abs() < 1e-4);
    assert!((m.hit_rate.unwrap() - 1.0).abs() < 1e-9);
    assert!((m.mean_hold_days.unwrap() - 8.0).abs() < 1e-9);
    assert!(
        (m.return_on_locked_capital.unwrap() - 0.176471).abs() < 1e-5,
        "ROLC {:?}",
        m.return_on_locked_capital
    );
    assert!(
        (m.annualized_return_on_locked_capital.unwrap() - 8.051471).abs() < 1e-4,
        "annROLC {:?}",
        m.annualized_return_on_locked_capital
    );
    assert_eq!(r.by_side[0].key, "sell");
}

/// The same sell, but the barrier is touched: the seller loses the **whole**
/// locked collateral and nothing more. pnl = −$10, ROLC = −100%.
#[test]
fn a_losing_sell_loses_exactly_the_collateral() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z",
        "tokA",
        0.02,
        0.16,
        "0.15",
        "0.17",
        "",
        "",
        "Yes",
        "2026-05-09T12:00:00Z",
    )]);
    let m = simulate(&s, &base()).unwrap().metrics;
    assert!((m.net_pnl_usd + 10.0).abs() < 1e-6, "pnl {}", m.net_pnl_usd);
    assert!((m.return_on_locked_capital.unwrap() + 1.0).abs() < 1e-6);
    assert!((m.cents_per_trade.unwrap() + 85.0).abs() < 1e-4, "c/trade {:?}", m.cents_per_trade);
    assert_eq!(m.longest_losing_streak, 1);
}

// ----------------------------------------------------------------- the buy case

/// **Hand-computed buy.**
///
/// Book 0.95 / 0.97, we think 0.99. Flat $10.
///
/// - side: p = 0.99 > ask = 0.97 → BUY at 0.97.
/// - capital per share = **0.97**; shares = 10 / 0.97 = **10.3093**.
/// - token wins ⇒ per-share profit = 1 − 0.97 = **0.03** ⇒ pnl = **$0.3093**.
/// - cents/trade = **3.00c**; ROLC = 0.3093/10 = **3.093%**; held 6 days
///   ⇒ annualized = 0.03093 × 365 / 6 = **188.1%**.
///
/// This is DESIGN.md §3's whole argument in one test: a +3c buy of a 97c
/// favourite ties up 97c per share to earn it.
#[test]
fn buy_case_is_hand_computable() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z",
        "tokB",
        0.99,
        0.96,
        "0.95",
        "0.97",
        "",
        "",
        "Yes",
        "2026-05-07T12:00:00Z",
    )]);
    let m = simulate(&s, &base()).unwrap().metrics;
    assert!((m.staked_usd - 10.0).abs() < 1e-6);
    assert!((m.net_pnl_usd - 0.309278).abs() < 1e-5, "pnl {}", m.net_pnl_usd);
    assert!((m.cents_per_trade.unwrap() - 3.0).abs() < 1e-4);
    assert!((m.return_on_locked_capital.unwrap() - 0.030928).abs() < 1e-5);
    assert!(
        (m.annualized_return_on_locked_capital.unwrap() - 1.881443).abs() < 1e-4,
        "annROLC {:?}",
        m.annualized_return_on_locked_capital
    );
}

/// The two cases side by side: identical cents/trade, wildly different
/// businesses. 15c premium on 85c of collateral for 8 days vs 15c on 85c of
/// price for 8 days — the engine must rank them by annROLC, not by cents.
#[test]
fn cents_per_trade_flatters_the_favourite_annualized_return_does_not() {
    // Sell a 15c wing: +15c/share on 85c locked.
    let sell = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);
    // Buy an 85c favourite: +15c/share on 85c locked, same 8 days — same ROLC.
    let buy = signals(&[row(
        "2026-05-01T12:00:00Z", "tokB", 0.99, 0.84, "0.83", "0.85", "", "", "Yes",
        "2026-05-09T12:00:00Z",
    )]);
    let a = simulate(&sell, &base()).unwrap().metrics;
    let b = simulate(&buy, &base()).unwrap().metrics;
    assert!((a.cents_per_trade.unwrap() - 15.0).abs() < 1e-4);
    assert!((b.cents_per_trade.unwrap() - 15.0).abs() < 1e-4);
    // Same cents, same days, same locked capital => same annualized return.
    assert!(
        (a.annualized_return_on_locked_capital.unwrap()
            - b.annualized_return_on_locked_capital.unwrap())
        .abs()
            < 1e-6
    );
    // Now the DESIGN.md §3 contrast: a +2c buy of a 97c favourite held 6 days.
    let fav = signals(&[row(
        "2026-05-01T12:00:00Z", "tokC", 0.999, 0.96, "0.95", "0.97", "", "", "Yes",
        "2026-05-07T12:00:00Z",
    )]);
    let c = simulate(&fav, &base()).unwrap().metrics;
    assert!(
        c.annualized_return_on_locked_capital.unwrap()
            < a.annualized_return_on_locked_capital.unwrap() / 4.0,
        "favourite {:?} should be far below the wing {:?}",
        c.annualized_return_on_locked_capital,
        a.annualized_return_on_locked_capital
    );
}

// --------------------------------------------------------------- the depth cap

/// **Depth cap.** Bid-side depth within 5c is $200 and `max_book_fraction` is
/// 0.15 ⇒ the stake may not exceed **$30**, whatever the policy asked for.
/// Slippage then walks the fill: 30/200 = 15% of the depth ⇒ 0.15 × 5c = 0.75c,
/// so we sell at 0.15 − 0.0075 = **0.1425** and lock (1 − 0.1425) per share.
#[test]
fn depth_caps_the_stake_and_walks_the_fill() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "200", "200", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let p = pol(
        "",
        "method = \"flat\"\nstake_usd = 500.0\nmax_per_market_usd = 1000.0",
        "",
        "assumed_spread = 0.03\nmax_book_fraction = 0.15\nfee_bps = 0",
    );
    let r = simulate(&s, &p).unwrap();
    assert_eq!(r.metrics.n, 1);
    assert!((r.metrics.staked_usd - 30.0).abs() < 1e-6, "staked {}", r.metrics.staked_usd);
    // shares = 30 / (1 - 0.1425) = 34.9854; pnl = 0.1425 * shares = 4.9854
    assert!((r.metrics.net_pnl_usd - 4.985423).abs() < 1e-5, "pnl {}", r.metrics.net_pnl_usd);
    assert!((r.metrics.cents_per_trade.unwrap() - 14.25).abs() < 1e-4);
}

/// **The unfundable path.** `max_book_fraction × depth` = 0.15 × $5 = $0.75,
/// below the $1 minimum ticket ⇒ the signal is *unfundable*: counted and
/// reported, never silently dropped.
#[test]
fn thin_depth_makes_a_signal_unfundable_and_it_is_counted() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "5", "5", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let p = pol("", "", "", "assumed_spread = 0.03\nmax_book_fraction = 0.15\nfee_bps = 0");
    let r = simulate(&s, &p).unwrap();
    assert_eq!(r.metrics.n, 0);
    assert_eq!(r.counts.unfundable, 1);
    assert_eq!(r.counts.signals, 1);
    assert!(r.notes.iter().any(|n| n.contains("UNFUNDABLE")), "{:?}", r.notes);
}

/// Nothing is ever silently dropped: every signal lands in exactly one terminal
/// bucket, and they sum to the number of signals.
#[test]
fn every_signal_lands_in_exactly_one_terminal_bucket() {
    let rows = vec![
        // tradeable sell
        row("2026-05-01T12:00:00Z", "t1", 0.02, 0.16, "0.15", "0.17", "1000", "1000", "No", "2026-05-09T12:00:00Z"),
        // inside the spread -> no executable edge
        row("2026-05-01T12:00:00Z", "t2", 0.16, 0.16, "0.15", "0.17", "1000", "1000", "No", "2026-05-09T12:00:00Z"),
        // edge too small
        row("2026-05-01T12:00:00Z", "t3", 0.149, 0.16, "0.15", "0.17", "1000", "1000", "No", "2026-05-09T12:00:00Z"),
        // depth too thin
        row("2026-05-01T12:00:00Z", "t4", 0.02, 0.16, "0.15", "0.17", "1", "1", "No", "2026-05-09T12:00:00Z"),
        // buy side
        row("2026-05-01T12:00:00Z", "t5", 0.99, 0.96, "0.95", "0.97", "1000", "1000", "Yes", "2026-05-07T12:00:00Z"),
    ];
    let s = signals(&rows);
    let p = pol(
        "min_edge = 0.05\nsides = [\"sell\"]\ndelay_hours = 0\nrequire_book = true\nmin_spread_ok = 0.05\nmin_depth_usd = 100.0\nrespect_venue_epsilon = true",
        "", "", "",
    );
    let r = simulate(&s, &p).unwrap();
    let c = &r.counts;
    let terminal = c.traded
        + c.no_quote
        + c.entry_after_resolution
        + c.delay_unavailable
        + c.no_executable_edge
        + c.side_excluded
        + c.below_min_edge
        + c.below_edge_percentile
        + c.spread_too_wide
        + c.depth_too_thin
        + c.unfundable
        + c.market_cap_full
        + c.stake_too_small;
    assert_eq!(terminal, c.signals, "counts do not account for every signal: {c:?}");
    assert_eq!(c.traded, 1);
    assert_eq!(c.no_executable_edge, 1);
    assert_eq!(c.below_min_edge, 1);
    assert_eq!(c.depth_too_thin, 1);
    assert_eq!(c.side_excluded, 1);
    // The epsilon screen was requested but no set carries the field: the one
    // sell we actually took is counted as unscreened.
    assert_eq!(c.epsilon_unavailable, 1);
    assert!(r.notes.iter().any(|n| n.contains("UNAPPLIED")), "{:?}", r.notes);
}

// ------------------------------------------------------------- synthetic fills

/// With only a midpoint, `assumed_spread` is applied symmetrically and the
/// trade is marked synthetic: mid 0.20, spread 3c ⇒ bid 0.185 / ask 0.215, and
/// a sell fills at **0.185**, never at 0.20.
#[test]
fn a_midpoint_only_row_never_fills_at_mid() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.05, 0.20, "", "", "", "", "No", "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&s, &base()).unwrap();
    assert_eq!(r.metrics.n, 1);
    assert!((r.metrics.mean_fill_price.unwrap() - 0.185).abs() < 1e-9);
    assert_eq!(r.metrics.synthetic_fill_share, Some(1.0));
    assert_eq!(r.counts.depth_unknown, 1);
    assert!(r.notes.iter().any(|n| n.contains("synthetic")), "{:?}", r.notes);
}

// ------------------------------------------------------------------ take-profit

/// **Take-profit exit.** Sell at 0.15 believing 0.05, `close_fraction = 0.60`
/// ⇒ target = 0.15 − 0.60 × (0.15 − 0.05) = **0.09**. Two days later the book
/// is 0.07 / 0.08: buying back at the ask (0.08 ≤ 0.09) triggers the exit.
///
/// shares = 10 / 0.85 = 11.7647; pnl = (0.15 − 0.08) × 11.7647 = **$0.8235**,
/// held **2 days** rather than 8 — the point of the policy.
#[test]
fn take_profit_exits_early_at_an_executable_price() {
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.05, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-03T12:00:00Z", "tokA", 0.05, 0.075, "0.07", "0.08", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let p = pol("", "", "rule = \"take-profit\"\nclose_fraction = 0.60\nelse_hold_to_resolution = true", "");
    let r = simulate(&s, &p).unwrap();
    // The second row is itself a signal; look at the first trade only.
    let first = r.equity_curve.first().unwrap();
    assert_eq!(first.t, "2026-05-01T12:00:00Z");
    assert_eq!(r.metrics.take_profit_exits, 1, "only the first row has a later price to exit into");
    // Trade 1: entered 05-01, exited 05-03 => 2 days, pnl 0.8235.
    // Trade 2 (from the 05-03 row): sells at 0.07 and holds to resolution.
    assert!(r.metrics.mean_hold_days.unwrap() < 8.0);

    // Isolate trade 1 by giving the second row a token the policy will not
    // trade (p inside its own spread), leaving it usable only as a price path.
    let s2 = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.05, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-03T12:00:00Z", "tokA", 0.075, 0.075, "0.07", "0.08", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let r2 = simulate(&s2, &p).unwrap();
    assert_eq!(r2.metrics.n, 1);
    assert_eq!(r2.metrics.take_profit_exits, 1);
    assert!((r2.metrics.net_pnl_usd - 0.823529).abs() < 1e-5, "pnl {}", r2.metrics.net_pnl_usd);
    assert!((r2.metrics.mean_hold_days.unwrap() - 2.0).abs() < 1e-9);
    // Same policy, hold-to-resolution: the full 15c premium over 8 days.
    let r3 = simulate(&s2, &base()).unwrap();
    assert!((r3.metrics.net_pnl_usd - 1.764706).abs() < 1e-5);
}

/// When the target is never reached, take-profit holds to resolution.
#[test]
fn take_profit_falls_back_to_holding() {
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.05, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-03T12:00:00Z", "tokA", 0.14, 0.14, "0.13", "0.15", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let p = pol("", "", "rule = \"take-profit\"\nclose_fraction = 0.60\nelse_hold_to_resolution = true", "");
    let r = simulate(&s, &p).unwrap();
    assert_eq!(r.metrics.n, 1);
    assert_eq!(r.metrics.take_profit_exits, 0);
    assert!((r.metrics.mean_hold_days.unwrap() - 8.0).abs() < 1e-9);
}

// ---------------------------------------------------------------- delayed entry

/// **Delayed entry.** `delay_hours = 24` enters at the price prevailing a day
/// later with the model frozen: the 05-01 signal (p = 0.02) fills against the
/// 05-02 book (bid 0.20), so it sells at **0.20**, not 0.15.
#[test]
fn delayed_entry_uses_the_later_price_with_a_frozen_model() {
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-02T12:00:00Z", "tokA", 0.02, 0.21, "0.20", "0.22", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let p = pol("min_edge = 0.01\nsides = [\"sell\"]\ndelay_hours = 24\nrequire_book = true\nmin_spread_ok = 1.0\nmin_depth_usd = 0.0\nrespect_venue_epsilon = false", "", "", "");
    let r = simulate(&s, &p).unwrap();
    // Signal 1 trades at the 05-02 price; signal 2 has no observation 24h later.
    assert_eq!(r.metrics.n, 1);
    assert_eq!(r.counts.delay_unavailable, 1);
    assert!((r.metrics.mean_fill_price.unwrap() - 0.20).abs() < 1e-9);
    // shares = 10/0.8 = 12.5, pnl = 0.20 * 12.5 = 2.50, held 05-02 -> 05-09 = 7d
    assert!((r.metrics.net_pnl_usd - 2.5).abs() < 1e-6);
    assert!((r.metrics.mean_hold_days.unwrap() - 7.0).abs() < 1e-9);
    assert!(
        r.notes.iter().any(|n| n.contains("EXCLUDED from this policy")),
        "{:?}",
        r.notes
    );
}

/// A delayed policy must not reach past the tolerance window and call a stale
/// observation "24h later".
#[test]
fn delayed_entry_refuses_an_observation_outside_the_tolerance() {
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        // 4 days later: far outside the default 12h tolerance on a 24h delay.
        row("2026-05-05T12:00:00Z", "tokA", 0.02, 0.21, "0.20", "0.22", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let p = pol("min_edge = 0.01\nsides = [\"sell\"]\ndelay_hours = 24\nrequire_book = true\nmin_spread_ok = 1.0\nmin_depth_usd = 0.0\nrespect_venue_epsilon = false", "", "", "");
    let r = simulate(&s, &p).unwrap();
    assert_eq!(r.metrics.n, 0);
    assert_eq!(r.counts.delay_unavailable, 2);
}

// ----------------------------------------------------------------- Kelly clamp

/// **Kelly and its clamps.** Sell at bid 0.20 believing 0.05:
/// full Kelly f = (0.20 − 0.05) / 0.20 = **0.75**; quarter-Kelly ⇒ 0.1875 of a
/// $1000 bankroll = **$187.50**, which `max_bankroll_fraction = 0.03` clamps to
/// **$30** and `max_per_market_usd = 20` clamps again to **$20**.
#[test]
fn kelly_sizing_is_clamped_by_bankroll_fraction_then_by_market_cap() {
    let mk = |max_per_market: f64, max_frac: f64| {
        format!(
            "method = \"fractional-kelly\"\nkelly_fraction = 0.25\nbankroll_usd = 1000.0\nmax_bankroll_fraction = {max_frac}\nmax_per_market_usd = {max_per_market}"
        )
    };
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.05, 0.21, "0.20", "0.22", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);

    // Unclamped: 0.75 * 0.25 * 1000 = 187.50
    let r = simulate(&s, &pol("", &mk(10_000.0, 1.0), "", "")).unwrap();
    assert!((r.metrics.staked_usd - 187.5).abs() < 1e-6, "staked {}", r.metrics.staked_usd);

    // Bankroll-fraction clamp bites: 3% of 1000 = 30
    let r = simulate(&s, &pol("", &mk(10_000.0, 0.03), "", "")).unwrap();
    assert!((r.metrics.staked_usd - 30.0).abs() < 1e-6, "staked {}", r.metrics.staked_usd);

    // Per-market clamp bites hardest: 20
    let r = simulate(&s, &pol("", &mk(20.0, 0.03), "", "")).unwrap();
    assert!((r.metrics.staked_usd - 20.0).abs() < 1e-6, "staked {}", r.metrics.staked_usd);

    // A buy: p = 0.99 vs ask 0.97 => f = (0.99-0.97)/(1-0.97) = 2/3;
    // quarter-Kelly on 1000 = 166.67, clamped by 5% of bankroll to 50.
    let b = signals(&[row(
        "2026-05-01T12:00:00Z", "tokB", 0.99, 0.96, "0.95", "0.97", "", "", "Yes",
        "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&b, &pol("", &mk(10_000.0, 1.0), "", "")).unwrap();
    assert!((r.metrics.staked_usd - 166.666667).abs() < 1e-4, "staked {}", r.metrics.staked_usd);
    let r = simulate(&b, &pol("", &mk(10_000.0, 0.05), "", "")).unwrap();
    assert!((r.metrics.staked_usd - 50.0).abs() < 1e-6);
}

/// Per-market exposure is cumulative across concurrently open positions.
#[test]
fn the_per_market_cap_counts_positions_still_open() {
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-02T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-03T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let p = pol("", "method = \"flat\"\nstake_usd = 10.0\nmax_per_market_usd = 15.0", "", "");
    let r = simulate(&s, &p).unwrap();
    // $10 then $5 (the room that is left) then nothing.
    assert_eq!(r.metrics.n, 2);
    assert!((r.metrics.staked_usd - 15.0).abs() < 1e-6);
    assert_eq!(r.counts.market_cap_full, 1);
}

/// `edge_percentile` is a property of the signal set: with ten signals and
/// `0.90`, only the largest claimed edge survives.
#[test]
fn edge_percentile_keeps_only_the_top_decile_of_the_set() {
    let rows: Vec<String> = (0..20)
        .map(|i| {
            let mid = 0.10 + 0.01 * i as f64; // claimed edge grows with i
            row(
                "2026-05-01T12:00:00Z",
                &format!("tok{i}"),
                0.01,
                mid,
                &format!("{:.4}", mid - 0.005),
                &format!("{:.4}", mid + 0.005),
                "",
                "",
                "No",
                "2026-05-09T12:00:00Z",
            )
        })
        .collect();
    let s = signals(&rows);
    let p = pol(
        "min_edge = 0.01\nedge_percentile = 0.90\nsides = [\"buy\", \"sell\"]\ndelay_hours = 0\nrequire_book = true\nmin_spread_ok = 1.0\nmin_depth_usd = 0.0\nrespect_venue_epsilon = false",
        "", "", "",
    );
    let r = simulate(&s, &p).unwrap();
    assert_eq!(r.metrics.n, 2, "top decile of 20 signals = 2 rows");
    assert_eq!(r.counts.below_edge_percentile, 18);
}

/// Fees are charged on the entry notional, and again on the exit only when the
/// position is closed in the market rather than settled.
#[test]
fn fees_are_charged_on_traded_notional_only() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokA", 0.02, 0.16, "0.15", "0.17", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let p = pol("", "", "", "assumed_spread = 0.03\nmax_book_fraction = 1.0\nfee_bps = 100");
    let r = simulate(&s, &p).unwrap();
    // shares = 11.7647; entry notional = 0.15 * 11.7647 = 1.7647; 1% = 0.017647
    assert!((r.metrics.net_pnl_usd - (1.764706 - 0.017647)).abs() < 1e-5, "pnl {}", r.metrics.net_pnl_usd);
}

// ============================================================ venue taker fees
//
// Polymarket charges takers `shares × rate × p × (1 − p)` USDC on every fill
// (docs.polymarket.com/trading/fees, verified 2026-07-25 against each market's
// own `feeSchedule` and against real executed fills). It is charged on entry,
// and again on an exit taken in the market; settlement at resolution is a
// redemption, not a match, and costs nothing. Every number below is worked out
// by hand in the comment above it.

/// A row with an explicit `asset`, so the per-category rate can be exercised.
#[allow(clippy::too_many_arguments)]
fn row_asset(
    asset: &str,
    t: &str,
    token: &str,
    p_model: f64,
    p_market: f64,
    bid: &str,
    ask: &str,
    resolved_outcome: &str,
    resolved: &str,
) -> String {
    format!(
        "s,{t},mkt-{token},0x{token},Yes,{token},fam,var,m,{p_model},{p_market},{bid},{ask},,,{resolved_outcome},{resolved},0,{asset}"
    )
}

/// Costs block with the real fee model on. `ast` — the asset the `row` helper
/// writes — is mapped to `category`.
fn fee_costs(category: &str) -> String {
    format!(
        "assumed_spread = 0.03\nmax_book_fraction = 1.0\n\
         fee_model = \"polymarket-taker\"\nfee_rate_default = 0.05\n\
         [costs.fee_rate]\ncrypto = 0.07\nfinance = 0.04\nsports = 0.05\n\
         [costs.asset_category]\nast = \"{category}\"\nbtc = \"crypto\"\nspy = \"finance\"\n"
    )
}

/// **The published fee table, reproduced.** docs.polymarket.com/trading/fees
/// tabulates the fee on 100 shares. If `taker_fee` disagrees with the venue's
/// own table, `taker_fee` is wrong.
#[test]
fn taker_fee_reproduces_the_published_fee_table() {
    // Crypto (0.07): $1.75 at p = 0.50, $0.63 at 0.10, $0.33 at 0.05, $0.07 at 0.01.
    assert!((engine::taker_fee(100.0, 0.07, 0.50) - 1.75).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.07, 0.10) - 0.63).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.07, 0.05) - 0.3325).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.07, 0.01) - 0.0693).abs() < 1e-9);
    // Sports / economics / culture / weather / other (0.05): $1.25 at p = 0.50.
    assert!((engine::taker_fee(100.0, 0.05, 0.50) - 1.25).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.05, 0.40) - 1.20).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.05, 0.10) - 0.45).abs() < 1e-9);
    // Finance / politics / tech / mentions (0.04): $1.00 at p = 0.50.
    assert!((engine::taker_fee(100.0, 0.04, 0.50) - 1.00).abs() < 1e-9);
    assert!((engine::taker_fee(100.0, 0.04, 0.20) - 0.64).abs() < 1e-9);
    // Geopolitics is fee-free at any price.
    assert_eq!(engine::taker_fee(100.0, 0.0, 0.50), 0.0);
}

/// The fee is symmetric about 0.50 — selling YES at `p` *is* buying NO at
/// `1 − p` (DESIGN.md §3), so the two must cost the same — peaks there, and
/// vanishes at the boundaries. Venue rounding: 5dp, and anything under
/// 0.00001 USDC is not charged.
#[test]
fn taker_fee_is_symmetric_peaks_at_a_half_and_rounds_like_the_venue() {
    for p in [0.01, 0.10, 0.23, 0.40, 0.49] {
        assert!(
            (engine::taker_fee(37.0, 0.05, p) - engine::taker_fee(37.0, 0.05, 1.0 - p)).abs() < 1e-12,
            "asymmetric at {p}"
        );
        assert!(engine::taker_fee(37.0, 0.05, p) < engine::taker_fee(37.0, 0.05, 0.50));
    }
    assert_eq!(engine::taker_fee(100.0, 0.05, 0.0), 0.0);
    assert_eq!(engine::taker_fee(100.0, 0.05, 1.0), 0.0);
    // 0.001 shares at 1c: 0.001 × 0.05 × 0.01 × 0.99 = 4.95e-7 -> rounds to zero.
    assert_eq!(engine::taker_fee(0.001, 0.05, 0.01), 0.0);
    // Nothing is charged when the model is off (rate 0) or the trade is empty.
    assert_eq!(engine::taker_fee(0.0, 0.05, 0.5), 0.0);
}

/// **Hand-computed fee at p = 0.50** — the worst price on the curve.
///
/// Book 0.50 / 0.52, we think 0.40, flat $10, sports/economics rate 0.05,
/// hold to resolution, market resolves **No** (our YES token loses ⇒ the wing
/// seller keeps the premium).
///
/// - sell at the bid **0.50**; capital per share = 1 − 0.50 = **0.50**;
///   shares = 10 / 0.50 = **20**.
/// - gross pnl = 0.50 × 20 = **$10.00**.
/// - fee = 20 × 0.05 × 0.50 × 0.50 = **$0.25** — i.e. **1.25c/share**, the
///   published peak, against a 3c assumed spread.
/// - net pnl = 10.00 − 0.25 = **$9.75**; cents/trade = 100 × 9.75 / 20 = **48.75c**.
#[test]
fn fee_at_a_half_is_hand_computable() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokH", 0.40, 0.51, "0.50", "0.52", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&s, &pol("", "", "", &fee_costs("sports"))).unwrap();
    let m = &r.metrics;
    assert_eq!(m.n, 1);
    assert!((m.gross_pnl_usd - 10.0).abs() < 1e-9, "gross {}", m.gross_pnl_usd);
    assert!((m.fees_usd - 0.25).abs() < 1e-9, "fees {}", m.fees_usd);
    assert!((m.net_pnl_usd - 9.75).abs() < 1e-9, "net {}", m.net_pnl_usd);
    assert!((m.fee_cents_per_share.unwrap() - 1.25).abs() < 1e-9);
    assert!((m.cents_per_trade.unwrap() - 48.75).abs() < 1e-6);
    // ROLC uses the fee-bearing pnl over collateral only: 9.75 / 10.
    assert!((m.return_on_locked_capital.unwrap() - 0.975).abs() < 1e-9);
}

/// **Hand-computed fee at p = 0.10** — deep in the fundable wing band.
///
/// Book 0.10 / 0.12, we think 0.02, flat **$9**, rate 0.05, resolves **No**.
///
/// - sell at **0.10**; capital per share = **0.90**; shares = 9 / 0.90 = **10**.
/// - gross pnl = 0.10 × 10 = **$1.00**.
/// - fee = 10 × 0.05 × 0.10 × 0.90 = **$0.045** = **0.45c/share**.
/// - net pnl = **$0.955**; the fee is 4.5% of the gross edge.
#[test]
fn fee_at_a_tenth_is_hand_computable() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokT", 0.02, 0.11, "0.10", "0.12", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let sizing = "method = \"flat\"\nstake_usd = 9.0\nmax_per_market_usd = 1000.0";
    let r = simulate(&s, &pol("", sizing, "", &fee_costs("sports"))).unwrap();
    let m = &r.metrics;
    assert_eq!(m.n, 1);
    assert!((m.gross_pnl_usd - 1.0).abs() < 1e-9, "gross {}", m.gross_pnl_usd);
    assert!((m.fees_usd - 0.045).abs() < 1e-9, "fees {}", m.fees_usd);
    assert!((m.net_pnl_usd - 0.955).abs() < 1e-9, "net {}", m.net_pnl_usd);
    assert!((m.fee_cents_per_share.unwrap() - 0.45).abs() < 1e-9);
    assert!((m.fee_share_of_gross.unwrap() - 0.045).abs() < 1e-9);
}

/// **A round trip pays the fee twice; holding to resolution pays it once.**
/// This is the whole reason an early-exit policy's ranking had to be re-checked.
///
/// Same entry both ways: sell at 0.50, shares = **20**, entry fee = **$0.25**.
///
/// *Held to resolution:* market resolves No ⇒ gross **$10.00**, fees **$0.25**
/// (redemption is not a match), net **$9.75**.
///
/// *Taken in the market:* `close_fraction = 0.60` ⇒ target = 0.50 − 0.60 ×
/// (0.50 − 0.40) = **0.44**; two days later the book is 0.42 / 0.44 and we buy
/// the short back at the ask **0.44**.
/// - gross = 20 × (0.50 − 0.44) = **$1.20**.
/// - exit fee = 20 × 0.05 × 0.44 × 0.56 = **$0.2464**.
/// - fees = 0.25 + 0.2464 = **$0.4964**; net = 1.20 − 0.4964 = **$0.7036**.
///
/// The exit alone costs 41% of the gross move it captured.
#[test]
fn a_round_trip_pays_the_fee_twice_and_settlement_pays_it_once() {
    // The second row is only a price path: its own p sits inside its spread, so
    // the policy will not open a position on it.
    let s = signals(&[
        row("2026-05-01T12:00:00Z", "tokR", 0.40, 0.51, "0.50", "0.52", "", "", "No", "2026-05-09T12:00:00Z"),
        row("2026-05-03T12:00:00Z", "tokR", 0.43, 0.43, "0.42", "0.44", "", "", "No", "2026-05-09T12:00:00Z"),
    ]);
    let costs = fee_costs("sports");
    let tp = pol("", "", "rule = \"take-profit\"\nclose_fraction = 0.60\nelse_hold_to_resolution = true", &costs);
    let hold = pol("", "", "", &costs);

    let a = simulate(&s, &tp).unwrap().metrics;
    assert_eq!(a.n, 1);
    assert_eq!(a.take_profit_exits, 1);
    assert!((a.gross_pnl_usd - 1.20).abs() < 1e-9, "gross {}", a.gross_pnl_usd);
    assert!((a.fees_usd - 0.4964).abs() < 1e-9, "fees {}", a.fees_usd);
    assert!((a.net_pnl_usd - 0.7036).abs() < 1e-9, "net {}", a.net_pnl_usd);

    let b = simulate(&s, &hold).unwrap().metrics;
    assert_eq!(b.n, 1);
    assert_eq!(b.take_profit_exits, 0);
    assert!((b.fees_usd - 0.25).abs() < 1e-9, "fees {}", b.fees_usd);
    assert!((b.net_pnl_usd - 9.75).abs() < 1e-9, "net {}", b.net_pnl_usd);

    // The entry fee is identical, so the whole difference is the second fill.
    assert!((a.fees_usd - b.fees_usd - 0.2464).abs() < 1e-9);
    assert!(a.fees_usd > b.fees_usd * 1.9, "a round trip must cost roughly twice the entry");
}

/// The rate follows the market's category, not the policy: the identical trade
/// costs 75% more on a crypto market than on a finance one (0.07 vs 0.04).
#[test]
fn the_rate_comes_from_the_assets_category() {
    let mk = |asset: &str, token: &str| {
        signals(&[row_asset(asset, "2026-05-01T12:00:00Z", token, 0.40, 0.51, "0.50", "0.52", "No", "2026-05-09T12:00:00Z")])
    };
    let p = pol("", "", "", &fee_costs("sports"));
    // shares = 20 either way; crypto 20 × 0.07 × 0.25 = 0.35, finance = 0.20.
    let c = simulate(&mk("btc", "tokX"), &p).unwrap().metrics;
    let f = simulate(&mk("spy", "tokY"), &p).unwrap().metrics;
    assert!((c.fees_usd - 0.35).abs() < 1e-9, "crypto fees {}", c.fees_usd);
    assert!((f.fees_usd - 0.20).abs() < 1e-9, "finance fees {}", f.fees_usd);
    assert!((c.net_pnl_usd - 9.65).abs() < 1e-9);
    assert!((f.net_pnl_usd - 9.80).abs() < 1e-9);
}

/// An asset nobody mapped must never trade fee-free by accident: it is charged
/// the declared fallback, counted, and named in the notes.
#[test]
fn an_unmapped_asset_is_charged_the_default_and_counted() {
    let s = signals(&[row_asset(
        "dogecoin", "2026-05-01T12:00:00Z", "tokU", 0.40, 0.51, "0.50", "0.52", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&s, &pol("", "", "", &fee_costs("sports"))).unwrap();
    assert_eq!(r.counts.fee_rate_unmapped, 1);
    // fee_rate_default = 0.05 => 20 × 0.05 × 0.25 = 0.25
    assert!((r.metrics.fees_usd - 0.25).abs() < 1e-9);
    assert!(
        r.notes.iter().any(|n| n.contains("no costs.asset_category entry")),
        "{:?}",
        r.notes
    );
}

/// A policy with no fee model charges nothing — and is required to *say* so, so
/// a v1 result can never be mistaken for a costed one.
#[test]
fn a_fee_free_policy_charges_nothing_and_admits_it() {
    let s = signals(&[row(
        "2026-05-01T12:00:00Z", "tokF", 0.40, 0.51, "0.50", "0.52", "", "", "No",
        "2026-05-09T12:00:00Z",
    )]);
    let r = simulate(&s, &base()).unwrap();
    assert_eq!(r.metrics.fees_usd, 0.0);
    assert!((r.metrics.net_pnl_usd - r.metrics.gross_pnl_usd).abs() < 1e-12);
    assert!((r.metrics.net_pnl_usd - 10.0).abs() < 1e-9);
    assert!(r.fee_model.starts_with("none"), "{}", r.fee_model);
    assert!(
        r.notes.iter().any(|n| n.contains("NO VENUE FEE IS CHARGED")),
        "a fee-free result must say so: {:?}",
        r.notes
    );
}

/// The fee model refuses to load half-specified, because a cost model that can
/// silently charge nothing is the bug this version exists to fix.
#[test]
fn an_incomplete_fee_model_is_rejected() {
    let head = "name = \"t\"\nversion = 2\n[entry]\nmin_edge = 0.01\nsides = [\"sell\"]\nmin_spread_ok = 1.0\nmin_depth_usd = 0.0\n[sizing]\nmethod = \"flat\"\nstake_usd = 10.0\nmax_per_market_usd = 100.0\n[exit]\nrule = \"hold-to-resolution\"\n[costs]\n";
    let err = |c: &str| Policy::from_toml(&format!("{head}{c}")).unwrap_err();

    assert!(err("fee_model = \"polymarket-taker\"\n").contains("[costs.fee_rate]"));
    assert!(err("fee_model = \"polymarket-taker\"\n[costs.fee_rate]\ncrypto = 0.07\n")
        .contains("fee_rate_default"));
    assert!(err(
        "fee_model = \"polymarket-taker\"\nfee_rate_default = 0.05\n[costs.fee_rate]\ncrypto = 0.07\n[costs.asset_category]\nbtc = \"cryptoo\"\n"
    )
    .contains("has no rate"));
    assert!(err("fee_model = \"polymarket-taker\"\nfee_rate_default = 9.0\n[costs.fee_rate]\ncrypto = 0.07\n")
        .contains("not a rate"));
    assert!(err("[costs.fee_rate]\ncrypto = 7.0\n").contains("not a rate"));
    // ...and a fully specified one loads.
    assert!(Policy::from_toml(&format!(
        "{head}fee_model = \"polymarket-taker\"\nfee_rate_default = 0.05\n[costs.fee_rate]\ncrypto = 0.07\n[costs.asset_category]\nbtc = \"crypto\"\n"
    ))
    .is_ok());
}
