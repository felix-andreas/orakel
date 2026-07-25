//! `/strategies` — the research unit of the firm, three levels deep.
//!
//!   /strategies                       every family, with its variants
//!   /strategies/<family>              FAMILY.md, variants, family scoring
//!   /strategies/<family>/<variant>    STRATEGY.md, strategy.toml facts,
//!                                     applications, results, worklog,
//!                                     scoring and the variant's predictions
//!
//! A variant page can also render one of that variant's markdown documents
//! directly: `?doc=results/backtest-2026-07-23.md`.

use crate::data::{self, Table};
use crate::render::{
    self, badge, chip_row, doc, esc, fmt_int, fmt_prob, fmt_signed, fmt_ts, icon, item, items, kpi,
    kpi_row, markdown_body, panel, panel_flush, panel_foot, table, table_scroll,
};
use crate::{shell, snapshot_banner, trail};
use worker::Env;

// ---------------------------------------------------------------------------
// /strategies
// ---------------------------------------------------------------------------

pub async fn index(env: &Env) -> String {
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let state_doc = data::text(env, "ops/state.toml").await;
    let mut all_live = tree_live && preds.live && scores.live && state_doc.live;

    let p = Table::parse(&preds.text);
    let s = Table::parse(&scores.text);
    let state = data::toml_of(&state_doc.text);
    let families = data::families(&paths);
    let variants = data::variants(&paths);

    // Per-variant facts come from each strategy.toml.
    let mut metas: Vec<(String, String, toml::Table)> = Vec::new();
    for (family, variant) in &variants {
        let f = data::text(env, &format!("strategies/{family}/{variant}/strategy.toml")).await;
        all_live &= f.live;
        metas.push((family.clone(), variant.clone(), data::toml_of(&f.text)));
    }

    let count_status = |want: &str| {
        metas
            .iter()
            .filter(|(_, _, t)| data::tstr(t, "status") == want)
            .count()
    };

    let kpis = kpi_row(&[
        kpi("Families", &fmt_int(families.len() as i64), "layers", 4)
            .context("a thesis shared by its variants"),
        kpi("Variants", &fmt_int(variants.len() as i64), "flask", 1)
            .context("the research unit — one folder each"),
        kpi("In trial", &fmt_int(count_status("trial") as i64), "clock", 3)
            .delta(
                &format!(
                    "{} of {} slots",
                    data::int_at(&state, &["research", "slots_active"]).unwrap_or(0),
                    data::int_at(&state, &["research", "slots_total"]).unwrap_or(0)
                ),
                "warn",
            )
            .context("running on a slot clock"),
        kpi("Retired", &fmt_int(count_status("retired") as i64), "check", 2)
            .context("folders stay, post-mortems in results/"),
    ]);

    let mut cards = String::new();
    for family in &families {
        let fam_doc = data::text(env, &format!("strategies/{family}/FAMILY.md")).await;
        all_live &= fam_doc.live;
        let summary = first_paragraph(&fam_doc.text);

        let mut inner = String::new();
        for (f, v, t) in metas.iter().filter(|(f, _, _)| f == family) {
            let status = data::tstr(t, "status");
            let n = p
                .rows
                .iter()
                .filter(|row| p.cell(row, "family") == *f && p.cell(row, "variant") == *v)
                .count();
            let score = s
                .rows
                .iter()
                .find(|row| s.cell(row, "level") == "variant" && s.cell(row, "key") == format!("{f}/{v}"));
            let trailing = match score {
                Some(row) => {
                    let imp = s.num(row, "mean_improvement");
                    badge(&fmt_signed(imp), if imp > 0.0 { "ok" } else { "bad" })
                }
                None => render::status_badge(status),
            };
            inner.push_str(&item(
                &format!("/strategies/{f}/{v}"),
                &esc(v),
                &esc(&format!(
                    "{} · created {} · {} predictions",
                    status,
                    data::tstr(t, "created"),
                    n
                )),
                &trailing,
            ));
        }
        if inner.is_empty() {
            inner.push_str(&item("", "No variants yet", "", ""));
        }

        cards.push_str(&panel_foot(
            family,
            &summary,
            &badge(
                &render::count(metas.iter().filter(|(f, _, _)| f == family).count(), "variant"),
                "",
            ),
            &items(&inner),
            &format!(
                "<span class=\"mono\">strategies/{family}/</span><a href=\"/strategies/{family}\">Family overview →</a>"
            ),
            false,
        ));
    }

    let body = format!(
        "{}{}<div class=\"grid-pair\">{}</div>",
        if all_live { String::new() } else { snapshot_banner() },
        kpis,
        cards
    );

    shell(
        env,
        "/strategies",
        trail(&[("Research", ""), ("Strategies", "")]),
        all_live,
        &body,
    )
    .await
}

/// First non-heading paragraph of a markdown file, collapsed to one line and
/// clipped — used as a panel subtitle.
fn first_paragraph(src: &str) -> String {
    let mut buf = String::new();
    for line in src.lines() {
        let l = line.trim();
        if l.is_empty() {
            if !buf.is_empty() {
                break;
            }
            continue;
        }
        if l.starts_with('#') {
            continue;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(l);
    }
    // The subtitle is plain text: drop the markdown syntax that would
    // otherwise show up literally (**bold**, `code`, [text](link)).
    let buf = buf
        .replace("**", "")
        .replace('`', "")
        .replace('*', "")
        .replace('_', "");
    let buf = strip_links(&buf);
    let clipped: String = buf.chars().take(150).collect();
    if buf.chars().count() > 150 {
        format!("{}…", clipped.trim_end())
    } else {
        clipped
    }
}

/// `[text](href)` → `text`.
fn strip_links(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](") else { break };
        let Some(end) = rest[open + close..].find(')') else { break };
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..open + close]);
        rest = &rest[open + close + end..];
        rest = rest.strip_prefix(')').unwrap_or(rest);
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// /strategies/<family>
// ---------------------------------------------------------------------------

pub async fn family(env: &Env, family: &str) -> String {
    let crumbs = trail(&[
        ("Research", ""),
        ("Strategies", "/strategies"),
        (family, ""),
    ]);
    if !data::safe_segment(family) {
        return shell(
            env,
            "/strategies",
            crumbs,
            true,
            &render::empty_state("Unknown family", ""),
        )
        .await;
    }

    let fam_doc = data::text(env, &format!("strategies/{family}/FAMILY.md")).await;
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let mut all_live = fam_doc.live && tree_live && preds.live && scores.live;

    if fam_doc.is_empty() {
        let body = panel(
            "Family not found",
            &format!("strategies/{family}/FAMILY.md"),
            "",
            &render::empty_state(
                "Nothing here",
                "<div>No such family in the repo. <a class=\"link\" href=\"/strategies\">Back to strategies</a>.</div>",
            ),
        );
        return shell(env, "/strategies", crumbs, all_live, &body).await;
    }

    let p = Table::parse(&preds.text);
    let s = Table::parse(&scores.text);
    let variants: Vec<(String, String)> = data::variants(&paths)
        .into_iter()
        .filter(|(f, _)| f == family)
        .collect();

    let mut metas: Vec<(String, toml::Table)> = Vec::new();
    for (_, v) in &variants {
        let f = data::text(env, &format!("strategies/{family}/{v}/strategy.toml")).await;
        all_live &= f.live;
        metas.push((v.clone(), data::toml_of(&f.text)));
    }

    let fam_rows: Vec<&Vec<String>> = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "family") == family)
        .collect();
    let fam_score = s
        .rows
        .iter()
        .find(|row| s.cell(row, "level") == "family" && s.cell(row, "key") == family);

    let kpis = kpi_row(&[
        kpi("Variants", &fmt_int(variants.len() as i64), "flask", 1).context(
            &metas
                .iter()
                .map(|(v, t)| format!("{v} ({})", data::tstr(t, "status")))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        kpi("Predictions", &fmt_int(fam_rows.len() as i64), "list", 4).context("logged by this family"),
        kpi(
            "Scored",
            &fam_score
                .map(|row| s.cell(row, "n").to_string())
                .unwrap_or_else(|| "0".to_string()),
            "check",
            2,
        )
        .context("rows with a resolution"),
        kpi(
            "Mean improvement",
            &fam_score
                .map(|row| fmt_signed(s.num(row, "mean_improvement")))
                .unwrap_or_else(|| "—".to_string()),
            "trend-up",
            2,
        )
        .delta(
            match fam_score.map(|row| s.num(row, "mean_improvement")) {
                Some(v) if v > 0.0 => "beats market",
                Some(_) => "behind market",
                None => "not scored yet",
            },
            match fam_score.map(|row| s.num(row, "mean_improvement")) {
                Some(v) if v > 0.0 => "up",
                Some(_) => "down",
                None => "",
            },
        )
        .context("market Brier minus ours"),
    ]);

    let mut variant_items = String::new();
    for (v, t) in &metas {
        let status = data::tstr(t, "status");
        let n = p
            .rows
            .iter()
            .filter(|row| p.cell(row, "family") == family && p.cell(row, "variant") == *v)
            .count();
        let supersedes = data::tstr(t, "supersedes");
        let sub = if supersedes.is_empty() {
            format!("created {} · {} predictions", data::tstr(t, "created"), n)
        } else {
            format!(
                "created {} · supersedes {} · {} predictions",
                data::tstr(t, "created"),
                supersedes,
                n
            )
        };
        variant_items.push_str(&item(
            &format!("/strategies/{family}/{v}"),
            &esc(v),
            &esc(&sub),
            &render::status_badge(status),
        ));
    }

    let scoring_rows: Vec<&Vec<String>> = s
        .rows
        .iter()
        .filter(|row| {
            (s.cell(row, "level") == "family" && s.cell(row, "key") == family)
                || (s.cell(row, "level") == "variant"
                    && s.cell(row, "key").starts_with(&format!("{family}/")))
        })
        .collect();

    let scoring = if scoring_rows.is_empty() {
        panel(
            "Scoring",
            "aggregates for this family",
            "",
            &render::empty_state("Nothing scored yet", ""),
        )
    } else {
        let body: Vec<Vec<String>> = scoring_rows
            .iter()
            .map(|row| {
                let imp = s.num(row, "mean_improvement");
                vec![
                    esc(s.cell(row, "level")),
                    esc(s.cell(row, "key")),
                    s.cell(row, "n").to_string(),
                    badge(&fmt_signed(imp), if imp > 0.0 { "ok" } else { "bad" }),
                    format!("{:.4}", s.num(row, "mean_brier")),
                    format!("{:.4}", s.num(row, "mean_market_brier")),
                    format!("{:.4}", s.num(row, "mean_logloss")),
                ]
            })
            .collect();
        panel_flush(
            "Scoring",
            "aggregates for this family and its variants",
            &badge(&render::count(scoring_rows.len(), "row"), ""),
            &table(
                &[
                    ("Level", ""),
                    ("Key", ""),
                    ("n", "num"),
                    ("Improvement", "num"),
                    ("Brier", "num"),
                    ("Market Brier", "num"),
                    ("Log loss", "num"),
                ],
                &body,
            ),
        )
    };

    let body = format!(
        "{banner}{kpis}<div class=\"grid-main\">{thesis}{variants}</div>{scoring}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        kpis = kpis,
        thesis = panel_foot(
            "Thesis",
            "what this family trades, and who is on the other side",
            "",
            &format!("<div class=\"prose\">{}</div>", markdown_body(&fam_doc.text)),
            &format!("<span class=\"mono\">strategies/{family}/FAMILY.md</span><a href=\"/predictions\">Predictions →</a>"),
            false,
        ),
        variants = panel(
            "Variants",
            "each one is a separate research unit",
            &badge(&render::count(variants.len(), "variant"), ""),
            &items(&variant_items),
        ),
        scoring = scoring,
    );

    shell(env, "/strategies", crumbs, all_live, &body).await
}

// ---------------------------------------------------------------------------
// /strategies/<family>/<variant>
// ---------------------------------------------------------------------------

pub async fn variant(env: &Env, family: &str, variant: &str, doc_path: Option<String>) -> String {
    let crumbs = trail(&[
        ("Research", ""),
        ("Strategies", "/strategies"),
        (family, &format!("/strategies/{family}")),
        (variant, ""),
    ]);
    if !data::safe_segment(family) || !data::safe_segment(variant) {
        return shell(
            env,
            "/strategies",
            crumbs,
            true,
            &render::empty_state("Unknown variant", ""),
        )
        .await;
    }
    let base = format!("strategies/{family}/{variant}");

    // ?doc=<relative path> renders one of the variant's markdown documents.
    if let Some(rel) = doc_path {
        return variant_doc(env, family, variant, &base, &rel).await;
    }

    let strategy_md = data::text(env, &format!("{base}/STRATEGY.md")).await;
    let toml_doc = data::text(env, &format!("{base}/strategy.toml")).await;
    let worklog = data::text(env, &format!("{base}/memory/WORKLOG.md")).await;
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let mut all_live = strategy_md.live
        && toml_doc.live
        && worklog.live
        && tree_live
        && preds.live
        && scores.live
        && detail.live;

    if strategy_md.is_empty() && toml_doc.is_empty() {
        let body = panel(
            "Variant not found",
            &base,
            "",
            &render::empty_state(
                "Nothing here",
                "<div>No such variant in the repo. <a class=\"link\" href=\"/strategies\">Back to strategies</a>.</div>",
            ),
        );
        return shell(env, "/strategies", crumbs, all_live, &body).await;
    }

    let meta = data::toml_of(&toml_doc.text);
    let p = Table::parse(&preds.text);
    let s = Table::parse(&scores.text);
    let d = Table::parse(&detail.text);
    let key = format!("{family}/{variant}");

    let rows: Vec<&Vec<String>> = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "family") == family && p.cell(row, "variant") == variant)
        .collect();
    let scored: Vec<&Vec<String>> = d
        .rows
        .iter()
        .filter(|row| d.cell(row, "family") == family && d.cell(row, "variant") == variant)
        .collect();
    let var_score = s
        .rows
        .iter()
        .find(|row| s.cell(row, "level") == "variant" && s.cell(row, "key") == key);

    let status = data::tstr(&meta, "status");
    let started = data::str_at(&meta, &["trial", "started"]);
    let review = data::str_at(&meta, &["trial", "review_due"]);
    let ahead = scored
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();

    let kpis = kpi_row(&[
        kpi("Status", status, "flask", 1)
            .delta(
                &format!("slot {}", data::int_at(&meta, &["trial", "slot"]).unwrap_or(0)),
                if status == "trial" { "warn" } else { "" },
            )
            .context(&format!("created {}", data::tstr(&meta, "created"))),
        kpi("Predictions", &fmt_int(rows.len() as i64), "list", 4).context(&format!(
            "{} markets",
            {
                let mut m: Vec<&str> = rows.iter().map(|row| p.cell(row, "market_slug")).collect();
                m.sort_unstable();
                m.dedup();
                m.len()
            }
        )),
        kpi("Scored", &fmt_int(scored.len() as i64), "check", 2)
            .delta(
                &format!("{ahead} ahead"),
                if !scored.is_empty() && ahead == scored.len() { "up" } else { "" },
            )
            .context("rows with a resolution"),
        kpi(
            "Mean improvement",
            &var_score
                .map(|row| fmt_signed(s.num(row, "mean_improvement")))
                .unwrap_or_else(|| "—".to_string()),
            "trend-up",
            2,
        )
        .context(
            &var_score
                .map(|row| {
                    format!(
                        "Brier {:.4} vs market {:.4}",
                        s.num(row, "mean_brier"),
                        s.num(row, "mean_market_brier")
                    )
                })
                .unwrap_or_else(|| "no resolutions yet".to_string()),
        ),
        kpi(
            "Trial review",
            if review.is_empty() { "—" } else { review },
            "clock",
            3,
        )
        .context(&if started.is_empty() {
            "not on a trial clock".to_string()
        } else {
            format!("started {started}")
        }),
    ]);

    // --- facts + documents (side panel) ---
    let mut facts = String::new();
    facts.push_str(&render::row("Status", &render::status_badge(status)));
    facts.push_str(&render::row("Created", &esc(data::tstr(&meta, "created"))));
    let supersedes = data::tstr(&meta, "supersedes");
    if !supersedes.is_empty() {
        facts.push_str(&render::row(
            "Supersedes",
            &format!(
                "<a href=\"/strategies/{family}/{0}\">{0}</a>",
                esc(supersedes)
            ),
        ));
    }
    facts.push_str(&render::row(
        "Family",
        &format!("<a href=\"/strategies/{0}\">{0}</a>", esc(family)),
    ));
    if !started.is_empty() {
        facts.push_str(&render::row("Trial started", &esc(started)));
        facts.push_str(&render::row("Review due", &esc(review)));
    }
    let labels = data::tlist(&meta, "labels");
    if !labels.is_empty() {
        facts.push_str(&render::row("Labels", &chip_row(&labels)));
    }
    let guideline = data::str_at(&meta, &["trial", "success_guideline"]);
    if !guideline.is_empty() {
        facts.push_str(&format!(
            "<div class=\"row row-block\"><span class=\"k\">Success guideline</span><span class=\"v\">{}</span></div>",
            esc(guideline)
        ));
    }

    // Documents: results/*.md, memory, applications — linked, not inlined.
    let results = data::files_in(&paths, &format!("{base}/results"), ".md");
    let mut doc_links = String::new();
    for r in &results {
        let name = r.rsplit('/').next().unwrap_or(r);
        doc_links.push_str(&format!(
            "<li><a href=\"/strategies/{family}/{variant}?doc=results/{0}\">{icon} {0}</a></li>",
            esc(name),
            icon = icon("book")
        ));
    }
    for (label, rel) in [
        ("STRATEGY.md", "STRATEGY.md"),
        ("memory/MEMORY.md", "memory/MEMORY.md"),
        ("memory/WORKLOG.md", "memory/WORKLOG.md"),
    ] {
        doc_links.push_str(&format!(
            "<li><a href=\"/strategies/{family}/{variant}?doc={rel}\">{icon} {label}</a></li>",
            icon = icon("book")
        ));
    }

    let side = format!(
        "{}{}",
        panel(
            "Facts",
            "status, trial clock and labels as recorded in the repo",
            &render::status_badge(status),
            &render::rows(&facts),
        ),
        panel(
            "Documents",
            "the variant's own write-ups",
            &badge(&render::count(results.len() + 3, "document"), ""),
            &format!("<ul class=\"link-list\">{doc_links}</ul>"),
        )
    );

    // --- applications ---
    let app_paths = data::files_in(&paths, &format!("{base}/applications"), ".toml");
    let mut app_rows: Vec<Vec<String>> = Vec::new();
    for path in app_paths.iter().take(24) {
        let f = data::text(env, path).await;
        all_live &= f.live;
        let t = data::toml_of(&f.text);
        let name = path.rsplit('/').next().unwrap_or(path);
        let slug = data::tstr(&t, "market_slug");
        let legs = data::arr_at(&t, &["legs"]).len();
        let active = data::tbool(&t, "active");
        app_rows.push(vec![
            format!("<span class=\"mono\">{}</span>", esc(name)),
            if slug.is_empty() {
                "<span class=\"muted\">—</span>".to_string()
            } else {
                format!("<a href=\"/markets/{0}\">{0}</a>", esc(slug))
            },
            esc(data::tstr(&t, "added")),
            match active {
                Some(true) => badge("active", "ok"),
                Some(false) => badge("inactive", ""),
                None => "<span class=\"muted\">—</span>".to_string(),
            },
            if legs > 0 {
                fmt_int(legs as i64)
            } else {
                "<span class=\"muted\">—</span>".to_string()
            },
            format!(
                "<span class=\"muted\">{}</span>",
                esc(data::str_at(&t, &["params", "asset"]))
            ),
            format!(
                "<span class=\"muted\">{}</span>",
                esc(data::str_at(&t, &["params", "tier"]))
            ),
        ]);
    }
    let applications = if app_rows.is_empty() {
        panel(
            "Applications",
            "the markets this variant is pointed at",
            "",
            &render::empty_state(
                "No applications",
                "<div>Nothing under <span class=\"mono\">applications/</span> yet.</div>",
            ),
        )
    } else {
        panel_flush(
            "Applications",
            "the boards and markets this variant is pointed at",
            &badge(&render::count(app_paths.len(), "file"), ""),
            &table(
                &[
                    ("File", ""),
                    ("Market", ""),
                    ("Added", ""),
                    ("Active", ""),
                    ("Legs", "num"),
                    ("Asset", ""),
                    ("Tier", ""),
                ],
                &app_rows,
            ),
        )
    };

    // --- this variant's predictions ---
    let pred_panel = if rows.is_empty() {
        panel(
            "Predictions",
            "rows logged by this variant",
            "",
            &render::empty_state("Nothing logged yet", ""),
        )
    } else {
        let body: Vec<Vec<String>> = rows
            .iter()
            .rev()
            .map(|row| {
                let slug = p.cell(row, "market_slug");
                let ts = p.cell(row, "timestamp");
                let outcome = p.cell(row, "outcome");
                let score = d.rows.iter().find(|sr| {
                    d.cell(sr, "market_slug") == slug
                        && d.cell(sr, "outcome") == outcome
                        && d.cell(sr, "timestamp") == ts
                });
                let ours = p.num(row, "prediction");
                let mkt = p.num(row, "market_price");
                vec![
                    format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(ts))),
                    format!("<a href=\"/markets/{0}\">{0}</a>", esc(slug)),
                    esc(outcome),
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
                    format!("<span class=\"mono\">{}</span>", esc(p.cell(row, "model"))),
                ]
            })
            .collect();
        panel_flush(
            "Predictions",
            "rows logged by this variant, newest first",
            &badge(&render::count(rows.len(), "row"), ""),
            &table_scroll(
                &[
                    ("Logged", ""),
                    ("Market", ""),
                    ("Outcome", ""),
                    ("Ours", "num"),
                    ("Market", "num"),
                    ("Edge", "num"),
                    ("Improvement", ""),
                    ("Model", ""),
                ],
                &body,
            ),
        )
    };

    let worklog_panel = if worklog.is_empty() {
        String::new()
    } else {
        doc(
            "Worklog",
            &esc(&format!("{base}/memory/WORKLOG.md")),
            &format!(
                "<div class=\"prose prose-wide\">{}</div>",
                markdown_body(&worklog.text)
            ),
            false,
        )
    };

    let body = format!(
        "{banner}{kpis}<div class=\"grid-main\">{thesis}{side}</div>{applications}{preds}{worklog}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        kpis = kpis,
        thesis = panel_foot(
            "Strategy",
            "how this variant models the market and what would kill it",
            "",
            &format!("<div class=\"prose\">{}</div>", markdown_body(&strategy_md.text)),
            &format!("<span class=\"mono\">{base}/STRATEGY.md</span><a href=\"/strategies/{family}\">Family →</a>"),
            false,
        ),
        side = format!("<div class=\"stack\">{side}</div>"),
        applications = applications,
        preds = pred_panel,
        worklog = worklog_panel,
    );

    shell(env, "/strategies", crumbs, all_live, &body).await
}

/// `?doc=` view: one markdown document belonging to a variant.
async fn variant_doc(env: &Env, family: &str, variant: &str, base: &str, rel: &str) -> String {
    let ok = data::safe_path(rel)
        && rel.ends_with(".md")
        && (rel.starts_with("results/") || rel.starts_with("memory/") || rel == "STRATEGY.md");
    let crumbs = trail(&[
        ("Research", ""),
        ("Strategies", "/strategies"),
        (family, &format!("/strategies/{family}")),
        (variant, &format!("/strategies/{family}/{variant}")),
        (rel, ""),
    ]);
    if !ok {
        return shell(
            env,
            "/strategies",
            crumbs,
            true,
            &render::empty_state("Not a document of this variant", ""),
        )
        .await;
    }

    let f = data::text(env, &format!("{base}/{rel}")).await;
    let body = if f.is_empty() {
        panel(
            rel,
            &format!("{base}/{rel}"),
            "",
            &render::empty_state("Not found", ""),
        )
    } else {
        panel_foot(
            &render::md_title(&f.text).unwrap_or_else(|| rel.to_string()),
            &format!("{base}/{rel}"),
            "",
            &format!(
                "<div class=\"prose prose-wide\">{}</div>",
                markdown_body(&f.text)
            ),
            &format!(
                "<span class=\"mono\">{base}/{rel}</span><a href=\"/strategies/{family}/{variant}\">← back to {variant}</a>"
            ),
            false,
        )
    };
    shell(env, "/strategies", crumbs, f.live, &body).await
}
