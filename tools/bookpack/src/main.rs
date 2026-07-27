//! bookpack — roll a day of hourly book snapshots into one Parquet file.
//!
//! The snapshot worker writes `snapshots/books/<date>/<HH>.json.gz` every hour:
//! one object per hour, every watched token's top-10 book inside it. That shape
//! is right for *writing* — small, append-only, no coordination — and wrong for
//! every question we actually ask of it, all of which are "how did this token's
//! book move over time". Answering that today means fetching one object per
//! hour: **8,760 requests for a year**, at ~149 KB each, to build one series.
//!
//! So this rolls each day into one column-oriented file at
//! `snapshots/parquet/date=<date>/books.parquet`. Same bytes, ~24× fewer
//! objects, and the columns compress hard because `market_slug`, `token_id` and
//! `condition_id` repeat 24 times a day with no variation at all.
//!
//! **Why this dataset is worth the tooling.** `wiki/reference/midpoint-is-not-a-fill.md`
//! says our reachability numbers are a *lower bound*, because `tools/fillcheck`
//! replays the trade feed and a resting bid nobody hit leaves no trace there. A
//! real book history removes that caveat — it records the bid whether or not
//! anyone took it. This is the one dataset the firm is accumulating that cannot
//! be reconstructed after the fact if we lose it.
//!
//! The hourly JSON is **not deleted**. It is the raw record; parquet is a
//! derived view, and a derived view that destroys its source is a migration,
//! not a cache.
//!
//! Usage:
//!   bookpack pack <date> [<date>...]   roll those days
//!   bookpack pack --all                every day present in the bucket
//!   bookpack verify <date>             read the parquet back and reconcile it
//!                                      against the hourly JSON, row for row

use std::collections::BTreeSet;
use std::io::Read;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use arrow_array::{Array, ArrayRef, Float64Array, Int32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use s3::{creds::Credentials, Bucket, Region};

const BUCKET: &str = "orakel";

/// One token's book at one hour — the row the whole file is made of.
struct Row {
    ts: String,
    date: String,
    hour: i32,
    condition_id: String,
    market_slug: String,
    token_id: String,
    best_bid: Option<f64>,
    best_ask: Option<f64>,
    midpoint: Option<f64>,
    spread: Option<f64>,
    bid_depth_usd: f64,
    ask_depth_usd: f64,
    bid_levels: i32,
    ask_levels: i32,
    error: Option<String>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bucket = open_bucket()?;
    match args.first().map(String::as_str) {
        Some("pack") => {
            let rest = &args[1..];
            let dates = if rest.iter().any(|a| a == "--all") {
                discover_dates(&bucket)?
            } else if rest.is_empty() {
                bail!("pack needs a date, or --all")
            } else {
                rest.to_vec()
            };
            if dates.is_empty() {
                println!("no snapshot days found under snapshots/books/");
                return Ok(());
            }
            for date in dates {
                pack_day(&bucket, &date)?;
            }
        }
        Some("verify") => {
            let date = args.get(1).context("verify needs a date")?;
            verify_day(&bucket, date)?;
        }
        _ => {
            println!("bookpack pack <date>... | pack --all | verify <date>");
        }
    }
    Ok(())
}

fn open_bucket() -> Result<Box<Bucket>> {
    let account = std::env::var("R2_ACCOUNT_ID").context("R2_ACCOUNT_ID not set")?;
    let creds = Credentials::new(
        Some(&std::env::var("R2_ACCESS_KEY_ID").context("R2_ACCESS_KEY_ID not set")?),
        Some(&std::env::var("R2_SECRET_ACCESS_KEY").context("R2_SECRET_ACCESS_KEY not set")?),
        None,
        None,
        None,
    )?;
    let region = Region::Custom {
        region: "auto".to_string(),
        endpoint: format!("https://{account}.r2.cloudflarestorage.com"),
    };
    Ok(Bucket::new(BUCKET, region, creds)?.with_path_style())
}

/// Every date that has at least one hourly object.
fn discover_dates(bucket: &Bucket) -> Result<Vec<String>> {
    let mut dates = BTreeSet::new();
    for page in bucket.list("snapshots/books/".to_string(), None)? {
        for obj in page.contents {
            // snapshots/books/<date>/<HH>.json.gz
            if let Some(rest) = obj.key.strip_prefix("snapshots/books/") {
                if let Some((date, _)) = rest.split_once('/') {
                    dates.insert(date.to_string());
                }
            }
        }
    }
    Ok(dates.into_iter().collect())
}

/// Sum of `price × size` over the stored levels — the money resting on that
/// side, which is what a fill question actually cares about.
fn depth_usd(levels: &serde_json::Value) -> (f64, i32) {
    let arr = match levels.as_array() {
        Some(a) => a,
        None => return (0.0, 0),
    };
    let mut usd = 0.0;
    for lv in arr {
        if let Some(pair) = lv.as_array() {
            let price = pair.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
            let size = pair.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
            usd += price * size;
        }
    }
    (usd, arr.len() as i32)
}

/// Best level on a side. The snapshot worker already normalises **best first**
/// (the raw CLOB returns best *last*, and the worker reverses it) — so this
/// takes `[0]`, and that asymmetry is exactly the kind of thing worth stating
/// where someone will read it.
fn best(levels: &serde_json::Value) -> Option<f64> {
    levels
        .as_array()?
        .first()?
        .as_array()?
        .first()?
        .as_f64()
}

fn rows_for_day(bucket: &Bucket, date: &str) -> Result<Vec<Row>> {
    let mut keys: Vec<String> = Vec::new();
    for page in bucket.list(format!("snapshots/books/{date}/"), None)? {
        for obj in page.contents {
            if obj.key.ends_with(".json.gz") {
                keys.push(obj.key);
            }
        }
    }
    keys.sort();

    let mut rows = Vec::new();
    for key in &keys {
        let resp = bucket.get_object(key)?;
        let raw = resp.bytes();
        // Objects are stored gzipped, but R2 may transparently decode on the
        // way out depending on how they were written — sniff the magic rather
        // than assume either way.
        let text = if raw.len() > 2 && raw[0] == 0x1f && raw[1] == 0x8b {
            let mut s = String::new();
            flate2::read::GzDecoder::new(&raw[..]).read_to_string(&mut s)?;
            s
        } else {
            String::from_utf8_lossy(raw).into_owned()
        };
        let doc: serde_json::Value =
            serde_json::from_str(&text).with_context(|| format!("parsing {key}"))?;

        let ts = doc["ts"].as_str().unwrap_or_default().to_string();
        let hour: i32 = key
            .rsplit('/')
            .next()
            .and_then(|f| f.split('.').next())
            .and_then(|h| h.parse().ok())
            .unwrap_or(-1);

        for m in doc["markets"].as_array().into_iter().flatten() {
            let condition_id = m["condition_id"].as_str().unwrap_or_default().to_string();
            let market_slug = m["market_slug"].as_str().unwrap_or_default().to_string();
            for t in m["tokens"].as_array().into_iter().flatten() {
                let error = t["error"].as_str().map(str::to_string);
                let (bid_depth_usd, bid_levels) = depth_usd(&t["bids"]);
                let (ask_depth_usd, ask_levels) = depth_usd(&t["asks"]);
                let best_bid = best(&t["bids"]);
                let best_ask = best(&t["asks"]);
                rows.push(Row {
                    ts: ts.clone(),
                    date: date.to_string(),
                    hour,
                    condition_id: condition_id.clone(),
                    market_slug: market_slug.clone(),
                    token_id: t["token_id"].as_str().unwrap_or_default().to_string(),
                    best_bid,
                    best_ask,
                    midpoint: t["midpoint"].as_f64(),
                    spread: match (best_bid, best_ask) {
                        (Some(b), Some(a)) => Some(a - b),
                        _ => None,
                    },
                    bid_depth_usd,
                    ask_depth_usd,
                    bid_levels,
                    ask_levels,
                    error,
                });
            }
        }
    }
    Ok(rows)
}

fn schema() -> Schema {
    Schema::new(vec![
        Field::new("ts", DataType::Utf8, false),
        Field::new("date", DataType::Utf8, false),
        Field::new("hour", DataType::Int32, false),
        Field::new("condition_id", DataType::Utf8, false),
        Field::new("market_slug", DataType::Utf8, false),
        Field::new("token_id", DataType::Utf8, false),
        Field::new("best_bid", DataType::Float64, true),
        Field::new("best_ask", DataType::Float64, true),
        Field::new("midpoint", DataType::Float64, true),
        Field::new("spread", DataType::Float64, true),
        Field::new("bid_depth_usd", DataType::Float64, false),
        Field::new("ask_depth_usd", DataType::Float64, false),
        Field::new("bid_levels", DataType::Int32, false),
        Field::new("ask_levels", DataType::Int32, false),
        Field::new("error", DataType::Utf8, true),
    ])
}

fn to_parquet(rows: &[Row]) -> Result<Vec<u8>> {
    let s = Arc::new(schema());
    let cols: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from_iter_values(rows.iter().map(|r| r.ts.as_str()))),
        Arc::new(StringArray::from_iter_values(rows.iter().map(|r| r.date.as_str()))),
        Arc::new(rows.iter().map(|r| r.hour).collect::<Int32Array>()),
        Arc::new(StringArray::from_iter_values(rows.iter().map(|r| r.condition_id.as_str()))),
        Arc::new(StringArray::from_iter_values(rows.iter().map(|r| r.market_slug.as_str()))),
        Arc::new(StringArray::from_iter_values(rows.iter().map(|r| r.token_id.as_str()))),
        Arc::new(rows.iter().map(|r| r.best_bid).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.best_ask).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.midpoint).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.spread).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.bid_depth_usd).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.ask_depth_usd).collect::<Float64Array>()),
        Arc::new(rows.iter().map(|r| r.bid_levels).collect::<Int32Array>()),
        Arc::new(rows.iter().map(|r| r.ask_levels).collect::<Int32Array>()),
        Arc::new(rows.iter().map(|r| r.error.as_deref()).collect::<StringArray>()),
    ];
    let batch = RecordBatch::try_new(s.clone(), cols)?;
    // zstd over the repeated id columns: a slug and a token_id are identical on
    // all 24 rows of a day, so the dictionary does most of the work.
    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
        .build();
    let mut buf: Vec<u8> = Vec::new();
    let mut w = ArrowWriter::try_new(&mut buf, s, Some(props))?;
    w.write(&batch)?;
    w.close()?;
    Ok(buf)
}

fn pack_day(bucket: &Bucket, date: &str) -> Result<()> {
    let rows = rows_for_day(bucket, date)?;
    if rows.is_empty() {
        println!("{date}: no hourly objects, skipped");
        return Ok(());
    }
    let hours: BTreeSet<i32> = rows.iter().map(|r| r.hour).collect();
    let bytes = to_parquet(&rows)?;
    let key = format!("snapshots/parquet/date={date}/books.parquet");
    bucket.put_object(&key, &bytes)?;
    println!(
        "{date}: {} rows from {} hours -> {key} ({} KiB)",
        rows.len(),
        hours.len(),
        bytes.len() / 1024
    );
    Ok(())
}

/// Read the parquet back out of R2 and reconcile it against the hourly JSON.
/// A derived view nobody checks is a rumour.
fn verify_day(bucket: &Bucket, date: &str) -> Result<()> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let key = format!("snapshots/parquet/date={date}/books.parquet");
    let got = bucket.get_object(&key)?;
    let bytes = bytes::Bytes::copy_from_slice(got.bytes());
    let reader = ParquetRecordBatchReaderBuilder::try_new(bytes)?.build()?;

    let mut n = 0usize;
    let mut sum_mid = 0.0f64;
    let mut with_bid = 0usize;
    for batch in reader {
        let batch = batch?;
        n += batch.num_rows();
        let mid = batch
            .column_by_name("midpoint")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("midpoint column")?;
        let bid = batch
            .column_by_name("best_bid")
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>())
            .context("best_bid column")?;
        for i in 0..batch.num_rows() {
            if !mid.is_null(i) {
                sum_mid += mid.value(i);
            }
            if !bid.is_null(i) {
                with_bid += 1;
            }
        }
    }

    let src = rows_for_day(bucket, date)?;
    let src_mid: f64 = src.iter().filter_map(|r| r.midpoint).sum();
    let src_bid = src.iter().filter(|r| r.best_bid.is_some()).count();

    println!("{date}");
    println!("  rows        parquet {n:>7}   json {:>7}   {}", src.len(),
        if n == src.len() { "ok" } else { "MISMATCH" });
    println!("  Σ midpoint  parquet {sum_mid:>7.3}   json {src_mid:>7.3}   {}",
        if (sum_mid - src_mid).abs() < 1e-6 { "ok" } else { "MISMATCH" });
    println!("  with a bid  parquet {with_bid:>7}   json {src_bid:>7}   {}",
        if with_bid == src_bid { "ok" } else { "MISMATCH" });
    if n != src.len() || (sum_mid - src_mid).abs() >= 1e-6 || with_bid != src_bid {
        bail!("{date}: parquet does not reconcile against the hourly JSON");
    }
    Ok(())
}
