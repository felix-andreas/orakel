//! orakel snapshot worker — cron-triggered Cloudflare Worker (workers-rs).
//!
//! Every hour (cron `7 * * * *`) it reads `config/watchlist.json` from the R2
//! bucket `orakel`, fetches Polymarket order books + midpoints for every
//! watched outcome token via the CLOB **batch** endpoints (`POST /books`,
//! `POST /midpoints` — batching keeps us inside the Workers free-plan limit of
//! 50 subrequests per invocation), and writes one gzipped JSON document to
//! `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` (UTC, derived from the
//! scheduled event time — reruns overwrite the same key, idempotent).
//!
//! Book normalization: the CLOB returns bids/asks with the BEST level LAST in
//! each array. We store the top `BOOK_DEPTH` levels **best first** as
//! `[price, size]` pairs of f64 (the API sends decimal strings).
//!
//! Per-token failures (missing book, failed chunk request) put an `"error"`
//! field on that token's entry instead of aborting the run. A missing
//! midpoint is stored as `null`.

use flate2::{write::GzEncoder, Compression};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use worker::{
    console_error, console_log, event, Context, Env, Error, Fetch, Headers, HttpMetadata, Method,
    Request, RequestInit, Response, Result, ScheduleContext, ScheduledEvent,
};

/// Default CLOB base URL. Overridable with a `CLOB_BASE` var (used only for
/// local testing against a mock, e.g. `wrangler dev --var CLOB_BASE:http://...`;
/// never set in wrangler.toml).
const CLOB: &str = "https://clob.polymarket.com";
const WATCHLIST_KEY: &str = "config/watchlist.json";
/// Book levels stored per side (best first).
const BOOK_DEPTH: usize = 10;
/// Max token_ids per batch call — defensive; the CLOB accepts large batches,
/// but we chunk so a huge watchlist degrades gracefully.
const CHUNK: usize = 100;

// ---------------------------------------------------------------------------
// Input schemas
// ---------------------------------------------------------------------------

/// `config/watchlist.json` — mirrored into R2 by the CEO from active
/// applications. `updated` (RFC3339) is informational; we don't read it.
#[derive(Deserialize)]
struct Watchlist {
    #[serde(default)]
    markets: Vec<WatchMarket>,
}

#[derive(Deserialize)]
struct WatchMarket {
    condition_id: String,
    market_slug: String,
    #[serde(default)]
    token_ids: Vec<String>,
}

/// One price level in a CLOB book: `{"price": "0.55", "size": "100.2"}`.
#[derive(Deserialize)]
struct BookLevel {
    price: String,
    size: String,
}

/// One entry of the `POST /books` response array.
#[derive(Deserialize)]
struct Book {
    asset_id: String,
    #[serde(default)]
    bids: Vec<BookLevel>,
    #[serde(default)]
    asks: Vec<BookLevel>,
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

#[event(fetch)]
async fn fetch(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Response::ok(
        "orakel snapshot worker; writes snapshots/books/YYYY-MM-DD/HH.json.gz hourly; \
         watchlist: config/watchlist.json",
    )
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    if let Err(e) = run(&event, &env).await {
        console_error!("snapshot run failed: {}", e);
    }
}

// ---------------------------------------------------------------------------
// The run
// ---------------------------------------------------------------------------

async fn run(event: &ScheduledEvent, env: &Env) -> Result<()> {
    let bucket = env.bucket("ORAKEL")?;
    let clob = env
        .var("CLOB_BASE")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| CLOB.to_string());

    // 1. Watchlist. Absent / empty / no markets → nothing to do (normal state
    //    until strategies exist).
    let watchlist_text = match bucket.get(WATCHLIST_KEY).execute().await? {
        Some(obj) => match obj.body() {
            Some(body) => body.text().await?,
            None => String::new(),
        },
        None => String::new(),
    };
    let watchlist: Watchlist = if watchlist_text.trim().is_empty() {
        Watchlist { markets: vec![] }
    } else {
        serde_json::from_str(&watchlist_text)
            .map_err(|e| Error::RustError(format!("bad {WATCHLIST_KEY}: {e}")))?
    };
    if watchlist.markets.iter().all(|m| m.token_ids.is_empty()) {
        console_log!("watchlist empty, nothing to do");
        return Ok(());
    }

    // 2. Collect token ids (deduped, order-preserving) and batch-fetch.
    let mut seen = HashSet::new();
    let token_ids: Vec<String> = watchlist
        .markets
        .iter()
        .flat_map(|m| m.token_ids.iter())
        .filter(|t| seen.insert((*t).clone()))
        .cloned()
        .collect();

    let mut books: HashMap<String, Book> = HashMap::new();
    let mut midpoints: HashMap<String, f64> = HashMap::new();
    let mut errors: HashMap<String, String> = HashMap::new();

    for chunk in token_ids.chunks(CHUNK) {
        let body: Value = chunk.iter().map(|t| json!({ "token_id": t })).collect();

        match post_json(&format!("{clob}/books"), &body).await {
            Ok(v) => match serde_json::from_value::<Vec<Book>>(v) {
                Ok(list) => {
                    for b in list {
                        books.insert(b.asset_id.clone(), b);
                    }
                }
                Err(e) => {
                    let msg = format!("unexpected /books response shape: {e}");
                    for t in chunk {
                        errors.insert(t.clone(), msg.clone());
                    }
                }
            },
            Err(e) => {
                let msg = format!("books request failed: {e}");
                for t in chunk {
                    errors.insert(t.clone(), msg.clone());
                }
            }
        }

        match post_json(&format!("{clob}/midpoints"), &body).await {
            Ok(v) => collect_midpoints(&v, &mut midpoints),
            // A missing midpoint is just `null` in the output; don't mark the
            // whole token failed over it.
            Err(e) => console_error!("midpoints request failed: {}", e),
        }
    }

    // 3. Assemble the snapshot document.
    let ts = rfc3339_utc(event.schedule() as u64 / 1000);
    let markets: Vec<Value> = watchlist
        .markets
        .iter()
        .map(|m| {
            let tokens: Vec<Value> = m
                .token_ids
                .iter()
                .map(|t| {
                    let mut entry = json!({
                        "token_id": t,
                        "midpoint": midpoints.get(t), // Option<f64> → number|null
                        "bids": [],
                        "asks": [],
                    });
                    if let Some(book) = books.get(t) {
                        entry["bids"] = best_levels(&book.bids);
                        entry["asks"] = best_levels(&book.asks);
                    } else {
                        let msg = errors
                            .get(t)
                            .cloned()
                            .unwrap_or_else(|| "no book returned for token".to_string());
                        entry["error"] = json!(msg);
                    }
                    entry
                })
                .collect();
            json!({
                "condition_id": m.condition_id,
                "market_slug": m.market_slug,
                "tokens": tokens,
            })
        })
        .collect();
    let doc = json!({ "ts": ts, "markets": markets });

    // 4. Gzip + PUT. Key is deterministic from the scheduled hour (UTC), so a
    //    rerun in the same hour overwrites — idempotent by design.
    let json_bytes =
        serde_json::to_vec(&doc).map_err(|e| Error::RustError(format!("serialize: {e}")))?;
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&json_bytes)
        .map_err(|e| Error::RustError(format!("gzip: {e}")))?;
    let gz = enc
        .finish()
        .map_err(|e| Error::RustError(format!("gzip: {e}")))?;

    let key = snapshot_key(event.schedule() as u64 / 1000);
    console_log!(
        "snapshot {}: {} markets, {} tokens, {} token errors, {} bytes json, {} bytes gz",
        key,
        watchlist.markets.len(),
        token_ids.len(),
        errors.len(),
        json_bytes.len(),
        gz.len()
    );
    bucket
        .put(&key, gz)
        .http_metadata(HttpMetadata {
            content_type: Some("application/json".to_string()),
            content_encoding: Some("gzip".to_string()),
            ..HttpMetadata::default()
        })
        .execute()
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// POST a JSON body, expect a 200 with a JSON response.
async fn post_json(url: &str, body: &Value) -> Result<Value> {
    let headers = Headers::new();
    headers.set("content-type", "application/json")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(
            serde_json::to_string(body)
                .map_err(|e| Error::RustError(format!("encode body: {e}")))?
                .into(),
        ));
    let req = Request::new_with_init(url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;
    let status = resp.status_code();
    let text = resp.text().await?;
    if status != 200 {
        let brief: String = text.chars().take(200).collect();
        return Err(Error::RustError(format!("{url} -> HTTP {status}: {brief}")));
    }
    serde_json::from_str(&text).map_err(|e| Error::RustError(format!("{url} -> bad JSON: {e}")))
}

/// `POST /midpoints` responds with a map `{"<token_id>": "0.55", ...}`
/// (values are decimal strings). Parse defensively: also accept numeric
/// values, and an array-of-objects form `[{"token_id": ..., "mid": ...}]`.
fn collect_midpoints(v: &Value, out: &mut HashMap<String, f64>) {
    let as_f64 = |v: &Value| -> Option<f64> {
        v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    };
    match v {
        Value::Object(map) => {
            for (token, mid) in map {
                if let Some(m) = as_f64(mid) {
                    out.insert(token.clone(), m);
                }
            }
        }
        Value::Array(list) => {
            for item in list {
                let token = item.get("token_id").and_then(|t| t.as_str());
                let mid = item.get("mid").or_else(|| item.get("midpoint")).and_then(as_f64);
                if let (Some(t), Some(m)) = (token, mid) {
                    out.insert(t.to_string(), m);
                }
            }
        }
        _ => {}
    }
}

/// CLOB books list levels with the BEST price LAST. Return the top
/// `BOOK_DEPTH` levels normalized **best first** as `[price, size]` f64 pairs.
/// Levels that fail to parse as decimals are skipped.
fn best_levels(levels: &[BookLevel]) -> Value {
    let start = levels.len().saturating_sub(BOOK_DEPTH);
    let out: Vec<Value> = levels[start..]
        .iter()
        .rev()
        .filter_map(|l| {
            let p: f64 = l.price.parse().ok()?;
            let s: f64 = l.size.parse().ok()?;
            Some(json!([p, s]))
        })
        .collect();
    Value::Array(out)
}

/// `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` from unix seconds (UTC).
fn snapshot_key(secs: u64) -> String {
    let (y, mo, d, h, _, _) = utc_parts(secs);
    format!("snapshots/books/{y:04}-{mo:02}-{d:02}/{h:02}.json.gz")
}

fn rfc3339_utc(secs: u64) -> String {
    let (y, mo, d, h, mi, s) = utc_parts(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Days-from-epoch to civil date (Howard Hinnant's algorithm), no deps.
/// Copied from dashboard/build.rs (CODING.md: copy freely).
fn utc_parts(secs: u64) -> (i64, u64, u64, u64, u64, u64) {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo as u64, d as u64, h, m, s)
}
