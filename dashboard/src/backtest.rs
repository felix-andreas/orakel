//! `/backtest` — what eight ways of trading our signals would have earned.
//!
//! The surface was called "Execution" until 2026-07-26. It never executed
//! anything: the firm places no orders and never will (CONSTITUTION.md §5).
//! Everything here is a replay of stored signals against stored prices, which
//! is a backtest, so that is what it is called. The repo directory keeps its
//! name — `execution/` is the engine's home and is referenced from
//! ARCHITECTURE.md, ops/decisions.md and several strategy manifests — and every
//! file path this page prints is the true one.
//!
//! Source of truth is `execution/results/summary.csv`, one row per
//! (signal set × policy × policy version). The page never recomputes a metric;
//! it reads them, ranks only the rows the engine is willing to rank, and says
//! in plain English what the matrix means.
//!
//! Three tabs, in the order a reader's questions actually arrive:
//!
//!   ""        our own signals    — does any of this trade at all? (7 of 8: no)
//!   history   historical signals — where there ARE trades, which policy wins
//!   method    how it works       — the accounting rule, the fees, the limits
//!
//! One tab per signal set is the honest split, because "which policy is best"
//! is meaningless without saying *on whose signals* (DESIGN.md §2) and the two
//! sets we have return opposite verdicts. Two rules the page must never blur:
//!
//!   1. **The deciding metric is annualized return on LOCKED CAPITAL**, not
//!      cents per trade (DESIGN.md §3). Selling a 15c wing locks 85c of
//!      collateral to earn that premium; the same cents on a 97c favourite is a
//!      much weaker business. Both are shown, the first one decides.
//!   2. **v1 is fee-free, v2 charges the venue's real taker fee.** A fee-free
//!      number must never be mistakable for a costed one, so v2 is the default,
//!      the version is a visible switch, and picking v1 raises a banner.
//!
//! Sample size sits beside every number and the engine's n < 30 rule is
//! respected: underpowered rows are shown, labelled, and NOT ranked (§7).

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, markdown_body, notes, section, section_foot, stat_line, table,
    table_sortable,
};
use crate::{shell, shell_sub, trail};
use worker::Env;

/// Canonical policy order (execution/README.md): a designed progression from
/// "no discipline at all" to the house style. It is also the colour order in
/// the equity chart, so a policy keeps its colour whichever set is shown.
const POLICIES: [&str; 8] = [
    "mirror", "gate", "kelly", "anchor", "fade", "patient", "sniper", "harvest",
];

/// A signal set, the tab it owns, and the plain-English paragraph introducing
/// it. A set the CSV carries but this list does not still gets a tab (keyed by
/// its own name) and a generic opener — the page must never hide a result
/// because nobody wrote copy for it.
struct SetInfo {
    key: &'static str,
    set: &'static str,
    label: &'static str,
    lede: &'static str,
}

const SETS: [SetInfo; 2] = [
    SetInfo {
        key: "",
        set: "orakel-live",
        label: "Our own signals",
        lede: "A backtest replays a decision the firm already made. For every prediction we \
               have published and later scored, it takes the probability we gave, the price \
               the market was quoting at that moment, and the outcome that eventually \
               happened — then asks what eight different ways of turning a probability into \
               a trade would have earned, after the spread they would have had to cross and \
               the fee the venue charges. No order is placed and no money is ever at risk: \
               this is arithmetic over records we already hold. These are our own signals, \
               the only set with no hindsight anywhere in it.",
    },
    SetInfo {
        key: "history",
        set: "ladder-rv-hist",
        label: "Historical signals",
        lede: "The same eight policies replayed over a far larger set of signals: every \
               resolved checkpoint from the <a class=\"link\" \
               href=\"/strategies/barrier-touch/ladder-rv\">ladder-rv</a> strategy's own \
               backtest, each one a probability we would have published and an outcome that \
               is now known. It is one strategy in one market regime, so it cannot say what \
               works in general — but it is the only set we have that is large enough to \
               tell eight policies apart at all.",
    },
];

// ---------------------------------------------------------------------------
// One row of the matrix
// ---------------------------------------------------------------------------

struct Run {
    set: String,
    policy: String,
    version: u32,
    fee_model: String,
    n_signals: i64,
    n_trades: i64,
    ann: Option<f64>,
    cents: Option<f64>,
    se: Option<f64>,
    hit: Option<f64>,
    hold: Option<f64>,
    cap_eff: Option<f64>,
    max_cap_eff: Option<f64>,
    max_dd: Option<f64>,
    net: f64,
    fees: f64,
    fee_share: Option<f64>,
    synthetic: Option<f64>,
    date_start: String,
    date_end: String,
    underpowered: bool,
    delay_na: i64,
}

impl Run {
    fn ranked(&self) -> bool {
        !self.underpowered && self.n_trades >= 30 && self.ann.is_some()
    }
}

fn parse(t: &Table) -> Vec<Run> {
    t.rows
        .iter()
        .map(|r| Run {
            set: t.cell(r, "signal_set").to_string(),
            policy: t.cell(r, "policy").to_string(),
            version: t.cell(r, "policy_version").parse().unwrap_or(0),
            fee_model: t.cell(r, "fee_model").to_string(),
            n_signals: t.numo(r, "n_signals").unwrap_or(0.0) as i64,
            n_trades: t.numo(r, "n_trades").unwrap_or(0.0) as i64,
            ann: t.numo(r, "annualized_return_on_locked_capital"),
            cents: t.numo(r, "cents_per_trade"),
            se: t.numo(r, "cents_per_trade_se"),
            hit: t.numo(r, "hit_rate"),
            hold: t.numo(r, "mean_hold_days"),
            cap_eff: t.numo(r, "capital_efficiency"),
            max_cap_eff: t.numo(r, "max_capital_efficiency"),
            max_dd: t.numo(r, "max_drawdown_usd"),
            net: t.numo(r, "net_pnl_usd").unwrap_or(0.0),
            fees: t.numo(r, "fees_usd").unwrap_or(0.0),
            fee_share: t.numo(r, "fee_share_of_gross"),
            synthetic: t.numo(r, "synthetic_fill_share"),
            date_start: t.cell(r, "date_start").to_string(),
            date_end: t.cell(r, "date_end").to_string(),
            underpowered: t.cell(r, "underpowered") == "yes",
            delay_na: t.numo(r, "delay_unavailable").unwrap_or(0.0) as i64,
        })
        .collect()
}

/// The runs of one set at one fee version, in canonical policy order — which is
/// the progression the policies were designed as, and therefore what makes a
/// column of zeros legible ("the naive one fired, everything disciplined did
/// not"). Ranking re-sorts only where there is something to rank.
fn set_runs<'a>(runs: &'a [Run], set: &str, version: u32) -> Vec<&'a Run> {
    let mut out: Vec<&Run> = runs
        .iter()
        .filter(|r| r.set == set && r.version == version)
        .collect();
    out.sort_by_key(|r| {
        POLICIES
            .iter()
            .position(|p| *p == r.policy)
            .unwrap_or(POLICIES.len())
    });
    out
}

// ---------------------------------------------------------------------------
// Formatting — every figure carries its basis, or a dash
// ---------------------------------------------------------------------------

const DASH: &str = "<span class=\"muted\">—</span>";

/// A ratio like 13.69 → "1,369%". This is a RATE, not a fund return.
fn ann(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{}%", fmt_int((v * 100.0).round() as i64)),
        None => DASH.to_string(),
    }
}

/// A fraction like 0.1721 → "17.2%".
fn pct(v: Option<f64>, dp: usize) -> String {
    match v {
        Some(v) => render::fmt_pct(v, dp),
        None => DASH.to_string(),
    }
}

fn cents(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:.2}c"),
        None => DASH.to_string(),
    }
}

fn usd(v: f64) -> String {
    if v.abs() < 100.0 {
        format!("${v:.2}")
    } else {
        format!("${}", fmt_int(v.round() as i64))
    }
}

/// The second line inside a table cell. Only legal inside `table.data`.
fn sub(text: &str) -> String {
    format!("<span class=\"sub\">{}</span>", esc(text))
}

// ---------------------------------------------------------------------------
// The plain-English policy characters, read from execution/README.md
// ---------------------------------------------------------------------------

/// `| `mirror` | Every signal, flat stake, no gates. … |` → (mirror, character).
/// Kept in the repo rather than the page so the eight descriptions have exactly
/// one home; PRINCIPLES requires a policy to be readable cold, and a bare name
/// is not.
fn policy_characters(readme: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in readme.lines() {
        let l = line.trim();
        let Some(rest) = l.strip_prefix("| `") else { continue };
        let Some((name, tail)) = rest.split_once('`') else { continue };
        let character = tail
            .trim_start_matches([' ', '|'])
            .trim_end_matches(['|', ' '])
            .trim()
            .to_string();
        if !character.is_empty() {
            out.push((name.to_string(), character));
        }
    }
    out
}

fn lookup(list: &[(String, String)], key: &str) -> String {
    list.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Addresses — the tab and the fee version are both real query parameters, so
// every state of this page is linkable and the back button behaves.
// ---------------------------------------------------------------------------

fn href(tab: &str, version: u32) -> String {
    let mut q: Vec<String> = Vec::new();
    if !tab.is_empty() {
        q.push(format!("tab={tab}"));
    }
    // v2 charges the venue's real fee and is the default, so it carries no
    // parameter: the bare URL is the canonical address of the honest numbers.
    if version == 1 {
        q.push("fees=v1".to_string());
    }
    if q.is_empty() {
        "/backtest".to_string()
    } else {
        format!("/backtest?{}", q.join("&"))
    }
}

/// One entry in the secondary bar. Same markup `render::tabbar` emits — but the
/// hrefs have to carry the fee version as well as the tab, because switching
/// tabs must not silently change which cost model you are reading, and
/// `render::tabbar` only knows about one parameter.
struct Tab {
    key: String,
    label: String,
    count: String,
}

fn tabbar(tabs: &[Tab], active: &str, version: u32) -> String {
    let mut out = String::new();
    for t in tabs {
        let n = if t.count.is_empty() {
            String::new()
        } else {
            format!("<span class=\"tab-n\">{}</span>", esc(&t.count))
        };
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}{}</a>",
            esc(&href(&t.key, version)),
            if t.key == active { " aria-current=\"page\"" } else { "" },
            esc(&t.label),
            n
        ));
    }
    format!("<nav class=\"subbar\" aria-label=\"Sections of this page\"><div class=\"subbar-in\">{out}</div></nav>")
}

// ---------------------------------------------------------------------------
// The page
// ---------------------------------------------------------------------------

pub async fn page(env: &Env, version: u32, want_tab: Option<String>, doc: Option<String>) -> String {
    if let Some(d) = doc {
        return document(env, &d).await;
    }

    // Independent reads, issued together: neither decides the other.
    let (csv, readme) = futures::join!(
        data::text(env, "execution/results/summary.csv"),
        data::text(env, "execution/README.md")
    );
    let all_live = csv.live && readme.live;
    let t = Table::parse(&csv.text);
    let runs = parse(&t);

    if runs.is_empty() {
        let body = render::empty_state(
            "No results yet",
            "<div>The engine writes <span class=\"mono\">execution/results/summary.csv</span> when it runs.</div>",
        );
        return shell(
            env,
            "/backtest",
            trail(&[("Overview", ""), ("Backtest", "")]),
            all_live,
            &body,
        )
        .await;
    }

    let chars = policy_characters(&readme.text);

    // --- which tabs exist -------------------------------------------------
    // One per signal set the CSV actually carries, in the order above, then
    // anything unrecognised, then the method tab.
    let mut tabs: Vec<Tab> = Vec::new();
    for info in &SETS {
        let rows = set_runs(&runs, info.set, version);
        if !rows.is_empty() {
            tabs.push(Tab {
                key: info.key.to_string(),
                label: info.label.to_string(),
                count: fmt_int(rows[0].n_signals),
            });
        }
    }
    // A set the CSV grows that nobody has written copy for still gets a tab,
    // keyed by its own name: a result must never be invisible because the page
    // has not been updated for it.
    let mut extra: Vec<String> = runs
        .iter()
        .filter(|r| r.version == version)
        .map(|r| r.set.clone())
        .filter(|s| !SETS.iter().any(|i| i.set == s))
        .collect();
    extra.sort();
    extra.dedup();
    for set in &extra {
        let rows = set_runs(&runs, set, version);
        tabs.push(Tab {
            key: set.clone(),
            label: set.clone(),
            count: fmt_int(rows[0].n_signals),
        });
    }
    tabs.push(Tab {
        key: "method".to_string(),
        label: "How it works".to_string(),
        count: String::new(),
    });

    // An unknown ?tab= lands on the default tab, never on an error.
    let want = want_tab.as_deref().unwrap_or("");
    let active = tabs
        .iter()
        .find(|t| t.key == want)
        .map(|t| t.key.clone())
        .unwrap_or_default();
    let active = active.as_str();
    let bar = tabbar(&tabs, active, version);

    // The breadcrumb is the title, and the default tab is the page itself.
    let leaf = tabs
        .iter()
        .find(|t| t.key == active && !t.key.is_empty())
        .map(|t| t.label.clone())
        .unwrap_or_default();
    let crumbs = if leaf.is_empty() {
        trail(&[("Overview", ""), ("Backtest", "")])
    } else {
        trail(&[("Overview", ""), ("Backtest", "/backtest"), (&leaf, "")])
    };

    let body = if active == "method" {
        method(&runs)
    } else {
        let set = SETS
            .iter()
            .find(|i| i.key == active)
            .map(|i| i.set)
            .unwrap_or(active);
        let lede = SETS
            .iter()
            .find(|i| i.key == active)
            .map(|i| i.lede)
            .unwrap_or(
                "The same eight policies replayed over one more collection of signals with \
                 known outcomes. No order is placed and no money is ever at risk.",
            );
        set_tab(set, lede, &runs, &chars, active, version)
    };

    shell_sub(env, "/backtest", crumbs, all_live, &bar, &body).await
}

// ---------------------------------------------------------------------------
// One signal set: the finding, the switch, the matrix, the curves, the limits
// ---------------------------------------------------------------------------

fn set_tab(
    set: &str,
    lede: &str,
    runs: &[Run],
    chars: &[(String, String)],
    tab: &str,
    version: u32,
) -> String {
    let mine = set_runs(runs, set, version);
    let other = set_runs(runs, set, if version == 1 { 2 } else { 1 });
    let Some(first) = mine.first() else {
        return render::empty_state("Nothing replayed on this set", "");
    };

    let traded: Vec<&&Run> = mine.iter().filter(|r| r.n_trades > 0).collect();
    let silent: Vec<&&Run> = mine.iter().filter(|r| r.n_trades == 0).collect();
    let ranked: Vec<&&Run> = mine.iter().filter(|r| r.ranked()).collect();
    let trades: i64 = mine.iter().map(|r| r.n_trades).sum();
    let days = data::days_between(&first.date_start, &first.date_end).unwrap_or(0) + 1;

    // --- the frame, in one line -------------------------------------------
    let stats = stat_line(&[
        (fmt_int(first.n_signals), "signals replayed".to_string(), ""),
        (
            format!("{} of {}", traded.len(), mine.len()),
            "policies that traded at all".to_string(),
            if traded.len() * 2 <= mine.len() { "warn" } else { "" },
        ),
        (fmt_int(trades), "trades taken".to_string(), ""),
        (
            fmt_int(days),
            if days == 1 { "day covered" } else { "days covered" }.to_string(),
            "",
        ),
    ]);

    // --- the verdict, in plain English ------------------------------------
    let finding = finding(&mine, &traded, &silent, &ranked, first, tab, version);

    // --- what these numbers charge ----------------------------------------
    let fee_free = first.fee_model.starts_with("none");
    let switch = format!(
        "<div class=\"tabs\"><a href=\"{v2}\"{a2}>Charging the venue's fee (v2)</a><a href=\"{v1}\"{a1}>Fee-free (v1, superseded)</a></div>",
        v2 = esc(&href(tab, 2)),
        v1 = esc(&href(tab, 1)),
        a2 = if version == 2 { " aria-current=\"true\"" } else { "" },
        a1 = if version == 1 { " aria-current=\"true\"" } else { "" },
    );
    let cost = if fee_free {
        format!(
            "<div class=\"banner\">{} <span><b>No venue fee is charged in these rows.</b> Polymarket has taken a fee on every fill since 2026-01-05, so every number below is too generous — most of all for a policy that closes in the market and pays it twice. These rows exist only so earlier reports stay attributable; read every conclusion off the fee-charging version.</span></div>",
            render::icon("clock")
        )
    } else {
        // The category rate table is 11 entries long and belongs on the method
        // tab; what a reader needs here is the formula and that it is charged.
        let formula = first
            .fee_model
            .split_once(" [")
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| first.fee_model.clone());
        render::note(&format!(
            "Every fill below pays the venue's real taker fee, in the engine's own words: <span class=\"mono\">{}</span> — charged on entry, and again on an exit taken in the market. <a class=\"link\" href=\"{}\">What the rates are →</a>",
            esc(&formula),
            esc(&href("method", version))
        ))
    };

    // --- the matrix -------------------------------------------------------
    let matrix = if ranked.is_empty() {
        untraded_table(&mine, chars)
    } else {
        ranked_table(&mine, &other, chars, fee_free)
    };

    // --- the curves, where more than one line makes a comparison ----------
    let curves = if traded.len() >= 2 {
        equity_section(set, &traded, version)
    } else {
        String::new()
    };

    let limits = section(
        "What this sample cannot tell us",
        "generated from the same rows, so it cannot drift from the numbers",
        "",
        &notes(&caveats(&mine, &traded, &ranked, first)),
    );

    // The paths shown are the real ones. The surface is called Backtest; the
    // engine's home in the repo is still `execution/`, and printing anything
    // else here would send a reader to a directory that does not exist.
    let foot = format!(
        "<div class=\"sec-foot\"><span class=\"mono\">execution/results/{}/ · execution/results/summary.csv</span><span><a href=\"/backtest?doc=summary\">The engine's full write-up →</a></span></div>",
        esc(set)
    );

    format!("<p class=\"lede\">{lede}</p>{stats}{finding}{switch}{cost}{matrix}{curves}{limits}{foot}")
}

/// The one-sentence answer the tab exists to give.
fn finding(
    mine: &[&Run],
    traded: &[&&Run],
    silent: &[&&Run],
    ranked: &[&&Run],
    first: &Run,
    tab: &str,
    version: u32,
) -> String {
    // The most important result we have is a negative one: on our own live
    // signals almost nothing fires. It leads whenever it is true.
    if silent.len() * 2 > mine.len() {
        let names: Vec<String> = silent
            .iter()
            .map(|r| format!("<span class=\"mono\">{}</span>", esc(&r.policy)))
            .collect();
        // `patient` enters a day late, so a set with no observation 24h after
        // the signal starves it for a reason that is not about edge at all.
        let starved: Vec<&&&Run> = silent
            .iter()
            .filter(|r| r.delay_na >= first.n_signals && first.n_signals > 0)
            .collect();
        let because = match starved.first() {
            Some(p) => format!(
                " Of those, {} screen for a minimum edge over the quoted price and nothing here cleared it once the spread came off; <span class=\"mono\">{}</span> waits a day before entering, and not one signal in this set has a price recorded that late for it to act on.",
                silent.len() - starved.len(),
                esc(&p.policy)
            ),
            None => " Every one of them screens for a minimum edge over the quoted price, and nothing here cleared it once the spread came off.".to_string(),
        };
        let fired = match traded.first() {
            Some(r) => format!(
                " Only <span class=\"mono\">{}</span> fired — {}, for {}.",
                esc(&r.policy),
                render::count(r.n_trades as usize, "trade"),
                usd(r.net)
            ),
            None => " No policy fired at all.".to_string(),
        };
        let elsewhere = if tab.is_empty() {
            format!(
                " The eight do pull apart given enough signals: <a class=\"link\" href=\"{}\">see what they did on the historical set →</a>",
                esc(&href("history", version))
            )
        } else {
            String::new()
        };
        return format!(
            "<p class=\"finding\"><b>{} of the {} policies took no trades at all</b> — {}.{because}{fired} This is a result, not an empty page: being well calibrated and being tradeable are different properties, and on our own predictions, scored as they stand, there was nothing worth trading.{elsewhere}</p>",
            silent.len(),
            mine.len(),
            names.join(", "),
        );
    }

    let by_ann = ranked
        .iter()
        .max_by(|a, b| a.ann.unwrap_or(f64::MIN).total_cmp(&b.ann.unwrap_or(f64::MIN)));
    let by_cents = ranked
        .iter()
        .max_by(|a, b| a.cents.unwrap_or(f64::MIN).total_cmp(&b.cents.unwrap_or(f64::MIN)));
    match (by_ann, by_cents) {
        (Some(a), Some(c)) if a.policy != c.policy => format!(
            "<p class=\"finding\"><b><span class=\"mono\">{a_name}</span> earns the most on the money it ties up — {a_ann} a year over {a_n} trades — while <span class=\"mono\">{c_name}</span> earns the most per trade ({c_cents} over {c_n}).</b> They disagree because <span class=\"mono\">{a_name}</span> holds a position {a_hold:.1} days and <span class=\"mono\">{c_name}</span> holds it {c_hold:.1}: the same cents recovered twice as fast are worth twice as much on the capital they lock up. That disagreement is the reason this page ranks on locked capital and not on cents.</p>",
            a_name = esc(&a.policy),
            a_ann = ann(a.ann),
            a_n = fmt_int(a.n_trades),
            c_name = esc(&c.policy),
            c_cents = cents(c.cents),
            c_n = render::count(c.n_trades as usize, "trade"),
            a_hold = a.hold.unwrap_or(0.0),
            c_hold = c.hold.unwrap_or(0.0),
        ),
        (Some(a), _) => format!(
            "<p class=\"finding\"><b><span class=\"mono\">{}</span> leads on both measures</b>: {} a year on the capital it locks, and {} per trade, over {} trades.</p>",
            esc(&a.policy),
            ann(a.ann),
            cents(a.cents),
            fmt_int(a.n_trades)
        ),
        _ => String::new(),
    }
}

/// The table for a set where nothing reaches n = 30. Ranking columns would be
/// eight dashes, so the table answers the question that is actually live: what
/// each policy is, and whether it fired.
fn untraded_table(mine: &[&Run], chars: &[(String, String)]) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for r in mine {
        // Why each one stopped is in the finding above — repeating it on seven
        // consecutive rows would add no information and would widen a number
        // column by the length of a sentence.
        let dead = r.n_trades == 0;
        rows.push(vec![
            format!(
                "<b>{}</b>{}",
                esc(&r.policy),
                sub(&lookup(chars, &r.policy))
            ),
            fmt_int(r.n_trades),
            if dead { DASH.to_string() } else { usd(r.net) },
            format!(
                "{}{}",
                if dead { DASH.to_string() } else { ann(r.ann) },
                if dead {
                    String::new()
                } else {
                    sub(&format!("not ranked — n = {}", fmt_int(r.n_trades)))
                }
            ),
            if dead { DASH.to_string() } else { cents(r.cents) },
        ]);
    }
    table(
        &[
            ("Policy", "wrap"),
            ("Trades", "num"),
            ("Net", "num"),
            ("Annual return on locked capital", "num"),
            ("Cents per trade", "num"),
        ],
        &rows,
    )
}

/// The full matrix, for a set with rows the engine is willing to rank.
fn ranked_table(
    mine: &[&Run],
    other: &[&Run],
    chars: &[(String, String)],
    fee_free: bool,
) -> String {
    let mut order: Vec<&&Run> = mine.iter().collect();
    order.sort_by(|a, b| {
        b.ranked()
            .cmp(&a.ranked())
            .then(b.ann.unwrap_or(f64::MIN).total_cmp(&a.ann.unwrap_or(f64::MIN)))
            .then(b.n_trades.cmp(&a.n_trades))
    });

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rank = 0;
    for r in &order {
        let counterpart = other.iter().find(|o| o.policy == r.policy);
        let delta = match (r.ann, counterpart.and_then(|o| o.ann)) {
            (Some(a), Some(b)) => sub(&format!(
                "{}{} pp vs {} {}",
                if a >= b { "+" } else { "−" },
                fmt_int(((a - b) * 100.0).abs().round() as i64),
                ann(Some(b)),
                if fee_free { "with the fee" } else { "fee-free" }
            )),
            _ => String::new(),
        };
        if r.ranked() {
            rank += 1;
        }
        rows.push(vec![
            if r.ranked() { format!("<b>{rank}</b>") } else { DASH.to_string() },
            format!(
                "<b>{}</b>{}{}",
                esc(&r.policy),
                if r.ranked() {
                    String::new()
                } else {
                    format!(" {}", badge("n < 30 — not ranked", "warn"))
                },
                sub(&lookup(chars, &r.policy))
            ),
            fmt_int(r.n_trades),
            format!("<b>{}</b>{}", ann(r.ann), delta),
            format!(
                "{}{}",
                cents(r.cents),
                match r.se {
                    Some(se) => sub(&format!("± {se:.2}")),
                    None => String::new(),
                }
            ),
            format!(
                "{}{}",
                pct(r.hit, 1),
                match r.hold {
                    Some(h) => sub(&format!("{h:.1} days held")),
                    None => String::new(),
                }
            ),
            format!(
                "{}{}",
                pct(r.cap_eff, 0),
                match r.max_dd {
                    Some(dd) if r.n_trades > 0 => sub(&format!("worst dip {}", usd(dd))),
                    _ => String::new(),
                }
            ),
            format!(
                "{}{}",
                pct(r.fee_share, 1),
                if r.n_trades == 0 {
                    String::new()
                } else {
                    sub(&format!("{} of gross", usd(r.fees)))
                }
            ),
        ]);
    }

    table_sortable(
        &[
            ("#", "num"),
            ("Policy", "wrap"),
            ("Trades", "num"),
            ("Annual return on locked capital", "num"),
            ("Cents per trade", "num"),
            ("Hit rate", "num"),
            ("Capital used", "num"),
            ("Fees", "num"),
        ],
        &rows,
    )
}

/// The limits of this sample, every one of them read off the same rows.
fn caveats(mine: &[&Run], traded: &[&&Run], ranked: &[&&Run], first: &Run) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    out.push(format!(
        "<b>One regime.</b> Every row covers {} to {} and nothing else. Read them as conditional on that window, not as what these policies do in general.",
        esc(&first.date_start),
        esc(&first.date_end)
    ));
    if ranked.is_empty() {
        out.push(
            "<b>Nothing here is ranked, and nothing here is disproved.</b> No policy reaches 30 trades, which is the sample the engine requires before it will call one a winner or a loser (DESIGN.md §7). A policy that never fired is untested on this set, not beaten.".into(),
        );
    }
    let synth = traded
        .iter()
        .filter_map(|r| r.synthetic)
        .fold(0.0_f64, f64::max);
    if synth > 0.0 {
        out.push(format!(
            "<b>The fills are invented, not observed.</b> {} of them were priced at the midpoint plus or minus half an assumed 3c spread, because no order book was recorded for that moment. The result is a hypothesis about what execution would have cost, not a measurement of it (DESIGN.md §4).",
            render::fmt_pct(synth, 0)
        ));
    }
    // Annualising a sub-daily hold multiplies it by several hundred. The number
    // is still the right one to rank on; it is not an order of magnitude anyone
    // should quote.
    if let Some(r) = traded.iter().filter(|r| r.hold.unwrap_or(1.0) < 1.0).max_by(|a, b| {
        a.n_trades.cmp(&b.n_trades)
    }) {
        if let Some(h) = r.hold.filter(|h| *h > 0.0) {
            out.push(format!(
                "<b>The annual figure extrapolates hard.</b> <span class=\"mono\">{}</span> holds a position {:.2} days on average, so annualising multiplies its return by {}. Read it as an order of magnitude, not a forecast.",
                esc(&r.policy),
                h,
                fmt_int((365.0 / h).round() as i64)
            ));
        }
    }
    if let Some(p) = mine
        .iter()
        .find(|r| r.delay_na > 0 && r.n_trades > 0)
    {
        out.push(format!(
            "<b><span class=\"mono\">{}</span> is not measured on the same signals as the rest.</b> {} of them had no observation 24 hours later and were dropped — and that attrition is not random, because a barrier that touches stops producing later observations. Its number is an upper bound.",
            esc(&p.policy),
            fmt_int(p.delay_na)
        ));
    }
    let over: Vec<String> = traded
        .iter()
        .filter(|r| r.max_cap_eff.unwrap_or(0.0) > 1.0)
        .map(|r| r.policy.clone())
        .collect();
    if !over.is_empty() {
        out.push(format!(
            "<b>The stated $1,000 bankroll could not have funded {}.</b> Peak deployment went above 100% of it, so the dollar figures are scale-free rates rather than what a fund would have earned; the percentages compare, the dollars do not.",
            if over.len() == traded.len() {
                "a single one of these policies".to_string()
            } else {
                esc(&over.join(", "))
            }
        ));
    }
    if !traded.is_empty() {
        out.push(
            "<b>The standard errors are optimistic, and this page does not rank on them.</b> Repeated signals on one market share a single outcome, so treat a market's trades as one observation before believing any spread. A t-statistic also asks the wrong question — it tests whether the edge is zero, when what decides a favourite-side trade is whether it clears its <a class=\"link\" href=\"/wiki/reference/break-even-win-rate\">break-even win rate</a>.".into(),
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Equity curves
// ---------------------------------------------------------------------------

/// Drawn only where more than one policy traded — a single line from $1,000 to
/// $1,002 is not a comparison, and vertical space is the scarcest thing on the
/// page. The legend lists the policies that actually have a curve, each at its
/// canonical colour index, so a set with fewer policies never recolours them.
fn equity_section(set: &str, traded: &[&&Run], version: u32) -> String {
    let mut legend = String::new();
    for (i, p) in POLICIES.iter().enumerate() {
        if !traded.iter().any(|r| r.policy == *p) {
            continue;
        }
        legend.push_str(&format!("<span><i class=\"c{}\"></i>{}</span>", i + 1, esc(p)));
    }

    let body = format!(
        r#"<div class="legend">{legend}</div>
<div class="chart" id="chart-eq"></div>
<script src="/charts.js"></script>
<script>
(function () {{
  var box = document.getElementById("chart-eq");
  fetch({url})
    .then(function (r) {{ return r.json(); }})
    .then(function (d) {{ box.innerHTML = ""; Chart.line(box, d, {{yPrecision: 0}}); }})
    .catch(function () {{ box.textContent = "failed to load the equity curves"; }});
}})();
</script>"#,
        url = crate::json_str(&format!("/backtest/data/{set}/v{version}.json")),
    );

    section_foot(
        "What $1,000 would have become",
        "bankroll plus realized profit — an open position is not marked to market, because these signal sets carry no price for every day of every hold",
        "",
        &body,
        "<span class=\"mono\">execution/results/&lt;set&gt;/&lt;policy&gt;-v{ver}.json</span><span>drag to zoom, double-click to reset</span>"
            .replace("{ver}", &version.to_string())
            .as_str(),
    )
}

/// `/backtest/data/<set>/v<n>.json` — one series per policy, colour pinned to
/// the policy's canonical position so a set with fewer policies never
/// recolours the rest.
pub async fn equity_json(env: &Env, set: &str, version: u32) -> String {
    if !data::safe_segment(set) || !(1..=9).contains(&version) {
        return "{\"series\":[]}".to_string();
    }
    let mut series: Vec<String> = Vec::new();
    let curves = data::read_all(
        env,
        POLICIES
            .iter()
            .map(|policy| format!("execution/results/{set}/{policy}-v{version}.json")),
    )
    .await;
    for (i, (policy, f)) in POLICIES.iter().zip(&curves).enumerate() {
        if f.is_empty() {
            continue;
        }
        let Ok(doc) = serde_json::from_str::<serde_json::Value>(&f.text) else {
            continue;
        };
        let empty = Vec::new();
        let curve = doc["equity_curve"].as_array().unwrap_or(&empty);
        let mut points: Vec<String> = Vec::new();
        for p in curve {
            let (Some(t), Some(v)) = (p["t"].as_str().and_then(data::ts_ms), p["equity"].as_f64())
            else {
                continue;
            };
            points.push(format!("{{\"t\":{t},\"v\":{v:.2}}}"));
        }
        if points.len() < 2 {
            continue;
        }
        series.push(format!(
            "{{\"label\":{},\"color\":{},\"points\":[{}]}}",
            crate::json_str(policy),
            i,
            points.join(",")
        ));
    }
    format!("{{\"series\":[{}]}}", series.join(","))
}

// ---------------------------------------------------------------------------
// The method tab — why the numbers are the numbers
// ---------------------------------------------------------------------------

fn method(runs: &[Run]) -> String {
    // The costed model, whichever version is being read on the results tabs:
    // this bullet explains what the venue actually charges, so quoting the
    // fee-free string here because someone arrived via ?fees=v1 would make the
    // page contradict itself.
    let model = runs
        .iter()
        .find(|r| !r.fee_model.starts_with("none") && !r.fee_model.is_empty())
        .map(|r| r.fee_model.clone())
        .unwrap_or_default();

    let metric = section(
        "The number that decides",
        "annual return on locked capital, and why it is not cents per trade",
        "",
        &notes(&[
            "<b>A binary outcome token trades between 0 and 1, and selling one costs collateral.</b> Selling YES at 15c means receiving 15c and posting the other 85c until the market settles — so the money a trade ties up, not the cents it earns, is what the return has to be measured against.".into(),
            "<b>The formula.</b> Net profit ÷ (capital locked × days held) × 365. Three cents of edge on a 15c wing locks 85c for a few days; the same three cents on a 97c favourite locks 97c to earn it. Cents per trade calls those the same business. This does not (DESIGN.md §3).".into(),
            "<b>It is a rate, not a fund result.</b> Multiply by the capital-used column to see what a bankroll would actually have earned — and where capital used exceeds 100%, the stated $1,000 bankroll could not have funded the policy at all.".into(),
            "<b>Capital used is reported beside it, always.</b> A policy earning 40% a year on 3% of the bankroll is a rounding error on the firm, and a matrix that hides that is lying by omission.".into(),
        ]),
    );

    let fills = section(
        "What a fill costs",
        "the simulator is conservative on purpose",
        "",
        &notes(&[
            "<b>Never at the midpoint.</b> A buy lifts the ask and a sell hits the bid. Where only a midpoint was recorded, an assumed 3c spread is applied symmetrically and the trade is marked synthetic — flagged, never celebrated (DESIGN.md §4).".into(),
            "<b>Size is capped by the depth that was visible</b> within 5c of the touch. Below the minimum stake the signal is unfundable: counted and reported, never quietly dropped.".into(),
            format!(
                "<b>The venue's taker fee is charged per fill.</b> Polymarket has taken one since 2026-01-05 — <span class=\"mono\">shares × rate × p × (1 − p)</span>, by market category. It peaks at p = 0.50, which is exactly the band these policies target. Charged on entry, and again on an exit taken in the market; settling at resolution is a redemption, not a match, and costs nothing. In the engine's own words: <span class=\"mono\">{}</span>.",
                esc(&model)
            ),
            "<b>Two policy versions exist for exactly one reason.</b> <span class=\"mono\">v1</span> charged no fee, which was wrong; <span class=\"mono\">v2</span> is identical in every other respect and charges the real one. v1 rows are kept only so earlier reports stay attributable, and a policy is never edited in place — a change means a new version file.".into(),
        ]),
    );

    let limits = section_foot(
        "What we refuse to conclude",
        "the rules the engine holds itself to, and this page with it",
        "",
        &notes(&[
            "<b>Nothing below 30 trades is ranked.</b> The engine will not call a policy a winner or a loser on a smaller sample, and neither will this page. An underpowered row is shown and labelled, never quietly promoted (DESIGN.md §7).".into(),
            "<b>Every signal set is one regime.</b> Each result is reported with its date span, and a span is a caveat, not a footnote.".into(),
            "<b>A policy that never fired has not been beaten.</b> Taking no trades is a finding about the policy and the data together; it is not evidence that the policy is bad.".into(),
            "<b>No real money is involved and none ever will be.</b> This is a measurement instrument that tells us which of our beliefs survive a spread, before anything is at stake (CONSTITUTION.md §5).".into(),
        ]),
        "<span class=\"mono\">execution/DESIGN.md · execution/results/summary.csv</span><span><a href=\"/backtest?doc=summary\">The engine's full write-up →</a> · <a href=\"/backtest?doc=design\">The design and its accounting rules →</a></span>",
    );

    format!(
        "<p class=\"lede\">Everything on this page is a replay. A signal is a probability the firm published for one outcome at one moment; a policy is a named way of turning that probability into a trade; a signal set is a frozen collection of signals whose outcomes are now known. The engine walks each policy through each set, prices every fill against the book that was recorded (or an assumed spread where none was), charges the venue's fee, and settles at the outcome that actually happened. Nothing is ordered, and the firm holds no positions.</p>{metric}{fills}{limits}"
    )
}

// ---------------------------------------------------------------------------
// `?doc=` — the engine's own write-ups, rendered whole
// ---------------------------------------------------------------------------

async fn document(env: &Env, which: &str) -> String {
    let (path, label) = match which {
        "summary" => ("execution/results/SUMMARY.md", "The full write-up"),
        "design" => ("execution/DESIGN.md", "Design"),
        _ => ("", ""),
    };
    let crumbs = trail(&[("Overview", ""), ("Backtest", "/backtest"), (label, "")]);
    if path.is_empty() {
        return shell(
            env,
            "/backtest",
            crumbs,
            true,
            &render::empty_state(
                "No such document",
                "<div><a class=\"link\" href=\"/backtest\">Back to the backtest</a>.</div>",
            ),
        )
        .await;
    }
    let f = data::text(env, path).await;
    let body = section_foot(
        &render::md_title(&f.text).unwrap_or_else(|| label.to_string()),
        path,
        "",
        &format!(
            "<div class=\"prose prose-wide\">{}</div>",
            markdown_body(&f.text)
        ),
        &format!("<span class=\"mono\">{path}</span><a href=\"/backtest\">← back to the results</a>"),
    );
    shell(env, "/backtest", crumbs, f.live, &body).await
}
