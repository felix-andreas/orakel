//! `/execution` — the paper book: what the firm holds right now, and what a
//! chosen execution policy would do about it next.
//!
//! Two tabs, both real links (`?tab=`), because a plan has to be linkable:
//!
//!   Holdings  cash, committed capital, every open position, and the ledger
//!             those two are derived from.
//!   Plan      pick strategies × markets × policy, get the target positions and
//!             the DIFF against what is open — `terraform plan`, for a book.
//!
//! **It is paper.** No wallet, no order, no venue (CONSTITUTION.md §5). The
//! page says so on its own surface, not only in this comment.
//!
//! Three properties this module is built to keep:
//!
//! 1. **Plan is a pure function.** Selection + current book + current prices in,
//!    diff out. No state is written, nothing is remembered between requests, and
//!    the whole selection lives in the URL — so a plan link re-runs to the same
//!    plan for anyone who opens it.
//! 2. **Apply cannot write, and does not pretend to.** The dashboard is a
//!    read-only Worker with no commit path by design (portfolio/README.md), so
//!    "apply" renders the exact `ledger.csv` rows the plan implies and the CEO
//!    commits them. There is no fake success state.
//! 3. **The accounting is the engine's, copied not re-derived.** Collateral,
//!    fees, Kelly sizing and slippage are ported line-for-line from
//!    `execution/engine/src/sim.rs` so the book and the backtest cannot drift
//!    (CODING.md: copy freely; only conventions are single-source).
//!
//! The gate order in `candidates()` mirrors `sim.rs::simulate` exactly. That is
//! deliberate: the most valuable screen here is not the trades a policy takes,
//! it is the ones it refuses and why — on our own live signals seven of the
//! eight policies take zero trades, and a plan that proposes nothing has to say
//! which gate stopped each candidate rather than render an empty table.

use std::collections::HashMap;

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_prob, fmt_signed, fmt_ts, section, section_foot, stat, stat_grid, table,
};
use crate::{shell_sub, trail};
use worker::{Date, Env, Url};

use crate::snapshots;

/// Depth is measured within this distance of the touch (DESIGN.md §4.2) and it
/// also sets the scale of the linear slippage penalty. Same constant as the
/// engine's `DEPTH_BAND`.
const DEPTH_BAND: f64 = 0.05;
/// Polymarket's minimum ticket, the engine's `DEFAULT_MIN_STAKE_USD`.
const DEFAULT_MIN_STAKE_USD: f64 = 1.0;
/// The engine's `DEFAULT_BANKROLL_USD`, used when a policy omits `bankroll_usd`.
const DEFAULT_BANKROLL_USD: f64 = 1000.0;
/// Chosen when neither the URL nor `portfolio.toml` names a policy. A v2: v1 is
/// fee-free and exists only so the v1→v2 delta is readable as the cost of the
/// fee, so it must never be the thing you get by not choosing.
const DEFAULT_POLICY: &str = "fade-v2";
/// Shares/collateral closer than this are the same position.
const EPS: f64 = 1e-6;

// ---------------------------------------------------------------------------
// Selection — the whole state of this page, and all of it in the URL
// ---------------------------------------------------------------------------

/// What the reader picked. Empty vectors mean "the default", which is resolved
/// against the data once it is read — so a bare `/execution?tab=plan` is a
/// complete, shareable plan and not an error state.
struct Sel {
    tab: String,
    /// `family/variant` keys.
    strategies: Vec<String>,
    /// Market slugs.
    markets: Vec<String>,
    /// Policy file stem, e.g. `fade-v2`.
    policy: String,
}

fn parse_sel(url: &Url) -> Sel {
    let mut sel = Sel {
        tab: String::new(),
        strategies: Vec::new(),
        markets: Vec::new(),
        policy: String::new(),
    };
    for (k, v) in url.query_pairs() {
        let v = v.trim().to_string();
        if v.is_empty() {
            continue;
        }
        match k.as_ref() {
            "tab" => sel.tab = v,
            // Repeated (`s=a&s=b`, what a checkbox form emits) and
            // comma-joined (`s=a,b`, what a human types) both work.
            "s" => sel.strategies.extend(split_list(&v).filter(|x| valid_variant_key(x))),
            "m" => sel.markets.extend(split_list(&v).filter(|x| data::safe_segment(x))),
            "p" => {
                if data::safe_segment(&v) {
                    sel.policy = v;
                }
            }
            _ => {}
        }
    }
    sel.strategies.sort();
    sel.strategies.dedup();
    sel.markets.sort();
    sel.markets.dedup();
    sel
}

fn split_list(v: &str) -> impl Iterator<Item = String> + '_ {
    v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

fn valid_variant_key(s: &str) -> bool {
    match s.split_once('/') {
        Some((f, v)) => data::safe_segment(f) && data::safe_segment(v),
        None => false,
    }
}

/// Percent-encode one query value. Only the unreserved set survives, so a
/// `family/variant` key round-trips through the URL intact.
fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// This page's address with `tab` swapped and everything else preserved —
/// switching tabs must never silently change what you are looking at.
fn href(sel: &Sel, tab: &str) -> String {
    let mut q: Vec<String> = Vec::new();
    if !tab.is_empty() {
        q.push(format!("tab={}", enc(tab)));
    }
    for s in &sel.strategies {
        q.push(format!("s={}", enc(s)));
    }
    for m in &sel.markets {
        q.push(format!("m={}", enc(m)));
    }
    if !sel.policy.is_empty() {
        q.push(format!("p={}", enc(&sel.policy)));
    }
    if q.is_empty() {
        "/execution".to_string()
    } else {
        format!("/execution?{}", q.join("&"))
    }
}

fn tabbar(sel: &Sel, open_positions: usize) -> String {
    let tabs = [("", "Holdings", render::fmt_int(open_positions as i64)), ("plan", "Plan", String::new())];
    let mut out = String::new();
    for (key, label, count) in &tabs {
        let n = if count.is_empty() {
            String::new()
        } else {
            format!("<span class=\"tab-n\">{}</span>", esc(count))
        };
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}{}</a>",
            esc(&href(sel, key)),
            if *key == sel.tab { " aria-current=\"page\"" } else { "" },
            esc(label),
            n
        ));
    }
    format!("<nav class=\"subbar\" aria-label=\"Sections of this page\"><div class=\"subbar-in\">{out}</div></nav>")
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn page(env: &Env, url: &Url) -> String {
    let sel = parse_sel(url);
    if sel.tab == "plan" {
        plan_page(env, &sel).await
    } else {
        holdings_page(env, &sel).await
    }
}

// ---------------------------------------------------------------------------
// Money, shares, dates
// ---------------------------------------------------------------------------

/// USDC to the cent, thousands-separated, with the house minus sign. No "$":
/// this is paper USDC and the column header says so — a dollar sign on a page
/// about pretend money is exactly the decoration PRINCIPLES.md forbids.
fn usd(v: f64) -> String {
    let neg = v < -0.005;
    let x = v.abs();
    let whole = x.trunc() as i64;
    let cents = ((x - whole as f64) * 100.0).round() as i64;
    // Rounding .995 up carries into the whole part.
    let (whole, cents) = if cents >= 100 { (whole + 1, 0) } else { (whole, cents) };
    format!(
        "{}{}.{:02}",
        if neg { "−" } else { "" },
        render::fmt_int(whole),
        cents
    )
}

/// Signed USDC — for anything that can go either way (P&L, cash deltas), where
/// the sign is the first thing the eye needs.
fn usd_signed(v: f64) -> String {
    if v >= -0.005 {
        format!("+{}", usd(v))
    } else {
        usd(v)
    }
}

fn shares_fmt(v: f64) -> String {
    format!("{v:.2}")
}

fn tone_of(v: f64) -> &'static str {
    if v > 0.005 {
        "ok"
    } else if v < -0.005 {
        "bad"
    } else {
        ""
    }
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    (if mo <= 2 { y + 1 } else { y }, mo, d)
}

fn now_ms() -> i64 {
    Date::now().as_millis() as i64
}

fn today_utc() -> String {
    let (y, m, d) = civil_from_days(now_ms().div_euclid(86_400_000));
    format!("{y:04}-{m:02}-{d:02}")
}

/// `2026-07-26T11:04:07Z` for the `applied_at` column of the rows we hand the
/// CEO to append.
fn now_iso() -> String {
    let ms = now_ms();
    let (y, m, d) = civil_from_days(ms.div_euclid(86_400_000));
    let secs = ms.rem_euclid(86_400_000) / 1000;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Whole days a position has been open (0 when it opened today).
fn days_held(opened_at: &str) -> i64 {
    let from = opened_at.get(..10).unwrap_or(opened_at);
    data::days_between(from, &today_utc()).unwrap_or(0).max(0)
}

// ---------------------------------------------------------------------------
// The accounting, ported from execution/engine/src/sim.rs
// ---------------------------------------------------------------------------

/// Capital committed per share (DESIGN.md §3). Buying costs the price; selling
/// YES at `p` is buying NO at `1 − p`, so it locks `1 − p` — for the whole
/// holding period. This one line is what makes "return on locked capital" the
/// honest metric and "cents per trade" the flattering one.
fn capital_per_share(side: &str, price: f64) -> f64 {
    if side == "sell" {
        1.0 - price
    } else {
        price
    }
}

/// Polymarket's published taker fee for ONE fill (DESIGN.md §4.4):
/// `fee = shares × rate × p × (1 − p)`, rounded to 5 decimals, with anything
/// under 0.00001 USDC rounding to zero. Charged on entry and again on an exit
/// taken in the market; settlement at resolution is a redemption and is free.
fn taker_fee(shares: f64, rate: f64, price: f64) -> f64 {
    if !shares.is_finite() || !rate.is_finite() || shares <= 0.0 || rate <= 0.0 {
        return 0.0;
    }
    if !(0.0..=1.0).contains(&price) {
        return 0.0;
    }
    let rounded = (shares * rate * price * (1.0 - price) * 1e5).round() / 1e5;
    if rounded < 1e-5 {
        0.0
    } else {
        rounded
    }
}

/// Full-Kelly fraction of bankroll at the *executable* price.
fn kelly_fraction(side: &str, p: f64, price: f64) -> f64 {
    let f = if side == "sell" {
        if price <= 0.0 {
            return 0.0;
        }
        (price - p) / price
    } else {
        if price >= 1.0 {
            return 0.0;
        }
        (p - price) / (1.0 - price)
    };
    f.clamp(0.0, 1.0)
}

/// Linear slippage: eating the whole depth within `DEPTH_BAND` of the touch
/// walks the price by that band, bounded by the room the price actually has (a
/// 1c bid cannot slip 5c, it can only slip to zero).
fn apply_slippage(side: &str, touch: f64, stake: f64, depth: Option<f64>) -> f64 {
    let room = if side == "sell" { touch } else { 1.0 - touch };
    let band = DEPTH_BAND.min(room.max(0.0));
    let slip = match depth {
        Some(d) if d > 0.0 => band * (stake / d).min(1.0),
        _ => 0.0,
    };
    if side == "sell" {
        (touch - slip).max(0.0)
    } else {
        (touch + slip).min(1.0)
    }
}

/// Unrealised P&L of an open position marked at `mark`, per DESIGN.md §3:
/// a long earns `mark − entry` per share, a short earns `entry − mark`.
fn pnl_at(side: &str, entry: f64, mark: f64, shares: f64) -> f64 {
    if side == "sell" {
        shares * (entry - mark)
    } else {
        shares * (mark - entry)
    }
}

/// Return on locked capital over the hold, and the same annualized
/// (`pnl / (capital × days) × 365`). `None` when the position is younger than a
/// day — annualizing a few hours produces a number that means nothing.
fn rolc(pnl: f64, capital: f64, days: i64) -> (Option<f64>, Option<f64>) {
    if capital <= EPS {
        return (None, None);
    }
    let r = pnl / capital;
    (Some(r), if days >= 1 { Some(r * 365.0 / days as f64) } else { None })
}

// ---------------------------------------------------------------------------
// Policies
// ---------------------------------------------------------------------------

/// One `execution/policies/<file>.toml`, flattened. Only the fields a live plan
/// can act on: the exit simulator's path-walking has no meaning for a book that
/// has not moved yet, so `exit` is carried but only `take-profit` is evaluated,
/// against today's quote.
struct Policy {
    file: String,
    name: String,
    version: i64,
    character: String,
    min_edge: f64,
    edge_percentile: Option<f64>,
    sides: Vec<String>,
    delay_hours: f64,
    min_spread_ok: f64,
    min_depth_usd: f64,
    respect_venue_epsilon: bool,
    sizing_method: String,
    stake_usd: Option<f64>,
    kelly: Option<f64>,
    bankroll: f64,
    max_bankroll_fraction: Option<f64>,
    max_per_market_usd: f64,
    min_stake: f64,
    exit_rule: String,
    close_fraction: Option<f64>,
    assumed_spread: f64,
    max_book_fraction: f64,
    taker_fees: bool,
    fee_rate: Vec<(String, f64)>,
    asset_category: Vec<(String, String)>,
    fee_rate_default: f64,
}

/// TOML numbers are floats or integers depending on how they were typed
/// (`delay_hours = 0` is an integer, `min_depth_usd = 200.0` a float), so both
/// have to be accepted or half the policy silently reads as zero.
fn fnum(t: &toml::Table, path: &[&str]) -> Option<f64> {
    match data::value_at(t, path)? {
        toml::Value::Float(f) => Some(*f),
        toml::Value::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

fn fnum_or(t: &toml::Table, path: &[&str], default: f64) -> f64 {
    fnum(t, path).unwrap_or(default)
}

fn parse_policy(file: &str, src: &str) -> Policy {
    let t = data::toml_of(src);
    let pairs = |path: &[&str]| -> Vec<(String, f64)> {
        data::table_at(&t, path)
            .map(|tb| {
                tb.iter()
                    .filter_map(|(k, v)| {
                        let n = match v {
                            toml::Value::Float(f) => Some(*f),
                            toml::Value::Integer(i) => Some(*i as f64),
                            _ => None,
                        }?;
                        Some((k.clone(), n))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let strs = |path: &[&str]| -> Vec<(String, String)> {
        data::table_at(&t, path)
            .map(|tb| {
                tb.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    };
    Policy {
        file: file.to_string(),
        name: data::tstr(&t, "name").to_string(),
        version: t.get("version").and_then(|v| v.as_integer()).unwrap_or(0),
        character: data::tstr(&t, "character").trim().to_string(),
        min_edge: fnum_or(&t, &["entry", "min_edge"], 0.0),
        edge_percentile: fnum(&t, &["entry", "edge_percentile"]),
        sides: data::str_list(&t, &["entry", "sides"])
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect(),
        delay_hours: fnum_or(&t, &["entry", "delay_hours"], 0.0),
        min_spread_ok: fnum_or(&t, &["entry", "min_spread_ok"], 1.0),
        min_depth_usd: fnum_or(&t, &["entry", "min_depth_usd"], 0.0),
        respect_venue_epsilon: data::bool_at(&t, &["entry", "respect_venue_epsilon"])
            .unwrap_or(false),
        sizing_method: data::str_at(&t, &["sizing", "method"]).to_string(),
        stake_usd: fnum(&t, &["sizing", "stake_usd"]),
        kelly: fnum(&t, &["sizing", "kelly_fraction"]),
        bankroll: fnum_or(&t, &["sizing", "bankroll_usd"], DEFAULT_BANKROLL_USD),
        max_bankroll_fraction: fnum(&t, &["sizing", "max_bankroll_fraction"]),
        max_per_market_usd: fnum_or(&t, &["sizing", "max_per_market_usd"], f64::INFINITY),
        min_stake: fnum_or(&t, &["sizing", "min_stake_usd"], DEFAULT_MIN_STAKE_USD),
        exit_rule: data::str_at(&t, &["exit", "rule"]).to_string(),
        close_fraction: fnum(&t, &["exit", "close_fraction"]),
        assumed_spread: fnum_or(&t, &["costs", "assumed_spread"], 0.03),
        max_book_fraction: fnum_or(&t, &["costs", "max_book_fraction"], 1.0),
        taker_fees: data::str_at(&t, &["costs", "fee_model"]) == "polymarket-taker",
        fee_rate: pairs(&["costs", "fee_rate"]),
        asset_category: strs(&["costs", "asset_category"]),
        fee_rate_default: fnum_or(&t, &["costs", "fee_rate_default"], 0.0),
    }
}

impl Policy {
    /// The taker rate for one asset, and whether an explicit mapping supplied
    /// it. `false` ⇒ the policy's `fee_rate_default` was used, which the engine
    /// counts rather than letting an unmapped asset trade fee-free by accident.
    fn rate_for(&self, asset: &str) -> (f64, bool) {
        if !self.taker_fees {
            return (0.0, true);
        }
        let key = asset.trim().to_ascii_lowercase();
        if let Some((_, cat)) = self.asset_category.iter().find(|(a, _)| *a == key) {
            if let Some((_, r)) = self.fee_rate.iter().find(|(c, _)| c == cat) {
                return (*r, true);
            }
        }
        (self.fee_rate_default, false)
    }

    fn fee_label(&self) -> String {
        if self.taker_fees {
            "the venue's real taker fee, charged per fill".to_string()
        } else {
            "no fee at all — this policy is fee-free and its numbers are not costed".to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// Quotes: the current book
// ---------------------------------------------------------------------------

/// One token's two-sided market right now. `synthetic` ⇒ there was no real book
/// and this was built from a midpoint plus the policy's assumed spread, which
/// is a guess about a price, not an observation of one.
struct Quote {
    bid: f64,
    ask: f64,
    mid: f64,
    depth_bid: Option<f64>,
    depth_ask: Option<f64>,
    synthetic: bool,
}

/// Every token in the newest hourly R2 book snapshot, with the USD notional
/// resting within 5c of each touch. Same construction as the engine's signal
/// adapter (`build-orakel-live.rs`), so a plan and a backtest read depth the
/// same way.
fn books_from_snapshot(doc: &serde_json::Value) -> HashMap<String, Quote> {
    let mut out = HashMap::new();
    let Some(markets) = doc["markets"].as_array() else {
        return out;
    };
    for m in markets {
        let Some(tokens) = m["tokens"].as_array() else { continue };
        for tok in tokens {
            if tok.get("error").is_some() {
                continue;
            }
            let Some(id) = tok["token_id"].as_str() else { continue };
            let bids = levels(&tok["bids"]);
            let asks = levels(&tok["asks"]);
            let (Some(&(bid, _)), Some(&(ask, _))) = (bids.first(), asks.first()) else {
                continue;
            };
            if ask <= bid {
                continue;
            }
            let depth_bid: f64 =
                bids.iter().filter(|(p, _)| *p >= bid - DEPTH_BAND).map(|(p, s)| p * s).sum();
            let depth_ask: f64 =
                asks.iter().filter(|(p, _)| *p <= ask + DEPTH_BAND).map(|(p, s)| p * s).sum();
            out.insert(
                id.to_string(),
                Quote {
                    bid,
                    ask,
                    mid: tok["midpoint"].as_f64().unwrap_or((bid + ask) / 2.0),
                    depth_bid: Some(depth_bid),
                    depth_ask: Some(depth_ask),
                    synthetic: false,
                },
            );
        }
    }
    out
}

fn levels(v: &serde_json::Value) -> Vec<(f64, f64)> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|l| Some((l.get(0)?.as_f64()?, l.get(1)?.as_f64()?)))
                .collect()
        })
        .unwrap_or_default()
}

/// The fallback when no snapshot carries this token: the midpoint we logged
/// with the prediction, spread apart by the policy's `assumed_spread`. Flagged
/// everywhere it is used — it is an assumption about a book, not a book.
fn synthetic_quote(mid: f64, assumed_spread: f64) -> Option<Quote> {
    let half = assumed_spread / 2.0;
    let bid = (mid - half).max(0.0);
    let ask = (mid + half).min(1.0);
    if ask <= bid {
        return None;
    }
    Some(Quote { bid, ask, mid, depth_bid: None, depth_ask: None, synthetic: true })
}

// ---------------------------------------------------------------------------
// The paper label. One line, at the top of both tabs, in the warn tone that is
// already reserved for "read this before you believe the numbers".
// ---------------------------------------------------------------------------

fn paper_banner() -> String {
    format!(
        "<div class=\"banner\">{} <div><strong>Paper book.</strong> No wallet, no order, no venue — \
         the firm places no trades at all (CONSTITUTION.md §5). Every figure below is what \
         we <em>would</em> hold if we had acted on our own signals, priced against real quotes.</div></div>",
        render::icon("alert")
    )
}

// ---------------------------------------------------------------------------
// Holdings
// ---------------------------------------------------------------------------

async fn holdings_page(env: &Env, sel: &Sel) -> String {
    let (docs, (paths, tree_live), snap) = futures::join!(
        data::read_all(
            env,
            [
                "portfolio/portfolio.toml",
                "portfolio/positions.csv",
                "portfolio/ledger.csv",
                "predictions/predictions.csv",
                "predictions/fills.csv",
                "predictions/resolutions.csv",
            ]
            .map(String::from)
        ),
        data::tree(env),
        snapshots::latest(env),
    );
    let [acct, pos, led, preds, fills, res]: [data::Doc; 6] = docs
        .try_into()
        .unwrap_or_else(|_| unreachable!("read_all returns one Doc per path"));
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let all_live = acct.live
        && pos.live
        && led.live
        && preds.live
        && fills.live
        && res.live
        && tree_live
        && metas_live;

    let account = data::toml_of(&acct.text);
    let starting_cash = fnum_or(&account, &["account", "starting_cash"], 0.0);
    let currency = data::str_at(&account, &["account", "currency"]);
    let opened = data::str_at(&account, &["account", "opened"]);

    let p = Table::parse(&pos.text);
    let l = Table::parse(&led.text);
    let pr = Table::parse(&preds.text);
    let fl = Table::parse(&fills.text);
    let rs = Table::parse(&res.text);

    let quotes = snap.as_ref().map(|s| books_from_snapshot(&s.doc)).unwrap_or_default();

    // Cash and collateral are DERIVED from the ledger; positions.csv is a
    // convenience index over the same history (portfolio/README.md). Both are
    // computed here so a disagreement is visible rather than assumed away.
    let ledger_cash_delta: f64 = l.rows.iter().map(|r| l.num(r, "cash_delta")).sum();
    let ledger_collateral: f64 = l.rows.iter().map(|r| l.num(r, "collateral_delta")).sum();
    let cash = starting_cash + ledger_cash_delta;
    let committed: f64 = p.rows.iter().map(|r| p.num(r, "collateral")).sum();
    let free = cash - committed;

    let bar = tabbar(sel, p.rows.len());
    let crumbs = trail(&[("Overview", ""), ("Paper book", "")]);

    if p.rows.is_empty() {
        let body = format!(
            "{banner}{empty}{ledger}",
            banner = paper_banner(),
            empty = empty_book(sel, starting_cash, cash, currency, opened, l.rows.len()),
            ledger = ledger_section(&l, starting_cash, cash),
        );
        return shell_sub(env, "/execution", crumbs, all_live, &bar, &body).await;
    }

    // --- mark every position, twice: at the midpoint and at the price we could
    // --- actually get out at. The gap between the two is the point.
    let mut pnl_mid = 0.0;
    let mut pnl_exit = 0.0;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut any_unpriced = false;
    for row in &p.rows {
        let token = p.cell(row, "token_id");
        let slug = p.cell(row, "market_slug");
        let outcome = p.cell(row, "outcome");
        let side = p.cell(row, "side");
        let shares = p.num(row, "shares");
        let entry = p.num(row, "entry_price");
        let collateral = p.num(row, "collateral");
        let days = days_held(p.cell(row, "opened_at"));

        // Live book first; the market price we logged with the last prediction
        // as a fallback; nothing at all if we have neither.
        let live = quotes.get(token);
        let logged_mid = latest_pred(&pr, slug, outcome).map(|r| pr.num(r, "market_price"));
        let mid = live.map(|q| q.mid).or(logged_mid);
        // Getting OUT of a long means hitting the bid; out of a short means
        // lifting the ask. Never the midpoint — nobody is offering it.
        let exit_px = live.map(|q| if side == "sell" { q.ask } else { q.bid });
        // `fills.csv` is the observed record of what somebody actually traded
        // after we spoke. It is the sanity check on the book quote.
        let observed = fills_bid(&fl, slug, outcome, side);

        if mid.is_none() {
            any_unpriced = true;
        }
        let m_pnl = mid.map(|m| pnl_at(side, entry, m, shares));
        let e_pnl = exit_px.map(|x| pnl_at(side, entry, x, shares));
        pnl_mid += m_pnl.unwrap_or(0.0);
        pnl_exit += e_pnl.or(m_pnl).unwrap_or(0.0);

        let (r, ann) = rolc(e_pnl.or(m_pnl).unwrap_or(0.0), collateral, days);

        // A position on a market that has already settled is not a holding, it
        // is an unclaimed redemption — and it will sit here silently until
        // somebody appends the close, so the row has to say so.
        let settled = rs
            .rows
            .iter()
            .find(|rr| rs.cell(rr, "market_slug") == slug)
            .map(|rr| rs.cell(rr, "winning_outcome"));

        rows.push(vec![
            format!(
                "<a href=\"/markets/{0}\">{0}</a><span class=\"sub\">{1}</span>{2}",
                esc(slug),
                esc(outcome),
                match settled {
                    Some(w) => badge(&format!("resolved {w} — settle it"), "warn"),
                    None => String::new(),
                }
            ),
            strategy_links(p.cell(row, "family"), p.cell(row, "variant"), &metas),
            side_badge(side),
            shares_fmt(shares),
            fmt_prob(entry),
            match mid {
                Some(m) => format!(
                    "{}<span class=\"sub\">{}</span>",
                    fmt_prob(m),
                    if live.is_some() { "live book" } else { "last logged" }
                ),
                None => "<span class=\"muted\">—</span>".to_string(),
            },
            match exit_px {
                Some(x) => format!(
                    "{}<span class=\"sub\">{}</span>",
                    fmt_prob(x),
                    if side == "sell" { "the ask" } else { "the bid" }
                ),
                None => match observed {
                    Some((v, label)) => format!(
                        "{}<span class=\"sub\">{}</span>",
                        fmt_prob(v),
                        esc(label)
                    ),
                    None => "<span class=\"muted\">—</span>".to_string(),
                },
            },
            match m_pnl {
                Some(v) => format!("<span class=\"{}\">{}</span>", tone_class(v), usd_signed(v)),
                None => "<span class=\"muted\">—</span>".to_string(),
            },
            match e_pnl {
                Some(v) => format!("<span class=\"{}\">{}</span>", tone_class(v), usd_signed(v)),
                None => "<span class=\"muted\">—</span>".to_string(),
            },
            usd(collateral),
            // Days held lives here rather than in its own column: it IS the
            // basis of the annualized figure, and a rate whose window is a
            // column away is a number without its basis.
            match (r, ann) {
                (Some(r), Some(a)) => format!(
                    "{}<span class=\"sub\">{}/yr over {}</span>",
                    render::fmt_pct(r, 1),
                    render::fmt_pct(a, 0),
                    render::count(days as usize, "day")
                ),
                (Some(r), None) => format!(
                    "{}<span class=\"sub\">held under a day — not annualized</span>",
                    render::fmt_pct(r, 1)
                ),
                _ => "<span class=\"muted\">—</span>".to_string(),
            },
        ]);
    }

    let mark_note = match &snap {
        Some(s) => format!(
            "marked against the order-book snapshot taken {} minutes ago",
            s.age_mins
        ),
        None => "no order-book snapshot was readable — marks fall back to the price logged with the last prediction".to_string(),
    };

    let kpis = stat_grid(&[
        stat(&usd(cash), "cash")
            .context(&format!("{starting} at open, {n} ledger rows since", starting = usd(starting_cash), n = l.rows.len())),
        stat(&usd(free), "free cash")
            .tone(if free < 0.0 { "bad" } else { "" })
            .context("cash minus the collateral every open position is holding down"),
        stat(&usd(committed), "committed")
            .context(&format!("{} of the book is deployed", render::fmt_pct(if cash > 0.0 { committed / cash } else { 0.0 }, 1))),
        stat(&render::fmt_int(p.rows.len() as i64), "open positions")
            .context(&format!("in {}", render::count(distinct_markets(&p), "market"))),
        stat(&usd_signed(pnl_mid), "unrealised at the midpoint")
            .tone(tone_of(pnl_mid))
            .context("an upper bound: a midpoint is not a price anyone offered"),
        stat(&usd_signed(pnl_exit), "unrealised at the exit price")
            .tone(tone_of(pnl_exit))
            .context("what closing everything into today's book would actually pay"),
    ]);

    let honesty = render::notes(&[
        format!(
            "<b>The midpoint column is not money.</b> A midpoint is the average of a bid and an ask, \
             and on a thin leg it is a number no counterparty ever offered. Our own first scored batch \
             beat the market on 21 of 21 predictions and had somebody reachable at the scored price on \
             <b>2 of 21</b> (<a href=\"/wiki/reference/midpoint-is-not-a-fill\">midpoint is not a fill</a>). \
             The exit-price column is the one to believe; the gap between the two is what the midpoint was flattering."
        ),
        format!(
            "<b>Exit price</b> is what getting out would cross to: a long has to hit the <b>bid</b>, \
             a short has to lift the <b>ask</b>. Marks: {}.",
            esc(&mark_note)
        ),
        format!(
            "Return on locked capital, not cents per trade (<span class=\"mono\">execution/DESIGN.md</span> §3). \
             A <b>buy</b> locks the price <span class=\"mono\">p</span> per share; a <b>sell</b> of YES at \
             <span class=\"mono\">p</span> is a purchase of NO at <span class=\"mono\">1 − p</span> and locks that, \
             for the whole holding period. So the denominator is the capital tied up, not the size of the ticket — \
             which is why a 10c gain on a 15c wing beats a 2c gain on a 97c favourite by six times, and cents \
             per trade says the opposite."
        ),
    ]);

    let positions = section_foot(
        "Open positions",
        "every one marked twice — at the quote, and at the price we could get",
        &badge(&render::count(p.rows.len(), "position"), ""),
        &format!(
            "{}{}",
            table(
                &[
                    ("Market", ""),
                    ("Strategy", ""),
                    ("Side", ""),
                    ("Shares", "num"),
                    ("Entry", "num"),
                    ("Mark (mid)", "num"),
                    ("Exit price", "num"),
                    ("P&L at mid", "num"),
                    ("P&L at exit", "num"),
                    ("Collateral", "num"),
                    ("Return on locked capital", "num"),
                ],
                &rows,
            ),
            honesty
        ),
        &format!(
            "<span class=\"mono\">portfolio/positions.csv</span><span>amounts in paper {}{}</span>",
            esc(currency),
            if any_unpriced { " · some positions could not be priced at all" } else { "" }
        ),
    );

    let body = format!(
        "{banner}{kpis}{recon}{positions}{ledger}{plan_link}",
        banner = paper_banner(),
        kpis = kpis,
        recon = reconciliation(ledger_collateral, committed, &l, &p),
        positions = positions,
        ledger = ledger_section(&l, starting_cash, cash),
        plan_link = section(
            "Change the book",
            "",
            "",
            &format!(
                "<p class=\"finding\">The book only moves when the CEO appends to the ledger. \
                 To see what a policy would do about today's signals — and what it would cost — \
                 open the <a href=\"{}\">Plan tab</a>.</p>",
                esc(&href(sel, "plan"))
            ),
        ),
    );

    shell_sub(env, "/execution", crumbs, all_live, &bar, &body).await
}

fn tone_class(v: f64) -> &'static str {
    match tone_of(v) {
        "ok" => "s-ok",
        "bad" => "s-bad",
        _ => "muted",
    }
}

fn distinct_markets(p: &Table) -> usize {
    let mut m: Vec<&str> = p.rows.iter().map(|r| p.cell(r, "market_slug")).collect();
    m.sort_unstable();
    m.dedup();
    m.len()
}

/// Just the word. What each side locks is explained once, in the notes under
/// the table — repeating "short — locks 1 − p" on every row costs exactly the
/// width the return-on-locked-capital column needs, and says nothing new after
/// the first read.
fn side_badge(side: &str) -> String {
    badge(side, "")
}

/// The most recent prediction row for one market outcome.
fn latest_pred<'a>(pr: &'a Table, slug: &str, outcome: &str) -> Option<&'a Vec<String>> {
    pr.rows
        .iter()
        .filter(|r| pr.cell(r, "market_slug") == slug && pr.cell(r, "outcome") == outcome)
        .max_by_key(|r| pr.cell(r, "timestamp"))
}

/// An observed price from `predictions/fills.csv` — what somebody demonstrably
/// traded after we spoke, on the side we would have to take to get out. Prefers
/// the first hour (when we would really have traded) over the market's whole
/// life (the most generous possible reading).
fn fills_bid(fl: &Table, slug: &str, outcome: &str, side: &str) -> Option<(f64, &'static str)> {
    let row = fl
        .rows
        .iter()
        .filter(|r| fl.cell(r, "market_slug") == slug && fl.cell(r, "outcome") == outcome)
        .max_by_key(|r| fl.cell(r, "timestamp"))?;
    // Closing a long means finding a buyer (a bid); closing a short means
    // finding a seller (an ask).
    let cols: [(&str, &'static str); 2] = if side == "sell" {
        [("ask_1h", "observed ask, first hour"), ("ask_life", "observed ask, ever")]
    } else {
        [("bid_1h", "observed bid, first hour"), ("bid_life", "observed bid, ever")]
    };
    for (col, label) in cols {
        if let Some(v) = fl.numo(row, col) {
            if v > 0.0 {
                return Some((v, label));
            }
        }
    }
    None
}

fn strategy_links(family: &str, variant: &str, metas: &[data::VariantMeta]) -> String {
    if family.is_empty() {
        return "<span class=\"muted\">—</span>".to_string();
    }
    if variant.is_empty() {
        return format!("<a href=\"/strategies/{0}\">{0}</a>", esc(family));
    }
    let title = metas
        .iter()
        .find(|m| m.family == family && m.variant == variant)
        .map(|m| format!(" title=\"{}\"", esc(&m.summary)))
        .unwrap_or_default();
    format!(
        "<a href=\"/strategies/{0}\">{0}</a><span class=\"muted\">/</span><a href=\"/strategies/{0}/{1}\"{2}>{1}</a>",
        esc(family),
        esc(variant),
        title
    )
}

/// The empty book, treated as a real view rather than a gap. It is today's
/// actual state and will be for a while, so it is the screen most likely to be
/// seen: it has to say what this is, that it is paper, that nothing is open,
/// and what the next move is.
fn empty_book(
    sel: &Sel,
    starting_cash: f64,
    cash: f64,
    currency: &str,
    opened: &str,
    ledger_rows: usize,
) -> String {
    let body = format!(
        "<p class=\"lede\">It holds {cash} paper {cur}, and every unit of that is free.</p>\
         {stats}\
         <p class=\"finding\">This is the firm's pretend trading account: it records what we \
         <b>would</b> be holding if we acted on our own signals, so that \"we were right\" and \
         \"we would have made money\" stop being the same sentence. It has never held a position — \
         the ledger has {rows}, and positions and cash are derived from it.</p>\
         <p class=\"finding\">Positions appear here only after the CEO appends rows to \
         <span class=\"mono\">portfolio/ledger.csv</span>. To see what a policy would open against \
         today's signals, what it would cost and which candidates it would refuse, \
         <a href=\"{plan}\">build a plan</a>.</p>",
        cash = usd(cash),
        cur = esc(currency),
        rows = render::count(ledger_rows, "row"),
        plan = esc(&href(sel, "plan")),
        stats = stat_grid(&[
            stat(&usd(cash), "cash").context(&format!("opened {} at {}", esc(opened), usd(starting_cash))),
            stat(&usd(cash), "free cash").context("nothing is committed, so free cash is all of it"),
            stat("0", "open positions").context("the book has never held one"),
            stat("—", "unrealised P&L").context("no position, nothing to mark"),
        ]),
    );
    section_foot(
        "The book is empty",
        "",
        // No "paper" chip here: the banner directly above already says it, and
        // saying it twice in 40px is the decoration PRINCIPLES.md forbids.
        "",
        &body,
        "<span class=\"mono\">portfolio/portfolio.toml · positions.csv · ledger.csv</span><a href=\"/wiki/reference/midpoint-is-not-a-fill\">Why marks are an upper bound →</a>",
    )
}

/// `positions.csv` and the cash line are derived from the ledger and must be
/// reproducible from it alone. When they disagree the ledger wins — and the
/// page has to say so rather than quietly show one of the two.
fn reconciliation(ledger_collateral: f64, positions_collateral: f64, l: &Table, p: &Table) -> String {
    let gap = ledger_collateral - positions_collateral;
    if gap.abs() <= 0.005 {
        return String::new();
    }
    format!(
        "<div class=\"banner banner-bad\">{icon} <div><strong>The ledger and the positions file disagree.</strong> \
         The ledger's collateral movements sum to {a} across {lr}, but the {pr} open sum to {b} — a gap of {gap}. \
         <span class=\"mono\">portfolio/ledger.csv</span> is the authority (portfolio/README.md): it is append-only \
         and everything else is derived from it, so <b>believe the ledger and treat the positions table below as \
         out of date</b>.</div></div>",
        icon = render::icon("alert"),
        a = usd(ledger_collateral),
        b = usd(positions_collateral),
        gap = usd_signed(gap),
        lr = render::count(l.rows.len(), "ledger row"),
        pr = render::count(p.rows.len(), "position"),
    )
}

fn ledger_section(l: &Table, starting_cash: f64, cash: f64) -> String {
    if l.rows.is_empty() {
        return section_foot(
            "Ledger",
            "the append-only history everything else is derived from",
            "",
            &render::empty_state(
                "Nothing has been applied",
                &format!(
                    "<div>The file holds only its header, so cash is still the opening balance of {}. \
                     Every position, fee and settlement the firm ever books lands here first; \
                     <span class=\"mono\">positions.csv</span> and the cash figure are read back out of it.</div>",
                    usd(starting_cash)
                ),
            ),
            "<span class=\"mono\">portfolio/ledger.csv</span>",
        );
    }
    let mut running = starting_cash;
    let mut rows: Vec<Vec<String>> = Vec::new();
    for r in &l.rows {
        running += l.num(r, "cash_delta");
        rows.push(vec![
            format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(l.cell(r, "applied_at")))),
            badge(l.cell(r, "action"), ""),
            format!(
                "<a href=\"/markets/{0}\">{0}</a><span class=\"sub\">{1}</span>",
                esc(l.cell(r, "market_slug")),
                esc(l.cell(r, "outcome"))
            ),
            esc(l.cell(r, "side")),
            shares_fmt(l.num(r, "shares")),
            fmt_prob(l.num(r, "price")),
            usd(l.num(r, "fee")),
            usd_signed(l.num(r, "cash_delta")),
            usd_signed(l.num(r, "collateral_delta")),
            usd(running),
            format!("<span class=\"mono\">{}</span>", esc(l.cell(r, "policy"))),
        ]);
    }
    rows.reverse();
    section_foot(
        "Ledger",
        "every applied action, newest first — the authority for cash and positions",
        &badge(&render::count(l.rows.len(), "row"), ""),
        &table(
            &[
                ("Applied", ""),
                ("Action", ""),
                ("Market", ""),
                ("Side", ""),
                ("Shares", "num"),
                ("Price", "num"),
                ("Fee", "num"),
                ("Cash", "num"),
                ("Collateral", "num"),
                ("Cash after", "num"),
                ("Policy", ""),
            ],
            &rows,
        ),
        &format!(
            "<span class=\"mono\">portfolio/ledger.csv</span><span>cash = {} at open {} in deltas = {}</span>",
            usd(starting_cash),
            usd_signed(cash - starting_cash),
            usd(cash)
        ),
    )
}

// ---------------------------------------------------------------------------
// Plan
// ---------------------------------------------------------------------------

/// One candidate signal, carried all the way through the gates so the page can
/// explain a rejection with the number that caused it. `gate` empty ⇒ it passed
/// everything and `stake`/`shares`/`fill` are meaningful.
struct Cand {
    slug: String,
    condition_id: String,
    outcome: String,
    token_id: String,
    family: String,
    variant: String,
    signal_ts: String,
    /// Our probability.
    p: f64,
    bid: f64,
    ask: f64,
    mid: f64,
    depth: Option<f64>,
    synthetic: bool,
    /// "buy" | "sell" | "" when the probability sits inside the spread.
    side: &'static str,
    edge: f64,
    gate: &'static str,
    detail: String,
    stake: f64,
    shares: f64,
    fill: f64,
    fee: f64,
    fee_unmapped: bool,
}

/// One line of the diff. `kind` is the terraform-shaped verb.
struct Action {
    kind: &'static str,
    slug: String,
    condition_id: String,
    outcome: String,
    token_id: String,
    family: String,
    variant: String,
    side: String,
    shares: f64,
    price: f64,
    collateral: f64,
    fee: f64,
    cash_delta: f64,
    why: String,
    note: String,
}

fn symbol(kind: &str) -> &'static str {
    match kind {
        "open" => "+",
        "increase" | "reduce" => "~",
        "close" => "−",
        _ => "=",
    }
}

fn symbol_tone(kind: &str) -> &'static str {
    match kind {
        "open" => "s-ok",
        "increase" | "reduce" => "s-warn",
        "close" => "s-bad",
        _ => "muted",
    }
}

async fn plan_page(env: &Env, sel: &Sel) -> String {
    let (docs, (paths, tree_live), snap) = futures::join!(
        data::read_all(
            env,
            [
                "portfolio/portfolio.toml",
                "portfolio/positions.csv",
                "predictions/predictions.csv",
                "predictions/resolutions.csv",
                "portfolio/ledger.csv",
            ]
            .map(String::from)
        ),
        data::tree(env),
        snapshots::latest(env),
    );
    let [acct, pos, preds, res, led]: [data::Doc; 5] = docs
        .try_into()
        .unwrap_or_else(|_| unreachable!("read_all returns one Doc per path"));

    let policy_paths = data::files_in(&paths, "execution/policies", ".toml");
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let policy_docs = data::read_all(env, policy_paths.iter().cloned()).await;
    let all_live = acct.live
        && pos.live
        && preds.live
        && res.live
        && led.live
        && tree_live
        && metas_live
        && policy_docs.iter().all(|d| d.live);

    let policies: Vec<Policy> = policy_paths
        .iter()
        .zip(&policy_docs)
        .map(|(path, doc)| {
            let file = path
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_end_matches(".toml")
                .to_string();
            parse_policy(&file, &doc.text)
        })
        .collect();

    let account = data::toml_of(&acct.text);
    let starting_cash = fnum_or(&account, &["account", "starting_cash"], 0.0);
    let bound_policy = data::str_at(&account, &["account", "policy"]);

    let p = Table::parse(&pos.text);
    let pr = Table::parse(&preds.text);
    let rs = Table::parse(&res.text);
    let l = Table::parse(&led.text);

    let bar = tabbar(sel, p.rows.len());
    let crumbs = trail(&[("Overview", ""), ("Paper book", ""), ("Plan", "")]);

    // --- resolve the selection against the data ----------------------------
    let default_strategies: Vec<String> = metas
        .iter()
        .filter(|m| m.status == "trial" || m.status == "live")
        .map(|m| m.key())
        .collect();
    let chosen_strategies: Vec<String> = if sel.strategies.is_empty() {
        default_strategies.clone()
    } else {
        sel.strategies.clone()
    };

    let policy_name = if !sel.policy.is_empty() {
        sel.policy.clone()
    } else if !bound_policy.is_empty() {
        bound_policy.to_string()
    } else {
        DEFAULT_POLICY.to_string()
    };
    let policy = policies.iter().find(|x| x.file == policy_name);

    // Live signals, grouped by token, newest first. Grouped rather than
    // flattened to "the latest row" because a delayed policy needs the latest
    // row that is ALSO old enough — `patient` enters on a probability frozen
    // 24h ago, and collapsing to the newest row first would make it
    // structurally unable to ever trade, which is a false result, not a gate.
    // A resolved market is not a thing to open a position on.
    let resolved: Vec<&str> = rs.rows.iter().map(|r| rs.cell(r, "market_slug")).collect();
    let mut ordered: Vec<&Vec<String>> = pr
        .rows
        .iter()
        .filter(|r| {
            let key = format!("{}/{}", pr.cell(r, "family"), pr.cell(r, "variant"));
            chosen_strategies.iter().any(|s| *s == key)
                && !resolved.contains(&pr.cell(r, "market_slug"))
        })
        .collect();
    ordered.sort_by(|a, b| pr.cell(b, "timestamp").cmp(pr.cell(a, "timestamp")));
    let mut signals: Vec<Vec<&Vec<String>>> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for r in ordered {
        let token = pr.cell(r, "token_id");
        let key = if token.is_empty() { pr.cell(r, "market_slug") } else { token };
        match seen.iter().position(|k| *k == key) {
            Some(i) => signals[i].push(r),
            None => {
                seen.push(key);
                signals.push(vec![r]);
            }
        }
    }
    signals.sort_by(|a, b| pr.cell(a[0], "market_slug").cmp(pr.cell(b[0], "market_slug")));

    // Selectable markets: everything the chosen strategies have a live signal
    // on, PLUS every market the book already holds a position in. The second
    // half is not optional — a market we are in is in scope by definition, and
    // leaving it out meant a position whose market had RESOLVED silently fell
    // outside every plan and was never proposed for settlement.
    let mut available_markets: Vec<String> =
        signals.iter().map(|g| pr.cell(g[0], "market_slug").to_string()).collect();
    available_markets.extend(p.rows.iter().map(|r| p.cell(r, "market_slug").to_string()));
    available_markets.sort();
    available_markets.dedup();

    let chosen_markets: Vec<String> = if sel.markets.is_empty() {
        available_markets.clone()
    } else {
        sel.markets.clone()
    };

    let picker = selection_form(
        sel,
        &metas,
        &default_strategies,
        &chosen_strategies,
        &available_markets,
        &chosen_markets,
        &policies,
        &policy_name,
    );

    let Some(policy) = policy else {
        let body = format!(
            "{banner}{picker}{err}",
            banner = paper_banner(),
            picker = picker,
            err = section(
                "No such policy",
                "",
                "",
                &render::empty_state(
                    "That policy does not exist",
                    &format!(
                        "<div>Nothing in <span class=\"mono\">execution/policies/</span> is called \
                         <span class=\"mono\">{}</span>. Pick one above.</div>",
                        esc(&policy_name)
                    ),
                ),
            ),
        );
        return shell_sub(env, "/execution", crumbs, all_live, &bar, &body).await;
    };

    // --- current book -------------------------------------------------------
    let quotes = snap.as_ref().map(|s| books_from_snapshot(&s.doc)).unwrap_or_default();
    let ledger_cash: f64 = l.rows.iter().map(|r| l.num(r, "cash_delta")).sum();
    let cash = starting_cash + ledger_cash;
    let committed: f64 = p.rows.iter().map(|r| p.num(r, "collateral")).sum();
    let free = cash - committed;

    // --- run the gates ------------------------------------------------------
    let cands = candidates(&pr, &signals, &chosen_markets, &quotes, policy, &p);
    let actions = diff(&cands, &p, &pr, &quotes, policy, &rs, &chosen_markets);

    let plan_id = plan_id(&policy_name, &chosen_strategies, &chosen_markets, &actions);

    let body = format!(
        "{banner}{picker}{summary}{v1}{funding}{table}{gates}{apply}",
        banner = paper_banner(),
        picker = picker,
        summary = plan_summary(policy, &actions, &cands, &snap, &plan_id),
        v1 = v1_warning(policy),
        funding = funding(&actions, cash, committed, free),
        table = actions_table(&actions, &metas),
        gates = gate_report(&cands, policy),
        apply = apply_section(&actions, policy, &plan_id),
    );

    shell_sub(env, "/execution", crumbs, all_live, &bar, &body).await
}

/// Every chosen signal put through the policy's gates, in the same order as
/// `execution/engine/src/sim.rs::simulate`. Nothing is dropped: a candidate
/// that fails carries the gate that stopped it and the number that did it.
fn candidates(
    pr: &Table,
    signals: &[Vec<&Vec<String>>],
    chosen_markets: &[String],
    quotes: &HashMap<String, Quote>,
    policy: &Policy,
    positions: &Table,
) -> Vec<Cand> {
    // Which row of each token's history this policy acts on. Normally the
    // newest; for a delayed policy the newest that is already `delay_hours`
    // old, because that is the probability the policy would be entering on.
    // (The engine's `delay_tolerance_hours` has no live analogue: it bounds how
    // stale the *entry observation* may be, and here the entry observation is
    // the current book.)
    let pick = |group: &[&Vec<String>]| -> Option<usize> {
        if policy.delay_hours <= 0.0 {
            return Some(0);
        }
        group
            .iter()
            .position(|r| age_hours(pr.cell(r, "timestamp")) >= policy.delay_hours)
    };

    // The percentile gate is relative to the whole selected set, so it needs
    // every claimed edge before any single candidate can be judged.
    let claimed: Vec<f64> = signals
        .iter()
        .filter(|g| chosen_markets.iter().any(|m| m == pr.cell(g[0], "market_slug")))
        .filter_map(|g| pick(g).map(|i| g[i]))
        .map(|r| (pr.num(r, "prediction") - pr.num(r, "market_price")).abs())
        .collect();
    let edge_floor = policy.edge_percentile.map(|q| quantile(&claimed, q));

    let mut out: Vec<Cand> = Vec::new();
    for group in signals {
        let slug = pr.cell(group[0], "market_slug").to_string();
        if !chosen_markets.iter().any(|m| *m == slug) {
            continue;
        }
        let due = pick(group);
        let r = group[due.unwrap_or(0)];
        let token = pr.cell(r, "token_id").to_string();
        let p_model = pr.num(r, "prediction");
        let logged_mid = pr.num(r, "market_price");
        let asset = data::asset_of(&slug);

        let live = quotes.get(&token);
        let quote = match live {
            Some(q) => Some(Quote {
                bid: q.bid,
                ask: q.ask,
                mid: q.mid,
                depth_bid: q.depth_bid,
                depth_ask: q.depth_ask,
                synthetic: false,
            }),
            None => synthetic_quote(logged_mid, policy.assumed_spread),
        };

        let mut c = Cand {
            slug: slug.clone(),
            condition_id: pr.cell(r, "condition_id").to_string(),
            outcome: pr.cell(r, "outcome").to_string(),
            token_id: token.clone(),
            family: pr.cell(r, "family").to_string(),
            variant: pr.cell(r, "variant").to_string(),
            signal_ts: pr.cell(r, "timestamp").to_string(),
            p: p_model,
            bid: 0.0,
            ask: 0.0,
            mid: logged_mid,
            depth: None,
            synthetic: true,
            side: "",
            edge: 0.0,
            gate: "",
            detail: String::new(),
            stake: 0.0,
            shares: 0.0,
            fill: 0.0,
            fee: 0.0,
            fee_unmapped: false,
        };

        // Entry delay: `patient` deliberately acts on a probability frozen
        // `delay_hours` ago, because the wing-drift finding says sell fills
        // improve with delay. If every prediction on this token is younger than
        // that, the policy is not refusing the trade — it is not due yet, which
        // is a different thing and reads differently.
        if due.is_none() {
            c.gate = "waiting out the entry delay";
            c.detail = format!(
                "this policy enters on a probability at least {:.0}h old, and the oldest on this market is {:.1}h",
                policy.delay_hours,
                group.iter().map(|r| age_hours(pr.cell(r, "timestamp"))).fold(0.0, f64::max)
            );
            out.push(c);
            continue;
        }

        let Some(q) = quote else {
            c.gate = "no quote";
            c.detail = "there is no two-sided book, and no midpoint to build one from".to_string();
            out.push(c);
            continue;
        };
        c.bid = q.bid;
        c.ask = q.ask;
        c.mid = q.mid;
        c.synthetic = q.synthetic;

        // Executable side. Buys lift the ask, sells hit the bid; a probability
        // inside the spread is not tradeable at any size.
        if p_model > q.ask {
            c.side = "buy";
            c.edge = p_model - q.ask;
        } else if p_model < q.bid {
            c.side = "sell";
            c.edge = q.bid - p_model;
        } else {
            c.gate = "no executable edge";
            c.detail = format!(
                "our {:.4} sits inside the {:.4} / {:.4} spread — there is no side to take",
                p_model, q.bid, q.ask
            );
            out.push(c);
            continue;
        }

        if !policy.sides.iter().any(|s| s == c.side) {
            c.gate = "side excluded";
            c.detail = format!(
                "the edge is on the {} side and this policy trades {} only",
                c.side,
                policy.sides.join(" and ")
            );
            out.push(c);
            continue;
        }
        if c.edge < policy.min_edge {
            c.gate = "below the minimum edge";
            c.detail = format!(
                "{:.4} of executable edge against a floor of {:.4}",
                c.edge, policy.min_edge
            );
            out.push(c);
            continue;
        }
        // The book already holds the other side of this token. Opening now
        // would propose a self-cancelling pair — economically it is closing the
        // old position, but it is not what the diff would say, and a plan whose
        // rows contradict each other is a plan you cannot apply. The exit rule
        // owns that decision, so this refuses rather than guesses.
        let opposite = if c.side == "sell" { "buy" } else { "sell" };
        let held_other: f64 = positions
            .rows
            .iter()
            .filter(|row| {
                positions.cell(row, "token_id") == c.token_id
                    && positions.cell(row, "side") == opposite
            })
            .map(|row| positions.num(row, "shares"))
            .sum();
        if held_other > EPS {
            c.gate = "the book holds the other side";
            c.detail = format!(
                "the edge is on the {} side and the book is already {} {:.2} shares of this token — \
                 taking both is a self-cancelling pair, and unwinding the old one is the exit rule's call, not the entry rule's",
                c.side,
                if opposite == "buy" { "long" } else { "short" },
                held_other
            );
            out.push(c);
            continue;
        }
        if let Some(floor) = edge_floor {
            if (p_model - logged_mid).abs() < floor {
                c.gate = "below the edge percentile";
                c.detail = format!(
                    "claimed edge {:.4}, and this policy takes only the top {:.0}% of the set (floor {:.4})",
                    (p_model - logged_mid).abs(),
                    100.0 * (1.0 - policy.edge_percentile.unwrap_or(0.0)),
                    floor
                );
                out.push(c);
                continue;
            }
        }
        if q.ask - q.bid > policy.min_spread_ok {
            c.gate = "spread too wide";
            c.detail = format!(
                "the book is {:.4} wide and this policy will not cross more than {:.4}",
                q.ask - q.bid,
                policy.min_spread_ok
            );
            out.push(c);
            continue;
        }

        let depth = if c.side == "sell" { q.depth_bid } else { q.depth_ask };
        c.depth = depth;
        if let Some(d) = depth {
            if d < policy.min_depth_usd {
                c.gate = "depth too thin";
                c.detail = format!(
                    "{} of depth within 5c of the touch, against a floor of {}",
                    usd(d),
                    usd(policy.min_depth_usd)
                );
                out.push(c);
                continue;
            }
            if policy.max_book_fraction * d < policy.min_stake {
                c.gate = "unfundable";
                c.detail = format!(
                    "this policy will take at most {} of the resting {}, which is under the {} minimum ticket",
                    usd(policy.max_book_fraction * d),
                    usd(d),
                    usd(policy.min_stake)
                );
                out.push(c);
                continue;
            }
        }

        // Per-market exposure cap, counting what the book already holds.
        let market_key = if c.condition_id.is_empty() { &c.slug } else { &c.condition_id };
        let exposure: f64 = positions
            .rows
            .iter()
            .filter(|row| {
                let cid = positions.cell(row, "condition_id");
                let k = if cid.is_empty() { positions.cell(row, "market_slug") } else { cid };
                k == market_key
            })
            .map(|row| positions.num(row, "collateral"))
            .sum();
        let room = policy.max_per_market_usd - exposure;
        if room < policy.min_stake {
            c.gate = "market cap full";
            c.detail = format!(
                "{} is already committed to this market and the cap is {}, leaving {}",
                usd(exposure),
                usd(policy.max_per_market_usd),
                usd(room.max(0.0))
            );
            out.push(c);
            continue;
        }

        let touch = if c.side == "sell" { q.bid } else { q.ask };
        let mut stake = if policy.sizing_method == "flat" {
            policy.stake_usd.unwrap_or(0.0)
        } else {
            kelly_fraction(c.side, p_model, touch) * policy.kelly.unwrap_or(0.0) * policy.bankroll
        };
        if let Some(maxf) = policy.max_bankroll_fraction {
            stake = stake.min(maxf * policy.bankroll);
        }
        stake = stake.min(room);
        if let Some(d) = depth {
            stake = stake.min(policy.max_book_fraction * d);
        }
        if !(stake >= policy.min_stake) {
            c.gate = "stake too small";
            c.detail = format!(
                "sizing lands on {}, under the {} minimum ticket",
                usd(stake.max(0.0)),
                usd(policy.min_stake)
            );
            out.push(c);
            continue;
        }

        let fill = apply_slippage(c.side, touch, stake, depth);
        let per_share = capital_per_share(c.side, fill);
        if per_share <= 0.0 {
            c.gate = "no quote";
            c.detail = "the fill price leaves no capital to lock — the book cannot be traded"
                .to_string();
            out.push(c);
            continue;
        }
        let (rate, mapped) = policy.rate_for(&asset);
        c.stake = stake;
        c.fill = fill;
        c.shares = stake / per_share;
        c.fee = taker_fee(c.shares, rate, fill);
        c.fee_unmapped = !mapped;
        out.push(c);
    }
    out
}

fn age_hours(ts: &str) -> f64 {
    match data::ts_ms(ts) {
        Some(t) => ((now_ms() - t) as f64 / 3_600_000.0).max(0.0),
        None => f64::INFINITY,
    }
}

fn quantile(xs: &[f64], q: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pos = q.clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    if lo == hi {
        v[lo]
    } else {
        v[lo] + (pos - lo as f64) * (v[hi] - v[lo])
    }
}

/// Target minus current. Only positions inside the selection are touched: a
/// plan that quietly closed a position you did not select would be a plan you
/// cannot trust, so anything outside is reported as unmanaged instead.
fn diff(
    cands: &[Cand],
    positions: &Table,
    pr: &Table,
    quotes: &HashMap<String, Quote>,
    policy: &Policy,
    rs: &Table,
    chosen_markets: &[String],
) -> Vec<Action> {
    let mut out: Vec<Action> = Vec::new();

    // --- what the gates want to put on ------------------------------------
    for c in cands {
        if !c.gate.is_empty() {
            continue;
        }
        let held: f64 = positions
            .rows
            .iter()
            .filter(|r| {
                positions.cell(r, "token_id") == c.token_id && positions.cell(r, "side") == c.side
            })
            .map(|r| positions.num(r, "shares"))
            .sum();
        out.push(Action {
            kind: if held > EPS { "increase" } else { "open" },
            slug: c.slug.clone(),
            condition_id: c.condition_id.clone(),
            outcome: c.outcome.clone(),
            token_id: c.token_id.clone(),
            family: c.family.clone(),
            variant: c.variant.clone(),
            side: c.side.to_string(),
            shares: c.shares,
            price: c.fill,
            collateral: c.stake,
            fee: c.fee,
            // Opening moves no cash out of the account: the money becomes
            // collateral, which is tracked separately. Only the venue's fee
            // actually leaves (portfolio/README.md: free = cash − Σ collateral).
            cash_delta: -c.fee,
            // How old the signal is belongs on the row: a plan acting on a
            // three-day-old probability against today's book is a different
            // proposition from one acting on this morning's.
            why: format!(
                "we say {ours} (logged {age:.0}h ago), the book is {bid} / {ask}; {side}ing at {fill} is {edge} of executable edge, over this policy's {floor} floor",
                ours = fmt_prob(c.p),
                age = age_hours(&c.signal_ts),
                bid = fmt_prob(c.bid),
                ask = fmt_prob(c.ask),
                side = c.side,
                fill = fmt_prob(c.fill),
                edge = fmt_prob(c.edge),
                floor = fmt_prob(policy.min_edge),
            ),
            note: format!(
                "signal {}{}",
                c.signal_ts,
                if c.synthetic { "; synthetic book" } else { "" }
            ),
        });
    }

    // --- what the book already holds ---------------------------------------
    for r in &positions.rows {
        let slug = positions.cell(r, "market_slug").to_string();
        let token = positions.cell(r, "token_id").to_string();
        let side = positions.cell(r, "side").to_string();
        let outcome = positions.cell(r, "outcome").to_string();
        let family = positions.cell(r, "family").to_string();
        let variant = positions.cell(r, "variant").to_string();
        let shares = positions.num(r, "shares");
        let entry = positions.num(r, "entry_price");
        let collateral = positions.num(r, "collateral");
        let in_scope = chosen_markets.iter().any(|m| *m == slug);
        if !in_scope {
            out.push(Action {
                kind: "unmanaged",
                slug,
                condition_id: positions.cell(r, "condition_id").to_string(),
                outcome,
                token_id: token,
                family: family.clone(),
                variant: variant.clone(),
                side,
                shares,
                price: entry,
                collateral,
                fee: 0.0,
                cash_delta: 0.0,
                why: "this market is not in the plan's selection, so the plan leaves it alone"
                    .to_string(),
                note: String::new(),
            });
            continue;
        }

        // A resolved market is a redemption, not a trade: it settles at 1 or 0
        // and the venue charges nothing for it.
        if let Some(rr) = rs.rows.iter().find(|rr| rs.cell(rr, "market_slug") == slug) {
            let winner = rs.cell(rr, "winning_outcome");
            let token_won = winner == outcome;
            let settle = if token_won { 1.0 } else { 0.0 };
            let pnl = pnl_at(&side, entry, settle, shares);
            out.push(Action {
                kind: "close",
                slug,
                condition_id: positions.cell(r, "condition_id").to_string(),
                outcome,
                token_id: token,
                family: family.clone(),
                variant: variant.clone(),
                side,
                shares,
                price: settle,
                collateral: -collateral,
                fee: 0.0,
                cash_delta: pnl,
                why: format!(
                    "the market resolved {} — the position settles at {settle:.0} and the collateral comes back",
                    esc(winner)
                ),
                note: "redemption, no fee".to_string(),
            });
            continue;
        }

        let token_for_why = token.clone();
        let quote = quotes.get(&token);
        // Getting out means crossing the spread the other way.
        let exit_px = quote.map(|q| if side == "sell" { q.ask } else { q.bid });
        let exit_depth = quote.and_then(|q| if side == "sell" { q.depth_ask } else { q.depth_bid });

        // Take-profit: the policy exits once the price has closed
        // `close_fraction` of the way from our fill to our probability.
        if policy.exit_rule == "take-profit" {
            let cf = policy.close_fraction.unwrap_or(0.0);
            let p_model = latest_pred(pr, &positions.cell(r, "market_slug").to_string(), &outcome)
                .map(|row| pr.num(row, "prediction"));
            if let (Some(px), Some(p_model)) = (exit_px, p_model) {
                let target = if side == "sell" {
                    entry - cf * (entry - p_model)
                } else {
                    entry + cf * (p_model - entry)
                };
                // Closing a short means buying it back (lift the ask); closing a
                // long means selling it (hit the bid). Slippage applies again.
                let exit_side = if side == "sell" { "buy" } else { "sell" };
                let px = apply_slippage(exit_side, px, collateral, exit_depth);
                let hit = if side == "sell" { px <= target } else { px >= target };
                if hit {
                    let (rate, _) = policy.rate_for(&data::asset_of(&slug));
                    let fee = taker_fee(shares, rate, px);
                    let pnl = pnl_at(&side, entry, px, shares);
                    out.push(Action {
                        kind: "close",
                        slug,
                        condition_id: positions.cell(r, "condition_id").to_string(),
                        outcome,
                        token_id: token,
                        family: family.clone(),
                        variant: variant.clone(),
                        side,
                        shares,
                        price: px,
                        collateral: -collateral,
                        fee,
                        cash_delta: pnl - fee,
                        // Stated mechanically, not as "60% of the edge has
                        // closed": when the signal has moved AGAINST an open
                        // position the take-profit level sits on the wrong side
                        // of the fill and the rule still fires, which is the
                        // engine's behaviour and must not be narrated as profit.
                        why: format!(
                            "this policy exits at {:.0}% of the way from our fill ({}) to our probability ({}), which is {:.4}; the market is at {}",
                            cf * 100.0,
                            fmt_prob(entry),
                            fmt_prob(p_model),
                            target,
                            fmt_prob(px)
                        ),
                        note: "exit in the market, second taker fee".to_string(),
                    });
                    continue;
                }
            }
        }

        // Config drift: a policy with a tighter per-market cap than the one the
        // position was opened under wants the excess given back.
        if collateral > policy.max_per_market_usd + EPS {
            let excess = collateral - policy.max_per_market_usd;
            let per_share = capital_per_share(&side, entry);
            let cut = if per_share > 0.0 { excess / per_share } else { 0.0 };
            // Selling part of it back crosses the spread and walks the book,
            // exactly as a full exit would.
            let exit_side = if side == "sell" { "buy" } else { "sell" };
            let px = apply_slippage(exit_side, exit_px.unwrap_or(entry), excess, exit_depth);
            let (rate, _) = policy.rate_for(&data::asset_of(&slug));
            let fee = taker_fee(cut, rate, px);
            let pnl = pnl_at(&side, entry, px, cut);
            out.push(Action {
                kind: "reduce",
                slug,
                condition_id: positions.cell(r, "condition_id").to_string(),
                outcome,
                token_id: token,
                family: family.clone(),
                variant: variant.clone(),
                side,
                shares: cut,
                price: px,
                collateral: -excess,
                fee,
                cash_delta: pnl - fee,
                why: format!(
                    "{} is committed to this market and this policy caps it at {}",
                    usd(collateral),
                    usd(policy.max_per_market_usd)
                ),
                note: String::new(),
            });
            continue;
        }

        out.push(Action {
            kind: "no change",
            slug,
            condition_id: positions.cell(r, "condition_id").to_string(),
            outcome,
            token_id: token,
            family,
            variant,
            side,
            shares,
            price: entry,
            collateral: 0.0,
            fee: 0.0,
            cash_delta: 0.0,
            why: match cands.iter().find(|c| {
                c.token_id == token_for_why && c.gate == "the book holds the other side"
            }) {
                // Worth saying on this row too: the signal has turned against
                // the position and the exit rule is the only thing holding it.
                Some(c) => format!(
                    "the current signal wants the {} side of this token, but this policy {} — so it stays",
                    c.side,
                    match policy.exit_rule.as_str() {
                        "take-profit" => "only exits on a take-profit",
                        _ => "holds to resolution",
                    }
                ),
                None => match policy.exit_rule.as_str() {
                    "take-profit" => "the edge has not closed far enough to take profit yet".to_string(),
                    _ => "this policy holds to resolution, and the market has not resolved".to_string(),
                },
            },
            note: String::new(),
        });
    }

    out.sort_by(|a, b| {
        rank(a.kind)
            .cmp(&rank(b.kind))
            .then_with(|| a.slug.cmp(&b.slug))
            .then_with(|| a.outcome.cmp(&b.outcome))
    });
    out
}

fn rank(kind: &str) -> u8 {
    match kind {
        "open" => 0,
        "increase" => 1,
        "reduce" => 2,
        "close" => 3,
        "no change" => 4,
        _ => 5,
    }
}

fn count_kind(actions: &[Action], kind: &str) -> usize {
    actions.iter().filter(|a| a.kind == kind).count()
}

/// A short stable id for exactly this plan — selection, policy and the prices
/// it was computed against. It goes in the `plan_id` column of the ledger rows,
/// so an applied trade can be traced back to the plan that proposed it. FNV-1a;
/// this identifies a plan, it does not protect anything.
fn plan_id(policy: &str, strategies: &[String], markets: &[String], actions: &[Action]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |s: &str| {
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    eat(policy);
    for s in strategies {
        eat(s);
    }
    for m in markets {
        eat(m);
    }
    for a in actions {
        eat(a.kind);
        eat(&a.token_id);
        eat(&a.side);
        eat(&format!("{:.6}/{:.6}", a.shares, a.price));
    }
    format!("p-{:08x}", (h >> 32) as u32)
}

// ---------------------------------------------------------------------------
// Plan rendering
// ---------------------------------------------------------------------------

fn plan_summary(
    policy: &Policy,
    actions: &[Action],
    cands: &[Cand],
    snap: &Option<snapshots::Latest>,
    plan_id: &str,
) -> String {
    let (o, i, rd, c) = (
        count_kind(actions, "open"),
        count_kind(actions, "increase"),
        count_kind(actions, "reduce"),
        count_kind(actions, "close"),
    );
    let unchanged = count_kind(actions, "no change");
    let unmanaged = count_kind(actions, "unmanaged");
    let considered = cands.len();
    let refused = cands.iter().filter(|x| !x.gate.is_empty()).count();

    let line = format!(
        "<p class=\"plan-line\"><b>Plan:</b> {o} to open, {i} to increase, {rd} to reduce, {c} to close.</p>"
    );
    let prices = match snap {
        Some(s) => format!(
            "priced against the order book as of {} minutes ago",
            s.age_mins
        ),
        None => "no order-book snapshot could be read, so every price below is the last logged midpoint spread apart by this policy's assumed spread — an assumption about a book, not a book".to_string(),
    };

    let stats = stat_grid(&[
        stat(&esc(&policy.name), "policy")
            .href("/backtest")
            .context(if policy.character.is_empty() {
                "no character line in the policy file"
            } else {
                &policy.character
            }),
        stat(&render::fmt_int(considered as i64), "signals considered").context(
            if policy.delay_hours > 0.0 {
                // A delayed policy does not act on the newest number it has.
                "the latest unresolved prediction on each selected market that is already old enough for this policy's entry delay"
            } else {
                "the latest unresolved prediction on each selected market"
            },
        ),
        stat(&render::fmt_int((considered - refused) as i64), "passed every gate")
            .tone(if considered > 0 && considered == refused { "warn" } else { "" })
            .context(&if refused == 1 {
                "1 candidate was refused, and the table below says by which rule".to_string()
            } else {
                format!("{refused} candidates were refused, and the table below says by which rule")
            }),
        stat(&render::fmt_int((o + i + rd + c) as i64), "actions")
            .context(&format!("{unchanged} unchanged · {unmanaged} outside this plan")),
    ]);

    section_foot(
        "What this plan would do",
        "target positions minus what is open, the way terraform reads a diff",
        &badge(&format!("version {}", policy.version), if policy.version >= 2 { "ok" } else { "warn" }),
        &format!("{line}{stats}"),
        &format!(
            "<span class=\"mono\">execution/policies/{}.toml</span><span class=\"mono\">plan {}</span><span>{}</span>",
            esc(&policy.file),
            esc(plan_id),
            esc(&prices)
        ),
    )
}

fn v1_warning(policy: &Policy) -> String {
    if policy.taker_fees {
        return String::new();
    }
    format!(
        "<div class=\"banner\">{} <div><strong>This policy is fee-free.</strong> \
         <span class=\"mono\">{}</span> charges no venue fee at all, and the venue has charged takers since \
         2026-01-05 ({}). Every cost below is therefore missing the fee, and a fee-free number \
         must never be read as a costed one — the v1 files exist so the v1→v2 difference is readable as \
         the price of the fee, not because anyone trades this way. Pick a <span class=\"mono\">-v2</span> policy \
         to see what it would actually cost.</div></div>",
        render::icon("alert"),
        esc(&policy.file),
        esc(&policy.fee_label()),
    )
}

/// Whether the plan fits in the money we have. It is never silently truncated:
/// a plan that cannot be funded is still shown in full, and said to be
/// unfundable, because "here is a smaller plan" hides the constraint.
fn funding(actions: &[Action], cash: f64, committed: f64, free: f64) -> String {
    let required: f64 = actions
        .iter()
        .filter(|a| a.kind == "open" || a.kind == "increase")
        .map(|a| a.collateral)
        .sum();
    let released: f64 = actions
        .iter()
        .filter(|a| a.kind == "close" || a.kind == "reduce")
        .map(|a| -a.collateral)
        .sum();
    let fees: f64 = actions.iter().map(|a| a.fee).sum();
    let cash_delta: f64 = actions.iter().map(|a| a.cash_delta).sum();
    if required <= EPS && released <= EPS && fees <= EPS {
        return String::new();
    }
    let cash_after = cash + cash_delta;
    let committed_after = committed + required - released;
    let free_after = cash_after - committed_after;

    let short = required > free + released + EPS;
    let warning = if short {
        format!(
            "<div class=\"banner banner-bad\">{} <div><strong>This plan does not fit.</strong> \
             It needs {} of collateral and only {} is free (plus {} that closing would release). \
             Nothing below has been trimmed to fit — a truncated plan would hide the constraint. \
             Either reduce the selection, or accept that the book cannot take all of it.</div></div>",
            render::icon("alert"),
            usd(required),
            usd(free),
            usd(released)
        )
    } else {
        String::new()
    };

    let rows = vec![
        vec![
            "Cash".to_string(),
            usd(cash),
            usd_signed(cash_delta),
            usd(cash_after),
            "fees leave the account; collateral does not — it moves into the position".to_string(),
        ],
        vec![
            "Committed as collateral".to_string(),
            usd(committed),
            usd_signed(required - released),
            usd(committed_after),
            format!("{} to post, {} released", usd(required), usd(released)),
        ],
        vec![
            "Free cash".to_string(),
            usd(free),
            usd_signed(free_after - free),
            format!(
                "<span class=\"{}\">{}</span>",
                if free_after < 0.0 { "s-bad" } else { "" },
                usd(free_after)
            ),
            "cash minus collateral — what the next plan has to work with".to_string(),
        ],
        vec![
            "Fees".to_string(),
            "—".to_string(),
            usd_signed(-fees),
            "—".to_string(),
            "taker fee on every entry, and again on any exit taken in the market".to_string(),
        ],
    ];

    format!(
        "{warning}{}",
        section_foot(
            "Where this leaves the book",
            "the same three numbers the Holdings tab shows, before and after",
            "",
            &table(
                &[("", ""), ("Now", "num"), ("Change", "num"), ("After", "num"), ("What it means", "wrap")],
                &rows,
            ),
            "<span>collateral is capital locked until the position closes, not money spent (<span class=\"mono\">execution/DESIGN.md</span> §3)</span>",
        )
    )
}

fn actions_table(actions: &[Action], metas: &[data::VariantMeta]) -> String {
    let live: Vec<&Action> = actions.iter().filter(|a| a.kind != "unmanaged").collect();
    let acting: Vec<&&Action> = live.iter().filter(|a| a.kind != "no change").collect();

    if acting.is_empty() {
        let unchanged = live.len();
        let detail = if unchanged == 0 {
            "<div>The book is empty and this policy opens nothing against today's signals. \
             That is a result, not a blank screen: the table below names the rule that stopped each candidate. \
             It is also the backtest's headline finding — on our own live signals, seven of the eight \
             policies take zero trades.</div>"
                .to_string()
        } else {
            format!(
                "<div>Every one of the {} open positions is where this policy wants it, and no new \
                 candidate cleared the gates. The table below names the rule that stopped each one.</div>",
                unchanged
            )
        };
        return section(
            "The diff",
            "nothing to apply",
            &badge("no changes", ""),
            &render::empty_state("This plan proposes no trades", &detail),
        );
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    for a in &live {
        let strategy = strategy_links(&a.family, &a.variant, metas);
        rows.push(vec![
            format!(
                "<span class=\"plan-sym {}\">{}</span>",
                symbol_tone(a.kind),
                symbol(a.kind)
            ),
            format!("<b>{}</b>", esc(a.kind)),
            format!(
                "<a href=\"/markets/{0}\">{0}</a><span class=\"sub\">{1}</span>",
                esc(&a.slug),
                esc(&a.outcome)
            ),
            strategy,
            esc(&a.side),
            if a.shares > 0.0 && a.kind != "no change" {
                shares_fmt(a.shares)
            } else {
                "—".to_string()
            },
            fmt_prob(a.price),
            if a.collateral.abs() > EPS { usd_signed(a.collateral) } else { "—".to_string() },
            if a.fee > 0.0 { usd(a.fee) } else { "—".to_string() },
            format!(
                "{}{}",
                esc(&a.why),
                if a.note.is_empty() {
                    String::new()
                } else {
                    format!("<span class=\"sub\">{}</span>", esc(&a.note))
                }
            ),
        ]);
    }

    // A fill that slippage has walked all the way to 0 or 1 is not a bug and is
    // not a trade anyone would make — it is what a policy with no depth cap
    // does when the stake exceeds the whole visible book. Say so, because a
    // reader who does not know `apply_slippage` will read it as broken.
    let walked = live
        .iter()
        .any(|a| (a.kind == "open" || a.kind == "increase") && (a.price <= 0.0 || a.price >= 1.0));
    let caveat = if walked {
        render::notes(&[
            "Rows priced at <b>0.0000</b> or <b>1.0000</b> have had the fill walked to the end of the book by \
             slippage: this policy caps how much of the resting depth it will take at 100%, so the stake eats \
             the whole book and the engine's linear penalty (<span class=\"mono\">apply_slippage</span>) moves the \
             price the full width it has room for. It is the engine's arithmetic unchanged, and it is also \
             a trade no counterparty would fill — read it as the policy failing a capacity test, not as a price."
                .to_string(),
        ])
    } else {
        String::new()
    };

    let unmanaged = count_kind(actions, "unmanaged");
    let foot = if unmanaged > 0 {
        format!(
            "<span>{} not shown: outside this plan's market selection, so it leaves them alone</span>",
            render::count(unmanaged, "open position")
        )
    } else {
        "<span>+ open · ~ change · − close · = leave alone</span>".to_string()
    };

    section_foot(
        "The diff",
        "one row per position the plan touches, and why",
        &badge(&render::count(acting.len(), "change"), ""),
        &format!(
            "{}{}",
            table(
                &[
                    ("", ""),
                    ("Action", ""),
                    ("Market", ""),
                    ("Strategy", ""),
                    ("Side", ""),
                    ("Shares", "num"),
                    ("Price", "num"),
                    ("Collateral", "num"),
                    ("Fee", "num"),
                    ("Why", "wrap"),
                ],
                &rows,
            ),
            caveat
        ),
        &foot,
    )
}

/// The screen this page exists for. A policy that trades nothing has told you
/// something precise, and the precision is in which rule fired: "no trades" is
/// useless, "no trades, and 9 of them because our probability sits inside the
/// spread" is a research finding.
fn gate_report(cands: &[Cand], policy: &Policy) -> String {
    let refused: Vec<&Cand> = cands.iter().filter(|c| !c.gate.is_empty()).collect();
    if refused.is_empty() {
        if cands.is_empty() {
            return section(
                "Why the rest were not traded",
                "",
                "",
                &render::empty_state(
                    "There were no candidates at all",
                    "<div>No selected strategy has an unresolved prediction on a selected market. \
                     Widen the selection above, or wait for the next run to log signals.</div>",
                ),
            );
        }
        return String::new();
    }

    // The counts first: which rule is doing the work.
    let mut tally: Vec<(&str, usize)> = Vec::new();
    for c in &refused {
        match tally.iter_mut().find(|(g, _)| *g == c.gate) {
            Some((_, n)) => *n += 1,
            None => tally.push((c.gate, 1)),
        }
    }
    tally.sort_by(|a, b| b.1.cmp(&a.1));
    let strip: Vec<(String, String, &str)> = tally
        .iter()
        .map(|(g, n)| (render::fmt_int(*n as i64), g.to_string(), ""))
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for c in &refused {
        rows.push(vec![
            format!(
                "<a href=\"/markets/{0}\">{0}</a><span class=\"sub\">{1}</span>",
                esc(&c.slug),
                esc(&c.outcome)
            ),
            format!(
                "{}<span class=\"sub\">{}</span>",
                fmt_prob(c.p),
                esc(&fmt_ts(&c.signal_ts))
            ),
            if c.bid > 0.0 || c.ask > 0.0 {
                format!(
                    "{} / {}{}",
                    fmt_prob(c.bid),
                    fmt_prob(c.ask),
                    if c.synthetic {
                        "<span class=\"sub\">assumed spread, not a real book</span>"
                    } else {
                        ""
                    }
                )
            } else {
                "<span class=\"muted\">—</span>".to_string()
            },
            if c.side.is_empty() {
                "<span class=\"muted\">none</span>".to_string()
            } else {
                format!("{}<span class=\"sub\">{}</span>", esc(c.side), fmt_signed(c.edge))
            },
            format!("<b>{}</b>", esc(c.gate)),
            esc(&c.detail),
        ]);
    }

    let mut notes = vec![format!(
        "Gates are applied in the same order as the backtest engine \
         (<span class=\"mono\">execution/engine/src/sim.rs</span>), so a candidate refused here would be \
         refused there. The first rule to fire is the one reported."
    )];
    if policy.respect_venue_epsilon {
        notes.push(
            "This policy also declines to sell a barrier sitting within 0.2% of its running extreme. \
             No signal set carries that distance yet, so the screen could not be applied to any row above — \
             counted, not silently passed."
                .to_string(),
        );
    }
    if cands.iter().any(|c| c.synthetic) {
        notes.push(
            "Rows marked <b>assumed spread</b> had no live order book. Their bid and ask were built by \
             spreading the last logged midpoint apart, which is a guess about a price rather than an \
             observation of one — treat any verdict on those rows as provisional."
                .to_string(),
        );
    }

    section_foot(
        "Why the rest were not traded",
        "every candidate that did not make it, and the rule that stopped it",
        &badge(&render::count(refused.len(), "refusal"), ""),
        &format!(
            "{}{}{}",
            render::stat_line(&strip),
            table(
                &[
                    ("Market", ""),
                    ("Our probability", "num"),
                    ("Book (bid / ask)", "num"),
                    ("Side and edge", ""),
                    ("Stopped by", ""),
                    ("The numbers", "wrap"),
                ],
                &rows,
            ),
            render::notes(&notes)
        ),
        "<span class=\"mono\">predictions/predictions.csv</span><a href=\"/backtest\">How these policies did in the backtest →</a>",
    )
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

/// Apply, honestly. The dashboard is a read-only Worker with no commit path, so
/// this renders the rows and the command and stops. There is no success state
/// to fake, and making application a deliberate act by the CEO is the point:
/// the ledger is the audit trail, and an audit trail nobody signs is decoration.
fn apply_section(actions: &[Action], policy: &Policy, plan_id: &str) -> String {
    let acting: Vec<&Action> = actions
        .iter()
        .filter(|a| a.kind != "no change" && a.kind != "unmanaged")
        .collect();
    if acting.is_empty() {
        return section(
            "Apply",
            "",
            "",
            &render::empty_state(
                "Nothing to apply",
                "<div>The plan proposes no changes, so there are no ledger rows to append. \
                 Nothing is written either way — this page only reads.</div>",
            ),
        );
    }

    let now = now_iso();
    let mut csv = String::from(
        "applied_at,action,market_slug,condition_id,outcome,token_id,side,shares,price,fee,cash_delta,collateral_delta,policy,plan_id,note\n",
    );
    for a in &acting {
        csv.push_str(&format!(
            "{now},{kind},{slug},{cond},{outcome},{token},{side},{shares:.4},{price:.4},{fee:.5},{cash:.5},{col:.4},{policy},{plan},{note}\n",
            kind = a.kind,
            slug = a.slug,
            cond = a.condition_id,
            outcome = a.outcome,
            token = a.token_id,
            side = a.side,
            shares = a.shares,
            price = a.price,
            fee = a.fee,
            cash = a.cash_delta,
            col = a.collateral,
            policy = policy.file,
            plan = plan_id,
            note = a.note.replace(',', ";"),
        ));
    }

    let body = format!(
        "<p class=\"finding\">Applying is something the CEO does in a commit, not something this page can do. \
         The dashboard reads the repository over the network and has <b>no write path at all</b> — granting a \
         web-facing service commit access to the firm's own state is Felix's decision, not a convenience. \
         It is also the better shape: a plan that has to be applied by hand leaves a signed, auditable trail, \
         and that trail is exactly what <span class=\"mono\">portfolio/ledger.csv</span> is for.</p>\
         <p class=\"finding\">Append these {n} to the ledger, then rebuild \
         <span class=\"mono\">positions.csv</span> from it — the ledger is the authority and the positions file \
         is only an index over it.</p>\
         <pre class=\"codeblock\">{csv}</pre>\
         <p class=\"finding\">Or, from a checkout:</p>\
         <pre class=\"codeblock\">{cmd}</pre>{notes}",
        n = render::count(acting.len(), "row"),
        csv = esc(&csv),
        cmd = esc(&format!(
            "cat >> portfolio/ledger.csv <<'EOF'\n{}EOF\ngit commit portfolio/ledger.csv portfolio/positions.csv \\\n  -m \"paper book: apply plan {} under {}\"",
            csv.lines().skip(1).map(|l| format!("{l}\n")).collect::<String>(),
            plan_id,
            policy.file
        )),
        notes = render::notes(&[
            format!(
                "<b><span class=\"mono\">applied_at</span> is this page's clock.</b> Set it to the moment you actually commit; \
                 the rows are otherwise exact."
            ),
            format!(
                "<b><span class=\"mono\">cash_delta</span> on an entry is the fee alone.</b> Posting collateral does not spend cash, \
                 it commits it — cash minus the sum of open collateral is the free cash figure (portfolio/README.md)."
            ),
            format!(
                "<b>Prices move.</b> These rows were priced against the book at the moment this page rendered; \
                 re-run the plan before committing if any time has passed. The plan id \
                 <span class=\"mono\">{}</span> changes when they do.",
                esc(plan_id)
            ),
        ]),
    );

    section_foot(
        "Apply",
        "the exact rows this plan implies, for the CEO to append",
        &badge("nothing is written by this page", "warn"),
        &body,
        "<span class=\"mono\">portfolio/ledger.csv</span><span>append-only, and the authority for every number on the Holdings tab</span>",
    )
}

// ---------------------------------------------------------------------------
// The selection form — a GET form, so the whole plan stays in the URL and the
// back button is correct without a line of JavaScript.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn selection_form(
    sel: &Sel,
    metas: &[data::VariantMeta],
    default_strategies: &[String],
    chosen_strategies: &[String],
    available_markets: &[String],
    chosen_markets: &[String],
    policies: &[Policy],
    policy_name: &str,
) -> String {
    let mut strat = String::new();
    for m in metas {
        // A retired variant produces no new signals; offering it would be a
        // control that cannot do anything.
        if m.status == "retired" && !chosen_strategies.contains(&m.key()) {
            continue;
        }
        let key = m.key();
        strat.push_str(&format!(
            "<label><input type=\"checkbox\" name=\"s\" value=\"{v}\"{on}><span><b>{k}</b> {badge}<span class=\"pick-sub\">{sum}</span></span></label>",
            v = esc(&key),
            on = if chosen_strategies.contains(&key) { " checked" } else { "" },
            k = esc(&key),
            badge = render::status_badge(&m.status),
            sum = esc(&m.summary),
        ));
    }
    if strat.is_empty() {
        strat = "<p class=\"pick-empty\">No strategy has a manifest to select.</p>".to_string();
    }

    let mut mkt = String::new();
    for slug in available_markets {
        mkt.push_str(&format!(
            "<label><input type=\"checkbox\" name=\"m\" value=\"{v}\"{on}><span>{s}</span></label>",
            v = esc(slug),
            on = if chosen_markets.contains(slug) { " checked" } else { "" },
            s = esc(slug),
        ));
    }
    if mkt.is_empty() {
        mkt = "<p class=\"pick-empty\">The selected strategies have no unresolved predictions, so there is nothing to pick.</p>".to_string();
    }

    let mut opts = String::new();
    for p in policies {
        opts.push_str(&format!(
            "<option value=\"{v}\"{on}>{name} — {ch}</option>",
            v = esc(&p.file),
            on = if p.file == policy_name { " selected" } else { "" },
            name = esc(&p.file),
            ch = esc(if p.character.is_empty() { "no description" } else { &p.character }),
        ));
    }

    let markets_open = !sel.markets.is_empty();
    let market_summary = if sel.markets.is_empty() {
        format!(
            "all {} with a live prediction",
            render::count(available_markets.len(), "market")
        )
    } else {
        format!(
            "{} of {} chosen",
            chosen_markets.len(),
            available_markets.len()
        )
    };

    let body = format!(
        r#"<form class="pick" method="get" action="/execution">
<input type="hidden" name="tab" value="plan">
<div class="pick-group">
  <div class="pick-label">Strategies <span class="pick-hint">whose signals this plan may act on. Default: everything on trial or live.</span></div>
  <div class="pick-opts">{strat}</div>
</div>
<details class="pick-group"{mopen}>
  <summary><span class="pick-label">Markets <span class="pick-hint">{msum}</span></span></summary>
  <div class="pick-opts pick-opts-narrow">{mkt}</div>
</details>
<div class="pick-group">
  <div class="pick-label">Execution policy <span class="pick-hint">the rules that turn a probability into a position. A <span class="mono">-v1</span> policy charges no fee and will say so.</span></div>
  <select name="p" aria-label="Execution policy">{opts}</select>
</div>
<div class="pick-actions">
  <button type="submit" class="btn">Plan</button>
  <a class="link" href="/execution?tab=plan">Reset to defaults</a>
  <a class="link" href="{back}">Back to holdings</a>
</div>
</form>"#,
        strat = strat,
        mkt = mkt,
        opts = opts,
        msum = esc(&market_summary),
        mopen = if markets_open { " open" } else { "" },
        back = esc(&href(sel, "")),
    );

    section_foot(
        "What to plan",
        "strategies, markets and the policy — all three live in the address, so this plan is a link",
        "",
        &body,
        &format!(
            "<span>defaults: {} · every market they have a live prediction on · <span class=\"mono\">{}</span></span>",
            if default_strategies.is_empty() {
                "no strategy is on trial or live".to_string()
            } else {
                default_strategies.join(", ")
            },
            DEFAULT_POLICY
        ),
    )
}
