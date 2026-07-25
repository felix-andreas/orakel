//! `/` — the dashboard.
//!
//! One question per band, and as many answers per screen as the data supports
//! (PRINCIPLES.md: density is a feature; vertical space is the scarcest
//! resource). No cards: a headline stat grid, then sections separated by space
//! and a heading.
//!
//!   1. eight headline numbers, each with the basis it rests on
//!   2. what we are betting on (chart) beside what is running (strategies)
//!   3. how we scored (chart) beside where we play (coverage table)
//!   4. what the firm did lately (runs) beside what it said lately (predictions)
//!
//! Nothing here needs a click to be understood; everything links deeper.

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, fmt_prob, fmt_signed, fmt_tokens, item, items, minibar, section,
    section_foot, stat, stat_grid, table,
};
use crate::{json_str, shell, snapshot_banner, trail};
use worker::Env;

pub async fn page(env: &Env) -> String {
    let preds = data::text(env, "predictions/predictions.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let resolutions = data::text(env, "predictions/resolutions.csv").await;
    let state_doc = data::text(env, "ops/state.toml").await;
    let exec_doc = data::text(env, "execution/results/summary.csv").await;
    let (paths, tree_live) = data::tree(env).await;
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let mut all_live = preds.live
        && detail.live
        && scores.live
        && resolutions.live
        && state_doc.live
        && exec_doc.live
        && tree_live
        && metas_live;

    let p = Table::parse(&preds.text);
    let d = Table::parse(&detail.text);
    let s = Table::parse(&scores.text);
    let r = Table::parse(&resolutions.text);
    let x = Table::parse(&exec_doc.text);
    let state = data::toml_of(&state_doc.text);

    // Run manifests, newest first — spend and the recent-runs strip.
    let mut runs: Vec<(String, toml::Table)> = Vec::new();
    for path in data::run_paths(&paths) {
        let doc = data::text(env, &path).await;
        all_live &= doc.live;
        let stem = path
            .trim_start_matches("ops/runs/")
            .trim_end_matches(".toml")
            .to_string();
        runs.push((stem, data::toml_of(&doc.text)));
    }

    let banner = if all_live { String::new() } else { snapshot_banner() };
    let body = format!(
        "{banner}{headline}{band2}{band3}{band4}",
        headline = headline(&p, &d, &s, &r, &x, &state, &metas, &runs),
        band2 = format!(
            "<div class=\"grid-main\">{}{}</div>",
            model_vs_market(&p),
            strategies_section(&p, &d, &metas)
        ),
        band3 = format!(
            "<div class=\"grid-pair\">{}{}</div>",
            scoring_section(&d),
            coverage_section(&p, &d)
        ),
        band4 = format!(
            "<div class=\"grid-pair\">{}{}</div>",
            recent_runs(&runs),
            latest_predictions(&p, &d)
        ),
    );

    shell(
        env,
        "/",
        trail(&[("Overview", ""), ("Dashboard", "")]),
        all_live,
        &body,
    )
    .await
}

// ---------------------------------------------------------------------------
// Band 1 — the headline numbers
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn headline(
    p: &Table,
    d: &Table,
    s: &Table,
    r: &Table,
    x: &Table,
    state: &toml::Table,
    metas: &[data::VariantMeta],
    runs: &[(String, toml::Table)],
) -> String {
    let mut markets: Vec<&str> = p.rows.iter().map(|row| p.cell(row, "market_slug")).collect();
    markets.sort_unstable();
    markets.dedup();
    let latest_run = p
        .rows
        .last()
        .map(|row| p.cell(row, "run_id").to_string())
        .unwrap_or_default();
    let latest_n = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "run_id") == latest_run)
        .count();

    let n_scored = d.rows.len();
    let n_ahead = d
        .rows
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();

    let overall = s
        .rows
        .iter()
        .find(|row| s.cell(row, "level") == "overall")
        .or_else(|| s.rows.iter().find(|row| s.cell(row, "level") == "variant"));
    let mean_imp = overall.map(|row| s.num(row, "mean_improvement"));
    let our_brier = overall.map(|row| s.num(row, "mean_brier")).unwrap_or(0.0);
    let mkt_brier = overall.map(|row| s.num(row, "mean_market_brier")).unwrap_or(0.0);

    let slots_total = data::int_at(state, &["research", "slots_total"]).unwrap_or(0);
    let slots_active = data::int_at(state, &["research", "slots_active"]).unwrap_or(0);
    let next_review = data::arr_at(state, &["research", "slot"])
        .iter()
        .filter_map(|v| v.get("trial_review_due").and_then(|x| x.as_str()))
        .min()
        .unwrap_or("")
        .to_string();

    let n_live = metas.iter().filter(|m| m.status == "live").count();
    let n_trial = metas.iter().filter(|m| m.status == "trial").count();
    let n_retired = metas.iter().filter(|m| m.status == "retired").count();

    let total_tokens: i64 = runs
        .iter()
        .filter_map(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .sum();
    let last_tokens = runs
        .first()
        .and_then(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .unwrap_or(0);

    // Best execution policy the engine is willing to rank, fee-charging version.
    let best = x
        .rows
        .iter()
        .filter(|row| {
            x.cell(row, "policy_version") == "2"
                && x.cell(row, "underpowered") == "no"
                && x.numo(row, "annualized_return_on_locked_capital").is_some()
        })
        .max_by(|a, b| {
            x.num(a, "annualized_return_on_locked_capital")
                .total_cmp(&x.num(b, "annualized_return_on_locked_capital"))
        });

    stat_grid(&[
        stat(&fmt_int(p.rows.len() as i64), "predictions logged")
            .href("/predictions")
            .context(&format!(
                "{} markets · {} runs · {latest_n} in the last run",
                markets.len(),
                runs.len().max(1)
            )),
        stat(&fmt_int(n_scored as i64), "scored so far")
            .context(&format!(
                "{n_ahead} beat the market · {} markets settled",
                r.rows.len()
            ))
            .tone(if n_scored > 0 && n_ahead == n_scored { "ok" } else { "" }),
        stat(
            &match mean_imp {
                Some(v) => fmt_signed(v),
                None => "—".to_string(),
            },
            "better than the market, per prediction",
        )
        .tone(match mean_imp {
            Some(v) if v > 0.0 => "ok",
            Some(_) => "bad",
            None => "",
        })
        .context(&format!(
            "Brier {our_brier:.4} ours vs {mkt_brier:.4} theirs"
        )),
        stat(
            &format!("{slots_active}<small> / {slots_total}</small>"),
            "research slots in use",
        )
        .href("/state")
        .context(&if next_review.is_empty() {
            "no trial running".to_string()
        } else {
            format!("next review {next_review}")
        }),
        stat(&fmt_int(metas.len() as i64), "strategies")
            .href("/strategies")
            .context(&format!(
                "{n_live} live · {n_trial} in trial · {n_retired} dropped"
            )),
        match best {
            Some(row) => stat(&esc(x.cell(row, "policy")), "best way to trade the signals")
                .href("/execution")
                .tone("ok")
                .context(&format!(
                    "{}%/yr on locked capital · {} trades",
                    fmt_int((x.num(row, "annualized_return_on_locked_capital") * 100.0).round() as i64),
                    fmt_int(x.num(row, "n_trades") as i64)
                )),
            None => stat("—", "best way to trade the signals")
                .href("/execution")
                .context("no policy reaches n = 30 yet"),
        },
        stat(&fmt_int(markets.len() as i64), "markets covered")
            .context(&format!("{} of them have settled", r.rows.len())),
        stat(&fmt_tokens(total_tokens), "tokens spent")
            .context(&format!("{} in the last run", fmt_tokens(last_tokens))),
    ])
}

// ---------------------------------------------------------------------------
// Band 2 — what we are betting on, and what is running
// ---------------------------------------------------------------------------

/// Every logged prediction, sorted by the market's price at the time: the
/// market's own curve, with our probability as dots underneath it. The gap is
/// the claimed edge — one picture of what the firm is betting on.
fn model_vs_market(p: &Table) -> String {
    if p.rows.is_empty() {
        return section(
            "Model vs market",
            "our probability against the market's price, per prediction",
            "",
            &render::empty_state("No predictions yet", "<div>predictions/predictions.csv holds only its header.</div>"),
        );
    }

    let mut pts: Vec<(f64, f64, String)> = p
        .rows
        .iter()
        .map(|row| {
            (
                p.num(row, "market_price"),
                p.num(row, "prediction"),
                format!(
                    "{} {}",
                    data::short_market(p.cell(row, "market_slug")),
                    p.cell(row, "outcome")
                ),
            )
        })
        .collect();
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut market = String::new();
    let mut ours = String::new();
    for (i, (mp, our, label)) in pts.iter().enumerate() {
        if i > 0 {
            market.push(',');
            ours.push(',');
        }
        let l = json_str(label);
        market.push_str(&format!("{{\"t\":{},\"v\":{:.4},\"label\":{}}}", i + 1, mp, l));
        ours.push_str(&format!("{{\"t\":{},\"v\":{:.4},\"label\":{}}}", i + 1, our, l));
    }

    let above = pts.iter().filter(|(mp, our, _)| our < mp).count();
    let legend = "<div class=\"legend\"><span><i></i>market price</span><span><i class=\"c2 dotmark\"></i>our probability</span></div>";
    let script = format!(
        r#"<div class="chart" id="chart-mm"></div>
<script src="/charts.js"></script>
<script>
Chart.line(document.getElementById("chart-mm"), {{series:[
  {{label:"market price",points:[{market}]}},
  {{label:"our probability",points:[{ours}],mode:"dots"}}
]}}, {{x:"index", min:0, yPrecision:3}});
</script>"#
    );

    section_foot(
        "Model vs market",
        "every logged prediction, sorted by the market's price at the time",
        &badge(&render::count(pts.len(), "row"), ""),
        &format!("{legend}{script}"),
        &format!(
            "<span>{} of {} rows priced below the market — the wing-fade thesis</span><a href=\"/predictions\">All predictions →</a>",
            above,
            pts.len()
        ),
    )
}

/// Every strategy the firm has, in its own plain English, newest and most
/// active first. One line each.
fn strategies_section(p: &Table, d: &Table, metas: &[data::VariantMeta]) -> String {
    if metas.is_empty() {
        return section(
            "Strategies",
            "what the firm is running",
            "",
            &render::empty_state("No strategies yet", ""),
        );
    }
    let rank = |m: &data::VariantMeta| match m.status.as_str() {
        "live" => 0,
        "trial" => 1,
        "retired" => 3,
        _ => 2,
    };
    let mut sorted: Vec<&data::VariantMeta> = metas.iter().collect();
    sorted.sort_by(|a, b| rank(a).cmp(&rank(b)).then(b.created.cmp(&a.created)));

    let mut inner = String::new();
    for m in sorted.iter().take(5) {
        let n = p
            .rows
            .iter()
            .filter(|row| p.cell(row, "family") == m.family && p.cell(row, "variant") == m.variant)
            .count();
        let scored = d
            .rows
            .iter()
            .filter(|row| d.cell(row, "family") == m.family && d.cell(row, "variant") == m.variant)
            .count();
        let when = if m.status == "trial" && !m.review_due.is_empty() {
            format!("review due {}", m.review_due)
        } else if m.status == "retired" && !m.retired_on.is_empty() {
            format!("dropped {}", m.retired_on)
        } else {
            format!("created {}", m.created)
        };
        inner.push_str(&item(
            &m.href(),
            &esc(&m.key()),
            &format!(
                "{}{}",
                if m.summary.is_empty() {
                    String::new()
                } else {
                    format!("<span class=\"item-summary\">{}</span>", esc(&m.summary))
                },
                esc(&format!(
                    "{when} · {n} predictions{}",
                    if scored > 0 {
                        format!(", {scored} scored")
                    } else {
                        String::new()
                    }
                ))
            ),
            &render::status_badge(&m.status),
        ));
    }

    section_foot(
        "Strategies",
        "what each one does, in its own words",
        &badge(&render::count(metas.len(), "strategy"), ""),
        &items(&inner),
        "<span class=\"mono\">strategies/&lt;family&gt;/&lt;variant&gt;</span><a href=\"/strategies\">All strategies →</a>",
    )
}

// ---------------------------------------------------------------------------
// Band 3 — scoring and coverage
// ---------------------------------------------------------------------------

fn scoring_section(d: &Table) -> String {
    if d.rows.is_empty() {
        return section(
            "Scoring",
            "market Brier minus our Brier, per resolved prediction",
            "",
            &render::empty_state(
                "Nothing scored yet",
                "<div>predictions/scores_detail.csv fills in when the first markets resolve.</div>",
            ),
        );
    }

    let mut rows: Vec<(String, f64)> = d
        .rows
        .iter()
        .map(|row| {
            (
                data::short_market(d.cell(row, "market_slug")),
                d.num(row, "improvement"),
            )
        })
        .collect();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let bars: String = rows
        .iter()
        .enumerate()
        .map(|(i, (label, v))| {
            format!(
                "{}{{\"label\":{},\"v\":{:.6},\"tone\":\"{}\"}}",
                if i > 0 { "," } else { "" },
                json_str(label),
                v,
                if *v >= 0.0 { "ok" } else { "bad" }
            )
        })
        .collect();

    let ahead = rows.iter().filter(|(_, v)| *v > 0.0).count();
    section_foot(
        "Scoring",
        "market Brier minus our Brier, per resolved prediction",
        &badge(
            &format!("{ahead} of {} ahead", rows.len()),
            if ahead == rows.len() { "ok" } else { "warn" },
        ),
        &format!(
            r#"<div class="chart chart-sm" id="chart-score"></div>
<script src="/charts.js"></script>
<script>
Chart.bar(document.getElementById("chart-score"), {{bars:[{bars}]}}, {{yPrecision:4}});
</script>"#
        ),
        "<span>positive = we beat the market on that leg</span><span class=\"mono\">predictions/scores_detail.csv</span>",
    )
}

/// Predictions grouped by board (the asset token in the slug), with a mini-bar
/// for volume and the average edge we claimed.
fn coverage_section(p: &Table, d: &Table) -> String {
    if p.rows.is_empty() {
        return section("Coverage", "predictions by board", "", &render::empty_state("No predictions yet", ""));
    }

    struct Group {
        asset: String,
        n: usize,
        markets: Vec<String>,
        sum_our: f64,
        sum_mkt: f64,
        scored: usize,
        sum_imp: f64,
    }
    let mut groups: Vec<Group> = Vec::new();
    for row in &p.rows {
        let slug = p.cell(row, "market_slug");
        let asset = data::asset_of(slug);
        let g = match groups.iter_mut().find(|g| g.asset == asset) {
            Some(g) => g,
            None => {
                groups.push(Group {
                    asset,
                    n: 0,
                    markets: Vec::new(),
                    sum_our: 0.0,
                    sum_mkt: 0.0,
                    scored: 0,
                    sum_imp: 0.0,
                });
                groups.last_mut().unwrap()
            }
        };
        g.n += 1;
        if !g.markets.iter().any(|m| m == slug) {
            g.markets.push(slug.to_string());
        }
        g.sum_our += p.num(row, "prediction");
        g.sum_mkt += p.num(row, "market_price");
    }
    for row in &d.rows {
        let asset = data::asset_of(d.cell(row, "market_slug"));
        if let Some(g) = groups.iter_mut().find(|g| g.asset == asset) {
            g.scored += 1;
            g.sum_imp += d.num(row, "improvement");
        }
    }
    groups.sort_by(|a, b| b.n.cmp(&a.n));
    let max_n = groups.iter().map(|g| g.n).max().unwrap_or(1) as f64;

    let body: Vec<Vec<String>> = groups
        .iter()
        .map(|g| {
            let n = g.n as f64;
            let edge = g.sum_mkt / n - g.sum_our / n;
            vec![
                format!("<b>{}</b>", esc(&g.asset)),
                minibar(g.n as f64 / max_n, 1),
                fmt_int(g.n as i64),
                if g.scored > 0 {
                    fmt_int(g.scored as i64)
                } else {
                    "<span class=\"muted\">—</span>".to_string()
                },
                format!("{:.4}", g.sum_mkt / n),
                format!("{:.4}", g.sum_our / n),
                format!(
                    "<span class=\"{}\">{}</span>",
                    if edge > 0.0 { "" } else { "muted" },
                    fmt_signed(edge)
                ),
            ]
        })
        .collect();

    section(
        "Coverage by board",
        "predictions grouped by the asset in the market slug",
        &badge(
            &format!(
                "{} · {}",
                render::count(groups.len(), "board"),
                render::count(groups.iter().map(|g| g.markets.len()).sum::<usize>(), "market")
            ),
            "",
        ),
        &table(
            &[
                ("Board", ""),
                ("Share", "bar-cell"),
                ("Rows", "num"),
                ("Scored", "num"),
                ("Market", "num"),
                ("Ours", "num"),
                ("Edge", "num"),
            ],
            &body,
        ),
    )
}

// ---------------------------------------------------------------------------
// Band 4 — what the firm did, and what it last said
// ---------------------------------------------------------------------------

fn recent_runs(runs: &[(String, toml::Table)]) -> String {
    if runs.is_empty() {
        return section(
            "Recent runs",
            "one manifest per orchestrated run",
            "",
            &render::empty_state("No run manifests yet", ""),
        );
    }
    let body: Vec<Vec<String>> = runs
        .iter()
        .take(6)
        .map(|(_stem, t)| {
            let date = data::tstr(t, "date");
            let steps = data::arr_at(t, &["step"]).len();
            let rows_added = data::int_at(t, &["health", "csv_appended_rows"]).unwrap_or(0);
            let tokens = data::int_at(t, &["spend", "total_tokens"]).unwrap_or(0);
            let ok = data::bool_at(t, &["health", "all_slots_ran"]).unwrap_or(false)
                && data::bool_at(t, &["health", "pushed"]).unwrap_or(false);
            vec![
                format!(
                    "<a href=\"/runs\"><b>{}</b></a><span class=\"sub\">{}</span>",
                    esc(date),
                    esc(data::weekday(date))
                ),
                esc(data::tstr(t, "trigger")),
                fmt_int(steps as i64),
                fmt_int(rows_added),
                fmt_tokens(tokens),
                if ok {
                    badge("healthy", "ok")
                } else {
                    badge("check manifest", "warn")
                },
            ]
        })
        .collect();

    section_foot(
        "Recent runs",
        "what the firm did, newest first",
        &badge(&render::count(runs.len(), "run"), ""),
        &table(
            &[
                ("Date", ""),
                ("Trigger", ""),
                ("Steps", "num"),
                ("Rows", "num"),
                ("Tokens", "num"),
                ("Health", ""),
            ],
            &body,
        ),
        "<span class=\"mono\">ops/runs/</span><a href=\"/runs\">Read the narrative →</a>",
    )
}

/// The last thing the firm actually claimed, market by market.
fn latest_predictions(p: &Table, d: &Table) -> String {
    if p.rows.is_empty() {
        return section(
            "Latest predictions",
            "newest rows in the log",
            "",
            &render::empty_state("Nothing logged yet", ""),
        );
    }
    let body: Vec<Vec<String>> = p
        .rows
        .iter()
        .rev()
        .take(6)
        .map(|row| {
            let slug = p.cell(row, "market_slug");
            let outcome = p.cell(row, "outcome");
            let ts = p.cell(row, "timestamp");
            let ours = p.num(row, "prediction");
            let mkt = p.num(row, "market_price");
            let score = d.rows.iter().find(|sr| {
                d.cell(sr, "market_slug") == slug
                    && d.cell(sr, "outcome") == outcome
                    && d.cell(sr, "timestamp") == ts
            });
            vec![
                format!(
                    "<a href=\"/markets/{0}\">{1}</a><span class=\"sub\">{2} · {3}</span>",
                    esc(slug),
                    esc(&data::short_market(slug)),
                    esc(outcome),
                    esc(&render::fmt_ts(ts))
                ),
                fmt_prob(ours),
                fmt_prob(mkt),
                fmt_signed(mkt - ours),
                match score {
                    Some(sr) => {
                        let v = d.num(sr, "improvement");
                        badge(&fmt_signed(v), if v > 0.0 { "ok" } else { "bad" })
                    }
                    None => "<span class=\"muted\">open</span>".to_string(),
                },
            ]
        })
        .collect();

    section_foot(
        "Latest predictions",
        "the newest rows in the log, with what happened",
        &badge(&render::count(p.rows.len(), "row"), ""),
        &table(
            &[
                ("Market", ""),
                ("Ours", "num"),
                ("Market", "num"),
                ("Edge", "num"),
                ("Result", ""),
            ],
            &body,
        ),
        "<span class=\"mono\">predictions/predictions.csv</span><a href=\"/predictions\">The whole log →</a>",
    )
}
