//! /snapshots data plumbing — R2 reads of the snapshot worker's output.
//!
//! The snapshot worker (workers/snapshot) writes one gzipped JSON document
//! per hour to `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` (UTC, hour from
//! the cron schedule). Document shape:
//!   { ts, markets: [{ condition_id, market_slug,
//!                     tokens: [{ token_id, midpoint: num|null,
//!                                bids: [[price,size]...best first],
//!                                asks: [[price,size]...], error? }] }] }
//!
//! GZIP CAVEAT: the objects are PUT as gzipped bytes with
//! `content_encoding=gzip` HTTP metadata; an R2 *binding* get returns the
//! stored (compressed) bytes verbatim — sniff the 0x1f,0x8b magic and gunzip
//! here (flate2 rust_backend, same dep as workers/snapshot).

use flate2::read::GzDecoder;
use serde_json::Value;
use std::io::Read;
use worker::{Bucket, Cache, Date, Env, Headers, Response, Result};

const BOOKS_PREFIX: &str = "snapshots/books/";
/// Series responses cache TTL (Cache API), seconds.
const SERIES_TTL_SECS: u32 = 300;

// ---------------------------------------------------------------------------
// R2 access
// ---------------------------------------------------------------------------

async fn get_bytes(bucket: &Bucket, key: &str) -> Option<Vec<u8>> {
    let obj = bucket.get(key).execute().await.ok()??;
    obj.body()?.bytes().await.ok()
}

/// Gunzip iff the gzip magic is present, else pass through.
fn ungzip_if_needed(bytes: Vec<u8>) -> Vec<u8> {
    if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        let mut out = Vec::new();
        if GzDecoder::new(bytes.as_slice()).read_to_end(&mut out).is_ok() {
            return out;
        }
    }
    bytes
}

async fn get_doc(bucket: &Bucket, key: &str) -> Option<Value> {
    let bytes = ungzip_if_needed(get_bytes(bucket, key).await?);
    serde_json::from_slice(&bytes).ok()
}

/// Sorted keys under `snapshots/books/<date>/` (hourly, so ≤24).
async fn day_keys(bucket: &Bucket, date: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(list) = bucket
        .list()
        .prefix(format!("{BOOKS_PREFIX}{date}/"))
        .execute()
        .await
    {
        for obj in list.objects() {
            out.push(obj.key());
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// Latest snapshot (health header + table)
// ---------------------------------------------------------------------------

pub struct Latest {
    pub key: String,
    /// UTC day the key belongs to (YYYY-MM-DD) — the day the charts query.
    pub date: String,
    pub age_mins: u64,
    pub doc: Value,
}

/// Newest snapshot: today's last key, falling back to yesterday (UTC).
/// None ⇒ no binding available or nothing written in the last two days.
pub async fn latest(env: &Env) -> Option<Latest> {
    let bucket = env.bucket("ORAKEL").ok()?;
    let now_ms = Date::now().as_millis();
    let today = date_string(now_ms);
    let mut keys = day_keys(&bucket, &today).await;
    if keys.is_empty() {
        keys = day_keys(&bucket, &date_string(now_ms - 86_400_000)).await;
    }
    let key = keys.pop()?;
    let doc = get_doc(&bucket, &key).await?;
    let date = key
        .split('/')
        .nth(2)
        .unwrap_or_default()
        .to_string();
    let age_mins = key_epoch_ms(&key)
        .map(|t| now_ms.saturating_sub(t) / 60_000)
        .unwrap_or(0);
    Some(Latest { key, date, age_mins, doc })
}

/// Ordered market slugs of a snapshot document (watchlist order — the
/// snapshot worker builds the document by iterating config/watchlist.json).
pub fn market_slugs(doc: &Value) -> Vec<String> {
    doc["markets"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|m| m["market_slug"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Per-market midpoint series endpoint
// ---------------------------------------------------------------------------

/// GET /snapshots/data/<date>/<slug>.json → {label, points:[{t,v}]} for
/// Chart.line, assembled from the day's hourly objects. Cached 5 min.
pub async fn series_json(env: &Env, date: &str, slug: &str, req_url: &str) -> Result<Response> {
    if !valid_date(date) || !valid_slug(slug) {
        return Response::error("bad request", 400);
    }

    let cache = Cache::default();
    if let Ok(Some(hit)) = cache.get(req_url, true).await {
        return Ok(hit);
    }

    let bucket = env.bucket("ORAKEL")?;
    let mut points = Vec::new();
    for key in day_keys(&bucket, date).await {
        let Some(t) = key_epoch_ms(&key) else { continue };
        let Some(doc) = get_doc(&bucket, &key).await else { continue };
        if let Some(v) = market_mid(&doc, slug) {
            points.push(format!("{{\"t\":{t},\"v\":{v}}}"));
        }
    }
    // slug/date are validated (no quotes/backslashes) — safe to embed.
    let body = format!(
        "{{\"label\":\"{slug} — midpoint, {date} (UTC)\",\"points\":[{}]}}",
        points.join(",")
    );

    let headers = Headers::new();
    headers.set("content-type", "application/json; charset=utf-8")?;
    headers.set("cache-control", &format!("public, max-age={SERIES_TTL_SECS}"))?;
    let mut resp = Response::ok(body)?.with_headers(headers);
    if let Ok(clone) = resp.cloned() {
        let _ = cache.put(req_url, clone).await;
    }
    Ok(resp)
}

/// First token's midpoint (the primary outcome) for a market slug.
fn market_mid(doc: &Value, slug: &str) -> Option<f64> {
    doc["markets"]
        .as_array()?
        .iter()
        .find(|m| m["market_slug"] == slug)?["tokens"]
        .get(0)?["midpoint"]
        .as_f64()
}

fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ---------------------------------------------------------------------------
// UTC date math (no deps; Howard Hinnant's algorithms, as in workers/snapshot)
// ---------------------------------------------------------------------------

/// `YYYY-MM-DD` (UTC) from unix milliseconds.
fn date_string(ms: u64) -> String {
    let (y, mo, d) = civil_from_days((ms / 86_400_000) as i64);
    format!("{y:04}-{mo:02}-{d:02}")
}

/// Epoch ms of `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` (top of the hour;
/// the worker actually snapshots at :07, close enough for age/charting).
fn key_epoch_ms(key: &str) -> Option<u64> {
    let rest = key.strip_prefix(BOOKS_PREFIX)?;
    let (date, file) = rest.split_once('/')?;
    let hh: u64 = file.get(0..2)?.parse().ok()?;
    let mut it = date.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    let days = days_from_civil(y, m, d);
    if days < 0 || hh > 23 {
        return None;
    }
    Some(days as u64 * 86_400_000 + hh * 3_600_000)
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    (if mo <= 2 { y + 1 } else { y }, mo, d)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468 // 719468 = days from 0000-03-01 to 1970-01-01
}
