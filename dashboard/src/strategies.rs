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
    self, badge, chip_row, doc, esc, fmt_int, fmt_prob, fmt_signed, fmt_ts, icon, item, items,
    markdown_body, panel, panel_flush, panel_foot, stat_line, table, table_scroll,
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
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let mut all_live = tree_live && preds.live && scores.live && detail.live && metas_live;

    let p = Table::parse(&preds.text);
    let s = Table::parse(&scores.text);
    let d = Table::parse(&detail.text);
    let families = data::families(&paths);

    let count_status = |want: &str| metas.iter().filter(|m| m.status == want).count();
    let stats = stat_line(&[
        (fmt_int(families.len() as i64), "families".to_string(), ""),
        (fmt_int(metas.len() as i64), "strategies".to_string(), ""),
        (
            fmt_int(count_status("trial") as i64),
            "in trial".to_string(),
            "warn",
        ),
        (fmt_int(count_status("live") as i64), "live".to_string(), "ok"),
        (
            fmt_int(count_status("retired") as i64),
            "dropped".to_string(),
            "",
        ),
        (
            fmt_int(p.rows.len() as i64),
            "predictions logged".to_string(),
            "",
        ),
    ]);

    // One expandable row per family: the family's own thesis line collapsed,
    // its variants (with the plain-English summary that IS the description)
    // revealed on click. Table over cards; expansion over navigation.
    let mut rows = String::from(
        "<div class=\"fhead\"><span>Family</span><span>What it is</span><span>Strategies</span><span>Predictions</span><span>Improvement</span></div>",
    );
    for family in &families {
        let fam_doc = data::text(env, &format!("strategies/{family}/FAMILY.md")).await;
        all_live &= fam_doc.live;
        let mine: Vec<&data::VariantMeta> = metas.iter().filter(|m| &m.family == family).collect();
        let n_pred = p
            .rows
            .iter()
            .filter(|row| p.cell(row, "family") == family.as_str())
            .count();
        let fam_score = s
            .rows
            .iter()
            .find(|row| s.cell(row, "level") == "family" && s.cell(row, "key") == family.as_str());
        let imp_cell = match fam_score {
            Some(row) => {
                let v = s.num(row, "mean_improvement");
                format!(
                    "<span class=\"{}\">{}</span>",
                    if v > 0.0 { "s-ok" } else { "s-bad" },
                    fmt_signed(v)
                )
            }
            None => "<span class=\"muted\">—</span>".to_string(),
        };
        let active = mine.iter().any(|m| m.status == "trial" || m.status == "live");

        let mut vars = String::new();
        for m in &mine {
            let n = p
                .rows
                .iter()
                .filter(|row| {
                    p.cell(row, "family") == m.family && p.cell(row, "variant") == m.variant
                })
                .count();
            let scored = d
                .rows
                .iter()
                .filter(|row| {
                    d.cell(row, "family") == m.family && d.cell(row, "variant") == m.variant
                })
                .count();
            let vscore = s.rows.iter().find(|row| {
                s.cell(row, "level") == "variant" && s.cell(row, "key") == m.key()
            });
            let vimp = match vscore {
                Some(row) => {
                    let v = s.num(row, "mean_improvement");
                    format!(
                        "<span class=\"{}\">{}</span>",
                        if v > 0.0 { "s-ok" } else { "s-bad" },
                        fmt_signed(v)
                    )
                }
                None => "<span class=\"muted\">—</span>".to_string(),
            };
            let when = if m.status == "retired" && !m.retired_on.is_empty() {
                format!("{} → dropped {}", m.created, m.retired_on)
            } else if !m.review_due.is_empty() && m.status == "trial" {
                format!("trial since {} · review {}", m.trial_started, m.review_due)
            } else {
                format!("created {}", m.created)
            };
            vars.push_str(&format!(
                "<div class=\"fvar\"><span class=\"fvar-name\"><a href=\"{href}\">{name}</a><span class=\"fvar-when\">{when}</span></span><span class=\"fsum\">{summary}</span><span class=\"fnum\">{status}</span><span class=\"fnum\">{n}{scored_sub}</span><span class=\"fnum\">{vimp}</span></div>",
                href = esc(&m.href()),
                name = esc(&m.variant),
                when = esc(&when),
                summary = esc(&m.summary),
                status = render::status_badge(&m.status),
                n = n,
                scored_sub = if scored > 0 {
                    format!("<span class=\"fvar-when\"> {scored} scored</span>")
                } else {
                    String::new()
                },
                vimp = vimp,
            ));
        }

        rows.push_str(&format!(
            "<details class=\"frow\"{open}><summary><span class=\"fname\">{family}</span><span class=\"fsum\">{thesis}</span><span class=\"fnum\">{nvar}</span><span class=\"fnum\">{npred}</span><span class=\"fnum\">{imp}</span></summary><div class=\"fvariants\">{vars}<p class=\"note\"><a class=\"link\" href=\"/strategies/{family}\">Family page: the full thesis, cross-variant lessons and scoring →</a></p></div></details>",
            open = if active { " open" } else { "" },
            family = esc(family),
            thesis = esc(&first_paragraph(&fam_doc.text)),
            nvar = mine.len(),
            npred = n_pred,
            imp = imp_cell,
            vars = vars,
        ));
    }

    let body = format!(
        "{banner}{stats}{table}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        table = panel_foot(
            "Strategies",
            "click a family to see its strategies",
            &badge(&render::count(metas.len(), "strategy"), ""),
            &format!("<div class=\"ftable\">{rows}</div>"),
            "<span class=\"mono\">strategies/&lt;family&gt;/&lt;variant&gt;/strategy.toml</span><span>improvement = how much better than the market, per scored prediction</span>",
            true,
        ),
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
/// clipped — used as a family's one-line description.
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
    let clipped: String = buf.chars().take(130).collect();
    if buf.chars().count() > 130 {
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

    let mut metas: Vec<data::VariantMeta> = Vec::new();
    for (_, v) in &variants {
        let f = data::text(env, &format!("strategies/{family}/{v}/strategy.toml")).await;
        all_live &= f.live;
        metas.push(data::parse_variant_meta(family, v, &f.text));
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

    let stats = stat_line(&[
        (fmt_int(variants.len() as i64), "strategies".to_string(), ""),
        (
            fmt_int(fam_rows.len() as i64),
            "predictions".to_string(),
            "",
        ),
        (
            fam_score
                .map(|row| s.cell(row, "n").to_string())
                .unwrap_or_else(|| "0".to_string()),
            "scored".to_string(),
            "",
        ),
        (
            fam_score
                .map(|row| fmt_signed(s.num(row, "mean_improvement")))
                .unwrap_or_else(|| "—".to_string()),
            "better than the market, per prediction".to_string(),
            match fam_score.map(|row| s.num(row, "mean_improvement")) {
                Some(v) if v > 0.0 => "ok",
                Some(_) => "bad",
                None => "",
            },
        ),
    ]);

    let mut variant_items = String::new();
    for m in &metas {
        let n = p
            .rows
            .iter()
            .filter(|row| p.cell(row, "family") == family && p.cell(row, "variant") == m.variant)
            .count();
        let when = if m.status == "retired" && !m.retired_on.is_empty() {
            format!("dropped {} · {} predictions", m.retired_on, n)
        } else if m.status == "trial" && !m.review_due.is_empty() {
            format!("review due {} · {} predictions", m.review_due, n)
        } else {
            format!("created {} · {} predictions", m.created, n)
        };
        variant_items.push_str(&item(
            &m.href(),
            &esc(&m.variant),
            &format!(
                "<span class=\"item-summary\">{}</span>{}",
                esc(&m.summary),
                esc(&when)
            ),
            &render::status_badge(&m.status),
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
        "{banner}{stats}<div class=\"grid-main\">{thesis}{variants}</div>{scoring}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        thesis = panel_foot(
            "The idea in full",
            "what this family trades, and who is on the other side",
            "",
            &format!("<div class=\"prose\">{}</div>", markdown_body(&fam_doc.text)),
            &format!("<span class=\"mono\">strategies/{family}/FAMILY.md</span><a href=\"/predictions\">Predictions →</a>"),
            false,
        ),
        variants = panel(
            "Strategies in this family",
            "each one is a separate experiment with its own clock",
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

    let m = data::parse_variant_meta(family, variant, &toml_doc.text);
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

    let status = m.status.clone();
    let ahead = scored
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();

    // The plain-English summary is the description of this strategy — first
    // thing on the page, above every number and before the runbook prose.
    let summary_html = if m.summary.is_empty() {
        format!(
            "<p class=\"lede lede-missing\">No plain-English summary yet — <span class=\"mono\">{base}/strategy.toml</span> is missing its required <span class=\"mono\">summary</span> field.</p>"
        )
    } else {
        format!("<p class=\"lede\">{}</p>", esc(&m.summary))
    };

    let stats = stat_line(&[
        (
            render::status_badge(&status),
            match status.as_str() {
                "trial" => format!(
                    "on slot {} since {}",
                    m.slot.unwrap_or(0),
                    m.trial_started
                ),
                "retired" => format!("dropped {}", m.retired_on),
                _ => format!("created {}", m.created),
            },
            "",
        ),
        (fmt_int(rows.len() as i64), "predictions".to_string(), ""),
        (
            fmt_int(scored.len() as i64),
            if scored.is_empty() {
                "scored".to_string()
            } else {
                format!("scored, {ahead} beat the market")
            },
            "",
        ),
        (
            var_score
                .map(|row| fmt_signed(s.num(row, "mean_improvement")))
                .unwrap_or_else(|| "—".to_string()),
            "better than the market, per prediction".to_string(),
            match var_score.map(|row| s.num(row, "mean_improvement")) {
                Some(v) if v > 0.0 => "ok",
                Some(_) => "bad",
                None => "",
            },
        ),
        (
            if m.review_due.is_empty() {
                "—".to_string()
            } else {
                m.review_due.clone()
            },
            "trial review due".to_string(),
            if status == "trial" { "warn" } else { "" },
        ),
    ]);

    // --- facts + documents (side panel) ---
    let mut facts = String::new();
    facts.push_str(&render::row("Status", &render::status_badge(&status)));
    facts.push_str(&render::row("Created", &esc(&m.created)));
    if !m.supersedes.is_empty() {
        facts.push_str(&render::row(
            "Replaces",
            &format!(
                "<a href=\"/strategies/{family}/{0}\">{0}</a>",
                esc(&m.supersedes)
            ),
        ));
    }
    facts.push_str(&render::row(
        "Family",
        &format!("<a href=\"/strategies/{0}\">{0}</a>", esc(family)),
    ));
    if !m.trial_started.is_empty() {
        facts.push_str(&render::row("Trial started", &esc(&m.trial_started)));
        facts.push_str(&render::row("Review due", &esc(&m.review_due)));
    }
    if !m.labels.is_empty() {
        facts.push_str(&render::row("Labels", &chip_row(&m.labels)));
    }
    if !m.retire_reason.is_empty() {
        facts.push_str(&format!(
            "<div class=\"row row-block\"><span class=\"k\">Why it was dropped</span><span class=\"v\">{}</span></div>",
            esc(&m.retire_reason)
        ));
    }
    if !m.success_guideline.is_empty() {
        facts.push_str(&format!(
            "<div class=\"row row-block\"><span class=\"k\">What would make it a success</span><span class=\"v\">{}</span></div>",
            esc(&m.success_guideline)
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
            &render::status_badge(&status),
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
        "{banner}{summary}{stats}<div class=\"grid-main\">{thesis}{side}</div>{applications}{preds}{worklog}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        summary = summary_html,
        stats = stats,
        thesis = panel_foot(
            "How it works",
            "the runbook: method, gates, and what would kill it",
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
