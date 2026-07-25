//! `/execution` — what our signals would have earned, and whether the sample is
//! big enough to believe.
//!
//! Source of truth is `execution/results/summary.csv`: one row per
//! (signal set × policy × policy version). The page never recomputes a metric —
//! it reads them, ranks the rows the engine is willing to rank, and says in
//! plain English what the matrix means.
//!
//! Two things this page must never blur:
//!
//!   1. **The deciding metric is annualized return on LOCKED CAPITAL**, not
//!      cents per trade (execution/DESIGN.md §3). Selling a 15c wing locks 85c
//!      of collateral to earn that premium; a fat cents-per-trade number on a
//!      97c favourite is a much weaker business. Both are shown, the first one
//!      decides, and the two leaders disagreeing is the headline, not a bug.
//!   2. **v1 is fee-free, v2 charges the venue's real taker fee.** A fee-free
//!      number must never be mistakable for a costed one, so the version is a
//!      visible switch, the fee model is printed in the engine's own words, and
//!      picking v1 raises a banner.
//!
//! Sample size sits next to every number and the engine's n < 30 rule is
//! respected: underpowered rows are shown, labelled, and NOT ranked
//! (DESIGN.md §7).

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, markdown_body, notes, section, section_foot, stat, stat_grid,
    table_sortable,
};
use crate::{shell, trail};
use worker::Env;

/// Canonical policy order (execution/README.md) — also the colour order in the
/// equity chart, so a policy keeps its colour whichever set is shown.
const POLICIES: [&str; 8] = [
    "mirror", "gate", "kelly", "anchor", "fade", "patient", "sniper", "harvest",
];

/// Sets in the order they should be read: the big one that can separate
/// policies first, our own live signals second.
const SETS: [&str; 2] = ["ladder-rv-hist", "orakel-live"];

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
    t_stat: Option<f64>,
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
            t_stat: t.numo(r, "t_stat"),
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
    format!("${}", fmt_int(v.round() as i64))
}

fn sub(text: &str) -> String {
    format!("<span class=\"sub\">{}</span>", esc(text))
}

// ---------------------------------------------------------------------------
// The plain-English characters, read from execution/README.md
// ---------------------------------------------------------------------------

/// `| `mirror` | Every signal, flat stake, no gates. … |` → (mirror, character).
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

/// ``- `orakel-live` — our own predictions ⋈ resolutions. …`` → (set, what it is).
/// Markdown bullets wrap, so indented continuation lines are folded back in.
fn set_characters(readme: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut open = false;
    for line in readme.lines() {
        let l = line.trim();
        if let Some(rest) = line.strip_prefix("- `") {
            open = false;
            if let Some((name, tail)) = rest.split_once('`') {
                let what = tail.trim_start_matches([' ', '—', '-']).trim().to_string();
                if !what.is_empty() {
                    out.push((name.to_string(), what));
                    open = true;
                }
            }
            continue;
        }
        if open && line.starts_with("  ") && !l.is_empty() {
            if let Some(last) = out.last_mut() {
                last.1.push(' ');
                last.1.push_str(l);
            }
        } else if l.is_empty() {
            open = false;
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
// The page
// ---------------------------------------------------------------------------

pub async fn page(env: &Env, version: u32, doc: Option<String>) -> String {
    if let Some(d) = doc {
        return document(env, &d).await;
    }

    let csv = data::text(env, "execution/results/summary.csv").await;
    let readme = data::text(env, "execution/README.md").await;
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
            "/execution",
            trail(&[("Overview", ""), ("Execution", "")]),
            all_live,
            &body,
        )
        .await;
    }

    let chars = policy_characters(&readme.text);
    let sets = set_characters(&readme.text);
    let here: Vec<&Run> = runs.iter().filter(|r| r.version == version).collect();
    let other: Vec<&Run> = runs.iter().filter(|r| r.version != version).collect();

    let fee_model = here
        .first()
        .map(|r| r.fee_model.clone())
        .unwrap_or_default();
    let fee_free = fee_model.starts_with("none");

    // --- headline ---------------------------------------------------------
    let best_ann = here
        .iter()
        .filter(|r| r.ranked())
        .max_by(|a, b| a.ann.unwrap().total_cmp(&b.ann.unwrap()));
    let best_cents = here
        .iter()
        .filter(|r| r.ranked())
        .max_by(|a, b| a.cents.unwrap_or(0.0).total_cmp(&b.cents.unwrap_or(0.0)));
    let trades: i64 = here.iter().map(|r| r.n_trades).sum();
    let signals: i64 = SETS
        .iter()
        .map(|s| {
            here.iter()
                .find(|r| r.set == *s)
                .map(|r| r.n_signals)
                .unwrap_or(0)
        })
        .sum();
    let fees_total: f64 = here.iter().map(|r| r.fees).sum();

    let headline = stat_grid(&[
        stat(
            &best_ann
                .map(|r| esc(&r.policy))
                .unwrap_or_else(|| "—".into()),
            "best on locked capital",
        )
        .tone("ok")
        .context(&match best_ann {
            Some(r) => format!(
                "{} a year · {} trades · {}",
                ann(r.ann),
                fmt_int(r.n_trades),
                r.set
            ),
            None => "nothing reaches n = 30".into(),
        }),
        stat(
            &best_cents
                .map(|r| esc(&r.policy))
                .unwrap_or_else(|| "—".into()),
            "best per trade",
        )
        .context(&match best_cents {
            Some(r) => format!(
                "{} per trade · {} trades · {}",
                cents(r.cents),
                fmt_int(r.n_trades),
                r.set
            ),
            None => "nothing reaches n = 30".into(),
        }),
        stat(&fmt_int(trades), "trades simulated")
            .context(&format!("out of {} signals replayed", fmt_int(signals))),
        stat(
            &best_ann.map(|r| usd(r.net)).unwrap_or_else(|| "—".into()),
            "net profit, best policy",
        )
        .context(&match best_ann {
            Some(r) => format!(
                "{} on a $1,000 bankroll, {} → {}",
                r.policy, r.date_start, r.date_end
            ),
            None => "nothing ranked".into(),
        }),
        stat(&usd(fees_total), "venue fees charged").tone(if fee_free { "bad" } else { "" }).context(
            if fee_free {
                "nothing — this version is fee-free"
            } else {
                "Polymarket taker fee, per fill"
            },
        ),
        stat("none", "money at risk").context("paper only — CONSTITUTION.md §5"),
    ]);

    // --- fee-version switch ----------------------------------------------
    let switch = format!(
        "<div class=\"tabs\"><a href=\"/execution\"{a2}>With the venue's fee (v2)</a><a href=\"/execution?fees=v1\"{a1}>Fee-free (v1, superseded)</a></div>",
        a2 = if version == 2 { " aria-current=\"true\"" } else { "" },
        a1 = if version == 1 { " aria-current=\"true\"" } else { "" },
    );
    let fee_banner = if fee_free {
        format!(
            "<div class=\"banner\">{} <span><b>No venue fee is charged in these rows.</b> Polymarket has taken a fee on every fill since 2026-01-05, so every number below is too generous — most of all for policies that exit in the market and pay it twice. They are kept only so earlier reports stay attributable; read conclusions off the fee-charging version.</span></div>",
            render::icon("clock")
        )
    } else {
        String::new()
    };
    let fee_line = render::note(&format!(
        "Cost model, in the engine's own words: <span class=\"mono\">{}</span>",
        esc(&fee_model)
    ));

    let control = section(
        "What these numbers charge",
        "the only difference between the two versions is the cost model",
        "",
        &format!("{switch}{fee_banner}{fee_line}"),
    );

    // --- one section per signal set ---------------------------------------
    let mut set_sections = String::new();
    for set in SETS {
        let mut mine: Vec<&&Run> = here.iter().filter(|r| r.set == set).collect();
        if mine.is_empty() {
            continue;
        }
        mine.sort_by(|a, b| {
            b.ranked()
                .cmp(&a.ranked())
                .then(b.ann.unwrap_or(f64::MIN).total_cmp(&a.ann.unwrap_or(f64::MIN)))
                .then(b.n_trades.cmp(&a.n_trades))
        });
        set_sections.push_str(&set_section(set, &mine, &other, &chars, &sets, fee_free));
    }

    // --- equity curves ----------------------------------------------------
    let curves = equity_section(version);

    let reading = section_foot(
        "How to read this",
        "the rules the engine holds itself to",
        "",
        &notes(&[
            "<b>Annualized return on locked capital</b> = net pnl ÷ (capital locked × days held) × 365. Selling a 15c wing receives 15c and locks the other 85c until the market settles, so the capital a trade ties up — not the cents it earns — is what the return is measured against (DESIGN.md §3).".into(),
            "<b>It is a rate, not a fund result.</b> Multiply by the capital-used column to see what a bankroll would actually have earned; where capital used exceeds 100% the stated $1,000 bankroll could not have funded the policy at all.".into(),
            "<b>Nothing below n = 30 is ranked.</b> The engine refuses to call a policy a winner or a loser on a smaller sample, and so does this page (DESIGN.md §7).".into(),
            "<b>No real money is involved and none ever will be</b> — this is a measurement instrument that says which of our beliefs survive a spread (CONSTITUTION.md §5).".into(),
        ]),
        "<span class=\"mono\">execution/results/summary.csv</span><span><a href=\"/execution?doc=summary\">The full write-up →</a> · <a href=\"/execution?doc=design\">The design and its accounting rules →</a></span>",
    );

    let body = format!(
        "<p class=\"lede\">Eight named ways of trading our signals, replayed against every signal set we have and priced against a spread. The question is not how many cents a trade earns — it is how much a year the capital each trade ties up earns, and whether the sample is big enough to believe.</p>{headline}{control}{sets}{curves}{reading}",
        headline = headline,
        control = control,
        sets = set_sections,
        curves = curves,
        reading = reading,
    );

    shell(
        env,
        "/execution",
        trail(&[("Overview", ""), ("Execution", "")]),
        all_live,
        &body,
    )
    .await
}

// ---------------------------------------------------------------------------
// One signal set: the finding in words, the matrix, the caveats
// ---------------------------------------------------------------------------

fn set_section(
    set: &str,
    mine: &[&&Run],
    other: &[&Run],
    chars: &[(String, String)],
    sets: &[(String, String)],
    fee_free: bool,
) -> String {
    let first = mine[0];
    let traded: Vec<&&&Run> = mine.iter().filter(|r| r.n_trades > 0).collect();
    let silent: Vec<&&&Run> = mine.iter().filter(|r| r.n_trades == 0).collect();
    let ranked: Vec<&&&Run> = mine.iter().filter(|r| r.ranked()).collect();

    // --- the finding, in plain English ---
    let finding = if silent.len() >= mine.len() / 2 {
        let names: Vec<String> = silent
            .iter()
            .map(|r| format!("<span class=\"mono\">{}</span>", esc(&r.policy)))
            .collect();
        let fired: Vec<String> = traded
            .iter()
            .map(|r| {
                format!(
                    "<span class=\"mono\">{}</span> took {}",
                    esc(&r.policy),
                    render::count(r.n_trades as usize, "trade")
                )
            })
            .collect();
        format!(
            "<p class=\"finding\"><b>{} of the {} policies took no trades at all</b> — {}. After a 3c spread there was no executable edge left in these {} signals for any of them; the minimum-edge screen alone rejected most of what was left. {}. That is a result, not an empty page: on our own scored predictions, as scored, there was nothing to trade.</p>",
            silent.len(),
            mine.len(),
            names.join(", "),
            fmt_int(first.n_signals),
            if fired.is_empty() {
                "No policy fired".to_string()
            } else {
                format!("Only {}", fired.join(" and "))
            }
        )
    } else {
        let by_ann = ranked.first();
        let by_cents = ranked
            .iter()
            .max_by(|a, b| a.cents.unwrap_or(0.0).total_cmp(&b.cents.unwrap_or(0.0)));
        match (by_ann, by_cents) {
            (Some(a), Some(c)) if a.policy != c.policy => format!(
                "<p class=\"finding\">The two metrics disagree, which is the whole point of measuring against locked capital. <b><span class=\"mono\">{}</span> earns {} a year</b> on the money it ties up ({} trades, {:.1} days per trade), while <span class=\"mono\">{}</span> earns the most per trade ({} on {} trades, {:.1} days). Holding time, not headline cents, is what separates them.</p>",
                esc(&a.policy),
                ann(a.ann),
                fmt_int(a.n_trades),
                a.hold.unwrap_or(0.0),
                esc(&c.policy),
                cents(c.cents),
                fmt_int(c.n_trades),
                c.hold.unwrap_or(0.0),
            ),
            (Some(a), _) => format!(
                "<p class=\"finding\"><b><span class=\"mono\">{}</span> leads on both metrics</b>: {} a year on locked capital and {} per trade, over {} trades.</p>",
                esc(&a.policy),
                ann(a.ann),
                cents(a.cents),
                fmt_int(a.n_trades)
            ),
            _ => String::new(),
        }
    };

    // --- the matrix ---
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rank = 0;
    for r in mine {
        let counterpart = other
            .iter()
            .find(|o| o.set == r.set && o.policy == r.policy);
        let delta = match (r.ann, counterpart.and_then(|o| o.ann)) {
            (Some(a), Some(b)) => {
                let pp = (a - b) * 100.0;
                format!(
                    "{}{} pp vs {} {}",
                    if pp >= 0.0 { "+" } else { "−" },
                    fmt_int(pp.abs().round() as i64),
                    ann(Some(b)),
                    if fee_free { "with fees" } else { "fee-free" }
                )
            }
            _ => String::new(),
        };
        let name = format!(
            "<b>{}</b>{}{}",
            esc(&r.policy),
            if r.n_trades == 0 {
                format!(" {}", badge("no trades", "warn"))
            } else if !r.ranked() {
                format!(" {}", badge("n < 30 — not ranked", "warn"))
            } else {
                String::new()
            },
            sub(&lookup(chars, &r.policy))
        );
        if r.ranked() {
            rank += 1;
        }
        rows.push(vec![
            if r.ranked() {
                format!("<b>{rank}</b>")
            } else {
                DASH.to_string()
            },
            name,
            if r.n_trades == 0 {
                DASH.to_string()
            } else {
                fmt_int(r.n_trades)
            },
            format!(
                "<b>{}</b>{}",
                ann(r.ann),
                if r.n_trades == 0 {
                    String::new()
                } else {
                    sub(&format!("n = {}{}", fmt_int(r.n_trades), if delta.is_empty() { String::new() } else { format!(" · {delta}") }))
                }
            ),
            format!(
                "{}{}",
                cents(r.cents),
                match (r.se, r.t_stat) {
                    (Some(se), Some(t)) => sub(&format!("± {se:.2} · t = {t:.2}")),
                    _ => String::new(),
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

    let matrix = table_sortable(
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
    );

    // --- what this sample cannot tell us, generated from the same CSV ---
    let mut caveats: Vec<String> = Vec::new();
    caveats.push(format!(
        "<b>One regime.</b> Every row covers {} to {} and nothing else; read them as conditional on that window.",
        esc(&first.date_start),
        esc(&first.date_end)
    ));
    let synth = traded
        .iter()
        .filter_map(|r| r.synthetic)
        .fold(0.0_f64, f64::max);
    if synth > 0.0 {
        caveats.push(format!(
            "<b>The fills are made up, not observed.</b> {} of them were priced at the midpoint ± half an assumed 3c spread because no order book was recorded for that moment. The ranking is a hypothesis about execution, not a measurement of it (DESIGN.md §4).",
            render::fmt_pct(synth, 0)
        ));
    }
    let under: Vec<String> = mine
        .iter()
        .filter(|r| !r.ranked())
        .map(|r| r.policy.clone())
        .collect();
    if !under.is_empty() {
        caveats.push(format!(
            "<b>Not ranked: {}.</b> Fewer than 30 trades each, which the engine refuses to rank in either direction (DESIGN.md §7).",
            esc(&under.join(", "))
        ));
    }
    if let Some(p) = mine.iter().find(|r| r.delay_na > 0) {
        caveats.push(format!(
            "<b><span class=\"mono\">{}</span> is not measured on the same sample as the others.</b> {} signals had no observation 24 hours later and were dropped — and that attrition is not random, because a barrier that touches stops producing later observations. Its number is an upper bound.",
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
        caveats.push(format!(
            "<b>The stated $1,000 bankroll could not have funded {}.</b> Peak deployment went above 100% of it, so the dollar figures are scale-free rates rather than fund results.",
            if over.len() == traded.len() {
                "a single one of these policies".to_string()
            } else {
                esc(&over.join(", "))
            }
        ));
    }
    caveats.push(
        "<b>The t-statistics are optimistic.</b> Repeated signals on one market share a single outcome, so collapse a market's trades to one observation before believing them.".into(),
    );

    let span = format!(
        "{} signals · {} → {}",
        fmt_int(first.n_signals),
        first.date_start,
        first.date_end
    );
    section_foot(
        set,
        &lookup(sets, set),
        &badge(&span, ""),
        &format!("{finding}{matrix}{}", notes(&caveats)),
        &format!(
            "<span class=\"mono\">execution/results/{}/</span><span>{} of {} policies traded · {} ranked</span>",
            esc(set),
            traded.len(),
            mine.len(),
            ranked.len()
        ),
    )
}

// ---------------------------------------------------------------------------
// Equity curves
// ---------------------------------------------------------------------------

fn equity_section(version: u32) -> String {
    let mut legend = String::new();
    for (i, p) in POLICIES.iter().enumerate() {
        legend.push_str(&format!(
            "<span><i class=\"c{}\"></i>{}</span>",
            i + 1,
            esc(p)
        ));
    }
    let mut options = String::new();
    for s in SETS {
        options.push_str(&format!("<option value=\"{0}\">{0}</option>", esc(s)));
    }

    let body = format!(
        r#"<div class="chart-controls"><label for="eq-set">Signal set</label><select id="eq-set" aria-label="Signal set">{options}</select><span>bankroll plus realized profit — open positions are not marked to market</span></div>
<div class="legend">{legend}</div>
<div class="chart" id="chart-eq"></div>
<script src="/charts.js"></script>
<script>
(function () {{
  var box = document.getElementById("chart-eq");
  var sel = document.getElementById("eq-set");
  function show(set) {{
    fetch("/execution/data/" + set + "/v{version}.json")
      .then(function (r) {{ return r.json(); }})
      .then(function (d) {{ box.innerHTML = ""; Chart.line(box, d, {{yPrecision: 0}}); }})
      .catch(function () {{ box.textContent = "failed to load the equity curves"; }});
  }}
  sel.addEventListener("change", function () {{ show(sel.value); }});
  show(sel.value);
}})();
</script>"#
    );

    section_foot(
        "Equity, day by day",
        "what $1,000 would have become under each policy, on the selected signal set",
        &badge(&format!("policy version v{version}"), ""),
        &body,
        "<span>drag to zoom, double-click to reset</span><span class=\"mono\">execution/results/&lt;set&gt;/&lt;policy&gt;-v{ver}.json</span>"
            .replace("{ver}", &version.to_string())
            .as_str(),
    )
}

/// `/execution/data/<set>/v<n>.json` — one series per policy, colour pinned to
/// the policy's canonical position so a set with fewer policies never
/// recolours the rest.
pub async fn equity_json(env: &Env, set: &str, version: u32) -> String {
    if !data::safe_segment(set) || !(1..=9).contains(&version) {
        return "{\"series\":[]}".to_string();
    }
    let mut series: Vec<String> = Vec::new();
    for (i, policy) in POLICIES.iter().enumerate() {
        let path = format!("execution/results/{set}/{policy}-v{version}.json");
        let f = data::text(env, &path).await;
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
// `?doc=` — the engine's own write-ups, rendered whole
// ---------------------------------------------------------------------------

async fn document(env: &Env, which: &str) -> String {
    let (path, label) = match which {
        "summary" => ("execution/results/SUMMARY.md", "The full write-up"),
        "design" => ("execution/DESIGN.md", "Design"),
        _ => ("", ""),
    };
    let crumbs = trail(&[
        ("Overview", ""),
        ("Execution", "/execution"),
        (label, ""),
    ]);
    if path.is_empty() {
        return shell(
            env,
            "/execution",
            crumbs,
            true,
            &render::empty_state("No such document", "<div><a class=\"link\" href=\"/execution\">Back to execution</a>.</div>"),
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
        &format!("<span class=\"mono\">{path}</span><a href=\"/execution\">← back to the matrix</a>"),
    );
    shell(env, "/execution", crumbs, f.live, &body).await
}
