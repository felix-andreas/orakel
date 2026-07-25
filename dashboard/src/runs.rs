//! `/runs` — what the firm did, day by day, in plain English.
//!
//! The manifests in `ops/runs/*.toml` are written by the CEO in internal
//! shorthand ("gate-0 251/255", "paired Brier", "delayed-exec verified"). A
//! smart outsider cannot read that, so it is NOT the headline here. Each run's
//! story is instead ASSEMBLED FROM REPO FACTS THAT CARRY A DATE:
//!
//!   predictions.csv    timestamps on this day       → "Logged 51 predictions"
//!   resolutions.csv    resolved_date                → "18 markets settled"
//!   scores_detail.csv  rows for those markets       → "21 beat the market"
//!   strategy.toml      created / [retirement].date  → started / dropped, each
//!                                                     described by its own
//!                                                     plain-English summary
//!   ideas/*.md         frontmatter date             → "added to the backlog"
//!
//! Nothing is invented: every sentence is a count or a quoted `summary` field.
//! The CEO's own notes and the health flags stay one click down, subordinate.
//!
//! Settlements are credited to the first run AFTER the resolution date — that
//! is the run that could act on them, which is where a reader expects to see
//! them.

use crate::data::{self, Table, VariantMeta};
use crate::render::{self, badge, esc, fmt_int, fmt_tokens, panel_foot, stat_line, table};
use crate::{shell, snapshot_banner, trail};
use worker::Env;

pub async fn page(env: &Env) -> String {
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let resolutions = data::text(env, "predictions/resolutions.csv").await;
    let detail = data::text(env, "predictions/scores_detail.csv").await;
    let (metas, metas_live) = data::variant_metas(env, &paths).await;
    let mut all_live = tree_live && preds.live && resolutions.live && detail.live && metas_live;

    // Idea files carry a frontmatter date; read them once for the story.
    let mut ideas: Vec<(String, String)> = Vec::new(); // (date, name)
    for path in data::files_in(&paths, "ideas", ".md") {
        if path.ends_with("README.md") {
            continue;
        }
        let f = data::text(env, &path).await;
        all_live &= f.live;
        let (fields, _) = render::split_frontmatter(&f.text);
        let get = |k: &str| {
            fields
                .iter()
                .find(|(a, _)| a == k)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let date = get("date");
        let slug = get("slug");
        if !date.is_empty() {
            ideas.push((date, if slug.is_empty() { path.clone() } else { slug }));
        }
    }

    let mut runs: Vec<(String, toml::Table)> = Vec::new();
    for path in data::run_paths(&paths) {
        let doc = data::text(env, &path).await;
        all_live &= doc.live;
        if doc.is_empty() {
            continue;
        }
        let stem = path
            .trim_start_matches("ops/runs/")
            .trim_end_matches(".toml")
            .to_string();
        runs.push((stem, data::toml_of(&doc.text)));
    }

    let p = Table::parse(&preds.text);
    let r = Table::parse(&resolutions.text);
    let d = Table::parse(&detail.text);
    let banner = if all_live { String::new() } else { snapshot_banner() };

    if runs.is_empty() {
        let body = format!(
            "{banner}{}",
            render::empty_state(
                "No runs yet",
                "<div>The CEO writes one manifest to <span class=\"mono\">ops/runs/</span> at the end of every run.</div>",
            )
        );
        return shell(env, "/runs", trail(&[("Overview", ""), ("Daily runs", "")]), all_live, &body)
            .await;
    }

    // Ascending run dates — used to credit a settlement to the next run.
    let mut dates: Vec<String> = runs
        .iter()
        .map(|(_, t)| data::tstr(t, "date").to_string())
        .collect();
    dates.sort();
    dates.dedup();

    let mut blocks = String::new();
    for (stem, t) in &runs {
        blocks.push_str(&run_block(stem, t, &p, &r, &d, &metas, &ideas, &dates));
    }

    let body = format!(
        "{banner}{stats}{list}",
        stats = summary_line(&runs, &p),
        list = panel_foot(
            "What happened, day by day",
            "newest first",
            &badge(&render::count(runs.len(), "run"), ""),
            &format!("<div class=\"runs\">{blocks}</div>"),
            "<span class=\"mono\">ops/runs/*.toml · predictions/*.csv · strategies/*/*/strategy.toml</span><a href=\"/predictions\">Prediction log →</a>",
            true,
        ),
    );

    shell(
        env,
        "/runs",
        trail(&[("Overview", ""), ("Daily runs", "")]),
        all_live,
        &body,
    )
    .await
}

fn summary_line(runs: &[(String, toml::Table)], p: &Table) -> String {
    let tokens: i64 = runs
        .iter()
        .filter_map(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .sum();
    let steps: usize = runs
        .iter()
        .map(|(_, t)| data::arr_at(t, &["step"]).len())
        .sum();
    let first = runs
        .last()
        .map(|(_, t)| data::tstr(t, "date").to_string())
        .unwrap_or_default();
    let last = runs
        .first()
        .map(|(_, t)| data::tstr(t, "date").to_string())
        .unwrap_or_default();
    let days = data::days_between(&first, &last).map(|d| d + 1).unwrap_or(0);
    let flagged = runs
        .iter()
        .filter(|(_, t)| {
            !(data::bool_at(t, &["health", "all_slots_ran"]).unwrap_or(false)
                && data::bool_at(t, &["health", "pushed"]).unwrap_or(false))
        })
        .count();

    stat_line(&[
        (fmt_int(runs.len() as i64), format!("runs in {days} days"), ""),
        (fmt_int(steps as i64), "pieces of work".to_string(), ""),
        (
            fmt_int(p.rows.len() as i64),
            "predictions logged".to_string(),
            "",
        ),
        (fmt_tokens(tokens), "tokens spent".to_string(), ""),
        (
            if flagged == 0 {
                "all".to_string()
            } else {
                fmt_int((runs.len() - flagged) as i64)
            },
            "runs healthy".to_string(),
            if flagged == 0 { "ok" } else { "warn" },
        ),
    ])
}

/// One run: date line, the derived plain-English story, then the raw manifest.
#[allow(clippy::too_many_arguments)]
fn run_block(
    stem: &str,
    t: &toml::Table,
    p: &Table,
    r: &Table,
    d: &Table,
    metas: &[VariantMeta],
    ideas: &[(String, String)],
    dates: &[String],
) -> String {
    let date = data::tstr(t, "date");
    let tokens = data::int_at(t, &["spend", "total_tokens"]).unwrap_or(0);
    let steps = data::arr_at(t, &["step"]);
    let all_ran = data::bool_at(t, &["health", "all_slots_ran"]).unwrap_or(false);
    let pushed = data::bool_at(t, &["health", "pushed"]).unwrap_or(false);
    let notes = data::str_at(t, &["health", "notes"]);

    // ---- the story, assembled from dated repo facts ----
    let mut lines: Vec<String> = Vec::new();

    let logged: Vec<&Vec<String>> = p
        .rows
        .iter()
        .filter(|row| p.cell(row, "timestamp").starts_with(date))
        .collect();
    if !logged.is_empty() {
        let mut by_variant: Vec<(String, usize)> = Vec::new();
        for row in &logged {
            let key = format!("{}/{}", p.cell(row, "family"), p.cell(row, "variant"));
            match by_variant.iter_mut().find(|(k, _)| *k == key) {
                Some(e) => e.1 += 1,
                None => by_variant.push((key, 1)),
            }
        }
        let which: Vec<String> = by_variant
            .iter()
            .map(|(k, n)| format!("<a href=\"/strategies/{0}\">{0}</a> ({n})", esc(k)))
            .collect();
        lines.push(format!(
            "Logged <b>{}</b> new predictions from {}.",
            logged.len(),
            which.join(", ")
        ));
    }

    let settled: Vec<&Vec<String>> = r
        .rows
        .iter()
        .filter(|row| {
            let rd = r.cell(row, "resolved_date");
            !rd.is_empty() && next_run_after(rd, dates).as_deref() == Some(date)
        })
        .collect();
    if !settled.is_empty() {
        let slugs: Vec<&str> = settled
            .iter()
            .map(|row| r.cell(row, "market_slug"))
            .collect();
        let scored: Vec<&Vec<String>> = d
            .rows
            .iter()
            .filter(|row| slugs.contains(&d.cell(row, "market_slug")))
            .collect();
        let ahead = scored
            .iter()
            .filter(|row| d.num(row, "improvement") > 0.0)
            .count();
        if scored.is_empty() {
            lines.push(format!(
                "<b>{}</b> markets we had predictions on settled.",
                settled.len()
            ));
        } else {
            lines.push(format!(
                "<b>{}</b> markets settled, so {} of our predictions could be marked — <b>{}</b> of them beat the market's own price.",
                settled.len(),
                scored.len(),
                ahead
            ));
        }
    }

    for m in metas.iter().filter(|m| m.created == date) {
        lines.push(format!(
            "Started a new strategy, <a href=\"{}\">{}</a>: {}",
            esc(&m.href()),
            esc(&m.key()),
            esc(&m.summary)
        ));
    }
    for m in metas
        .iter()
        .filter(|m| !m.retired_on.is_empty() && m.retired_on == date)
    {
        lines.push(format!(
            "Dropped <a href=\"{}\">{}</a>: {}",
            esc(&m.href()),
            esc(&m.key()),
            esc(&m.summary)
        ));
    }

    let filed: Vec<&(String, String)> = ideas.iter().filter(|(dt, _)| dt == date).collect();
    if !filed.is_empty() {
        let names: Vec<String> = filed
            .iter()
            .map(|(_, name)| format!("<span class=\"mono\">{}</span>", esc(name)))
            .collect();
        lines.push(format!(
            "Added {} to the idea backlog: {}.",
            render::count(filed.len(), "candidate"),
            names.join(", ")
        ));
    }

    let story = if lines.is_empty() {
        "<p class=\"run-story\">Maintenance only — no predictions, settlements or strategy changes on this day.</p>".to_string()
    } else {
        format!("<p class=\"run-story\">{}</p>", lines.join(" "))
    };

    // ---- subordinate: the CEO's own words, unedited ----
    let mut step_rows: Vec<Vec<String>> = Vec::new();
    for s in &steps {
        let role = s.get("role").and_then(|v| v.as_str()).unwrap_or("?");
        let variant = s.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        let note = s.get("note").and_then(|v| v.as_str()).unwrap_or("");
        let status = s.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let who = if variant.is_empty() {
            esc(role)
        } else {
            format!(
                "{}<span class=\"sub\"><a href=\"/strategies/{1}\">{1}</a></span>",
                esc(role),
                esc(variant)
            )
        };
        step_rows.push(vec![
            who,
            if note.is_empty() { esc(status) } else { esc(note) },
        ]);
    }
    let mut detail_html = table(&[("Who", ""), ("Note, as written", "wrap")], &step_rows);
    if !notes.is_empty() {
        detail_html.push_str(&format!(
            "<p class=\"note\"><b>Closing note:</b> {}</p>",
            esc(notes)
        ));
    }
    detail_html.push_str(&format!(
        "<p class=\"note\">Trigger: {} · models: {} · all slots ran: {} · pushed: {}</p>",
        esc(data::tstr(t, "trigger")),
        esc(data::tstr(t, "model")),
        if all_ran { "yes" } else { "no" },
        if pushed { "yes" } else { "no" },
    ));

    let health = if all_ran && pushed {
        String::new()
    } else {
        format!(" · {}", badge("check health", "warn"))
    };

    format!(
        r#"<section class="run"><div class="run-line"><span class="run-when"><b>{date}</b> {weekday}{suffix}</span><span class="run-meta">{steps_n} steps · {tokens} tokens{health}</span></div>{story}<details class="doc doc-plain"><summary>What the CEO wrote<span class="doc-meta">ops/runs/{stem}.toml</span></summary><div class="doc-body">{detail}</div></details></section>"#,
        date = esc(date),
        weekday = esc(data::weekday(date)),
        suffix = if stem.len() > 10 {
            format!(" · run {}", &stem[11..])
        } else {
            String::new()
        },
        steps_n = steps.len(),
        tokens = fmt_tokens(tokens),
        health = health,
        story = story,
        stem = esc(stem),
        detail = detail_html,
    )
}

/// The first run date strictly after `date` — the run that could act on a
/// market that settled that day.
fn next_run_after(date: &str, dates: &[String]) -> Option<String> {
    dates.iter().find(|d| d.as_str() > date).cloned()
}
