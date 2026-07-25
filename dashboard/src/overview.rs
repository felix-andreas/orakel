//! `/` — the dashboard, and `/execution` — the reserved slot for the
//! execution/backtest engine.
//!
//! Layout (the whole point of v3): one screen, four bands.
//!   1. five compact KPI cards
//!   2. dominant chart panel (2/3) + research-slot panel (1/3)
//!   3. scoring chart (1/2) + dense coverage table with mini-bars (1/2)
//!   4. recent runs, compact
//! Nothing here needs a click to be understood; everything links deeper.

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, fmt_signed, fmt_tokens, item, items, kpi, kpi_row, minibar, panel,
    panel_flush, panel_foot, table,
};
use crate::{json_str, shell, snapshot_banner, trail};
use worker::Env;

pub async fn page(env: &Env) -> String {
    let preds = data::text(env, "predictions/predictions.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let resolutions = data::text(env, "predictions/resolutions.csv").await;
    let state_doc = data::text(env, "ops/state.toml").await;
    let (paths, tree_live) = data::tree(env).await;
    let mut all_live = preds.live
        && detail.live
        && scores.live
        && resolutions.live
        && state_doc.live
        && tree_live;

    let p = Table::parse(&preds.text);
    let d = Table::parse(&detail.text);
    let s = Table::parse(&scores.text);
    let r = Table::parse(&resolutions.text);
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
        "{banner}{kpis}{band2}{band3}{band4}",
        kpis = kpis(&p, &d, &s, &r, &state, &runs),
        band2 = format!(
            "<div class=\"grid-main\">{}{}</div>",
            model_vs_market(&p),
            slots_panel(&state)
        ),
        band3 = format!(
            "<div class=\"grid-pair\">{}{}</div>",
            scoring_panel(&d),
            coverage_panel(&p, &d)
        ),
        band4 = recent_runs(&runs),
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
// Band 1 — KPI cards
// ---------------------------------------------------------------------------

fn kpis(
    p: &Table,
    d: &Table,
    s: &Table,
    r: &Table,
    state: &toml::Table,
    runs: &[(String, toml::Table)],
) -> String {
    // 1. predictions logged, and how many the newest run added
    let n_preds = p.rows.len();
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

    // 2. scored rows and how many beat the market
    let n_scored = d.rows.len();
    let n_ahead = d
        .rows
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();

    // 3. headline scoring numbers from the overall row of scores.csv
    let overall = s
        .rows
        .iter()
        .find(|row| s.cell(row, "level") == "overall")
        .or_else(|| s.rows.iter().find(|row| s.cell(row, "level") == "variant"));
    let mean_imp = overall.map(|row| s.num(row, "mean_improvement"));
    let our_brier = overall.map(|row| s.num(row, "mean_brier")).unwrap_or(0.0);
    let mkt_brier = overall.map(|row| s.num(row, "mean_market_brier")).unwrap_or(0.0);

    // 4. research slots
    let slots_total = data::int_at(state, &["research", "slots_total"]).unwrap_or(0);
    let slots_active = data::int_at(state, &["research", "slots_active"]).unwrap_or(0);
    let next_review = data::arr_at(state, &["research", "slot"])
        .iter()
        .filter_map(|v| v.get("trial_review_due").and_then(|x| x.as_str()))
        .min()
        .unwrap_or("")
        .to_string();

    // 5. token spend
    let total_tokens: i64 = runs
        .iter()
        .filter_map(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .sum();
    let last_tokens = runs
        .first()
        .and_then(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .unwrap_or(0);

    let cards = vec![
        kpi("Predictions logged", &fmt_int(n_preds as i64), "list", 1)
            .delta(&format!("{latest_n} new"), "up")
            .context(&format!(
                "{} markets · {} runs",
                markets.len(),
                runs.len().max(1)
            )),
        kpi("Scored", &fmt_int(n_scored as i64), "target", 2)
            .delta(
                &format!("{n_ahead} of {n_scored} ahead"),
                if n_scored > 0 && n_ahead == n_scored { "up" } else { "" },
            )
            .context(&format!("{} markets resolved", r.rows.len())),
        kpi(
            "Mean improvement",
            &match mean_imp {
                Some(v) => fmt_signed(v),
                None => "—".to_string(),
            },
            "trend-up",
            2,
        )
        .delta(
            match mean_imp {
                Some(v) if v > 0.0 => "beats market",
                Some(_) => "behind market",
                None => "not scored yet",
            },
            match mean_imp {
                Some(v) if v > 0.0 => "up",
                Some(_) => "down",
                None => "",
            },
        )
        .context(&format!(
            "Brier {our_brier:.4} ours vs {mkt_brier:.4} market"
        )),
        kpi(
            "Research slots",
            &format!("{slots_active}<small> / {slots_total}</small>"),
            "layers",
            4,
        )
        .delta(
            &format!("{slots_active} in trial"),
            if slots_active > 0 { "warn" } else { "" },
        )
        .context(&if next_review.is_empty() {
            "no trial running".to_string()
        } else {
            format!("next review {next_review}")
        }),
        kpi("Token spend", &fmt_tokens(total_tokens), "bolt", 3)
            .delta(&format!("{} last run", fmt_tokens(last_tokens)), "")
            .context("recorded per run, no cap set"),
    ];
    kpi_row(&cards)
}

// ---------------------------------------------------------------------------
// Band 2 — the dominant chart + research slots
// ---------------------------------------------------------------------------

/// Every logged prediction, sorted by the market's price at the time: the
/// market's own curve, with our probability as dots underneath it. The gap is
/// the claimed edge — one picture of what the firm is betting on.
fn model_vs_market(p: &Table) -> String {
    if p.rows.is_empty() {
        return panel(
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
    let chip = badge(&render::count(pts.len(), "row"), "");
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

    panel_foot(
        "Model vs market",
        "every logged prediction, sorted by the market's price at the time",
        &chip,
        &format!("{legend}{script}"),
        &format!(
            "<span>{} of {} rows priced below the market — the wing-fade thesis</span><a href=\"/predictions\">All predictions →</a>",
            above,
            pts.len()
        ),
        false,
    )
}

fn slots_panel(state: &toml::Table) -> String {
    let total = data::int_at(state, &["research", "slots_total"]).unwrap_or(0);
    let slots = data::arr_at(state, &["research", "slot"]);
    let updated = data::str_at(state, &["firm", "updated"]);

    let mut inner = String::new();
    for slot in &slots {
        let id = slot.get("id").and_then(|v| v.as_integer()).unwrap_or(0);
        let variant = slot.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        let started = slot.get("trial_started").and_then(|v| v.as_str()).unwrap_or("");
        let due = slot
            .get("trial_review_due")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let day = data::days_between(started, updated)
            .map(|d| format!("day {}", d + 1))
            .unwrap_or_default();
        let href = match variant.split_once('/') {
            Some((f, v)) => format!("/strategies/{f}/{v}"),
            None => String::new(),
        };
        inner.push_str(&item(
            &href,
            &esc(variant),
            &esc(&format!("slot {id} · {day} · review {due}")),
            &badge("trial", "warn"),
        ));
    }
    let used = slots.len() as i64;
    for n in used..total {
        inner.push_str(&format!(
            "<div class=\"item item-empty\"><div class=\"item-main\"><div class=\"item-title\">slot {}</div><div class=\"item-sub\">free — filled by the CEO from the ideas backlog</div></div></div>",
            n + 1
        ));
    }
    if inner.is_empty() {
        inner = "<div class=\"item item-empty\"><div class=\"item-main\"><div class=\"item-title\">No slots configured</div></div></div>".to_string();
    }

    panel_foot(
        "Research slots",
        "one trial variant per slot, reviewed on its own clock",
        &badge(&format!("{used} of {total} active"), if used > 0 { "warn" } else { "" }),
        &items(&inner),
        "<span class=\"mono\">ops/state.toml</span><a href=\"/state\">Firm state →</a>",
        false,
    )
}

// ---------------------------------------------------------------------------
// Band 3 — scoring chart + coverage table
// ---------------------------------------------------------------------------

fn scoring_panel(d: &Table) -> String {
    if d.rows.is_empty() {
        return panel(
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
    panel_foot(
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
        false,
    )
}

/// Predictions grouped by board (the asset token in the slug), with a mini-bar
/// for volume and the average edge we claimed.
fn coverage_panel(p: &Table, d: &Table) -> String {
    if p.rows.is_empty() {
        return panel("Coverage", "predictions by board", "", &render::empty_state("No predictions yet", ""));
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

    panel_flush(
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
// Band 4 — recent runs
// ---------------------------------------------------------------------------

fn recent_runs(runs: &[(String, toml::Table)]) -> String {
    if runs.is_empty() {
        return panel(
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

    panel_foot(
        "Recent runs",
        "what the firm did, newest first",
        &badge(&render::count(runs.len(), "run"), ""),
        &table(
            &[
                ("Date", ""),
                ("Trigger", ""),
                ("Steps", "num"),
                ("Rows added", "num"),
                ("Tokens", "num"),
                ("Health", ""),
            ],
            &body,
        ),
        "<span class=\"mono\">ops/runs/</span><a href=\"/runs\">Read the narrative →</a>",
        true,
    )
}

// ---------------------------------------------------------------------------
// /execution — reserved for the engine the CEO is building
// ---------------------------------------------------------------------------

pub async fn execution(env: &Env) -> String {
    let body = panel(
        "Execution engine",
        "paper execution and backtests — under construction",
        &badge("being built", "warn"),
        &format!(
            "{}{}",
            render::empty_state(
                "Not wired up yet",
                "<div>The CEO is building the execution/backtest engine. This page renders its output as soon as the schema lands: per-signal paper fills, slippage against the book we snapshot hourly, and equity curves per variant.</div>",
            ),
            "<p class=\"note\">Until then, everything the firm claims lives in <a class=\"link\" href=\"/predictions\">Predictions</a> (probability vs market price, scored on resolution) and in the per-variant backtests under <a class=\"link\" href=\"/strategies\">Strategies</a>. CONSTITUTION.md §5: no real trading, ever — paper execution and backtests only.</p>",
        ),
    );
    shell(
        env,
        "/execution",
        trail(&[("Overview", ""), ("Execution", "")]),
        true,
        &body,
    )
    .await
}
