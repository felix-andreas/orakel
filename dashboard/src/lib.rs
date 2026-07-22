//! orakel dashboard — Cloudflare Worker (workers-rs), v1 skeleton.
//!
//! Server-rendered HTML over repo data EMBEDDED AT BUILD TIME (include_str!).
//! Each deploy snapshots repo state; live GitHub API + R2 reads come later
//! (see README.md). No JS, no external assets — plain <a> navigation.

mod render;

use render::{
    badge, badge_row, card, chip_row, csv_table, empty_state, esc, kv, layout, markdown, note,
    parse_csv, tabs,
};
use worker::{event, Context, Env, Headers, Request, Response, Result, Router};

// ---------------------------------------------------------------------------
// Embedded repo snapshot. Paths are relative to THIS FILE (src/lib.rs).
// Specific known files only — no globs. Files that may not exist yet
// (predictions/scores.csv) are staged into OUT_DIR by build.rs so the crate
// always builds.
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

/// Set by build.rs at compile time — never Date::now at runtime.
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(page_operations()))
        .get("/predictions", |_, _| Response::from_html(page_predictions()))
        .get("/inboxes", |_, _| Response::from_html(page_inboxes()))
        .get("/wiki", |_, _| Response::from_html(page_wiki()))
        .get("/style.css", |_, _| {
            let headers = Headers::new();
            headers.set("content-type", "text/css; charset=utf-8")?;
            headers.set("cache-control", "public, max-age=300")?;
            Ok(Response::ok(CSS)?.with_headers(headers))
        })
        .run(req, env)
        .await
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

fn page_operations() -> String {
    // NB: parse as toml::Table — a document. (toml::Value::from_str parses a
    // single TOML *value* and rejects a document.)
    let state_html = match STATE_TOML.parse::<toml::Table>() {
        Ok(state) => state_cards(&state),
        Err(e) => empty_state(
            "Failed to parse ops/state.toml",
            &format!("<div class=\"mono\">{}</div>", esc(&e.to_string())),
        ),
    };

    let decisions_html = card(
        "ops/decisions.md",
        "Append-only. Every structural change: what, why, who. Newest first.",
        &format!("<div class=\"prose\">{}</div>", markdown(DECISIONS_MD)),
    );

    let runs_html = empty_state(
        "No run manifests yet",
        "<div>ops/runs/ contains no <span class=\"mono\">&lt;YYYY-MM-DD&gt;.toml</span> manifests in this build snapshot. The CEO writes one per orchestrated run.</div>",
    ) + &card(
        "ops/runs/README.md",
        "Manifest format reference.",
        &format!("<div class=\"prose\">{}</div>", markdown(RUNS_README_MD)),
    );

    let body = tabs(
        "ops",
        &[
            ("State", state_html),
            ("Decisions", decisions_html),
            ("Runs", runs_html),
        ],
    );

    layout(
        "/",
        "Operations",
        "Current operating state (ops/state.toml), decision log, and run manifests.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_predictions() -> String {
    // predictions.csv
    let rows = parse_csv(PREDICTIONS_CSV);
    let predictions_html = if rows.len() <= 1 {
        let header = rows.first().cloned().unwrap_or_default();
        empty_state(
            "No predictions yet",
            &format!(
                "<div>predictions/predictions.csv holds only its header in this build snapshot. Rows appear once research slots start producing signals.</div>{}",
                chip_row(&header)
            ),
        )
    } else {
        let n = rows.len() - 1;
        format!(
            "<p class=\"page-desc\" style=\"margin:0\">{} prediction{}</p>{}",
            n,
            if n == 1 { "" } else { "s" },
            csv_table(&rows)
        )
    };

    // resolutions.csv
    let res_rows = parse_csv(RESOLUTIONS_CSV);
    let resolutions_html = if res_rows.len() <= 1 {
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

    // scores.csv — generated by scoring/, may not exist yet (build.rs stages
    // an empty placeholder so this compiles either way).
    let score_rows = parse_csv(SCORES_CSV);
    let scores_html = if score_rows.len() <= 1 {
        empty_state(
            "No scores yet",
            "<div>predictions/scores.csv has not been generated — <span class=\"mono\">scoring/</span> runs once the first predictions resolve. It will be picked up automatically at the next build.</div>",
        )
    } else {
        csv_table(&score_rows)
    };

    let body = tabs(
        "predictions",
        &[
            ("Predictions", predictions_html),
            ("Resolutions", resolutions_html),
            ("Scores", scores_html),
        ],
    );

    layout(
        "/predictions",
        "Predictions",
        "The canonical append-only prediction log, resolutions, and scoring aggregates.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_inboxes() -> String {
    let inboxes: [(&str, &str, &str); 3] = [
        ("CEO", "roles/ceo/inbox/", INBOX_CEO_MD),
        ("Felix", "roles/felix/inbox/", INBOX_FELIX_MD),
        (
            "Market researcher",
            "roles/market-researcher/inbox/",
            INBOX_MARKET_RESEARCHER_MD,
        ),
    ];

    let mut cards = String::new();
    for (role, path, readme) in inboxes.iter() {
        cards.push_str(&card(
            role,
            path,
            &format!(
                "<div class=\"prose\">{}</div>{}",
                markdown(readme),
                note("No messages in this build snapshot — only the inbox README is embedded. Real message files will render here once the dashboard reads the repo live."),
            ),
        ));
    }

    let body = format!(
        "<div class=\"grid-cards\" style=\"grid-template-columns:repeat(auto-fill,minmax(340px,1fr))\">{}</div>",
        cards
    );

    layout(
        "/inboxes",
        "Inboxes",
        "Per-role message queues (roles/<role>/inbox/). Researcher and executor inboxes are created per slot / per live variant.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_wiki() -> String {
    let body = card(
        "wiki/index.md",
        "Durable, cross-strategy knowledge. Maintained by the market researcher.",
        &format!(
            "<div class=\"prose\">{}</div>{}",
            markdown(WIKI_INDEX_MD),
            note("v1 skeleton renders the index only — links to individual wiki pages are not routed yet; read them in the repo. Full wiki browsing arrives with live GitHub reads."),
        ),
    );

    layout(
        "/wiki",
        "Wiki",
        "Index of the firm's knowledge base.",
        &body,
        BUILD_TIMESTAMP,
    )
}

// ---------------------------------------------------------------------------
// ops/state.toml → cards
// ---------------------------------------------------------------------------

fn state_cards(state: &toml::Table) -> String {
    let mut cards = String::new();

    // Firm
    {
        let phase = str_at(state, &["firm", "phase"]).unwrap_or("?");
        let tone = if phase == "operating" { "ok" } else { "warn" };
        let updated = str_at(state, &["firm", "updated"]).unwrap_or("?");
        cards.push_str(&card(
            "Firm",
            "",
            &(kv("phase", &badge(phase, tone))
                + &kv("updated", &format!("<span class=\"mono\">{}</span>", esc(updated)))),
        ));
    }

    // Research slots
    {
        let total = int_at(state, &["research", "slots_total"]).unwrap_or(0);
        let active = int_at(state, &["research", "slots_active"]).unwrap_or(0);
        let mut content = format!(
            "<div class=\"big-number\">{} <small>/ {} slots active</small></div>",
            active, total
        );
        match arr_at(state, &["research", "slot"]) {
            Some(slots) if !slots.is_empty() => {
                for slot in slots {
                    let id = slot.get("id").and_then(|v| v.as_integer()).unwrap_or(0);
                    let variant = slot.get("variant").and_then(|v| v.as_str()).unwrap_or("?");
                    let started =
                        slot.get("trial_started").and_then(|v| v.as_str()).unwrap_or("?");
                    let due = slot
                        .get("trial_review_due")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    content.push_str(&kv(
                        &format!("slot {}", id),
                        &format!(
                            "<span class=\"mono\">{}</span> · {} → review {}",
                            esc(variant),
                            esc(started),
                            esc(due)
                        ),
                    ));
                }
            }
            _ => {
                content.push_str(
                    "<p class=\"card-desc\" style=\"margin-top:.5rem\">No trials running — slots are filled by the CEO from the ideas/ backlog.</p>",
                );
            }
        }
        cards.push_str(&card("Research slots", "", &content));
    }

    // Strategies
    {
        let mut content = String::new();
        for (key, tone) in [("live", "ok"), ("trial", "warn"), ("retired", "")] {
            let names = str_list(state, &["strategies", key]);
            let value = if names.is_empty() {
                "<span style=\"color:var(--muted-foreground)\">—</span>".to_string()
            } else {
                badge_row(&names.iter().map(|n| badge(n, tone)).collect::<Vec<_>>())
            };
            content.push_str(&kv(key, &value));
        }
        cards.push_str(&card("Strategies", "family/variant, by status", &content));
    }

    // Cadence
    {
        let ceo = str_at(state, &["cadence", "ceo_trigger"]).unwrap_or("?");
        let additional = str_list(state, &["cadence", "additional_triggers"]);
        let additional_html = if additional.is_empty() {
            "<span style=\"color:var(--muted-foreground)\">none</span>".to_string()
        } else {
            badge_row(&additional.iter().map(|t| badge(t, "")).collect::<Vec<_>>())
        };
        cards.push_str(&card(
            "Cadence",
            "All triggers inside the constitution's working window.",
            &(kv("ceo trigger", &esc(ceo)) + &kv("additional", &additional_html)),
        ));
    }

    // Models
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
        cards.push_str(&card("Model routing", "Per-task defaults (CONSTITUTION.md §4).", &content));
    }

    // Roles
    {
        let roles = str_list(state, &["roles", "active"]);
        let content = if roles.is_empty() {
            "<span style=\"color:var(--muted-foreground)\">—</span>".to_string()
        } else {
            badge_row(&roles.iter().map(|r| badge(r, "")).collect::<Vec<_>>())
        } + "<p class=\"card-desc\" style=\"margin-top:.6rem\">Researchers/executors are instantiated per slot / per live variant.</p>";
        cards.push_str(&card("Active roles", "", &content));
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
        cards.push_str(&card("Secrets", "Presence only — values never live in the repo.", &content));
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
        cards.push_str(&card("Cloudflare", "", &content));
    }

    format!("<div class=\"grid-cards\">{}</div>", cards)
        + &note(&format!(
            "Snapshot of <span class=\"mono\">ops/state.toml</span> as of this deploy (built {}). A structural change not reflected there didn't happen.",
            esc(BUILD_TIMESTAMP)
        ))
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
