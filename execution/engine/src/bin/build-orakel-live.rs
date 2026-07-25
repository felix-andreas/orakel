//! Adapter: our own predictions ⋈ resolutions -> `signals/orakel-live.csv`.
//!
//! ```sh
//! build-orakel-live --predictions predictions --out execution/signals/orakel-live.csv \
//!                   [--cache <dir>] [--offline]
//! ```
//!
//! Book data comes from the firm's own hourly R2 snapshots
//! (`snapshots/books/<YYYY-MM-DD>/<HH>.json.gz`, schema in
//! `workers/snapshot/README.md`). For each prediction we take the **latest
//! snapshot whose `ts` is at or before the prediction time** — never a later
//! one, which would be lookahead — and read the token's real bid/ask plus the
//! USD depth resting within 5c of each touch. A prediction with no such
//! snapshot, or whose token was not on the watchlist yet, gets `synthetic_book`
//! and the engine prices it off the midpoint with the policy's assumed spread.
//!
//! Downloaded snapshots are cached on disk, so a re-run is offline-repeatable
//! (`--offline` refuses to touch the network at all).

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

/// Depth is measured within this distance of the touch (DESIGN.md §4.2).
const DEPTH_BAND: f64 = 0.05;
/// A snapshot older than this is not a description of the book we traded into.
const MAX_SNAPSHOT_AGE_SECS: i64 = 24 * 3600;
const SNAPSHOT_PREFIX: &str = "snapshots/books/";

fn main() -> Result<()> {
    let mut preds_dir = PathBuf::from("predictions");
    let mut out: Option<PathBuf> = None;
    let mut cache = PathBuf::from(".snapshot-cache");
    let mut offline = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--predictions" => preds_dir = args.next().map(PathBuf::from).context("--predictions")?,
            "--out" => out = args.next().map(PathBuf::from),
            "--cache" => cache = args.next().map(PathBuf::from).context("--cache")?,
            "--offline" => offline = true,
            "--help" | "-h" => {
                println!("usage: build-orakel-live --predictions <dir> --out <csv> [--cache <dir>] [--offline]");
                return Ok(());
            }
            other => bail!("unknown argument '{other}'"),
        }
    }
    let out = out.context("--out is required")?;

    let (preds, preds_sha) = read_csv(&preds_dir.join("predictions.csv"))?;
    let (res, res_sha) = read_csv(&preds_dir.join("resolutions.csv"))?;

    // Resolutions keyed by condition_id and by slug, same as scoring/.
    let mut resolutions: HashMap<String, (String, String)> = HashMap::new();
    for r in &res {
        let v = (r["winning_outcome"].clone(), r["resolved_date"].clone());
        if !r["condition_id"].is_empty() {
            resolutions.insert(r["condition_id"].clone(), v.clone());
        }
        if !r["market_slug"].is_empty() {
            resolutions.insert(r["market_slug"].clone(), v);
        }
    }

    let snaps = load_snapshots(&cache, offline)?;
    eprintln!("{} snapshots available ({} .. {})",
        snaps.len(),
        snaps.first().map(|s| s.ts_raw.as_str()).unwrap_or("-"),
        snaps.last().map(|s| s.ts_raw.as_str()).unwrap_or("-"));

    let mut rows = Vec::new();
    let mut unresolved = 0usize;
    let mut with_book = 0usize;
    let mut used_keys: Vec<String> = Vec::new();
    for p in &preds {
        let key = if resolutions.contains_key(&p["condition_id"]) {
            p["condition_id"].clone()
        } else if resolutions.contains_key(&p["market_slug"]) {
            p["market_slug"].clone()
        } else {
            unresolved += 1;
            continue;
        };
        let (winning_outcome, resolved_date) = resolutions[&key].clone();
        let t = engine::parse_ts(&p["timestamp"]).map_err(anyhow::Error::msg)?;

        let book = latest_book_at_or_before(&snaps, t, &p["token_id"]);
        if let Some((k, _)) = &book {
            with_book += 1;
            if !used_keys.contains(k) {
                used_keys.push(k.clone());
            }
        }
        let (bid, ask, dbid, dask) = match &book {
            Some((_, b)) => (
                fmt(Some(b.bid)),
                fmt(Some(b.ask)),
                fmt(Some(b.depth_bid)),
                fmt(Some(b.depth_ask)),
            ),
            None => (String::new(), String::new(), String::new(), String::new()),
        };
        rows.push(
            [
                "orakel-live".to_string(),
                p["timestamp"].clone(),
                p["market_slug"].clone(),
                p["condition_id"].clone(),
                p["outcome"].clone(),
                p["token_id"].clone(),
                p["family"].clone(),
                p["variant"].clone(),
                p["model"].clone(),
                p["prediction"].clone(),
                p["market_price"].clone(),
                bid,
                ask,
                dbid,
                dask,
                winning_outcome,
                resolved_date,
                if book.is_some() { "0" } else { "1" }.to_string(),
                asset_of(&p["market_slug"]),
            ]
            .join(","),
        );
    }

    let header = format!("{},asset", engine::SIGNAL_HEADER.join(","));
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d).ok();
    }
    std::fs::write(&out, format!("{header}\n{}\n", rows.join("\n")))
        .with_context(|| format!("writing {}", out.display()))?;

    let sidecar = out.with_extension("source.json");
    let meta = serde_json::json!({
        "signal_set": "orakel-live",
        "built_by": "engine/src/bin/build-orakel-live.rs",
        "engine_version": engine::ENGINE_VERSION,
        "inputs": [
            {"file": "predictions/predictions.csv", "sha256": preds_sha, "rows": preds.len()},
            {"file": "predictions/resolutions.csv", "sha256": res_sha, "rows": res.len()},
        ],
        "r2_bucket": std::env::var("R2_BUCKET").unwrap_or_else(|_| "orakel".into()),
        "r2_snapshot_prefix": SNAPSHOT_PREFIX,
        "snapshots_available": snaps.iter().map(|s| serde_json::json!({
            "key": s.key, "ts": s.ts_raw, "sha256": s.sha256,
        })).collect::<Vec<_>>(),
        "snapshots_used_for_books": used_keys,
        "output_rows": rows.len(),
        "rows_with_real_book": with_book,
        "predictions_without_resolution": unresolved,
        "conventions": {
            "book_selection": "latest snapshot with ts <= prediction time, at most 24h stale; never a later snapshot",
            "depth": "USD notional resting within 5c of the touch, per side",
            "resolved_date": "bare date from resolutions.csv; the engine reads it as 12:00 UTC (scoring's convention)",
        },
    });
    std::fs::write(&sidecar, serde_json::to_string_pretty(&meta)? + "\n")?;

    println!(
        "{} resolved signals -> {} ({} with a real book, {} predictions unresolved)\n{}",
        rows.len(),
        out.display(),
        with_book,
        unresolved,
        sidecar.display()
    );
    Ok(())
}

fn fmt(v: Option<f64>) -> String {
    v.map(engine::fmt_f).unwrap_or_default()
}

/// Asset attribution key from the market slug (`will-<asset>-reach-...`).
fn asset_of(slug: &str) -> String {
    slug.strip_prefix("will-")
        .and_then(|s| s.split('-').next())
        .unwrap_or("")
        .to_string()
}

// ------------------------------------------------------------------ snapshots

struct Snapshot {
    key: String,
    ts_raw: String,
    ts: i64,
    sha256: String,
    /// token_id -> book
    books: HashMap<String, Book>,
}

#[derive(Clone)]
struct Book {
    bid: f64,
    ask: f64,
    depth_bid: f64,
    depth_ask: f64,
}

fn latest_book_at_or_before<'a>(
    snaps: &'a [Snapshot],
    t: i64,
    token_id: &str,
) -> Option<(String, &'a Book)> {
    snaps
        .iter()
        .rev()
        .filter(|s| s.ts <= t && t - s.ts <= MAX_SNAPSHOT_AGE_SECS)
        .find_map(|s| s.books.get(token_id).map(|b| (s.key.clone(), b)))
}

fn load_snapshots(cache: &Path, offline: bool) -> Result<Vec<Snapshot>> {
    std::fs::create_dir_all(cache).ok();
    let keys = if offline {
        let mut k: Vec<String> = std::fs::read_dir(cache)?
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|n| n.ends_with(".json.gz"))
            .map(|n| format!("{SNAPSHOT_PREFIX}{}", n.replacen('_', "/", 1)))
            .collect();
        k.sort();
        k
    } else {
        list_r2_keys()?
    };

    let mut out = Vec::new();
    for key in keys {
        let name = key.trim_start_matches(SNAPSHOT_PREFIX).replace('/', "_");
        let path = cache.join(&name);
        let gz = if path.is_file() {
            std::fs::read(&path)?
        } else if offline {
            bail!("--offline but {} is not cached", key);
        } else {
            let bytes = get_r2_object(&key)?;
            std::fs::write(&path, &bytes)?;
            bytes
        };
        let sha256 = format!("{:x}", Sha256::digest(&gz));
        let mut json = String::new();
        flate2::read::GzDecoder::new(&gz[..])
            .read_to_string(&mut json)
            .with_context(|| format!("gunzip {key}"))?;
        let v: serde_json::Value = serde_json::from_str(&json).with_context(|| format!("parse {key}"))?;
        let ts_raw = v["ts"].as_str().unwrap_or_default().to_string();
        let ts = engine::parse_ts(&ts_raw).map_err(anyhow::Error::msg)?;
        let mut books = HashMap::new();
        for m in v["markets"].as_array().into_iter().flatten() {
            for tok in m["tokens"].as_array().into_iter().flatten() {
                if tok.get("error").is_some() {
                    continue;
                }
                let Some(id) = tok["token_id"].as_str() else { continue };
                let bids = levels(&tok["bids"]);
                let asks = levels(&tok["asks"]);
                let (Some(&(bid, _)), Some(&(ask, _))) = (bids.first(), asks.first()) else {
                    continue;
                };
                if !(ask > bid) {
                    continue;
                }
                // Depth within 5c of each touch, in USD notional.
                let depth_bid: f64 =
                    bids.iter().filter(|(p, _)| *p >= bid - DEPTH_BAND).map(|(p, s)| p * s).sum();
                let depth_ask: f64 =
                    asks.iter().filter(|(p, _)| *p <= ask + DEPTH_BAND).map(|(p, s)| p * s).sum();
                books.insert(id.to_string(), Book { bid, ask, depth_bid, depth_ask });
            }
        }
        out.push(Snapshot { key, ts_raw, ts, sha256, books });
    }
    out.sort_by_key(|s| s.ts);
    Ok(out)
}

fn levels(v: &serde_json::Value) -> Vec<(f64, f64)> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|l| {
                    let p = l.get(0)?.as_f64()?;
                    let s = l.get(1)?.as_f64()?;
                    Some((p, s))
                })
                .collect()
        })
        .unwrap_or_default()
}

// ------------------------------------------------------------------- R2 access
// Same shape as tools/r2data/src/config.rs — copied rather than shared, per
// CODING.md ("copy freely; only conventions are single-source").

fn r2_bucket() -> Result<Box<s3::Bucket>> {
    let account = std::env::var("R2_ACCOUNT_ID").context("R2_ACCOUNT_ID not set")?;
    let key = std::env::var("R2_ACCESS_KEY_ID").context("R2_ACCESS_KEY_ID not set")?;
    let secret = std::env::var("R2_SECRET_ACCESS_KEY").context("R2_SECRET_ACCESS_KEY not set")?;
    let name = std::env::var("R2_BUCKET").unwrap_or_else(|_| "orakel".to_string());
    let region = s3::Region::Custom {
        region: "auto".to_string(),
        endpoint: format!("https://{account}.r2.cloudflarestorage.com"),
    };
    let creds = s3::creds::Credentials::new(Some(&key), Some(&secret), None, None, None)?;
    Ok(s3::Bucket::new(&name, region, creds)?
        .with_path_style()
        .with_request_timeout(Duration::from_secs(300))?)
}

fn list_r2_keys() -> Result<Vec<String>> {
    let bucket = r2_bucket()?;
    let mut keys: Vec<String> = bucket
        .list(SNAPSHOT_PREFIX.to_string(), None)?
        .into_iter()
        .flat_map(|r| r.contents)
        .map(|o| o.key)
        .filter(|k| k.ends_with(".json.gz"))
        .collect();
    keys.sort();
    Ok(keys)
}

fn get_r2_object(key: &str) -> Result<Vec<u8>> {
    let bucket = r2_bucket()?;
    let resp = bucket.get_object(key).with_context(|| format!("GET {key}"))?;
    let code = resp.status_code();
    if !(200..300).contains(&code) {
        bail!("GET {key} failed with HTTP {code}");
    }
    Ok(resp.bytes().to_vec())
}

// ------------------------------------------------------------------------ csv

fn read_csv(path: &Path) -> Result<(Vec<HashMap<String, String>>, String)> {
    let bytes = std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    let text = String::from_utf8(bytes).context("not utf-8")?;
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(text.as_bytes());
    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.trim().to_string()).collect();
    let mut rows = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        let mut m = HashMap::with_capacity(headers.len());
        for (i, h) in headers.iter().enumerate() {
            m.insert(h.clone(), rec.get(i).unwrap_or("").trim().to_string());
        }
        rows.push(m);
    }
    Ok((rows, sha))
}
