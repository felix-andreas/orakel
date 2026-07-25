//! `/strategies` — the research unit of the firm, three levels deep.
//!
//!   /strategies                       every family, with its variants
//!   /strategies/<family>              the family: what it is, its variants,
//!                                     FAMILY.md, its predictions
//!   /strategies/<family>/<variant>    the strategy: what it is, how it works,
//!                                     its results, predictions and logs
//!
//! Both detail pages are TABBED. The tabs are a secondary bar under the top bar
//! (`render::tabbar`, rendered into `layout`'s subbar slot) and every tab is a
//! real URL: `?tab=<key>`, with the default tab carrying no parameter at all so
//! the bare address stays the canonical one. Server-rendered — a fresh load of
//! `?tab=results` returns the results; nothing is hidden client-side.
//!
//! Why a query parameter and not a path segment: `/strategies/<f>/<seg>` is
//! ALREADY the variant route, so `/strategies/barrier-touch/results` could not
//! be told apart from a variant called `results`. One scheme at both levels
//! beats two.
//!
//! A tab that would be empty is not rendered, so the bar only ever offers what
//! the repo actually has. The legacy `?doc=<rel>` deep link redirects to
//! whichever tab now holds that document (`doc_target`, wired up in lib.rs).

use crate::data::{self, Table};
use crate::render::{
    self, badge, chip_row, esc, fmt_int, fmt_prob, fmt_signed, fmt_ts, icon, item, items,
    markdown_body, section, section_foot, stat_line, table, table_scroll,
};
use crate::{shell, shell_sub, snapshot_banner, trail};
use worker::Env;

/// Where a repo file can be read in full — for result artifacts the dashboard
/// cannot render itself.
const REPO_BLOB: &str = "https://github.com/felix-andreas/orakel/blob/main";

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
            if count_status("trial") > 0 { "warn" } else { "" },
        ),
        (
            fmt_int(count_status("live") as i64),
            "live".to_string(),
            if count_status("live") > 0 { "ok" } else { "" },
        ),
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
    //
    // The family NAME is a link to the family page; the rest of the row is the
    // disclosure. An <a> is itself an activation target, so a click on it
    // navigates and does NOT toggle the <details>, while a click anywhere else
    // in the summary toggles and does not navigate. No JavaScript involved.
    let mut rows = String::from(
        "<div class=\"fhead\"><span>Family</span><span>What it does</span><span>Status</span><span>Predictions</span><span>vs market</span></div>",
    );
    for family in &families {
        let fam_doc = data::text(env, &format!("strategies/{family}/FAMILY.md")).await;
        all_live &= fam_doc.live;
        let mine: Vec<&data::VariantMeta> = metas.iter().filter(|m| &m.family == family).collect();
        // Resolved below; a variant row repeats nothing the family row said.
        let family_line = mine
            .iter()
            .find(|m| m.status == "trial" || m.status == "live")
            .or_else(|| mine.iter().max_by(|a, b| a.created.cmp(&b.created)))
            .map(|m| m.summary.clone())
            .unwrap_or_default();
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
            let summary_cell = if m.summary == family_line {
                String::new() // already said on the family row
            } else {
                esc(&m.summary)
            };
            vars.push_str(&format!(
                "<div class=\"fvar\"><span class=\"fvar-name\"><a href=\"{href}\">{name}</a><span class=\"fvar-when\">{when}</span></span><span class=\"fsum\">{summary}</span><span class=\"fnum\">{status}</span><span class=\"fnum\">{n}{scored_sub}</span><span class=\"fnum\">{vimp}</span></div>",
                href = esc(&m.href()),
                name = esc(&m.variant),
                when = esc(&when),
                summary = summary_cell,
                status = render::status_badge(&m.status),
                n = n,
                scored_sub = if scored > 0 {
                    format!("<div class=\"fvar-when\">{scored} scored</div>")
                } else {
                    String::new()
                },
                vimp = vimp,
            ));
        }

        // The family line must be plain English. FAMILY.md is written in the
        // firm's own vocabulary, so the description shown here is the REQUIRED
        // summary of the family's current strategy (its trialling/live variant,
        // else its newest), falling back to the family's own plain-English
        // opener. FAMILY.md itself is one click away, in full.
        let what = if family_line.is_empty() {
            let (opener, _) = plain_english(&fam_doc.text);
            clip(&opener, 130)
        } else {
            family_line.clone()
        };
        let status_cell = if mine.is_empty() {
            "<span class=\"muted\">—</span>".to_string()
        } else {
            let n_trial = mine.iter().filter(|m| m.status == "trial").count();
            let n_live = mine.iter().filter(|m| m.status == "live").count();
            if n_live > 0 {
                badge(&format!("{n_live} live"), "ok")
            } else if n_trial > 0 {
                badge(&format!("{n_trial} in trial"), "warn")
            } else {
                badge("dropped", "")
            }
        };

        rows.push_str(&format!(
            "<details class=\"frow\"{open}><summary><span class=\"fname\"><a class=\"flink\" href=\"/strategies/{family}\">{family}</a><span class=\"fvar-when\">{nvar}</span></span><span class=\"fsum\">{what}</span><span class=\"fnum\">{status}</span><span class=\"fnum\">{npred}</span><span class=\"fnum\">{imp}</span></summary><div class=\"fvariants\">{vars}</div></details>",
            open = if active { " open" } else { "" },
            family = esc(family),
            nvar = render::count(mine.len(), "strategy"),
            what = esc(&what),
            status = status_cell,
            npred = n_pred,
            imp = imp_cell,
            vars = vars,
        ));
    }

    let body = format!(
        "{banner}{stats}{table}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        table = section_foot(
            "Families",
            "click a family's name to open it, anywhere else on the row to see the strategies inside",
            "",
            &format!("<div class=\"ftable\">{rows}</div>"),
            "<span class=\"mono\">strategies/&lt;family&gt;/&lt;variant&gt;/strategy.toml</span><span>improvement = how much better than the market's quoted price, per scored prediction — being right is not the same as being able to trade on it, so open a strategy for its reachable count</span>"
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

// ---------------------------------------------------------------------------
// Plain-English text helpers
// ---------------------------------------------------------------------------

/// Markdown syntax stripped, for text displayed as plain prose (**bold**,
/// `code` and [text](link) would otherwise show up literally).
fn plain_text(src: &str) -> String {
    let flat = src
        .replace("**", "")
        .replace('`', "")
        .replace('*', "")
        .replace('_', "");
    strip_links(&flat)
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

fn clip(src: &str, max: usize) -> String {
    let clipped: String = src.chars().take(max).collect();
    if src.chars().count() > max {
        format!("{}…", clipped.trim_end())
    } else {
        clipped
    }
}

/// Split a FAMILY.md into (plain-English lede, the rest of the document).
///
/// Every FAMILY.md opens with a `> **In plain English:** …` blockquote — the
/// one paragraph a reader with no prior knowledge can start from (PRINCIPLES:
/// self-contained, no jargon). It is the family page's lede, so it is lifted
/// OUT of the document rather than printed twice.
fn plain_english(src: &str) -> (String, String) {
    const MARK: &str = "**In plain English:**";
    let mut lede = String::new();
    let mut rest = String::new();
    let mut taking = false;
    for line in src.lines() {
        let t = line.trim_start();
        if !taking && t.starts_with('>') && t.contains(MARK) {
            taking = true;
        }
        if taking {
            if t.starts_with('>') {
                let body = t.trim_start_matches('>').trim();
                let body = body.strip_prefix(MARK).unwrap_or(body).trim();
                if !body.is_empty() {
                    if !lede.is_empty() {
                        lede.push(' ');
                    }
                    lede.push_str(body);
                }
                continue;
            }
            taking = false;
            if line.trim().is_empty() {
                continue; // the blank line the blockquote left behind
            }
        }
        rest.push_str(line);
        rest.push('\n');
    }
    (plain_text(&lede), rest)
}

/// The plain-English description of a thing, with the typographic prominence it
/// deserves: it is the most important text on the page, above every number.
/// `missing_html` is what to say when the repo has not got one yet.
fn lede(text: &str, missing_html: &str) -> String {
    if text.trim().is_empty() {
        format!("<p class=\"lede lede-missing\">{missing_html}</p>")
    } else {
        format!("<p class=\"lede\">{}</p>", esc(text.trim()))
    }
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

/// A tab that has something to show. `key` is the `?tab=` value; "" is the
/// default tab and carries no query parameter.
struct Tab {
    key: &'static str,
    label: &'static str,
    /// Shown small beside the label; "" for none.
    count: String,
}

fn tab(key: &'static str, label: &'static str) -> Tab {
    Tab { key, label, count: String::new() }
}

fn tab_n(key: &'static str, label: &'static str, n: usize) -> Tab {
    Tab { key, label, count: n.to_string() }
}

/// The tab actually being shown: the requested one when it exists, else the
/// default. An unknown `?tab=` therefore lands on the page, never on an error.
fn active_tab<'a>(tabs: &'a [Tab], want: &Option<String>) -> &'a str {
    let want = want.as_deref().unwrap_or("");
    tabs.iter()
        .find(|t| t.key == want)
        .map(|t| t.key)
        .unwrap_or("")
}

/// A tab's label, for the breadcrumb ("" for the default tab, which the
/// breadcrumb does not name — the page itself is the overview).
fn tab_label<'a>(tabs: &'a [Tab], key: &str) -> &'a str {
    if key.is_empty() {
        return "";
    }
    tabs.iter()
        .find(|t| t.key == key)
        .map(|t| t.label)
        .unwrap_or("")
}

fn tabbar(base: &str, tabs: &[Tab], active: &str) -> String {
    let entries: Vec<(&str, &str, String)> = tabs
        .iter()
        .map(|t| (t.key, t.label, t.count.clone()))
        .collect();
    render::tabbar(base, &entries, active)
}

/// Which tab now holds the document an old `?doc=<rel>` link pointed at, and
/// the anchor within it (the results tab can hold several documents).
pub fn doc_target(rel: &str) -> (String, String) {
    if rel == "STRATEGY.md" {
        ("how-it-works".to_string(), String::new())
    } else if rel.starts_with("memory/") {
        ("logs".to_string(), String::new())
    } else if let Some(name) = rel.strip_prefix("results/") {
        ("results".to_string(), anchor(name))
    } else {
        (String::new(), String::new())
    }
}

/// A file name → an HTML id: `backtest-2026-07-23.md` → `backtest-2026-07-23`.
fn anchor(name: &str) -> String {
    let stem = name.strip_suffix(".md").unwrap_or(name);
    stem.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

// ---------------------------------------------------------------------------
// /strategies/<family>
// ---------------------------------------------------------------------------

pub async fn family(env: &Env, family: &str, want_tab: Option<String>) -> String {
    let page = format!("/strategies/{family}");
    let crumbs_for = |leaf: &str| -> Vec<render::Crumb> {
        let mut parts: Vec<(&str, &str)> = vec![("Research", ""), ("Strategies", "/strategies")];
        if leaf.is_empty() {
            parts.push((family, ""));
        } else {
            parts.push((family, page.as_str()));
            parts.push((leaf, ""));
        }
        trail(&parts)
    };

    if !data::safe_segment(family) {
        return shell(
            env,
            "/strategies",
            crumbs_for(""),
            true,
            &render::empty_state("Unknown family", ""),
        )
        .await;
    }

    let fam_doc = data::text(env, &format!("strategies/{family}/FAMILY.md")).await;
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let mut all_live =
        fam_doc.live && tree_live && preds.live && scores.live && detail.live;

    if fam_doc.is_empty() {
        let body = section(
            "Family not found",
            &format!("strategies/{family}/FAMILY.md"),
            "",
            &render::empty_state(
                "Nothing here",
                "<div>No such family in the repo. <a class=\"link\" href=\"/strategies\">Back to strategies</a>.</div>",
            ),
        );
        return shell(env, "/strategies", crumbs_for(""), all_live, &body).await;
    }

    let p = Table::parse(&preds.text);
    let s = Table::parse(&scores.text);
    let d = Table::parse(&detail.text);
    let variants: Vec<(String, String)> = data::variants(&paths)
        .into_iter()
        .filter(|(f, _)| f == family)
        .collect();

    let fam_rows: Vec<&Vec<String>> = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "family") == family)
        .collect();
    let scoring_rows: Vec<&Vec<String>> = s
        .rows
        .iter()
        .filter(|row| {
            (s.cell(row, "level") == "family" && s.cell(row, "key") == family)
                || (s.cell(row, "level") == "variant"
                    && s.cell(row, "key").starts_with(&format!("{family}/")))
        })
        .collect();

    let (fam_lede, fam_rest) = plain_english(&fam_doc.text);
    let fam_prose = markdown_body(&fam_rest);

    // --- which tabs have something in them --------------------------------
    let mut tabs: Vec<Tab> = vec![tab("", "Overview")];
    if !fam_prose.trim().is_empty() {
        tabs.push(tab("how-it-works", "How it works"));
    }
    if !fam_rows.is_empty() || !scoring_rows.is_empty() {
        tabs.push(tab_n("predictions", "Predictions", fam_rows.len()));
    }
    let active = active_tab(&tabs, &want_tab).to_string();
    let bar = tabbar(&page, &tabs, &active);
    let crumbs = crumbs_for(tab_label(&tabs, &active));

    let fam_score = s
        .rows
        .iter()
        .find(|row| s.cell(row, "level") == "family" && s.cell(row, "key") == family);
    let stats = stat_line(&[
        (
            fmt_int(variants.len() as i64),
            if variants.len() == 1 { "strategy" } else { "strategies" }.to_string(),
            "",
        ),
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

    // --- the active tab's content -----------------------------------------
    let content = match active.as_str() {
        "how-it-works" => section_foot(
            "",
            "",
            "",
            &format!("<div class=\"prose\">{fam_prose}</div>"),
            &format!(
                "<span class=\"mono\">strategies/{family}/FAMILY.md</span><span>written for the firm — the plain-English version opens <a href=\"{page}\">Overview</a></span>"
            ),
        ),
        "predictions" => {
            let scoring = if scoring_rows.is_empty() {
                String::new()
            } else {
                let body: Vec<Vec<String>> = scoring_rows
                    .iter()
                    .map(|row| {
                        let imp = s.num(row, "mean_improvement");
                        let key = s.cell(row, "key");
                        vec![
                            esc(s.cell(row, "level")),
                            format!("<a href=\"/strategies/{0}\">{0}</a>", esc(key)),
                            s.cell(row, "n").to_string(),
                            badge(&fmt_signed(imp), if imp > 0.0 { "ok" } else { "bad" }),
                            format!("{:.4}", s.num(row, "mean_brier")),
                            format!("{:.4}", s.num(row, "mean_market_brier")),
                            format!("{:.4}", s.num(row, "mean_logloss")),
                        ]
                    })
                    .collect();
                section(
                    "Scoring",
                    "the family and each strategy in it — lower Brier is better, improvement is the gap to the market",
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
            format!("{scoring}{}", prediction_table(&p, &d, &fam_rows, true))
        }
        // Overview
        _ => {
            let mut metas: Vec<data::VariantMeta> = Vec::new();
            for (_, v) in &variants {
                let f = data::text(env, &format!("strategies/{family}/{v}/strategy.toml")).await;
                all_live &= f.live;
                metas.push(data::parse_variant_meta(family, v, &f.text));
            }
            let mut variant_items = String::new();
            for m in &metas {
                let n = p
                    .rows
                    .iter()
                    .filter(|row| {
                        p.cell(row, "family") == family && p.cell(row, "variant") == m.variant
                    })
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
            format!(
                "{lede}{variants}",
                lede = lede(
                    &fam_lede,
                    &format!(
                        "No plain-English description yet — <span class=\"mono\">strategies/{family}/FAMILY.md</span> does not open with its required <span class=\"mono\">&gt; **In plain English:**</span> paragraph."
                    ),
                ),
                variants = section(
                    "Strategies in this family",
                    "each one is a separate experiment with its own clock",
                    &badge(&render::count(variants.len(), "variant"), ""),
                    &items(&variant_items),
                ),
            )
        }
    };

    let body = format!(
        "{banner}{stats}{content}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        content = content,
    );

    shell_sub(env, "/strategies", crumbs, all_live, &bar, &body).await
}

// ---------------------------------------------------------------------------
// /strategies/<family>/<variant>
// ---------------------------------------------------------------------------

pub async fn variant(env: &Env, family: &str, variant: &str, want_tab: Option<String>) -> String {
    let page = format!("/strategies/{family}/{variant}");
    let fam_href = format!("/strategies/{family}");
    let crumbs_for = |leaf: &str| -> Vec<render::Crumb> {
        let mut parts: Vec<(&str, &str)> = vec![
            ("Research", ""),
            ("Strategies", "/strategies"),
            (family, fam_href.as_str()),
        ];
        if leaf.is_empty() {
            parts.push((variant, ""));
        } else {
            parts.push((variant, page.as_str()));
            parts.push((leaf, ""));
        }
        trail(&parts)
    };

    if !data::safe_segment(family) || !data::safe_segment(variant) {
        return shell(
            env,
            "/strategies",
            crumbs_for(""),
            true,
            &render::empty_state("Unknown variant", ""),
        )
        .await;
    }
    let base = format!("strategies/{family}/{variant}");

    let toml_doc = data::text(env, &format!("{base}/strategy.toml")).await;
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let scores = data::text(env, "predictions/scores.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let mut all_live = toml_doc.live && tree_live && preds.live && scores.live && detail.live;

    let has_strategy_md = paths.iter().any(|path| *path == format!("{base}/STRATEGY.md"));
    if toml_doc.is_empty() && !has_strategy_md {
        let body = section(
            "Variant not found",
            &base,
            "",
            &render::empty_state(
                "Nothing here",
                "<div>No such variant in the repo. <a class=\"link\" href=\"/strategies\">Back to strategies</a>.</div>",
            ),
        );
        return shell(env, "/strategies", crumbs_for(""), all_live, &body).await;
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

    // --- which tabs have something in them --------------------------------
    // Decided from the repo's FILE LISTING, not by reading the files: the tab
    // being shown costs a fetch, the tabs merely offered cost nothing.
    let results = data::files_in(&paths, &format!("{base}/results"), ".md");
    let logs: Vec<String> = ["memory/WORKLOG.md", "memory/MEMORY.md"]
        .iter()
        .map(|rel| format!("{base}/{rel}"))
        .filter(|path| paths.iter().any(|p| p == path))
        .collect();

    let mut tabs: Vec<Tab> = vec![tab("", "Overview")];
    if has_strategy_md {
        tabs.push(tab("how-it-works", "How it works"));
    }
    if !results.is_empty() {
        tabs.push(tab_n("results", "Results", results.len()));
    }
    if !rows.is_empty() {
        tabs.push(tab_n("predictions", "Predictions", rows.len()));
    }
    if !logs.is_empty() {
        tabs.push(tab("logs", "Logs"));
    }
    let active = active_tab(&tabs, &want_tab).to_string();
    let bar = tabbar(&page, &tabs, &active);
    let crumbs = crumbs_for(tab_label(&tabs, &active));

    let ahead = scored
        .iter()
        .filter(|row| d.num(row, "improvement") > 0.0)
        .count();
    // Beating the quoted price and being able to trade on it are separate
    // claims; "N beat the market" without "M of them were reachable" is the
    // misleading half. See wiki/reference/midpoint-is-not-a-fill.md.
    let fillable = var_score
        .map(|row| (s.num(row, "n_known_fill"), s.num(row, "n_fillable")))
        .filter(|(known, _)| *known > 0.0)
        .map(|(_, fill)| format!(", {} reachable", fill as i64))
        .unwrap_or_default();
    let stats = stat_line(&[
        (
            render::status_badge(&status),
            match status.as_str() {
                "trial" => format!("on slot {} since {}", m.slot.unwrap_or(0), m.trial_started),
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
                format!("scored, {ahead} beat the market{fillable}")
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

    // --- the active tab's content -----------------------------------------
    let content = match active.as_str() {
        "how-it-works" => {
            let f = data::text(env, &format!("{base}/STRATEGY.md")).await;
            all_live &= f.live;
            section_foot(
                "",
                "",
                "",
                &format!("<div class=\"prose\">{}</div>", markdown_body(&f.text)),
                &format!(
                    "<span class=\"mono\">{base}/STRATEGY.md</span><span>the runbook: method, gates, and what would kill it</span>"
                ),
            )
        }
        "results" => {
            let mut out = String::new();
            for path in &results {
                let name = path.rsplit('/').next().unwrap_or(path);
                let f = data::text(env, path).await;
                all_live &= f.live;
                // These documents keep their own `# ` title: it is distinctive
                // ("Metals backtest — gates 0/1/2 …"), it repeats no breadcrumb,
                // and rendered as prose it outranks the h2s beneath it — which
                // a 14px section heading would not, with two write-ups stacked.
                out.push_str(&format!(
                    "<div id=\"{id}\">{sec}</div>",
                    id = anchor(name),
                    sec = section_foot(
                        "",
                        "",
                        "",
                        &format!(
                            "<div class=\"prose prose-wide\">{}</div>",
                            render::markdown(&f.text)
                        ),
                        &format!("<span class=\"mono\">{}</span>", esc(path)),
                    ),
                ));
            }
            // Everything else the results folder holds: the numbers the
            // write-ups were generated from. The dashboard cannot render those,
            // so it links to the file itself rather than pretending otherwise.
            let artifacts: Vec<String> = data::files_in(&paths, &format!("{base}/results"), "")
                .into_iter()
                .filter(|path| !path.ends_with(".md"))
                .collect();
            if !artifacts.is_empty() {
                let mut list = String::new();
                for path in &artifacts {
                    let name = path.rsplit('/').next().unwrap_or(path);
                    list.push_str(&format!(
                        "<li><a href=\"{REPO_BLOB}/{path}\" target=\"_blank\" rel=\"noreferrer\">{icon} {name}</a></li>",
                        path = esc(path),
                        name = esc(name),
                        icon = icon("external"),
                    ));
                }
                out.push_str(&section(
                    "The numbers behind them",
                    "artifacts these write-ups were generated from, in the repo",
                    &badge(&render::count(artifacts.len(), "file"), ""),
                    &format!("<ul class=\"link-list\">{list}</ul>"),
                ));
            }
            out
        }
        "predictions" => prediction_table(&p, &d, &rows, false),
        "logs" => {
            let mut out = String::new();
            for path in &logs {
                let f = data::text(env, path).await;
                all_live &= f.live;
                let worklog = path.ends_with("WORKLOG.md");
                out.push_str(&section_foot(
                    if worklog { "Worklog" } else { "Memory" },
                    if worklog {
                        "what this strategy's agents did, run by run"
                    } else {
                        "what they know now — carried into the next run"
                    },
                    "",
                    &format!(
                        "<div class=\"prose prose-wide\">{}</div>",
                        markdown_body(&f.text)
                    ),
                    &format!("<span class=\"mono\">{}</span>", esc(path)),
                ));
            }
            out
        }
        // Overview
        _ => {
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

            let app_paths = data::files_in(&paths, &format!("{base}/applications"), ".toml");
            let mut app_rows: Vec<Vec<String>> = Vec::new();
            for path in app_paths.iter().take(24) {
                let f = data::text(env, path).await;
                all_live &= f.live;
                let t = data::toml_of(&f.text);
                let name = path.rsplit('/').next().unwrap_or(path);
                let slug = data::tstr(&t, "market_slug");
                let legs = data::arr_at(&t, &["legs"]).len();
                let is_active = data::tbool(&t, "active");
                app_rows.push(vec![
                    format!("<span class=\"mono\">{}</span>", esc(name)),
                    if slug.is_empty() {
                        "<span class=\"muted\">—</span>".to_string()
                    } else {
                        format!("<a href=\"/markets/{0}\">{0}</a>", esc(slug))
                    },
                    esc(data::tstr(&t, "added")),
                    match is_active {
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
                section(
                    "Applications",
                    "the markets this strategy is pointed at",
                    "",
                    &render::empty_state(
                        "No applications",
                        "<div>Nothing under <span class=\"mono\">applications/</span> yet — this strategy is not pointed at any market.</div>",
                    ),
                )
            } else {
                section(
                    "Applications",
                    "the boards and markets this strategy is pointed at",
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

            // The explainer is the description of this strategy: the single
            // most important text on the page, above every number and before
            // any prose written for the firm. `summary` is the one-liner used
            // in tables; this is the version a reader can start from cold.
            //
            // It gets a reading measure, which leaves the right-hand third of
            // the row free — so the facts list, the densest thing on the tab,
            // goes there instead of into empty space, and the applications
            // table gets the full width its seven columns need.
            format!(
                "<div class=\"grid-main\"><div>{lede}</div>{facts}</div>{applications}",
                lede = lede(
                    if m.explainer.is_empty() { &m.summary } else { &m.explainer },
                    &format!(
                        "No plain-English explainer yet — <span class=\"mono\">{base}/strategy.toml</span> is missing its required <span class=\"mono\">explainer</span> field."
                    ),
                ),
                facts = section(
                    "Facts",
                    "status, trial clock and labels as recorded in the repo",
                    "",
                    &render::rows(&facts),
                ),
                applications = applications,
            )
        }
    };

    let body = format!(
        "{banner}{stats}{content}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        content = content,
    );

    shell_sub(env, "/strategies", crumbs, all_live, &bar, &body).await
}

// ---------------------------------------------------------------------------
// Shared: the prediction log of one strategy or one family
// ---------------------------------------------------------------------------

/// Prediction rows, newest first, with the score where the market has resolved.
/// `with_variant` adds the strategy column (a family spans several).
fn prediction_table(p: &Table, d: &Table, rows: &[&Vec<String>], with_variant: bool) -> String {
    if rows.is_empty() {
        return section(
            "Predictions",
            "rows logged against this family",
            "",
            &render::empty_state("Nothing logged yet", ""),
        );
    }
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
            let mut cells = vec![format!("<span class=\"mono\">{}</span>", esc(&fmt_ts(ts)))];
            if with_variant {
                cells.push(format!(
                    "<a href=\"/strategies/{0}/{1}\">{1}</a>",
                    esc(p.cell(row, "family")),
                    esc(p.cell(row, "variant"))
                ));
            }
            cells.extend([
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
            ]);
            cells
        })
        .collect();

    let mut head: Vec<(&str, &str)> = vec![("Logged", "")];
    if with_variant {
        head.push(("Strategy", ""));
    }
    head.extend([
        ("Market", ""),
        ("Outcome", ""),
        ("Ours", "num"),
        ("Market", "num"),
        ("Edge", "num"),
        ("Improvement", ""),
        ("Model", ""),
    ]);

    section(
        "Predictions",
        "every row logged, newest first — ours against the market at the time",
        &badge(&render::count(rows.len(), "row"), ""),
        &table_scroll(&head, &body),
    )
}
