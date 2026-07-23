//! orakel dashboard — Cloudflare Worker (workers-rs), v2: live data.
//!
//! Primary data path: LIVE reads at request time — repo files from the GitHub
//! API (src/live.rs, 60s Cache API caching) and hourly book snapshots from R2
//! (src/snapshots.rs). The build-time include_str! snapshot remains as
//! FALLBACK: no GITHUB_TOKEN secret or a failed fetch → pages render the
//! embedded data with a "stale" notice, never an error page.
//!
//! No external assets. Navigation is plain <a> sub-pages in a grouped sidebar;
//! the only scripting is a tiny localStorage helper that remembers which nav
//! groups are folded (native <details> folds without it) and a CSS-only burger
//! menu on mobile. /dev and /snapshots load /charts.js, our hand-rolled
//! dependency-free chart framework.

mod live;
mod render;
mod snapshots;

use render::{
    badge, badge_row, breadcrumb, card, chip_row, csv_table, empty_state, esc, ext_link, file_ref,
    kv, layout, markdown, market_url, note, parse_csv, section, subsection,
};
use serde_json::Value;
use worker::{event, Context, Env, Headers, Request, Response, Result, Router};

// ---------------------------------------------------------------------------
// Embedded repo snapshot (FALLBACK when live reads are unavailable).
// Paths are relative to THIS FILE (src/lib.rs). Specific known files only —
// no globs. Files that may not exist yet (predictions/scores.csv) are staged
// into OUT_DIR by build.rs so the crate always builds.
// ---------------------------------------------------------------------------

const STATE_TOML: &str = include_str!("../../ops/state.toml");
const DECISIONS_MD: &str = include_str!("../../ops/decisions.md");
const RUNS_README_MD: &str = include_str!("../../ops/runs/README.md");
const PREDICTIONS_CSV: &str = include_str!("../../predictions/predictions.csv");
const RESOLUTIONS_CSV: &str = include_str!("../../predictions/resolutions.csv");
const SCORES_CSV: &str = include_str!(concat!(env!("OUT_DIR"), "/scores.csv"));
const INBOX_CEO_MD: &str = include_str!("../../roles/ceo/inbox/README.md");
const INBOX_FELIX_MD: &str = include_str!("../../roles/felix/inbox/README.md");
const INBOX_MARKET_RESEARCHER_MD: &str =
    include_str!("../../roles/market-researcher/inbox/README.md");
const WIKI_INDEX_MD: &str = include_str!("../../wiki/index.md");
const CSS: &str = include_str!("style.css");
const CHARTS_JS: &str = include_str!("charts.js");
const FAVICON_SVG: &str = include_str!("favicon.svg");

/// Set by build.rs at compile time — the build stamp is always compile-time.
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn static_response(body: &str, content_type: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("content-type", content_type)?;
    headers.set("cache-control", "public, max-age=300")?;
    Ok(Response::ok(body)?.with_headers(headers))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", |_, ctx| async move {
            Response::from_html(page_ops_state(&ctx.env).await)
        })
        .get_async("/decisions", |_, ctx| async move {
            Response::from_html(page_ops_decisions(&ctx.env).await)
        })
        .get_async("/runs", |_, ctx| async move {
            Response::from_html(page_ops_runs(&ctx.env).await)
        })
        .get_async("/ideas", |_, ctx| async move {
            Response::from_html(page_ops_ideas(&ctx.env).await)
        })
        .get_async("/strategies", |_, ctx| async move {
            Response::from_html(page_strategies(&ctx.env).await)
        })
        .get_async("/strategies/:family/:variant", |_, ctx| async move {
            let family = ctx.param("family").cloned().unwrap_or_default();
            let variant = ctx.param("variant").cloned().unwrap_or_default();
            Response::from_html(page_strategy_detail(&ctx.env, &family, &variant).await)
        })
        .get_async("/predictions", |_, ctx| async move {
            Response::from_html(page_predictions(&ctx.env).await)
        })
        .get_async("/resolutions", |_, ctx| async move {
            Response::from_html(page_resolutions(&ctx.env).await)
        })
        .get_async("/scores", |_, ctx| async move {
            Response::from_html(page_scores(&ctx.env).await)
        })
        .get_async("/snapshots", |_, ctx| async move {
            Response::from_html(page_snapshots(&ctx.env).await)
        })
        .get_async("/snapshots/data/:date/:file", |req, ctx| async move {
            let date = ctx.param("date").cloned().unwrap_or_default();
            let file = ctx.param("file").cloned().unwrap_or_default();
            let Some(slug) = file.strip_suffix(".json") else {
                return Response::error("bad request", 400);
            };
            let url = req.url()?;
            snapshots::series_json(&ctx.env, &date, slug, url.as_str()).await
        })
        .get_async("/inboxes", |_, ctx| async move {
            Response::from_html(page_inboxes(&ctx.env).await)
        })
        .get_async("/wiki", |req, ctx| async move {
            let url = req.url()?;
            let page = url
                .query_pairs()
                .find(|(k, _)| k == "page")
                .map(|(_, v)| v.into_owned());
            Response::from_html(page_wiki(&ctx.env, page).await)
        })
        .get("/dev", |_, _| Response::from_html(page_dev()))
        .get("/dev/endpoints", |_, _| {
            Response::from_html(page_dev_endpoints())
        })
        .get("/dev/data/line.json", |_, _| {
            static_response(&dev_line_json(), "application/json; charset=utf-8")
        })
        .get("/dev/data/bar.json", |_, _| {
            static_response(&dev_bar_json(), "application/json; charset=utf-8")
        })
        .get("/style.css", |_, _| {
            static_response(CSS, "text/css; charset=utf-8")
        })
        .get("/charts.js", |_, _| {
            static_response(CHARTS_JS, "text/javascript; charset=utf-8")
        })
        .get("/favicon.svg", |_, _| {
            static_response(FAVICON_SVG, "image/svg+xml")
        })
        .run(req, env)
        .await
}

/// Rendered at the top of a page when any live read fell back to the
/// embedded build-time snapshot.
fn stale_notice() -> String {
    format!(
        "<p class=\"note\">{} live GitHub read unavailable — showing the embedded snapshot from build {}. (Set the GITHUB_TOKEN Worker secret to enable live reads.)</p>",
        badge("stale", "warn"),
        esc(BUILD_TIMESTAMP)
    )
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

async fn page_ops_state(env: &Env) -> String {
    let state = live::repo_text(env, "ops/state.toml", STATE_TOML).await;

    // Parse whichever text we have (live or embedded). NB: parse as
    // toml::Table, a document (toml::Value::from_str rejects documents).
    let state_html = match state.text.parse::<toml::Table>() {
        Ok(t) => state_sections(&t),
        Err(e) => empty_state(
            "Failed to parse ops/state.toml",
            &format!("<div class=\"mono\">{}</div>", esc(&e.to_string())),
        ),
    };

    let notice = if state.live { String::new() } else { stale_notice() };
    layout(
        "/",
        "State",
        "The canonical current operating state of the firm — ops/state.toml.",
        &format!("{notice}{state_html}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_ops_decisions(env: &Env) -> String {
    let decisions = live::repo_text(env, "ops/decisions.md", DECISIONS_MD).await;
    let body = file_ref(
        "ops/decisions.md",
        "Append-only. Every structural change: what, why, who. Newest first.",
    ) + &format!("<div class=\"prose\">{}</div>", markdown(&decisions.text));
    let notice = if decisions.live { String::new() } else { stale_notice() };
    layout(
        "/decisions",
        "Decisions",
        "The append-only decision log — every structural change to the firm.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_ops_runs(env: &Env) -> String {
    let runs_readme = live::repo_text(env, "ops/runs/README.md", RUNS_README_MD).await;
    let tree = live::repo_tree(env).await;
    let mut all_live = runs_readme.live && tree.is_some();

    let mut body = String::new();
    match &tree {
        Some(paths) => {
            let mut manifests: Vec<&String> = paths
                .iter()
                .filter(|p| p.starts_with("ops/runs/") && p.ends_with(".toml"))
                .collect();
            // "<date>.toml" is run 1 of the day, "<date>-N.toml" run N.
            manifests.sort_by_key(|p| {
                let stem = p.trim_start_matches("ops/runs/").trim_end_matches(".toml");
                let (date, n) = if stem.len() > 10 && stem.as_bytes()[10] == b'-' {
                    (&stem[..10], stem[11..].parse::<u32>().unwrap_or(1))
                } else {
                    (stem, 1)
                };
                (date.to_string(), n)
            });
            manifests.reverse();
            if manifests.is_empty() {
                body.push_str(&empty_state(
                    "No run manifests yet",
                    "<div>ops/runs/ contains no <span class=\"mono\">&lt;YYYY-MM-DD&gt;.toml</span> manifests. The CEO writes one per orchestrated run.</div>",
                ));
            }
            for path in manifests {
                let m = live::repo_text(env, path, "").await;
                all_live &= m.live;
                let stem = path.trim_start_matches("ops/runs/").trim_end_matches(".toml");
                body.push_str(&section(stem, &run_manifest_html(&m.text)));
            }
        }
        None => {
            body.push_str(&empty_state(
                "Run listing unavailable",
                "<div>Live listing of <span class=\"mono\">ops/runs/</span> needs the GitHub tree API; manifests are not embedded at build time.</div>",
            ));
        }
    }
    body.push_str(&file_ref("ops/runs/README.md", "Manifest format reference."));
    body.push_str(&format!(
        "<div class=\"prose\">{}</div>",
        markdown(&runs_readme.text)
    ));

    let notice = if all_live { String::new() } else { stale_notice() };
    layout(
        "/runs",
        "Runs",
        "Orchestrated-run manifests — one per CEO run, newest first.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_ops_ideas(env: &Env) -> String {
    let tree = live::repo_tree(env).await;
    let mut all_live = tree.is_some();

    let mut body = String::new();
    match &tree {
        Some(paths) => {
            let mut files: Vec<&String> = paths
                .iter()
                .filter(|p| {
                    p.starts_with("ideas/")
                        && p.ends_with(".md")
                        && !p.ends_with("README.md")
                        && !p["ideas/".len()..].contains('/')
                })
                .collect();
            files.sort();
            files.reverse();
            if files.is_empty() {
                body.push_str(
                    "<p class=\"muted-line\">No ideas filed yet — the market researcher writes one per run.</p>",
                );
            }
            for path in files {
                let f = live::repo_text(env, path, "").await;
                all_live &= f.live;
                let name = path.trim_start_matches("ideas/");
                body.push_str(&message_html(name, &f.text));
            }
        }
        None => {
            body.push_str(&empty_state(
                "Ideas listing unavailable",
                "<div>Live listing of <span class=\"mono\">ideas/</span> needs the GitHub tree API; idea files are not embedded at build time.</div>",
            ));
        }
    }

    let notice = if all_live { String::new() } else { stale_notice() };
    layout(
        "/ideas",
        "Ideas",
        "The market researcher's backlog — one strategy idea per run, newest first.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_predictions(env: &Env) -> String {
    let predictions = live::repo_text(env, "predictions/predictions.csv", PREDICTIONS_CSV).await;

    let rows = parse_csv(&predictions.text);
    let body = if rows.len() <= 1 {
        let header = rows.first().cloned().unwrap_or_default();
        empty_state(
            "No predictions yet",
            &format!(
                "<div>predictions/predictions.csv holds only its header. Rows appear once research slots start producing signals.</div>{}",
                chip_row(&header)
            ),
        )
    } else {
        let n = rows.len() - 1;
        format!(
            "<p class=\"muted-line\">{} prediction{} · market slugs link to Polymarket.</p>{}",
            n,
            if n == 1 { "" } else { "s" },
            csv_table(&rows)
        )
    };

    let notice = if predictions.live { String::new() } else { stale_notice() };
    layout(
        "/predictions",
        "Prediction log",
        "The canonical append-only prediction log — predictions/predictions.csv.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_resolutions(env: &Env) -> String {
    let resolutions = live::repo_text(env, "predictions/resolutions.csv", RESOLUTIONS_CSV).await;
    let res_rows = parse_csv(&resolutions.text);
    let body = if res_rows.len() <= 1 {
        let header = res_rows.first().cloned().unwrap_or_default();
        empty_state(
            "No resolutions yet",
            &format!(
                "<div>predictions/resolutions.csv is appended by the CEO when a market resolves.</div>{}",
                chip_row(&header)
            ),
        )
    } else {
        csv_table(&res_rows)
    };
    let notice = if resolutions.live { String::new() } else { stale_notice() };
    layout(
        "/resolutions",
        "Resolutions",
        "Market resolutions — appended by the CEO when a market settles.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_scores(env: &Env) -> String {
    // scores.csv may not exist yet in the repo — a live 404 renders the empty
    // state, the embedded placeholder does the same on fallback.
    let scores = live::repo_text(env, "predictions/scores.csv", SCORES_CSV).await;
    let score_rows = parse_csv(&scores.text);
    let body = if score_rows.len() <= 1 {
        empty_state(
            "No scores yet",
            "<div>predictions/scores.csv has not been generated — <span class=\"mono\">scoring/</span> runs once the first predictions resolve.</div>",
        )
    } else {
        csv_table(&score_rows)
    };
    let notice = if scores.live { String::new() } else { stale_notice() };
    layout(
        "/scores",
        "Scores",
        "Scoring aggregates — Brier and calibration by application, variant, and family.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_snapshots(env: &Env) -> String {
    let body = match snapshots::latest(env).await {
        None => empty_state(
            "No snapshots found",
            "<div>Nothing under <span class=\"mono\">snapshots/books/</span> for today or yesterday (UTC). Either the R2 binding is unavailable in this environment (local <span class=\"mono\">wrangler dev</span> simulates an empty bucket — use <span class=\"mono\">--remote</span>) or the snapshot worker has not written yet (hourly at :07 UTC).</div>",
        ),
        Some(l) => {
            let slugs = snapshots::market_slugs(&l.doc);
            let age_html = if l.age_mins > 120 {
                badge(&format!("{} min — stale", l.age_mins), "bad")
            } else {
                badge(&format!("{} min", l.age_mins), "ok")
            };
            let stats = format!(
                "<div class=\"stats\">{}{}{}</div>",
                stat(
                    "latest snapshot",
                    &format!(
                        "<small><span class=\"mono\">{}</span></small>",
                        esc(l.key.trim_start_matches("snapshots/books/"))
                    ),
                ),
                stat("age", &age_html),
                stat("markets", &slugs.len().to_string()),
            );

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
      .then(function (d) {{ box.innerHTML = ""; Chart.line(box, d); }})
      .catch(function () {{ box.textContent = "failed to load series"; }});
  }}
  sel.addEventListener("change", function () {{ show(sel.value); }});
  if (sel.value) show(sel.value);
}})();
</script>"#,
                options = options,
                date = esc(&l.date),
            );

            stats
                + &section("Latest book", &snapshot_table(&l.doc))
                + &section("Midpoint series (hourly)", &chart)
                + &note("Written hourly at :07 UTC by workers/snapshot into R2 bucket <span class=\"mono\">orakel</span>; read here at request time (series cached 5 min). Table and chart use each market's first outcome token.")
        }
    };

    layout(
        "/snapshots",
        "Snapshots",
        "Hourly Polymarket order-book snapshots from R2 (snapshots/books/, watchlist-ordered).",
        &body,
        BUILD_TIMESTAMP,
    )
}

/// Latest snapshot document → per-market table (first outcome token:
/// midpoint, best bid/ask, summed size over the stored top-10 levels).
fn snapshot_table(doc: &Value) -> String {
    let mut rows: Vec<Vec<String>> = vec![
        ["market", "midpoint", "best bid", "best ask", "bid depth", "ask depth"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    ];
    let empty = Vec::new();
    for m in doc["markets"].as_array().unwrap_or(&empty) {
        let tok = &m["tokens"][0];
        let num = |v: &Value, prec: usize| -> String {
            v.as_f64()
                .map(|f| format!("{f:.prec$}"))
                .unwrap_or_else(|| "—".to_string())
        };
        let depth = |side: &str| -> String {
            tok[side]
                .as_array()
                .map(|ls| {
                    let sum: f64 = ls.iter().filter_map(|l| l[1].as_f64()).sum();
                    format!("{sum:.0}")
                })
                .unwrap_or_else(|| "—".to_string())
        };
        rows.push(vec![
            m["market_slug"].as_str().unwrap_or("?").to_string(),
            num(&tok["midpoint"], 4),
            num(&tok["bids"][0][0], 3),
            num(&tok["asks"][0][0], 3),
            depth("bids"),
            depth("asks"),
        ]);
    }
    csv_table(&rows)
}

async fn page_inboxes(env: &Env) -> String {
    let tree = live::repo_tree(env).await;

    let (body, notice) = match tree {
        Some(paths) => {
            // Discover every roles/<role>/inbox/ (role may be a nested path,
            // e.g. a per-variant researcher). README.md marks the inbox as
            // existing but is not a message.
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

            let mut html = String::new();
            let mut all_live = true;
            for (role, mut files) in by_role {
                files.sort();
                files.reverse(); // date-prefixed filenames → newest first
                let mut inner = String::new();
                if files.is_empty() {
                    inner.push_str("<p class=\"muted-line\">No messages.</p>");
                }
                for path in files {
                    let f = live::repo_text(env, &path, "").await;
                    all_live &= f.live;
                    let name = path.rsplit('/').next().unwrap_or(&path).to_string();
                    inner.push_str(&message_html(&name, &f.text));
                }
                html.push_str(&section(&role, &inner));
            }
            let notice = if all_live { String::new() } else { stale_notice() };
            (html, notice)
        }
        None => {
            // Embedded fallback: only the three inbox READMEs are baked in.
            let inboxes: [(&str, &str, &str); 3] = [
                ("ceo", "roles/ceo/inbox/", INBOX_CEO_MD),
                ("felix", "roles/felix/inbox/", INBOX_FELIX_MD),
                (
                    "market-researcher",
                    "roles/market-researcher/inbox/",
                    INBOX_MARKET_RESEARCHER_MD,
                ),
            ];
            let mut cards = String::new();
            for (role, path, readme) in inboxes.iter() {
                cards.push_str(&card(
                    role,
                    path,
                    &format!("<div class=\"prose\">{}</div>", markdown(readme)),
                ));
            }
            let html = format!(
                "<div class=\"grid-cards\" style=\"grid-template-columns:repeat(auto-fill,minmax(340px,1fr))\">{}</div>",
                cards
            );
            (html, stale_notice())
        }
    };

    layout(
        "/inboxes",
        "Inboxes",
        "Per-role message queues (roles/<role>/inbox/). Frontmatter status renders as a badge.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

async fn page_wiki(env: &Env, page: Option<String>) -> String {
    // Sub-page view: /wiki?page=<path-under-wiki-without-.md>
    if let Some(p) = page {
        let safe = !p.is_empty()
            && p.len() <= 200
            && !p.contains("..")
            && !p.starts_with('/')
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'));
        let back = "<p class=\"file-ref\"><a href=\"/wiki\">← wiki index</a></p>".to_string();
        let body = if !safe {
            back + &empty_state("Bad page path", "")
        } else {
            let f = live::repo_text(env, &format!("wiki/{p}.md"), "").await;
            if !f.live {
                format!(
                    "{}{}{}",
                    stale_notice(),
                    back,
                    empty_state(
                        "Live fetch unavailable",
                        "<div>Individual wiki pages are not embedded at build time.</div>",
                    )
                )
            } else if f.text.is_empty() {
                back + &empty_state("Not found", "")
            } else {
                format!(
                    "{back}{}<div class=\"prose\">{}</div>",
                    file_ref(&format!("wiki/{p}.md"), ""),
                    markdown(&f.text)
                )
            }
        };
        return layout("/wiki", "Wiki", "", &body, BUILD_TIMESTAMP);
    }

    let index = live::repo_text(env, "wiki/index.md", WIKI_INDEX_MD).await;
    let tree = live::repo_tree(env).await;
    let mut all_live = index.live && tree.is_some();

    let mut body = file_ref(
        "wiki/index.md",
        "Durable, cross-strategy knowledge. Maintained by the market researcher.",
    ) + &format!("<div class=\"prose\">{}</div>", markdown(&index.text));

    if let Some(paths) = &tree {
        let mut pages: Vec<&String> = paths
            .iter()
            .filter(|p| p.starts_with("wiki/") && p.ends_with(".md") && *p != "wiki/index.md")
            .collect();
        pages.sort();
        let mut list = String::new();
        for p in pages {
            let stem = p.trim_start_matches("wiki/").trim_end_matches(".md");
            list.push_str(&format!(
                "<li><a href=\"/wiki?page={0}\"><span class=\"mono\">{0}</span></a></li>",
                esc(stem)
            ));
        }
        if !list.is_empty() {
            body.push_str(&section(
                "Pages",
                &format!("<ul class=\"link-list\">{list}</ul>"),
            ));
        }
    } else {
        all_live = false;
    }

    let notice = if all_live { String::new() } else { stale_notice() };
    layout(
        "/wiki",
        "Wiki",
        "Index of the firm's knowledge base.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

// ---------------------------------------------------------------------------
// Strategies — family → variant catalogue (list) and variant detail
// ---------------------------------------------------------------------------

const GH_REPO: &str = "felix-andreas/orakel";

fn gh_blob(path: &str) -> String {
    format!("https://github.com/{GH_REPO}/blob/main/{path}")
}

fn gh_tree(path: &str) -> String {
    format!("https://github.com/{GH_REPO}/tree/main/{path}")
}

/// A single path segment is safe to interpolate into a repo path / URL when it
/// is a plain slug (no traversal, no slashes). Guards the :family/:variant route.
fn safe_segment(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && !s.contains("..")
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// First real paragraph of a markdown doc (skips a leading `# heading` and
/// blank lines), collapsed to one line. Used for family blurbs.
fn first_paragraph(md: &str) -> String {
    let mut para: Vec<&str> = Vec::new();
    for line in md.lines() {
        let t = line.trim();
        if para.is_empty() && (t.is_empty() || t.starts_with('#')) {
            continue;
        }
        if t.is_empty() {
            break;
        }
        para.push(t);
    }
    para.join(" ")
}

struct VariantRow {
    family: String,
    variant: String,
    status: String,
    created: String,
    labels: Vec<String>,
    apps: usize,
}

/// Count a variant's live/parked applications from the tree (application tomls
/// under applications/, excluding the `_market.toml` template).
fn count_apps(paths: &[String], family: &str, variant: &str) -> usize {
    let prefix = format!("strategies/{family}/{variant}/applications/");
    paths
        .iter()
        .filter(|p| {
            p.starts_with(&prefix)
                && p.ends_with(".toml")
                && !p.ends_with("/_market.toml")
                && p[prefix.len()..].matches('/').count() == 0
        })
        .count()
}

async fn page_strategies(env: &Env) -> String {
    let tree = live::repo_tree(env).await;
    let Some(paths) = tree else {
        let body = stale_notice()
            + &empty_state(
                "Strategy listing unavailable",
                "<div>Live listing of <span class=\"mono\">strategies/</span> needs the GitHub tree API; strategy files are not embedded at build time.</div>",
            );
        return layout(
            "/strategies",
            "Strategies",
            "The firm's strategy catalogue — families and their variants.",
            &body,
            BUILD_TIMESTAMP,
        );
    };

    // Discover variants: strategies/<family>/<variant>/strategy.toml (skip the
    // _template scaffold).
    let mut variants: Vec<(String, String)> = Vec::new();
    for p in &paths {
        let Some(rest) = p.strip_prefix("strategies/") else {
            continue;
        };
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 3 && parts[2] == "strategy.toml" && parts[0] != "_template" {
            variants.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    variants.sort();

    let mut all_live = true;
    let mut rows: Vec<VariantRow> = Vec::new();
    for (family, variant) in &variants {
        let f = live::repo_text(
            env,
            &format!("strategies/{family}/{variant}/strategy.toml"),
            "",
        )
        .await;
        all_live &= f.live;
        let t = f.text.parse::<toml::Table>().unwrap_or_default();
        rows.push(VariantRow {
            family: family.clone(),
            variant: variant.clone(),
            status: t
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            created: t
                .get("created")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            labels: t
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            apps: count_apps(&paths, family, variant),
        });
    }

    // Headline stats: families, variants, and status breakdown.
    let families: Vec<String> = {
        let mut fs: Vec<String> = rows.iter().map(|r| r.family.clone()).collect();
        fs.sort();
        fs.dedup();
        fs
    };
    let count = |st: &str| rows.iter().filter(|r| r.status == st).count();
    let stats = format!(
        "<div class=\"stats\">{}{}{}{}</div>",
        stat("families", &families.len().to_string()),
        stat("variants", &rows.len().to_string()),
        stat("live", &format!("{}", count("live"))),
        stat(
            "in trial",
            &format!("{}<small> / {} retired</small>", count("trial"), count("retired")),
        ),
    );

    let mut body = stats;
    if rows.is_empty() {
        body.push_str(&empty_state(
            "No strategies yet",
            "<div>No variants under <span class=\"mono\">strategies/</span>. The CEO fills research slots from the ideas backlog.</div>",
        ));
    }

    for family in &families {
        let fam_md = live::repo_text(env, &format!("strategies/{family}/FAMILY.md"), "").await;
        all_live &= fam_md.live;
        let blurb = first_paragraph(&fam_md.text);
        let blurb_html = if blurb.is_empty() {
            String::new()
        } else {
            format!("<p class=\"strat-family-blurb\">{}</p>", esc(&blurb))
        };
        let header = format!(
            "<div class=\"strat-family-head\"><h2 class=\"section-title\">{}</h2>{}</div>{}",
            esc(family),
            ext_link(&gh_blob(&format!("strategies/{family}/FAMILY.md")), "FAMILY.md"),
            blurb_html,
        );

        let mut cards = String::new();
        for r in rows.iter().filter(|r| &r.family == family) {
            cards.push_str(&strategy_card(r));
        }

        body.push_str(&format!(
            "<section class=\"section strat-family\">{header}<div class=\"strat-grid\">{cards}</div></section>"
        ));
    }

    body.push_str(&note(
        "Variants live under <span class=\"mono\">strategies/&lt;family&gt;/&lt;variant&gt;/</span>; the catalogue is derived live from the repo tree.",
    ));

    let notice = if all_live { String::new() } else { stale_notice() };
    layout(
        "/strategies",
        "Strategies",
        "The firm's strategy catalogue — families and their variants.",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

/// One variant as a clickable card linking to its detail page.
fn strategy_card(r: &VariantRow) -> String {
    let tone = match r.status.as_str() {
        "live" => "ok",
        "trial" => "warn",
        "retired" => "bad",
        _ => "",
    };
    let labels = if r.labels.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"strat-labels\">{}</div>",
            r.labels
                .iter()
                .map(|l| format!("<span class=\"chip\">{}</span>", esc(l)))
                .collect::<String>()
        )
    };
    format!(
        "<a class=\"strat-card\" href=\"/strategies/{fam}/{var}\"><div class=\"strat-card-head\"><span class=\"strat-card-title\">{var_esc}</span>{badge}</div><div class=\"strat-card-meta\">created {created} · {apps} application{plural}</div>{labels}</a>",
        fam = esc(&r.family),
        var = esc(&r.variant),
        var_esc = esc(&r.variant),
        badge = badge(&r.status, tone),
        created = esc(&r.created),
        apps = r.apps,
        plural = if r.apps == 1 { "" } else { "s" },
        labels = labels,
    )
}

async fn page_strategy_detail(env: &Env, family: &str, variant: &str) -> String {
    let crumbs = breadcrumb(&[
        ("/strategies", "Strategies"),
        ("", family),
        ("", variant),
    ]);

    if !safe_segment(family) || !safe_segment(variant) {
        return layout(
            "/strategies",
            "Strategy",
            "",
            &(crumbs + &empty_state("Bad strategy path", "")),
            BUILD_TIMESTAMP,
        );
    }

    let prefix = format!("strategies/{family}/{variant}");
    let toml_f = live::repo_text(env, &format!("{prefix}/strategy.toml"), "").await;

    if !toml_f.live && toml_f.text.is_empty() {
        return layout(
            "/strategies",
            "Strategy",
            "",
            &format!(
                "{}{}{}",
                stale_notice(),
                crumbs,
                empty_state(
                    "Live fetch unavailable",
                    "<div>Individual strategy files are not embedded at build time.</div>",
                )
            ),
            BUILD_TIMESTAMP,
        );
    }
    if toml_f.text.is_empty() {
        return layout(
            "/strategies",
            "Strategy",
            "",
            &(crumbs + &empty_state("Strategy not found", &format!("<div class=\"mono\">{}</div>", esc(&prefix)))),
            BUILD_TIMESTAMP,
        );
    }

    let meta = toml_f.text.parse::<toml::Table>().unwrap_or_default();
    let status = meta.get("status").and_then(|v| v.as_str()).unwrap_or("?");
    let created = meta.get("created").and_then(|v| v.as_str()).unwrap_or("?");
    let supersedes = meta
        .get("supersedes")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let labels: Vec<String> = meta
        .get("labels")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let status_tone = match status {
        "live" => "ok",
        "trial" => "warn",
        "retired" => "bad",
        _ => "",
    };

    // Stat strip: status, created, and trial/slot context when present.
    let mut stat_cells = format!(
        "{}{}",
        stat("status", &badge(status, status_tone)),
        stat("created", &format!("<span class=\"mono\">{}</span>", esc(created))),
    );
    if let Some(trial) = meta.get("trial").and_then(|v| v.as_table()) {
        if let Some(slot) = trial.get("slot").and_then(|v| v.as_integer()) {
            if slot > 0 {
                stat_cells.push_str(&stat("slot", &slot.to_string()));
            }
        }
        if let Some(due) = trial.get("review_due").and_then(|v| v.as_str()) {
            stat_cells.push_str(&stat(
                "review due",
                &format!("<span class=\"mono\">{}</span>", esc(due)),
            ));
        }
    }
    let stats = format!("<div class=\"stats\">{stat_cells}</div>");

    let mut body = format!("{crumbs}{stats}");

    if !labels.is_empty() {
        body.push_str(&format!(
            "<div class=\"badge-row strat-detail-labels\">{}</div>",
            labels
                .iter()
                .map(|l| format!("<span class=\"chip\">{}</span>", esc(l)))
                .collect::<String>()
        ));
    }
    if !supersedes.is_empty() {
        body.push_str(&note(&format!(
            "Supersedes <span class=\"mono\">{}</span>.",
            esc(supersedes)
        )));
    }

    // Retirement note, if any.
    if let Some(ret) = meta.get("retirement").and_then(|v| v.as_table()) {
        let date = ret.get("date").and_then(|v| v.as_str()).unwrap_or("?");
        let reason = ret.get("reason").and_then(|v| v.as_str()).unwrap_or("");
        body.push_str(&section(
            "Retirement",
            &format!(
                "{}<p>{}</p>",
                kv("date", &format!("<span class=\"mono\">{}</span>", esc(date))),
                esc(reason)
            ),
        ));
    }

    // Overview — STRATEGY.md rendered.
    let strat_md = live::repo_text(env, &format!("{prefix}/STRATEGY.md"), "").await;
    let mut all_live = toml_f.live && strat_md.live;
    if !strat_md.text.is_empty() {
        body.push_str(&section(
            "Overview",
            &format!(
                "{}<div class=\"prose\">{}</div>",
                file_ref(&format!("{prefix}/STRATEGY.md"), ""),
                markdown(&strat_md.text)
            ),
        ));
    }

    // Applications and results need the tree.
    let tree = live::repo_tree(env).await;
    match &tree {
        Some(paths) => {
            // Applications
            let app_prefix = format!("{prefix}/applications/");
            let mut app_paths: Vec<&String> = paths
                .iter()
                .filter(|p| {
                    p.starts_with(&app_prefix)
                        && p.ends_with(".toml")
                        && !p.ends_with("/_market.toml")
                        && p[app_prefix.len()..].matches('/').count() == 0
                })
                .collect();
            app_paths.sort();
            let mut app_rows = String::new();
            for p in &app_paths {
                let af = live::repo_text(env, p, "").await;
                all_live &= af.live;
                let at = af.text.parse::<toml::Table>().unwrap_or_default();
                let slug = at.get("market_slug").and_then(|v| v.as_str()).unwrap_or("");
                let active = at.get("active").and_then(|v| v.as_bool());
                let added = at.get("added").and_then(|v| v.as_str()).unwrap_or("—");
                let note_txt = at
                    .get("note")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        at.get("params")
                            .and_then(|p| p.get("note"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("");
                let market_cell = if slug.is_empty() {
                    "<span class=\"muted-line\">—</span>".to_string()
                } else {
                    ext_link(&market_url(slug), slug)
                };
                let active_cell = match active {
                    Some(true) => badge("active", "ok"),
                    Some(false) => badge("parked", "warn"),
                    None => badge("—", ""),
                };
                app_rows.push_str(&format!(
                    "<tr><td class=\"cell-market\">{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    market_cell,
                    active_cell,
                    esc(added),
                    esc(note_txt),
                ));
            }
            let apps_html = if app_paths.is_empty() {
                "<p class=\"muted-line\">No applications yet — a variant with no markets is legal.</p>".to_string()
            } else {
                format!(
                    "<div class=\"table-wrap wrap-cells\"><table class=\"data\"><thead><tr><th>market</th><th>status</th><th>added</th><th>note</th></tr></thead><tbody>{}</tbody></table></div>",
                    app_rows
                )
            };
            body.push_str(&section("Applications", &apps_html));

            // Results — rendered write-ups (backtests, post-mortems).
            let res_prefix = format!("{prefix}/results/");
            let mut res_paths: Vec<&String> = paths
                .iter()
                .filter(|p| p.starts_with(&res_prefix) && p.ends_with(".md"))
                .collect();
            res_paths.sort();
            res_paths.reverse();
            if !res_paths.is_empty() {
                let mut res_html = String::new();
                for p in &res_paths {
                    let rf = live::repo_text(env, p, "").await;
                    all_live &= rf.live;
                    let name = p.rsplit('/').next().unwrap_or(p);
                    res_html.push_str(&format!(
                        "<details class=\"msg\"><summary><span class=\"msg-title\">{}</span><span class=\"msg-meta\">{}</span></summary><div class=\"msg-body\"><div class=\"prose\">{}</div></div></details>",
                        esc(name),
                        ext_link(&gh_blob(p), "GitHub"),
                        markdown(&rf.text),
                    ));
                }
                body.push_str(&section("Results", &res_html));
            }
        }
        None => {
            all_live = false;
            body.push_str(&section(
                "Applications & results",
                &empty_state(
                    "Listing unavailable",
                    "<div>Live listing of the variant directory needs the GitHub tree API.</div>",
                ),
            ));
        }
    }

    // Files — jump straight to the source on GitHub.
    let mut file_links = format!(
        "<li>{}</li>",
        ext_link(&gh_tree(&prefix), "variant directory")
    );
    for (label, rel) in [
        ("STRATEGY.md", "STRATEGY.md"),
        ("strategy.toml", "strategy.toml"),
        ("memory/MEMORY.md", "memory/MEMORY.md"),
        ("memory/WORKLOG.md", "memory/WORKLOG.md"),
    ] {
        let full = format!("{prefix}/{rel}");
        if tree
            .as_ref()
            .map(|paths| paths.iter().any(|p| p == &full))
            .unwrap_or(false)
        {
            file_links.push_str(&format!("<li>{}</li>", ext_link(&gh_blob(&full), label)));
        }
    }
    body.push_str(&section(
        "Files",
        &format!("<ul class=\"link-list\">{file_links}</ul>"),
    ));

    let notice = if all_live { String::new() } else { stale_notice() };
    layout(
        "/strategies",
        &format!("{family}/{variant}"),
        "",
        &format!("{notice}{body}"),
        BUILD_TIMESTAMP,
    )
}

// ---------------------------------------------------------------------------
// ops/state.toml → flat State tab (stat strip + flat subsections, no cards)
// ---------------------------------------------------------------------------

fn stat(label: &str, value_html: &str) -> String {
    format!(
        "<div class=\"stat\"><div class=\"stat-label\">{}</div><div class=\"stat-value\">{}</div></div>",
        esc(label),
        value_html
    )
}

fn state_sections(state: &toml::Table) -> String {
    // Headline stats — the three numbers Felix actually checks.
    let phase = str_at(state, &["firm", "phase"]).unwrap_or("?");
    let tone = if phase == "operating" { "ok" } else { "warn" };
    let updated = str_at(state, &["firm", "updated"]).unwrap_or("?");
    let total = int_at(state, &["research", "slots_total"]).unwrap_or(0);
    let active = int_at(state, &["research", "slots_active"]).unwrap_or(0);
    let stats = format!(
        "<div class=\"stats\">{}{}{}</div>",
        stat("phase", &badge(phase, tone)),
        stat(
            "research slots",
            &format!("{}<small> / {} active</small>", active, total),
        ),
        stat(
            "state updated",
            &format!("<span class=\"mono\">{}</span>", esc(updated)),
        ),
    );

    let mut groups = String::new();

    // Research slots
    {
        let content = match arr_at(state, &["research", "slot"]) {
            Some(slots) if !slots.is_empty() => {
                let mut rows = String::new();
                for slot in slots {
                    let id = slot.get("id").and_then(|v| v.as_integer()).unwrap_or(0);
                    let variant = slot.get("variant").and_then(|v| v.as_str()).unwrap_or("?");
                    let started =
                        slot.get("trial_started").and_then(|v| v.as_str()).unwrap_or("?");
                    let due = slot
                        .get("trial_review_due")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    rows.push_str(&kv(
                        &format!("slot {}", id),
                        &format!(
                            "<span class=\"mono\">{}</span> · {} → review {}",
                            esc(variant),
                            esc(started),
                            esc(due)
                        ),
                    ));
                }
                rows
            }
            _ => "<p class=\"muted-line\">No trials running — slots are filled by the CEO from the ideas/ backlog.</p>".to_string(),
        };
        groups.push_str(&subsection("Research slots", &content));
    }

    // Strategies (family/variant, by status)
    {
        let mut content = String::new();
        for (key, tone) in [("live", "ok"), ("trial", "warn"), ("retired", "")] {
            let names = str_list(state, &["strategies", key]);
            let value = if names.is_empty() {
                "<span class=\"muted-line\">—</span>".to_string()
            } else {
                badge_row(&names.iter().map(|n| badge(n, tone)).collect::<Vec<_>>())
            };
            content.push_str(&kv(key, &value));
        }
        groups.push_str(&subsection("Strategies", &content));
    }

    // Cadence
    {
        let ceo = str_at(state, &["cadence", "ceo_trigger"]).unwrap_or("?");
        let additional = str_list(state, &["cadence", "additional_triggers"]);
        let additional_html = if additional.is_empty() {
            "<span class=\"muted-line\">none</span>".to_string()
        } else {
            badge_row(&additional.iter().map(|t| badge(t, "")).collect::<Vec<_>>())
        };
        groups.push_str(&subsection(
            "Cadence",
            &(kv("ceo trigger", &esc(ceo)) + &kv("additional", &additional_html)),
        ));
    }

    // Model routing (per-task defaults, CONSTITUTION.md §4)
    {
        let mut content = String::new();
        if let Some(models) = table_at(state, &["models"]) {
            for (role, model) in models {
                content.push_str(&kv(
                    role,
                    &format!(
                        "<span class=\"mono\">{}</span>",
                        esc(model.as_str().unwrap_or("?"))
                    ),
                ));
            }
        }
        groups.push_str(&subsection("Model routing", &content));
    }

    // Active roles
    {
        let roles = str_list(state, &["roles", "active"]);
        let content = if roles.is_empty() {
            "<span class=\"muted-line\">—</span>".to_string()
        } else {
            badge_row(&roles.iter().map(|r| badge(r, "")).collect::<Vec<_>>())
        } + "<p class=\"muted-line\" style=\"margin-top:.6rem\">Researchers/executors are instantiated per slot / per live variant.</p>";
        groups.push_str(&subsection("Active roles", &content));
    }

    // Secrets (presence only — never values)
    {
        let mut content = String::new();
        if let Some(secrets) = table_at(state, &["secrets"]) {
            for (name, v) in secrets {
                let val = v.as_str().unwrap_or("?");
                let b = if val == "missing" {
                    badge("missing", "bad")
                } else {
                    badge(val, "ok")
                };
                content.push_str(&kv(name, &b));
            }
        }
        groups.push_str(&subsection("Secrets", &content));
    }

    // Cloudflare
    {
        let mut content = String::new();
        if let Some(cf) = table_at(state, &["cloudflare"]) {
            for (name, v) in cf {
                let val = v.as_str().unwrap_or("?");
                let html = if name == "mcp" && val == "connected" {
                    badge(val, "ok")
                } else {
                    format!("<span class=\"mono\">{}</span>", esc(val))
                };
                content.push_str(&kv(name, &html));
            }
        }
        groups.push_str(&subsection("Cloudflare", &content));
    }

    stats
        + &format!("<div class=\"grid-flat\">{}</div>", groups)
        + &note(
            "From <span class=\"mono\">ops/state.toml</span> on main. A structural change not reflected there didn't happen.",
        )
}

// ---------------------------------------------------------------------------
// Markdown documents with frontmatter (inbox messages, ideas)
// ---------------------------------------------------------------------------

/// Split optional `---` frontmatter into (fields, body). Values keep only
/// the part before an inline ` #` comment (`status: trialing # -> ...`).
fn split_frontmatter(src: &str) -> (Vec<(String, String)>, &str) {
    let Some(rest) = src.strip_prefix("---") else {
        return (Vec::new(), src);
    };
    let Some(end) = rest.find("\n---") else {
        return (Vec::new(), src);
    };
    let head = &rest[..end];
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    let fields = head
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            let v = v.trim();
            let v = v.split(" #").next().unwrap_or(v).trim();
            Some((k.trim().to_string(), v.to_string()))
        })
        .collect();
    (fields, body)
}

fn status_badge(status: &str) -> String {
    let tone = match status {
        "done" | "answered" | "closed" | "resolved" | "adopted" | "ok" | "live" => "ok",
        "open" | "pending" | "trialing" | "trial" => "warn",
        s if s.starts_with("kill") || s == "rejected" || s == "dead" || s == "retired" => "bad",
        _ => "",
    };
    badge(status, tone)
}

/// One frontmattered markdown file as a collapsible block: summary line
/// (subject/slug, status badge, from → to · date · filename) + body
/// (remaining frontmatter fields as kv/chips, then the markdown).
fn message_html(filename: &str, src: &str) -> String {
    let (fields, body) = split_frontmatter(src);
    let get = |k: &str| fields.iter().find(|(f, _)| f == k).map(|(_, v)| v.as_str());
    let title = get("subject").or_else(|| get("slug")).unwrap_or(filename);
    let status = get("status").map(status_badge).unwrap_or_default();
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
            let items: Vec<String> = v
                .trim_matches(['[', ']'])
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            extra.push_str(&kv(k, &chip_row(&items)));
        } else {
            extra.push_str(&kv(k, &esc(v)));
        }
    }

    format!(
        "<details class=\"msg\"><summary><span class=\"msg-title\">{}</span>{}<span class=\"msg-meta\">{}</span></summary><div class=\"msg-body\">{}<div class=\"prose\">{}</div></div></details>",
        esc(title),
        status,
        meta,
        extra,
        markdown(body)
    )
}

/// One ops/runs manifest: scalar fields as kv rows, [[step]] as a wrapping
/// table. Unparseable TOML falls back to a verbatim block.
fn run_manifest_html(src: &str) -> String {
    let Ok(t) = src.parse::<toml::Table>() else {
        return format!("<pre class=\"mono\">{}</pre>", esc(src));
    };
    let mut html = String::new();
    for k in ["date", "trigger", "model"] {
        if let Some(v) = t.get(k).and_then(|v| v.as_str()) {
            html.push_str(&kv(k, &esc(v)));
        }
    }
    if let Some(tokens) = t
        .get("spend")
        .and_then(|s| s.get("total_tokens"))
        .and_then(|v| v.as_integer())
    {
        html.push_str(&kv("spend", &format!("~{}k tokens", tokens / 1000)));
    }
    if let Some(steps) = t.get("step").and_then(|v| v.as_array()) {
        let mut rows: Vec<Vec<String>> =
            vec![["role", "status", "note"].iter().map(|s| s.to_string()).collect()];
        for s in steps {
            let mut role = s
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            if let Some(slot) = s.get("slot").and_then(|v| v.as_integer()) {
                role.push_str(&format!(" · slot {slot}"));
            }
            if let Some(var) = s.get("variant").and_then(|v| v.as_str()) {
                role.push_str(&format!(" · {var}"));
            }
            let status = s.get("status").and_then(|v| v.as_str()).unwrap_or("?");
            let mut note_text = s
                .get("note")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if let Some(n) = s.get("predictions").and_then(|v| v.as_integer()) {
                note_text = format!("[{n} predictions] {note_text}");
            }
            rows.push(vec![role, status.to_string(), note_text]);
        }
        html.push_str(&format!("<div class=\"wrap-cells\">{}</div>", csv_table(&rows)));
    }
    html
}

// ---------------------------------------------------------------------------
// /dev — pre-production playground (charts framework)
// ---------------------------------------------------------------------------

fn page_dev() -> String {
    let body = file_ref(
        "dashboard/src/charts.js",
        "Hand-rolled, dependency-free SVG charts, themed by the shadcn tokens (dark mode for free). The /snapshots page renders real R2 series with the same framework.",
    ) + &section(
        "Line — market probability",
        "<p class=\"muted-line\">Drag horizontally to zoom into a range; double-click to reset; hover for values.</p><div class=\"chart\" id=\"chart-line\"></div>",
    ) + &section(
        "Bar — Brier score by variant",
        "<p class=\"muted-line\">Hover a bar for the exact value. Lower is better.</p><div class=\"chart\" id=\"chart-bar\"></div>",
    ) + &note("Example series only — deterministic data generated in the Worker (fixed seed, never Date::now), served from /dev/data/*.json.")
        + r#"<script src="/charts.js"></script>
<script>
(function () {
  function load(url, fn) {
    fetch(url).then(function (r) { return r.json(); }).then(fn)
      .catch(function (e) {
        document.querySelectorAll(".chart").forEach(function (c) {
          if (!c.firstChild) c.textContent = "failed to load " + url;
        });
      });
  }
  load("/dev/data/line.json", function (d) {
    Chart.line(document.getElementById("chart-line"), d);
  });
  load("/dev/data/bar.json", function (d) {
    Chart.bar(document.getElementById("chart-bar"), d);
  });
})();
</script>"#;

    layout(
        "/dev",
        "Charts",
        "Pre-production playground — the SVG chart framework before it graduates to real pages.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_dev_endpoints() -> String {
    let body = file_ref(
        "dashboard/src/lib.rs",
        "JSON endpoints — deterministic example data plus the live R2 series endpoint.",
    ) + &kv(
        "GET /dev/data/line.json",
        "<span class=\"mono\">{label, points:[{t, v}]}</span> — t = unix ms · <a href=\"/dev/data/line.json\">open</a>",
    ) + &kv(
        "GET /dev/data/bar.json",
        "<span class=\"mono\">{label, bars:[{label, v}]}</span> · <a href=\"/dev/data/bar.json\">open</a>",
    ) + &kv(
        "GET /charts.js",
        "<span class=\"mono\">Chart.line(el, data, opts)</span> / <span class=\"mono\">Chart.bar(el, data, opts)</span> · <a href=\"/charts.js\">open</a>",
    ) + &kv(
        "GET /snapshots/data/&lt;date&gt;/&lt;slug&gt;.json",
        "<span class=\"mono\">{label, points:[{t, v}]}</span> — midpoint series from R2, cached 5 min · used by /snapshots",
    );

    layout(
        "/dev/endpoints",
        "Endpoints",
        "The dashboard's JSON endpoints and the charts API surface.",
        &body,
        BUILD_TIMESTAMP,
    )
}

// Deterministic example data (CONSTITUTION: reproducible; never Date::now or
// OS randomness — a fixed-seed LCG so every deploy serves identical JSON).

/// Numerical-Recipes LCG → uniform f64 in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 33) as f64 / (1u64 << 31) as f64
}

/// 120 days of a plausible market probability: mean-reverting random walk
/// with occasional news jumps. Shape matches real data: ms timestamps.
fn dev_line_json() -> String {
    const START_MS: u64 = 1_772_323_200_000; // 2026-03-01T00:00:00Z, fixed
    const DAY_MS: u64 = 86_400_000;
    let mut seed: u64 = 42;
    let mut v: f64 = 0.34;
    let mut points = String::new();
    for i in 0..120u64 {
        let r1 = lcg(&mut seed);
        let r2 = lcg(&mut seed);
        let jump = if r2 > 0.97 { (r1 - 0.5) * 0.22 } else { 0.0 };
        v += (0.5 - v) * 0.01 + (r1 - 0.5) * 0.035 + jump;
        v = v.clamp(0.02, 0.98);
        if i > 0 {
            points.push(',');
        }
        points.push_str(&format!(
            "{{\"t\":{},\"v\":{:.3}}}",
            START_MS + i * DAY_MS,
            v
        ));
    }
    format!(
        "{{\"label\":\"example — daily close, p(YES)\",\"points\":[{}]}}",
        points
    )
}

/// Plausible per-variant Brier scores (lower is better).
fn dev_bar_json() -> String {
    let variants = [
        "weather/nyc-temp-v1",
        "weather/nyc-temp-v2",
        "crypto/btc-close-v1",
        "politics/approval-v1",
        "sports/nba-totals-v1",
        "macro/cpi-nowcast-v1",
    ];
    let mut seed: u64 = 7;
    let mut bars = String::new();
    for (i, name) in variants.iter().enumerate() {
        if i > 0 {
            bars.push(',');
        }
        bars.push_str(&format!(
            "{{\"label\":\"{}\",\"v\":{:.3}}}",
            name,
            0.09 + lcg(&mut seed) * 0.17
        ));
    }
    format!(
        "{{\"label\":\"example — Brier score by variant\",\"bars\":[{}]}}",
        bars
    )
}

// ---------------------------------------------------------------------------
// toml::Value access helpers
// ---------------------------------------------------------------------------

fn value_at<'a>(root: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let mut cur = root.get(*first)?;
    for key in rest {
        cur = cur.get(key)?;
    }
    Some(cur)
}

fn str_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a str> {
    value_at(v, path)?.as_str()
}

fn int_at(v: &toml::Table, path: &[&str]) -> Option<i64> {
    value_at(v, path)?.as_integer()
}

fn arr_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a Vec<toml::Value>> {
    value_at(v, path)?.as_array()
}

fn table_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Table> {
    value_at(v, path)?.as_table()
}

fn str_list(v: &toml::Table, path: &[&str]) -> Vec<String> {
    arr_at(v, path)
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
