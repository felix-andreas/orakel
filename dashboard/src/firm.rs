//! The firm's own pages: `/state`, `/decisions`, `/inboxes`, `/wiki`,
//! `/ideas` and `/snapshots`.
//!
//! These render repo documents nearly verbatim — the value is in finding them,
//! not in transforming them — so the layout work here is grouping, badges and
//! whitespace rather than charts.

use crate::data;
use crate::render::{
    self, badge, chip_row, doc, esc, fmt_int, icon, item, items, markdown, markdown_body, section,
    section_foot, stat_line, table,
};
use crate::snapshots;
use crate::{shell, snapshot_banner, trail};
use serde_json::Value;
use worker::Env;

// ---------------------------------------------------------------------------
// /state
// ---------------------------------------------------------------------------

pub async fn state(env: &Env) -> String {
    let doc_text = data::text(env, "ops/state.toml").await;
    let all_live = doc_text.live;
    let t = data::toml_of(&doc_text.text);

    let phase = data::str_at(&t, &["firm", "phase"]);
    let updated = data::str_at(&t, &["firm", "updated"]);
    let total = data::int_at(&t, &["research", "slots_total"]).unwrap_or(0);
    let active = data::int_at(&t, &["research", "slots_active"]).unwrap_or(0);
    let live_s = data::str_list(&t, &["strategies", "live"]);
    let trial_s = data::str_list(&t, &["strategies", "trial"]);
    let retired_s = data::str_list(&t, &["strategies", "retired"]);

    let stats = stat_line(&[
        (
            esc(if phase.is_empty() { "—" } else { phase }),
            "phase".to_string(),
            if phase == "operating" { "ok" } else { "warn" },
        ),
        (
            format!("{active} / {total}"),
            "research slots in use".to_string(),
            "",
        ),
        (
            fmt_int(trial_s.len() as i64),
            "strategies in trial".to_string(),
            "warn",
        ),
        (fmt_int(live_s.len() as i64), "live".to_string(), "ok"),
        (
            fmt_int(retired_s.len() as i64),
            "dropped".to_string(),
            "",
        ),
        (
            esc(if updated.is_empty() { "—" } else { updated }),
            "state last updated".to_string(),
            "",
        ),
    ]);

    // --- research slots ---
    let mut slot_items = String::new();
    for slot in data::arr_at(&t, &["research", "slot"]) {
        let id = slot.get("id").and_then(|v| v.as_integer()).unwrap_or(0);
        let variant = slot.get("variant").and_then(|v| v.as_str()).unwrap_or("");
        let started = slot.get("trial_started").and_then(|v| v.as_str()).unwrap_or("");
        let due = slot
            .get("trial_review_due")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let href = match variant.split_once('/') {
            Some((f, v)) => format!("/strategies/{f}/{v}"),
            None => String::new(),
        };
        slot_items.push_str(&item(
            &href,
            &esc(variant),
            &esc(&format!("slot {id} · started {started}")),
            &badge(&format!("review {due}"), "warn"),
        ));
    }
    if slot_items.is_empty() {
        slot_items = item("", "No trials running", "slots are filled by the CEO", "");
    }

    // --- strategies by status ---
    let mut strat = String::new();
    for (label, list, tone) in [
        ("live", &live_s, "ok"),
        ("trial", &trial_s, "warn"),
        ("retired", &retired_s, ""),
    ] {
        let value = if list.is_empty() {
            "<span class=\"muted\">none</span>".to_string()
        } else {
            let links: Vec<String> = list
                .iter()
                .map(|k| match k.split_once('/') {
                    Some((f, v)) => format!(
                        "<a class=\"badge badge-{tone}\" href=\"/strategies/{f}/{v}\">{}</a>",
                        esc(k)
                    ),
                    None => badge(k, tone),
                })
                .collect();
            format!("<div class=\"badge-row\">{}</div>", links.join(""))
        };
        strat.push_str(&render::row(label, &value));
    }

    // --- cadence ---
    let mut cadence = String::new();
    cadence.push_str(&render::row(
        "CEO trigger",
        &esc(data::str_at(&t, &["cadence", "ceo_trigger"])),
    ));
    let extra = data::str_list(&t, &["cadence", "additional_triggers"]);
    cadence.push_str(&render::row(
        "Additional",
        &if extra.is_empty() {
            "<span class=\"muted\">none</span>".to_string()
        } else {
            chip_row(&extra)
        },
    ));

    // --- model routing / roles / secrets / cloudflare ---
    let mut models = String::new();
    if let Some(tbl) = data::table_at(&t, &["models"]) {
        for (role, model) in tbl {
            models.push_str(&render::row(
                role,
                &format!(
                    "<span class=\"mono\">{}</span>",
                    esc(model.as_str().unwrap_or(""))
                ),
            ));
        }
    }

    let roles = data::str_list(&t, &["roles", "active"]);
    let roles_html = if roles.is_empty() {
        "<span class=\"muted\">none</span>".to_string()
    } else {
        chip_row(&roles)
    };

    let mut secrets = String::new();
    if let Some(tbl) = data::table_at(&t, &["secrets"]) {
        for (name, v) in tbl {
            let val = v.as_str().unwrap_or("");
            secrets.push_str(&render::row(
                name,
                &badge(val, if val == "missing" { "bad" } else { "ok" }),
            ));
        }
    }

    let mut cf = String::new();
    if let Some(tbl) = data::table_at(&t, &["cloudflare"]) {
        for (name, v) in tbl {
            let val = v.as_str().unwrap_or("");
            cf.push_str(&render::row(
                name,
                &if name == "mcp" && val == "connected" {
                    badge(val, "ok")
                } else {
                    format!("<span class=\"mono\">{}</span>", esc(val))
                },
            ));
        }
    }

    let body = format!(
        "{banner}{stats}<div class=\"grid-main\">{slots}{strategies}</div><div class=\"grid-3\">{cadence}{models}{roles}</div><div class=\"grid-pair\">{secrets}{cf}</div>{note}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        stats = stats,
        slots = section_foot(
            "Research slots",
            "one trial variant per slot",
            &badge(&format!("{active} of {total} active"), "warn"),
            &items(&slot_items),
            "<span class=\"mono\">ops/state.toml [[research.slot]]</span><a href=\"/strategies\">Strategies →</a>"
        ),
        strategies = section(
            "Strategies",
            "by status, as recorded in state",
            "",
            &render::rows(&strat),
        ),
        cadence = section("Cadence", "when work fires", "", &render::rows(&cadence)),
        models = section("Model routing", "per-task defaults", "", &render::rows(&models)),
        roles = section(
            "Active roles",
            "researchers are instantiated per slot",
            "",
            &roles_html,
        ),
        secrets = section(
            "Secrets",
            "presence only — never values",
            "",
            &render::rows(&secrets),
        ),
        cf = section("Cloudflare", "the firm's infrastructure", "", &render::rows(&cf)),
        note = render::note(
            "From <span class=\"mono\">ops/state.toml</span> on main. Every change here needs a dated entry in <a class=\"link\" href=\"/decisions\">decisions</a>.",
        ),
    );

    shell(env, "/state", trail(&[("Firm", ""), ("State", "")]), all_live, &body).await
}

// ---------------------------------------------------------------------------
// /decisions
// ---------------------------------------------------------------------------

pub async fn decisions(env: &Env) -> String {
    let f = data::text(env, "ops/decisions.md").await;

    // ops/decisions.md is append-only prose. Its "## <date> — <what changed>"
    // headings are already the scannable spine; the reasoning underneath is
    // detail. Split on them so the page reads as a list of changes, not a
    // wall (PRINCIPLES.md: no walls of text).
    let entries = split_entries(&f.text);

    let body = if entries.is_empty() {
        format!(
            "{}{}",
            if f.live { String::new() } else { snapshot_banner() },
            render::empty_state("No decisions recorded yet", ""),
        )
    } else {
        let mut list = String::new();
        for (date, what, detail) in &entries {
            list.push_str(&format!(
                r#"<section class="entry"><div class="entry-line"><b>{date}</b><span class="entry-meta">{weekday}</span></div><p class="entry-what">{what}</p><details class="doc doc-plain"><summary>Why</summary><div class="doc-body"><div class="prose">{detail}</div></div></details></section>"#,
                date = esc(date),
                weekday = esc(data::weekday(date)),
                what = esc(what),
                detail = markdown(detail),
            ));
        }
        format!(
            "{banner}{stats}{panel}",
            banner = if f.live { String::new() } else { snapshot_banner() },
            stats = stat_line(&[
                (fmt_int(entries.len() as i64), "structural changes".to_string(), ""),
                (
                    entries.last().map(|(d, _, _)| d.clone()).unwrap_or_default(),
                    "first entry".to_string(),
                    "",
                ),
                (
                    entries.first().map(|(d, _, _)| d.clone()).unwrap_or_default(),
                    "most recent".to_string(),
                    "",
                ),
            ]),
            panel = section_foot(
                "What changed, and when",
                "newest first — open an entry for the reasoning behind it",
                &badge(&render::count(entries.len(), "entry"), ""),
                &format!("<div class=\"entries\">{list}</div>"),
                "<span class=\"mono\">ops/decisions.md</span><a href=\"/state\">Current state →</a>"
            ),
        )
    };

    shell(
        env,
        "/decisions",
        trail(&[("Firm", ""), ("Decisions", "")]),
        f.live,
        &body,
    )
    .await
}

/// `## <date> — <what changed>` sections → (date, what, body). Anything before
/// the first heading (the file's own preamble) is dropped: it explains the
/// format, which the page header already does.
fn split_entries(src: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    for line in src.lines() {
        if let Some(head) = line.strip_prefix("## ") {
            let head = head.trim();
            // "2026-07-25 — Execution layer built" → ("2026-07-25", "Execution…")
            let (date, what) = match head.split_once(|c| c == '—' || c == '-') {
                Some((d, w)) if d.trim().len() == 10 => (d.trim().to_string(), w.trim().to_string()),
                _ => (String::new(), head.to_string()),
            };
            let (date, what) = if date.is_empty() {
                // Fall back to the leading 10 chars if they look like a date.
                let maybe = head.chars().take(10).collect::<String>();
                if maybe.len() == 10 && maybe.chars().filter(|c| *c == '-').count() == 2 {
                    (
                        maybe.clone(),
                        head[10..].trim_start_matches([' ', '—', '-']).trim().to_string(),
                    )
                } else {
                    (String::new(), head.to_string())
                }
            } else {
                (date, what)
            };
            out.push((date, what, String::new()));
        } else if let Some(last) = out.last_mut() {
            if line.trim() == "---" {
                continue;
            }
            last.2.push_str(line);
            last.2.push('\n');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// /inboxes
// ---------------------------------------------------------------------------

pub async fn inboxes(env: &Env) -> String {
    let (paths, tree_live) = data::tree(env).await;
    let mut all_live = tree_live;

    // roles/<role>/inbox/<file>.md — role may itself be a nested path.
    let mut by_role: Vec<(String, Vec<String>)> = Vec::new();
    for p in &paths {
        let Some(rest) = p.strip_prefix("roles/") else { continue };
        let Some((role, file)) = rest.split_once("/inbox/") else { continue };
        if !file.ends_with(".md") || file.contains('/') {
            continue;
        }
        let entry = match by_role.iter_mut().find(|(r, _)| r == role) {
            Some(e) => e,
            None => {
                by_role.push((role.to_string(), Vec::new()));
                by_role.last_mut().unwrap()
            }
        };
        if file != "README.md" {
            entry.1.push(p.clone());
        }
    }
    by_role.sort_by_key(|(r, _)| {
        let rank = ["ceo", "felix", "market-researcher"]
            .iter()
            .position(|x| x == r)
            .unwrap_or(99);
        (rank, r.clone())
    });

    let total: usize = by_role.iter().map(|(_, f)| f.len()).sum();
    let role_count = by_role.len();
    let mut panels = String::new();
    for (role, mut files) in by_role {
        files.sort();
        files.reverse(); // date-prefixed filenames → newest first
        let n = files.len();
        let mut inner = String::new();
        for path in files {
            let f = data::text(env, &path).await;
            all_live &= f.live;
            let name = path.rsplit('/').next().unwrap_or(&path).to_string();
            inner.push_str(&message_doc(&name, &f.text));
        }
        if inner.is_empty() {
            inner = "<p class=\"note\">No messages.</p>".to_string();
        }
        panels.push_str(&section_foot(
            &role,
            "messages waiting for this role",
            &badge(&format!("{n}"), if n > 0 { "info" } else { "" }),
            &inner,
            &format!("<span class=\"mono\">roles/{role}/inbox/</span>")
        ));
    }

    let body = format!(
        "{banner}{kpis}{panels}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        kpis = stat_line(&[
            (fmt_int(total as i64), "messages waiting".to_string(), ""),
            (fmt_int(role_count as i64), "role inboxes".to_string(), ""),
        ]),
        panels = panels,
    );

    shell(
        env,
        "/inboxes",
        trail(&[("Firm", ""), ("Inboxes", "")]),
        all_live,
        &body,
    )
    .await
}

/// A frontmattered markdown file as a collapsible document.
fn message_doc(filename: &str, src: &str) -> String {
    let (fields, body) = render::split_frontmatter(src);
    let get = |k: &str| fields.iter().find(|(f, _)| f == k).map(|(_, v)| v.as_str());
    let title = get("subject").or_else(|| get("slug")).unwrap_or(filename);
    let status = get("status").map(render::status_badge).unwrap_or_default();

    let mut meta = String::new();
    if let (Some(f), Some(t)) = (get("from"), get("to")) {
        meta.push_str(&format!("{} → {}", esc(f), esc(t)));
    }
    if let Some(d) = get("date") {
        if !meta.is_empty() {
            meta.push_str(" · ");
        }
        meta.push_str(&esc(d));
    }
    if !meta.is_empty() {
        meta.push_str(" · ");
    }
    meta.push_str(&format!("<span class=\"mono\">{}</span>", esc(filename)));

    let mut extra = String::new();
    for (k, v) in &fields {
        if matches!(k.as_str(), "subject" | "slug" | "status" | "from" | "to" | "date") {
            continue;
        }
        if v.starts_with('[') {
            let list: Vec<String> = v
                .trim_matches(['[', ']'])
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            extra.push_str(&render::row(k, &chip_row(&list)));
        } else {
            extra.push_str(&render::row(k, &esc(v)));
        }
    }

    doc(
        &format!("{}{}", esc(title), status),
        &meta,
        &format!(
            "{}<div class=\"prose\">{}</div>",
            if extra.is_empty() {
                String::new()
            } else {
                render::rows(&extra)
            },
            markdown(body)
        ),
        false,
    )
}

// ---------------------------------------------------------------------------
// /ideas
// ---------------------------------------------------------------------------

pub async fn ideas(env: &Env) -> String {
    let (paths, tree_live) = data::tree(env).await;
    let mut all_live = tree_live;

    let mut files = data::files_in(&paths, "ideas", ".md");
    files.retain(|p| !p.ends_with("README.md"));
    files.reverse(); // date-prefixed → newest first

    let mut docs = String::new();
    let mut statuses: Vec<String> = Vec::new();
    for path in &files {
        let f = data::text(env, path).await;
        all_live &= f.live;
        let (fields, _) = render::split_frontmatter(&f.text);
        if let Some((_, v)) = fields.iter().find(|(k, _)| k == "status") {
            statuses.push(v.clone());
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        docs.push_str(&message_doc(name, &f.text));
    }

    let trialing = statuses.iter().filter(|s| s.starts_with("trial")).count();
    let killed = statuses
        .iter()
        .filter(|s| s.starts_with("kill") || s.as_str() == "rejected")
        .count();

    let body = format!(
        "{banner}{kpis}{panel}",
        banner = if all_live { String::new() } else { snapshot_banner() },
        kpis = stat_line(&[
            (fmt_int(files.len() as i64), "ideas filed".to_string(), ""),
            (fmt_int(trialing as i64), "taken to trial".to_string(), "warn"),
            (fmt_int(killed as i64), "screened out".to_string(), ""),
        ]),
        panel = section_foot(
            "Backlog",
            "the market researcher's candidate strategies",
            &badge(&render::count(files.len(), "idea"), ""),
            &if docs.is_empty() {
                render::empty_state(
                    "No ideas filed yet",
                    "<div>The market researcher writes one per run.</div>",
                )
            } else {
                docs
            },
            "<span class=\"mono\">ideas/</span><a href=\"/strategies\">Strategies →</a>"
        ),
    );

    shell(
        env,
        "/ideas",
        trail(&[("Research", ""), ("Ideas", "")]),
        all_live,
        &body,
    )
    .await
}

// ---------------------------------------------------------------------------
// /wiki
// ---------------------------------------------------------------------------

pub async fn wiki(env: &Env) -> String {
    let index = data::text(env, "wiki/index.md").await;
    let (paths, tree_live) = data::tree(env).await;
    let all_live = index.live && tree_live;
    let pages = data::wiki_pages(&paths);

    // Group by folder: reference/, recipes/, top level.
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for p in &pages {
        let rest = p.trim_start_matches("wiki/").trim_end_matches(".md");
        let (folder, name) = match rest.split_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => ("pages".to_string(), rest.to_string()),
        };
        match groups.iter_mut().find(|(g, _)| *g == folder) {
            Some(g) => g.1.push(name),
            None => groups.push((folder, vec![name])),
        }
    }
    groups.sort_by(|a, b| a.0.cmp(&b.0));

    let mut group_panels = String::new();
    for (folder, names) in &groups {
        let mut list = String::new();
        for n in names {
            let href = if folder == "pages" {
                format!("/wiki/{n}")
            } else {
                format!("/wiki/{folder}/{n}")
            };
            list.push_str(&format!(
                "<li><a href=\"{}\">{} {}</a></li>",
                esc(&href),
                icon("book"),
                esc(n)
            ));
        }
        group_panels.push_str(&section(
            folder,
            &format!("wiki/{}", if folder == "pages" { "" } else { folder }),
            &badge(&format!("{}", names.len()), ""),
            &format!("<ul class=\"link-list\">{list}</ul>"),
        ));
    }

    let body = format!(
        "{banner}{kpis}<div class=\"grid-main\">{index_panel}{groups}</div>",
        banner = if all_live { String::new() } else { snapshot_banner() },
        kpis = stat_line(&[
            (fmt_int(pages.len() as i64), "pages".to_string(), ""),
            (fmt_int(groups.len() as i64), "sections".to_string(), ""),
        ]),
        index_panel = section_foot(
            "Index",
            "durable, cross-strategy knowledge — what generalises beyond one run",
            "",
            &format!("<div class=\"prose\">{}</div>", markdown_body(&index.text)),
            "<span class=\"mono\">wiki/index.md</span>"
        ),
        groups = format!("<div class=\"stack\">{group_panels}</div>"),
    );

    shell(env, "/wiki", trail(&[("Firm", ""), ("Wiki", "")]), all_live, &body).await
}

pub async fn wiki_page(env: &Env, page: &str) -> String {
    let page = page.trim_end_matches(".md");
    let crumbs = trail(&[("Firm", ""), ("Wiki", "/wiki"), (page, "")]);
    if !data::safe_path(page) {
        return shell(
            env,
            "/wiki",
            crumbs,
            true,
            &render::empty_state("Bad page path", ""),
        )
        .await;
    }
    let f = data::text(env, &format!("wiki/{page}.md")).await;
    let body = if f.is_empty() {
        section(
            page,
            &format!("wiki/{page}.md"),
            "",
            &render::empty_state(
                "Not found",
                "<div><a class=\"link\" href=\"/wiki\">Back to the wiki index</a>.</div>",
            ),
        )
    } else {
        section_foot(
            &render::md_title(&f.text).unwrap_or_else(|| page.to_string()),
            &format!("wiki/{page}.md"),
            "",
            &format!("<div class=\"prose prose-wide\">{}</div>", markdown_body(&f.text)),
            &format!("<span class=\"mono\">wiki/{page}.md</span><a href=\"/wiki\">← wiki index</a>")
        )
    };
    shell(env, "/wiki", crumbs, f.live, &body).await
}

// ---------------------------------------------------------------------------
// /snapshots
// ---------------------------------------------------------------------------

pub async fn snapshots(env: &Env) -> String {
    let body = match snapshots::latest(env).await {
        None => render::empty_state(
            "No snapshots found",
            "<div>Nothing under <span class=\"mono\">snapshots/books/</span> for today or yesterday (UTC). Either the R2 binding is unavailable in this environment (local <span class=\"mono\">wrangler dev</span> simulates an empty bucket — use <span class=\"mono\">--remote</span>) or the snapshot worker has not written yet (hourly at :07 UTC).</div>",
        ),
        Some(l) => {
            let slugs = snapshots::market_slugs(&l.doc);
            let fresh = l.age_mins <= 120;
            let kpis = stat_line(&[
                (
                    format!(
                        "<span class=\"mono\">{}</span>",
                        esc(l.key.trim_start_matches("snapshots/books/"))
                    ),
                    "latest snapshot".to_string(),
                    "",
                ),
                (
                    format!("{} min", l.age_mins),
                    "old".to_string(),
                    if fresh { "ok" } else { "bad" },
                ),
                (
                    fmt_int(slugs.len() as i64),
                    "markets watched".to_string(),
                    "",
                ),
            ]);

            let mut options = String::new();
            for s in &slugs {
                options.push_str(&format!("<option value=\"{0}\">{0}</option>", esc(s)));
            }
            let chart = format!(
                r#"<div class="chart-controls"><select id="snap-market" aria-label="Market">{options}</select></div>
<div class="chart" id="chart-snap"></div>
<script src="/charts.js"></script>
<script>
(function () {{
  var date = "{date}";
  var sel = document.getElementById("snap-market");
  var box = document.getElementById("chart-snap");
  function show(slug) {{
    fetch("/snapshots/data/" + date + "/" + slug + ".json")
      .then(function (r) {{ return r.json(); }})
      .then(function (d) {{ box.innerHTML = ""; Chart.line(box, d, {{yPrecision:4}}); }})
      .catch(function () {{ box.textContent = "failed to load series"; }});
  }}
  sel.addEventListener("change", function () {{ show(sel.value); }});
  if (sel.value) show(sel.value);
}})();
</script>"#,
                options = options,
                date = esc(&l.date),
            );

            format!(
                "{kpis}<div class=\"grid-main\">{chart_panel}{table_panel}</div>",
                kpis = kpis,
                chart_panel = section_foot(
                    "Midpoint series",
                    "hourly midpoints for one market, from the R2 objects",
                    &badge(&l.date, ""),
                    &chart,
                    "<span>drag to zoom, double-click to reset</span><span class=\"mono\">snapshots/books/</span>"
                ),
                table_panel = section(
                    "Latest book",
                    "first outcome token per market",
                    &badge(&render::count(slugs.len(), "market"), ""),
                    &book_table(&l.doc),
                ),
            )
        }
    };

    let body = format!(
        "{body}{}",
        render::note(
            "Written hourly at :07 UTC by <span class=\"mono\">workers/snapshot</span> into R2 bucket <span class=\"mono\">orakel</span>; read here at request time (series cached 5 minutes). Table and chart use each market's first outcome token.",
        )
    );

    shell(
        env,
        "/snapshots",
        trail(&[("Data", ""), ("Snapshots", "")]),
        true,
        &body,
    )
    .await
}

/// Latest snapshot document → per-market table (first outcome token).
fn book_table(doc_json: &Value) -> String {
    let empty = Vec::new();
    let markets = doc_json["markets"].as_array().unwrap_or(&empty);
    let num = |v: &Value, prec: usize| -> String {
        v.as_f64()
            .map(|f| format!("{f:.prec$}"))
            .unwrap_or_else(|| "—".to_string())
    };
    let rows: Vec<Vec<String>> = markets
        .iter()
        .map(|m| {
            let tok = &m["tokens"][0];
            let depth = |side: &str| -> String {
                tok[side]
                    .as_array()
                    .map(|ls| {
                        let sum: f64 = ls.iter().filter_map(|l| l[1].as_f64()).sum();
                        format!("{sum:.0}")
                    })
                    .unwrap_or_else(|| "—".to_string())
            };
            let slug = m["market_slug"].as_str().unwrap_or("?");
            vec![
                format!("<a href=\"/markets/{0}\">{0}</a>", esc(slug)),
                num(&tok["midpoint"], 4),
                num(&tok["bids"][0][0], 3),
                num(&tok["asks"][0][0], 3),
                depth("bids"),
                depth("asks"),
            ]
        })
        .collect();

    table(
        &[
            ("Market", ""),
            ("Midpoint", "num"),
            ("Best bid", "num"),
            ("Best ask", "num"),
            ("Bid depth", "num"),
            ("Ask depth", "num"),
        ],
        &rows,
    )
}
