//! `/runs` — daily runs, read as a narrative.
//!
//! Renders `ops/runs/*.toml` newest first, one panel per run: the date, what
//! fired it, which models ran, the CEO's own closing note, then one line per
//! step (who did what, what came of it), plus the health flags and token
//! spend. The goal is that a human understands the firm's last week in half a
//! minute without opening a single manifest.

use crate::data::{self, Table};
use crate::render::{
    self, badge, esc, fmt_int, fmt_tokens, kpi, kpi_row, panel, table,
};
use crate::{shell, snapshot_banner, trail};
use worker::Env;

pub async fn page(env: &Env) -> String {
    let (paths, tree_live) = data::tree(env).await;
    let preds = data::text(env, "predictions/predictions.csv").await;
    let readme = data::text(env, "ops/runs/README.md").await;
    let mut all_live = tree_live && preds.live && readme.live;

    let mut runs: Vec<(String, toml::Table)> = Vec::new();
    for path in data::run_paths(&paths) {
        let doc = data::text(env, &path).await;
        all_live &= doc.live;
        let stem = path
            .trim_start_matches("ops/runs/")
            .trim_end_matches(".toml")
            .to_string();
        if doc.is_empty() {
            continue;
        }
        runs.push((stem, data::toml_of(&doc.text)));
    }

    let p = Table::parse(&preds.text);
    let banner = if all_live { String::new() } else { snapshot_banner() };

    if runs.is_empty() {
        let body = format!(
            "{banner}{}",
            panel(
                "Daily runs",
                "one manifest per orchestrated run",
                "",
                &render::empty_state(
                    "No run manifests yet",
                    "<div><span class=\"mono\">ops/runs/</span> contains no <span class=\"mono\">&lt;YYYY-MM-DD&gt;.toml</span> manifests. The CEO writes one at the end of every run.</div>",
                ),
            )
        );
        return shell(env, "/runs", trail(&[("Overview", ""), ("Daily runs", "")]), all_live, &body).await;
    }

    let mut body = format!("{banner}{}", summary(&runs, &p));
    for (stem, t) in &runs {
        body.push_str(&run_panel(stem, t));
    }
    body.push_str(&render::note(&format!(
        "Manifest format: <span class=\"mono\">ops/runs/README.md</span>. {}",
        esc(
            "Every run records what fired, what each role did, rows appended, health flags and token spend (CONSTITUTION §1–2)."
        )
    )));

    shell(
        env,
        "/runs",
        trail(&[("Overview", ""), ("Daily runs", "")]),
        all_live,
        &body,
    )
    .await
}

/// Four numbers covering the whole log, so the page opens with a verdict.
fn summary(runs: &[(String, toml::Table)], p: &Table) -> String {
    let tokens: i64 = runs
        .iter()
        .filter_map(|(_, t)| data::int_at(t, &["spend", "total_tokens"]))
        .sum();
    let rows: i64 = runs
        .iter()
        .filter_map(|(_, t)| data::int_at(t, &["health", "csv_appended_rows"]))
        .sum();
    let steps: usize = runs.iter().map(|(_, t)| data::arr_at(t, &["step"]).len()).sum();
    let first = runs
        .last()
        .map(|(_, t)| data::tstr(t, "date").to_string())
        .unwrap_or_default();
    let last = runs
        .first()
        .map(|(_, t)| data::tstr(t, "date").to_string())
        .unwrap_or_default();
    let span = data::days_between(&first, &last)
        .map(|d| format!("{} days", d + 1))
        .unwrap_or_else(|| "—".to_string());
    let flagged = runs
        .iter()
        .filter(|(_, t)| {
            !(data::bool_at(t, &["health", "all_slots_ran"]).unwrap_or(false)
                && data::bool_at(t, &["health", "pushed"]).unwrap_or(false))
        })
        .count();

    kpi_row(&[
        kpi("Runs", &fmt_int(runs.len() as i64), "calendar", 1)
            .delta(&span, "")
            .context(&format!("{first} → {last}")),
        kpi("Steps executed", &fmt_int(steps as i64), "check", 2)
            .delta(
                if flagged == 0 { "all healthy" } else { "needs a look" },
                if flagged == 0 { "up" } else { "warn" },
            )
            .context("across every role"),
        kpi("Prediction rows added", &fmt_int(rows), "list", 4)
            .context(&format!("{} rows in the log now", p.rows.len())),
        kpi("Token spend", &fmt_tokens(tokens), "bolt", 3)
            .context("best-effort estimates, all subagents included"),
    ])
}

/// One manifest → one panel: header, the CEO's closing note, the step list,
/// then the health flags.
fn run_panel(stem: &str, t: &toml::Table) -> String {
    let date = data::tstr(t, "date");
    let trigger = data::tstr(t, "trigger");
    let model = data::tstr(t, "model");
    let tokens = data::int_at(t, &["spend", "total_tokens"]).unwrap_or(0);
    let rows_added = data::int_at(t, &["health", "csv_appended_rows"]).unwrap_or(0);
    let all_ran = data::bool_at(t, &["health", "all_slots_ran"]).unwrap_or(false);
    let pushed = data::bool_at(t, &["health", "pushed"]).unwrap_or(false);
    let notes = data::str_at(t, &["health", "notes"]);
    let steps = data::arr_at(t, &["step"]);

    let mut chips = vec![
        badge(&render::count(steps.len(), "step"), ""),
        badge(&render::count(rows_added.max(0) as usize, "row"), if rows_added > 0 { "info" } else { "" }),
        badge(&format!("{} tokens", fmt_tokens(tokens)), ""),
    ];
    chips.push(if all_ran && pushed {
        badge("healthy", "ok")
    } else {
        badge("check health", "warn")
    });

    let head = format!(
        r#"<div class="run-head"><div class="run-date"><b>{date}</b><span>{weekday}{suffix}</span></div><div class="badge-row">{chips}</div></div>"#,
        date = esc(date),
        weekday = esc(data::weekday(date)),
        suffix = if stem.len() > 10 {
            format!(" · run {}", &stem[11..])
        } else {
            String::new()
        },
        chips = chips.join(""),
    );

    let lead = if notes.is_empty() {
        String::new()
    } else {
        format!("<p class=\"run-lead\">{}</p>", esc(notes))
    };

    let mut step_html = String::new();
    for s in &steps {
        let role = s.get("role").and_then(|v| v.as_str()).unwrap_or("?");
        let status = s.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let note = s.get("note").and_then(|v| v.as_str()).unwrap_or("");
        let preds = s.get("predictions").and_then(|v| v.as_integer());
        let slot = s.get("slot").and_then(|v| v.as_integer());
        let variant = s.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        let mut sub = String::new();
        if let Some(n) = slot {
            sub.push_str(&format!("<span class=\"badge\">slot {n}</span>"));
        }
        if !variant.is_empty() {
            let href = match variant.split_once('/') {
                Some((f, v)) => format!("/strategies/{f}/{v}"),
                None => String::new(),
            };
            sub.push_str(&if href.is_empty() {
                format!("<span class=\"chip\">{}</span>", esc(variant))
            } else {
                format!(
                    "<a class=\"chip\" href=\"{}\">{}</a>",
                    esc(&href),
                    esc(variant)
                )
            });
        }
        if let Some(n) = preds {
            if n > 0 {
                sub.push_str(&badge(&format!("+{n} predictions"), "info"));
            }
        }

        step_html.push_str(&format!(
            r#"<div class="step"><div class="step-who"><span class="step-role"><span class="dot dot-{tone}"></span>{role}</span><div class="badge-row">{sub}</div></div><div class="step-note">{note}</div></div>"#,
            tone = match render::status_tone(status) {
                "ok" => "ok",
                "bad" => "bad",
                "warn" => "warn",
                _ => "",
            },
            role = esc(role),
            sub = sub,
            note = if note.is_empty() {
                format!("<span class=\"muted\">{}</span>", esc(status))
            } else {
                esc(note)
            },
        ));
    }

    let health = table(
        &[("Health flag", ""), ("Value", "")],
        &[
            vec![
                "all slots ran".to_string(),
                badge(if all_ran { "yes" } else { "no" }, if all_ran { "ok" } else { "bad" }),
            ],
            vec![
                "pushed to main".to_string(),
                badge(if pushed { "yes" } else { "no" }, if pushed { "ok" } else { "bad" }),
            ],
            vec!["prediction rows appended".to_string(), fmt_int(rows_added)],
            vec![
                "models".to_string(),
                format!("<span class=\"mono\">{}</span>", esc(model)),
            ],
            vec![
                "trigger".to_string(),
                esc(trigger),
            ],
        ],
    );

    format!(
        r#"<section class="panel">{head}{lead}<div class="steps">{steps}</div><details class="doc doc-flush"><summary>Run detail<span class="doc-meta">ops/runs/{stem}.toml</span></summary><div class="doc-body">{health}</div></details></section>"#,
        head = head,
        lead = lead,
        steps = step_html,
        stem = esc(stem),
        health = health,
    )
}
