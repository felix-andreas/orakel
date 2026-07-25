//! `/predictions` — the canonical prediction log, made navigable, and
//! `/markets/<slug>` — everything the firm ever said about one market.
//!
//! Every prediction row links out three ways: the market slug to its own page,
//! the family and the variant to their strategy pages. Scores and resolutions
//! are joined in, so a row shows what we said, what the market said, and what
//! happened, on one line.

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, fmt_prob, fmt_signed, fmt_ts, item, items, section, section_foot,
    stat, stat_grid, table, table_scroll,
};
use crate::{json_str, shell, trail};
use worker::Env;

// ---------------------------------------------------------------------------
// /predictions
// ---------------------------------------------------------------------------

pub async fn page(env: &Env) -> String {
    let preds = data::text(env, "predictions/predictions.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let resolutions = data::text(env, "predictions/resolutions.csv").await;
    let (paths, tree_live) = data::tree(env).await;
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let all_live =
        preds.live && detail.live && scores.live && resolutions.live && tree_live && metas_live;

    let p = Table::parse(&preds.text);
    let d = Table::parse(&detail.text);
    let s = Table::parse(&scores.text);
    let r = Table::parse(&resolutions.text);

    let body = format!(
        "{kpis}{who}{log}<div class=\"grid-pair\">{scoring}{res}</div>",
        kpis = kpi_strip(&p, &d, &s, &r),
        who = attribution(&p, &metas),
        log = log_panel(&p, &d, &r, &metas),
        scoring = scoring_table(&s),
        res = resolutions_table(&r, &p),
    );

    shell(
        env,
        "/predictions",
        trail(&[("Research", ""), ("Predictions", "")]),
        all_live,
        &body,
    )
    .await
}

fn kpi_strip(p: &Table, d: &Table, s: &Table, r: &Table) -> String {
    let mut markets: Vec<&str> = p.rows.iter().map(|row| p.cell(row, "market_slug")).collect();
    markets.sort_unstable();
    markets.dedup();
    let ahead = d
        .rows
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();
    let overall = s.rows.iter().find(|row| s.cell(row, "level") == "overall");
    let mean_imp = overall.map(|row| s.num(row, "mean_improvement"));

    // Beating the quoted price and being able to trade on it are separate
    // claims, and only the second one is money. `market_price` is a midpoint —
    // the average of a bid and an ask — so on a thin leg it can be a number
    // nobody ever offered. Never show the improvement without this beside it.
    let (n_known, n_fill) = overall
        .map(|row| (s.num(row, "n_known_fill"), s.num(row, "n_fillable")))
        .unwrap_or((0.0, 0.0));
    let reachable = (n_known > 0.0).then(|| {
        stat(
            &format!("{}/{}", n_fill as i64, n_known as i64),
            "reachable at that price",
        )
        .tone(if n_fill * 2.0 >= n_known { "ok" } else { "bad" })
        .context("a counterparty was actually observed there — the rest was scored against a midpoint nobody offered")
    });

    let mut stats = vec![
        stat(&fmt_int(p.rows.len() as i64), "rows logged")
            .context("append-only, one per market outcome per run"),
        stat(&fmt_int(markets.len() as i64), "markets")
            .context(&format!("{} resolved so far", r.rows.len())),
        stat(&fmt_int(d.rows.len() as i64), "scored")
            .tone(if !d.rows.is_empty() && ahead == d.rows.len() { "ok" } else { "" })
            .context(&format!(
                "{ahead} ahead of the market's own price at the time"
            )),
        stat(
            &mean_imp.map(fmt_signed).unwrap_or_else(|| "—".to_string()),
            "better than the market, per prediction",
        )
        .tone(match mean_imp {
            Some(v) if v > 0.0 => "ok",
            Some(_) => "bad",
            None => "",
        })
        .context("market Brier minus our Brier — calibration, not cash"),
    ];
    stats.extend(reachable);
    stat_grid(&stats)
}

/// Who produced these rows. A slug and a variant name say nothing about what
/// was claimed; the strategy's own plain-English summary does.
fn attribution(p: &Table, metas: &[data::VariantMeta]) -> String {
    let mut keys: Vec<String> = p
        .rows
        .iter()
        .map(|row| format!("{}/{}", p.cell(row, "family"), p.cell(row, "variant")))
        .collect();
    keys.sort();
    keys.dedup();
    if keys.is_empty() {
        return String::new();
    }
    let mut inner = String::new();
    for key in &keys {
        let n = p
            .rows
            .iter()
            .filter(|row| format!("{}/{}", p.cell(row, "family"), p.cell(row, "variant")) == *key)
            .count();
        let m = metas.iter().find(|m| m.key() == *key);
        inner.push_str(&item(
            &m.map(|m| m.href()).unwrap_or_default(),
            &esc(key),
            &m.map(|m| format!("<span class=\"item-summary\">{}</span>", esc(&m.summary)))
                .unwrap_or_default(),
            &badge(&render::count(n, "row"), ""),
        ));
    }
    section(
        "Where these predictions come from",
        "the strategies that produced the rows below, in their own words",
        "",
        &items(&inner),
    )
}

/// The log itself. Newest first; scores and resolutions joined on the row.
fn log_panel(p: &Table, d: &Table, r: &Table, metas: &[data::VariantMeta]) -> String {
    if p.rows.is_empty() {
        return section(
            "Prediction log",
            "predictions/predictions.csv",
            "",
            &render::empty_state(
                "No predictions yet",
                "<div>The file holds only its header. Rows appear once a research slot produces signals.</div>",
            ),
        );
    }

    let mut idx: Vec<usize> = (0..p.rows.len()).collect();
    idx.sort_by(|a, b| {
        p.cell(&p.rows[*b], "timestamp")
            .cmp(p.cell(&p.rows[*a], "timestamp"))
    });

    let body: Vec<Vec<String>> = idx
        .iter()
        .map(|i| {
            let row = &p.rows[*i];
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
            let resolved = r
                .rows
                .iter()
                .find(|rr| r.cell(rr, "market_slug") == slug);

            let result = match (score, resolved) {
                (Some(sr), res) => {
                    let imp = d.num(sr, "improvement");
                    let winner = res.map(|rr| r.cell(rr, "winning_outcome")).unwrap_or("");
                    let mut html = String::new();
                    if !winner.is_empty() {
                        html.push_str(&badge(&format!("{winner} wins"), ""));
                    }
                    html.push_str(&badge(
                        &fmt_signed(imp),
                        if imp > 0.0 { "ok" } else { "bad" },
                    ));
                    format!("<div class=\"badge-row\">{html}</div>")
                }
                (None, Some(rr)) => badge(&format!("{} wins", r.cell(rr, "winning_outcome")), ""),
                (None, None) => "<span class=\"muted\">open</span>".to_string(),
            };

            vec![
                format!(
                    "<span class=\"mono\">{}</span><span class=\"sub mono\">{}</span>",
                    esc(&fmt_ts(ts)),
                    esc(p.cell(row, "run_id"))
                ),
                format!(
                    "<a href=\"/markets/{0}\">{0}</a>",
                    esc(slug)
                ),
                esc(outcome),
                fmt_prob(ours),
                fmt_prob(mkt),
                format!(
                    "<span class=\"{}\">{}</span>",
                    if mkt - ours > 0.0 { "" } else { "muted" },
                    fmt_signed(mkt - ours)
                ),
                result,
                strategy_links(p.cell(row, "family"), p.cell(row, "variant"), metas),
                format!("<span class=\"mono\">{}</span>", esc(p.cell(row, "model"))),
            ]
        })
        .collect();

    section_foot(
        "Prediction log",
        "what we said, what the market said, and what happened",
        &badge(&render::count(p.rows.len(), "row"), ""),
        &table_scroll(
            &[
                ("Logged", ""),
                ("Market", ""),
                ("Outcome", ""),
                ("Ours", "num"),
                ("Market", "num"),
                ("Edge", "num"),
                ("Result", ""),
                ("Strategy", ""),
                ("Model", ""),
            ],
            &body,
        ),
        "<span class=\"mono\">predictions/predictions.csv · scores_detail.csv · resolutions.csv</span><span>edge = market price − our probability</span>"
    )
}

/// family / variant as two separate links, the variant carrying its
/// plain-English summary as a hover title (no vertical cost in a dense table).
fn strategy_links(family: &str, variant: &str, metas: &[data::VariantMeta]) -> String {
    if family.is_empty() {
        return String::new();
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

fn scoring_table(s: &Table) -> String {
    if s.rows.is_empty() {
        return section(
            "Scoring aggregates",
            "predictions/scores.csv",
            "",
            &render::empty_state(
                "Not generated yet",
                "<div><span class=\"mono\">scoring/</span> writes this file once predictions resolve.</div>",
            ),
        );
    }
    let body: Vec<Vec<String>> = s
        .rows
        .iter()
        .map(|row| {
            let imp = s.num(row, "mean_improvement");
            let key = s.cell(row, "key");
            let level = s.cell(row, "level");
            let key_html = match (level, key.split_once('/')) {
                ("variant", Some((f, v))) => {
                    format!("<a href=\"/strategies/{f}/{v}\">{}</a>", esc(key))
                }
                ("family", _) => format!("<a href=\"/strategies/{0}\">{0}</a>", esc(key)),
                _ => esc(key),
            };
            vec![
                esc(level),
                key_html,
                s.cell(row, "n").to_string(),
                format!(
                    "<span class=\"{}\">{}</span>",
                    if imp > 0.0 { "" } else { "muted" },
                    fmt_signed(imp)
                ),
                format!("{:.4}", s.num(row, "mean_brier")),
                format!("{:.4}", s.num(row, "mean_market_brier")),
            ]
        })
        .collect();

    section(
        "Scoring aggregates",
        "the same rows scored by every grouping we care about",
        &badge(&render::count(s.rows.len(), "group"), ""),
        &table(
            &[
                ("Level", ""),
                ("Key", ""),
                ("n", "num"),
                ("Improvement", "num"),
                ("Brier", "num"),
                ("Market Brier", "num"),
            ],
            &body,
        ),
    )
}

fn resolutions_table(r: &Table, p: &Table) -> String {
    if r.rows.is_empty() {
        return section(
            "Resolutions",
            "predictions/resolutions.csv",
            "",
            &render::empty_state(
                "Nothing resolved yet",
                "<div>The CEO appends a row when a market settles.</div>",
            ),
        );
    }
    let body: Vec<Vec<String>> = r
        .rows
        .iter()
        .rev()
        .map(|row| {
            let slug = r.cell(row, "market_slug");
            let has_pred = p.rows.iter().any(|pr| p.cell(pr, "market_slug") == slug);
            let note = r.cell(row, "note");
            vec![
                format!(
                    "{}{}",
                    if has_pred {
                        format!("<a href=\"/markets/{0}\">{0}</a>", esc(slug))
                    } else {
                        esc(slug)
                    },
                    if note.is_empty() {
                        String::new()
                    } else {
                        format!("<span class=\"sub\">{}</span>", esc(note))
                    }
                ),
                badge(r.cell(row, "winning_outcome"), ""),
                format!("<span class=\"mono\">{}</span>", esc(r.cell(row, "resolved_date"))),
            ]
        })
        .collect();

    section(
        "Resolutions",
        "how each market settled, newest first",
        &badge(&render::count(r.rows.len(), "market"), ""),
        &table(
            &[("Market", ""), ("Winner", ""), ("Resolved", "")],
            &body,
        ),
    )
}

// ---------------------------------------------------------------------------
// /markets/<slug>
// ---------------------------------------------------------------------------

pub async fn market(env: &Env, slug: &str) -> String {
    if !data::safe_path(slug) {
        return shell(
            env,
            "/predictions",
            trail(&[("Research", ""), ("Predictions", "/predictions"), ("Unknown market", "")]),
            true,
            &render::empty_state("Bad market slug", ""),
        )
        .await;
    }

    let preds = data::text(env, "predictions/predictions.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let resolutions = data::text(env, "predictions/resolutions.csv").await;
    let (paths, tree_live) = data::tree(env).await;
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let all_live = preds.live && detail.live && resolutions.live && tree_live && metas_live;

    let p = Table::parse(&preds.text);
    let d = Table::parse(&detail.text);
    let r = Table::parse(&resolutions.text);

    let rows: Vec<&Vec<String>> = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "market_slug") == slug)
        .collect();

    let crumbs = trail(&[
        ("Research", ""),
        ("Predictions", "/predictions"),
        (slug, ""),
    ]);

    if rows.is_empty() {
        let body = section(
            "Market",
            slug,
            "",
            &render::empty_state(
                "No predictions for this market",
                "<div>Nothing in <span class=\"mono\">predictions/predictions.csv</span> carries this slug.</div>",
            ),
        );
        return shell(env, "/predictions", crumbs, all_live, &body).await;
    }

    let resolution = r.rows.iter().find(|rr| r.cell(rr, "market_slug") == slug);
    let latest: &Vec<String> = rows[rows.len() - 1];
    let first: &Vec<String> = rows[0];
    let family = p.cell(latest, "family").to_string();
    let variant = p.cell(latest, "variant").to_string();

    let scored: Vec<&Vec<String>> = d
        .rows
        .iter()
        .filter(|row| d.cell(row, "market_slug") == slug)
        .collect();
    let mean_imp = data::mean(&d, &scored, "improvement");

    let kpis = stat_grid(&[
        stat(&fmt_int(rows.len() as i64), "predictions on this market").context(&format!(
            "{} distinct outcomes",
            {
                let mut o: Vec<&str> = rows.iter().map(|row| p.cell(row, "outcome")).collect();
                o.sort_unstable();
                o.dedup();
                o.len()
            }
        )),
        stat(&fmt_prob(p.num(latest, "prediction")), "our latest probability")
            .context(&format!("logged {}", fmt_ts(p.cell(latest, "timestamp")))),
        stat(&fmt_prob(p.num(latest, "market_price")), "the market's latest price").context(
            &format!(
                "{} edge claimed on the last row",
                fmt_signed(p.num(latest, "market_price") - p.num(latest, "prediction"))
            ),
        ),
        match resolution {
            Some(rr) => stat(&esc(r.cell(rr, "winning_outcome")), "how it settled")
                .tone(match mean_imp {
                    Some(v) if v > 0.0 => "ok",
                    Some(_) => "bad",
                    None => "",
                })
                .context(&format!(
                    "on {} · {} vs the market",
                    r.cell(rr, "resolved_date"),
                    mean_imp
                        .map(fmt_signed)
                        .unwrap_or_else(|| "not scored".to_string())
                )),
            None => stat("open", "how it settled").context("not settled yet"),
        },
    ]);

    let body = format!(
        "{kpis}<div class=\"grid-main\">{chart}{facts}</div>{log}",
        kpis = kpis,
        chart = market_chart(&p, &rows, slug),
        facts = market_facts(&p, &r, &rows, first, latest, resolution, &family, &variant, &metas),
        log = market_log(&p, &d, &rows, slug),
    );

    shell(env, "/predictions", crumbs, all_live, &body).await
}

/// Our probability and the market's price at each logged timestamp, plus the
/// hourly midpoint series we snapshot into R2 (fetched client-side; absent
/// when the bucket has nothing for this market).
fn market_chart(p: &Table, rows: &[&Vec<String>], slug: &str) -> String {
    let mut ours = String::new();
    let mut mkt = String::new();
    let mut n = 0;
    for row in rows {
        let Some(t) = data::ts_ms(p.cell(row, "timestamp")) else { continue };
        if n > 0 {
            ours.push(',');
            mkt.push(',');
        }
        let label = json_str(&format!(
            "{} · {}",
            fmt_ts(p.cell(row, "timestamp")),
            p.cell(row, "outcome")
        ));
        ours.push_str(&format!(
            "{{\"t\":{t},\"v\":{:.4},\"label\":{label}}}",
            p.num(row, "prediction")
        ));
        mkt.push_str(&format!(
            "{{\"t\":{t},\"v\":{:.4},\"label\":{label}}}",
            p.num(row, "market_price")
        ));
        n += 1;
    }

    let legend = "<div class=\"legend\"><span id=\"lg-mid\"><i></i>midpoint (hourly, R2)</span><span><i class=\"c2 dotmark\"></i>market price when logged</span><span><i class=\"c3 dotmark\"></i>our probability</span></div>";
    let script = format!(
        r#"<div class="chart" id="chart-market"></div>
<script src="/charts.js"></script>
<script>
(function () {{
  var box = document.getElementById("chart-market");
  var ours = [{ours}], mkt = [{mkt}];
  function draw(mid) {{
    var has = mid && mid.length > 1;
    var lg = document.getElementById("lg-mid");
    if (lg && !has) lg.style.display = "none";
    /* colour is pinned per series: the midpoint may be absent (empty bucket
       locally) and must not shift the other two. */
    var series = [
      {{label:"midpoint", points: has ? mid : [], mode:"line", color:0}},
      {{label:"market price", points:mkt, mode:"dots", color:1}},
      {{label:"our probability", points:ours, mode:"dots", color:2}}
    ];
    box.innerHTML = "";
    Chart.line(box, {{series:series}}, {{min:0, yPrecision:4}});
  }}
  fetch("/data/market-series/{slug}.json")
    .then(function (r) {{ return r.json(); }})
    .then(function (d) {{ draw(d.points || []); }})
    .catch(function () {{ draw([]); }});
}})();
</script>"#,
        ours = ours,
        mkt = mkt,
        slug = esc(slug),
    );

    section_foot(
        "Probability over time",
        "our number against the market, every time we logged it",
        &badge(&render::count(n, "point"), ""),
        &format!("{legend}{script}"),
        "<span>drag to zoom, double-click to reset</span><a href=\"/snapshots\">Order-book snapshots →</a>"
    )
}

#[allow(clippy::too_many_arguments)]
fn market_facts(
    p: &Table,
    r: &Table,
    rows: &[&Vec<String>],
    first: &Vec<String>,
    latest: &Vec<String>,
    resolution: Option<&Vec<String>>,
    family: &str,
    variant: &str,
    metas: &[data::VariantMeta],
) -> String {
    let cond = p.cell(latest, "condition_id");
    let token = p.cell(latest, "token_id");
    let mut inner = String::new();
    inner.push_str(&render::row("Strategy", &strategy_links(family, variant, metas)));
    if let Some(m) = metas.iter().find(|m| m.family == family && m.variant == variant) {
        if !m.summary.is_empty() {
            inner.push_str(&format!(
                "<div class=\"row row-block\"><span class=\"k\">What it claims</span><span class=\"v\">{}</span></div>",
                esc(&m.summary)
            ));
        }
    }
    inner.push_str(&render::row(
        "Status",
        &render::status_badge(p.cell(latest, "status")),
    ));
    inner.push_str(&render::row(
        "First logged",
        &format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(p.cell(first, "timestamp")))),
    ));
    inner.push_str(&render::row(
        "Last logged",
        &format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(p.cell(latest, "timestamp")))),
    ));
    inner.push_str(&render::row("Rows", &fmt_int(rows.len() as i64)));
    if !cond.is_empty() {
        inner.push_str(&render::row("Condition id", &render::cell_text(cond)));
    }
    if !token.is_empty() {
        inner.push_str(&render::row("Token id", &render::cell_text(token)));
    }
    match resolution {
        Some(rr) => {
            inner.push_str(&render::row(
                "Resolved",
                &format!(
                    "{} <span class=\"mono\">{}</span>",
                    badge(r.cell(rr, "winning_outcome"), "ok"),
                    esc(r.cell(rr, "resolved_date"))
                ),
            ));
            let note = r.cell(rr, "note");
            if !note.is_empty() {
                inner.push_str(&render::row("Note", &esc(note)));
            }
        }
        None => inner.push_str(&render::row("Resolved", "<span class=\"muted\">open</span>")),
    }

    section(
        "Market facts",
        "identifiers as logged with the prediction",
        "",
        &render::rows(&inner),
    )
}

fn market_log(p: &Table, d: &Table, rows: &[&Vec<String>], slug: &str) -> String {
    let body: Vec<Vec<String>> = rows
        .iter()
        .rev()
        .map(|row| {
            let outcome = p.cell(row, "outcome");
            let ts = p.cell(row, "timestamp");
            let ours = p.num(row, "prediction");
            let mkt = p.num(row, "market_price");
            let score = d.rows.iter().find(|sr| {
                d.cell(sr, "market_slug") == slug
                    && d.cell(sr, "outcome") == outcome
                    && d.cell(sr, "timestamp") == ts
            });
            let (actual, brier, mbrier, imp, horizon) = match score {
                Some(sr) => (
                    d.cell(sr, "actual").to_string(),
                    format!("{:.6}", d.num(sr, "brier")),
                    format!("{:.6}", d.num(sr, "market_brier")),
                    {
                        let v = d.num(sr, "improvement");
                        badge(&fmt_signed(v), if v > 0.0 { "ok" } else { "bad" })
                    },
                    d.cell(sr, "horizon").to_string(),
                ),
                None => (
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "<span class=\"muted\">not scored</span>".to_string(),
                    "—".to_string(),
                ),
            };
            vec![
                format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(ts))),
                esc(outcome),
                fmt_prob(ours),
                fmt_prob(mkt),
                fmt_signed(mkt - ours),
                actual,
                brier,
                mbrier,
                imp,
                esc(&horizon),
                format!("<span class=\"mono\">{}</span>", esc(p.cell(row, "model"))),
            ]
        })
        .collect();

    section(
        "Predictions and scoring",
        "one row per logged prediction, joined with its score",
        &badge(&render::count(rows.len(), "row"), ""),
        &table(
            &[
                ("Logged", ""),
                ("Outcome", ""),
                ("Ours", "num"),
                ("Market", "num"),
                ("Edge", "num"),
                ("Actual", "num"),
                ("Brier", "num"),
                ("Market Brier", "num"),
                ("Improvement", "num"),
                ("Horizon", ""),
                ("Model", ""),
            ],
            &body,
        ),
    )
}
