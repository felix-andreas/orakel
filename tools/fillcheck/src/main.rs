//! fillcheck — was the price we scored against a price anyone would trade at?
//!
//! Every row in `predictions/predictions.csv` records `market_price`, a CLOB
//! **midpoint**. A midpoint is the average of a bid and an ask. On a wing leg
//! quoted 0.001 / 0.08 the midpoint reads 4c, and 4c is a number no
//! counterparty ever offered. Scoring a "paired improvement vs the market"
//! against it measures forecasting skill against a price that does not exist.
//!
//! This tool replays Polymarket's public trade feed for every market we
//! predicted on and asks, per row: **after we spoke, what is the best price at
//! which somebody demonstrably traded each side?** That is a directly
//! observed, conservative bound on the price we could have got.
//!
//! Method. The Data API reports the *taker* side of each trade. A taker who
//! SELLS Yes at q was filled by a resting bid at q, so q is provably a price a
//! seller could hit; a taker who BUYS Yes at q proves a resting ask at q for a
//! buyer. Trading No at r is trading Yes at 1-r with the direction flipped, so
//! both outcomes fold into one YES-equivalent view and a single pass answers
//! the question for either leg and either direction.
//!
//! Output `predictions/fills.csv`, one row per prediction row:
//!
//! ```text
//! timestamp,market_slug,outcome,mid,
//! bid_1h,bid_24h,bid_life,ask_1h,ask_24h,ask_life,
//! bid_notional_24h,ask_notional_24h,n_trades_after
//! ```
//!
//! Prices are in the units of the row's own `outcome` token, so they compare
//! directly with `market_price`. An empty field means nothing traded on that
//! side in that window — read it as "no observed counterparty", never as a
//! price of zero. `*_notional_24h` is the USDC that changed hands within a day
//! at a price at least as good as the midpoint: the size behind the headline.
//!
//! **This measures a lower bound.** A resting bid that nobody ever hit leaves
//! no trace in a trade feed, so a row reported as unreachable might have had a
//! quiet bid sitting there the whole time. The number to trust is therefore
//! "at least this many rows were reachable", never "exactly this many". The
//! only way to settle it is to record the book itself at prediction time —
//! which is why prediction rows are getting `bid`/`ask`/`depth` columns and
//! why the snapshot worker exists. Until then this is the best evidence
//! available, and it is evidence in the conservative direction: a row with a
//! *confirmed* fill is confirmed.
//!
//! Usage: fillcheck [--dir <predictions-dir>]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};

const DATA_API: &str = "https://data-api.polymarket.com/trades";
const PAGE: usize = 500;
const MAX_PAGES: usize = 40;
const HOUR: i64 = 3600;
const DAY: i64 = 86_400;

const HEADER: [&str; 13] = [
    "timestamp",
    "market_slug",
    "outcome",
    "mid",
    "bid_1h",
    "bid_24h",
    "bid_life",
    "ask_1h",
    "ask_24h",
    "ask_life",
    "bid_notional_24h",
    "ask_notional_24h",
    "n_trades_after",
];

/// One prediction row, reduced to what this question needs.
struct Row {
    timestamp_raw: String,
    at: DateTime<Utc>,
    market_slug: String,
    condition_id: String,
    outcome: String,
    mid: f64,
}

/// A trade normalised into YES-equivalent units. `taker_sold` marks the case
/// that proves a resting *bid* existed; the other case proves a resting ask.
struct Fill {
    at: i64,
    price: f64,
    taker_sold: bool,
    size: f64,
}

#[derive(serde::Deserialize)]
struct RawTrade {
    side: String,
    outcome: String,
    price: f64,
    size: f64,
    timestamp: i64,
}

fn main() -> Result<()> {
    let mut dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dir" => dir = Some(PathBuf::from(args.next().context("--dir needs a path")?)),
            "-h" | "--help" => {
                println!("fillcheck [--dir <predictions-dir>]");
                return Ok(());
            }
            other => bail!("unknown argument {other}"),
        }
    }
    let dir = dir.unwrap_or_else(default_dir);
    let rows = read_predictions(&dir.join("predictions.csv"))?;
    if rows.is_empty() {
        bail!("no prediction rows in {}", dir.display());
    }

    // One fetch per market, however many rows point at it.
    let mut markets: Vec<&str> = rows.iter().map(|r| r.condition_id.as_str()).collect();
    markets.sort_unstable();
    markets.dedup();
    eprintln!(
        "fillcheck: {} rows over {} markets, replaying the trade feed",
        rows.len(),
        markets.len()
    );

    let mut feed: HashMap<String, Vec<Fill>> = HashMap::new();
    let mut truncated: Vec<String> = Vec::new();
    for (i, cid) in markets.iter().enumerate() {
        let (fills, cut) = fetch_market(cid).with_context(|| format!("fetching trades for {cid}"))?;
        eprintln!(
            "  [{:>3}/{}] {cid} — {} trades{}",
            i + 1,
            markets.len(),
            fills.len(),
            if cut { "  [TRUNCATED at the API's offset cap]" } else { "" }
        );
        if cut {
            truncated.push((*cid).to_string());
        }
        feed.insert((*cid).to_string(), fills);
    }

    // Write to a temp file and rename, so `fills.csv` is only ever replaced by a
    // COMPLETE file.
    //
    // Writing in place was a real, measured loss. On 2026-07-30 a transient
    // failure killed a run partway through and left a truncated `fills.csv`;
    // `scoring/` read it without complaint and reported tradeability at
    // **13/34 (38%)** where the complete file gives **15/35 (43%)**. Nothing
    // said the file was partial — it parsed, it had the right header, it was
    // simply short.
    //
    // That is the fourth instance this week of the same failure: a file's
    // existence being treated as evidence of its completeness. The others were
    // in the variant's candle archive; this one is in the firm's own tooling,
    // one step upstream of a headline number.
    let out = dir.join("fills.csv");
    let tmp = dir.join("fills.csv.partial");
    let mut w = csv::Writer::from_path(&tmp)?;
    w.write_record(HEADER)?;

    let (mut reachable, mut known, mut sum_mid, mut sum_best) = (0usize, 0usize, 0.0f64, 0.0f64);
    for r in &rows {
        let fills = feed.get(&r.condition_id).map(Vec::as_slice).unwrap_or(&[]);
        let t0 = r.at.timestamp();
        // The row is about one outcome token; flip YES-equivalent prices into
        // that token's units so everything compares with `market_price`.
        let yes = r.outcome.eq_ignore_ascii_case("yes");
        let after: Vec<&Fill> = fills.iter().filter(|f| f.at >= t0).collect();

        // Best price on each side within a window. A seller wants the highest
        // bid, a buyer the lowest ask; both are in this row's outcome units.
        let side = |window: Option<i64>, want_bid: bool| -> Option<f64> {
            let mut best: Option<f64> = None;
            for f in after.iter().filter(|f| window.map_or(true, |w| f.at < t0 + w)) {
                // `taker_sold` is expressed in YES terms; a taker selling YES
                // is a taker buying NO, so the side flips with the token.
                let is_bid = if yes { f.taker_sold } else { !f.taker_sold };
                if is_bid != want_bid {
                    continue;
                }
                let p = if yes { f.price } else { 1.0 - f.price };
                best = Some(match best {
                    Some(b) if want_bid => b.max(p),
                    Some(b) => b.min(p),
                    None => p,
                });
            }
            best
        };
        let notional = |want_bid: bool| -> f64 {
            after
                .iter()
                .filter(|f| f.at < t0 + DAY)
                .filter_map(|f| {
                    let is_bid = if yes { f.taker_sold } else { !f.taker_sold };
                    if is_bid != want_bid {
                        return None;
                    }
                    let p = if yes { f.price } else { 1.0 - f.price };
                    // "At least as good as the mid" means above it for a
                    // seller, below it for a buyer.
                    let good = if want_bid { p >= r.mid - 1e-9 } else { p <= r.mid + 1e-9 };
                    good.then_some(f.size * p)
                })
                .sum()
        };

        let (b1, b24, blife) = (side(Some(HOUR), true), side(Some(DAY), true), side(None, true));
        let (a1, a24, alife) = (side(Some(HOUR), false), side(Some(DAY), false), side(None, false));

        // Headline counter: rows where a *seller* could have reached the price
        // we scored against. Selling the overpriced side is what our variants
        // do; when that changes, report the buy column instead.
        known += 1;
        if blife.is_some_and(|b| b >= r.mid - 1e-9) {
            reachable += 1;
        }
        sum_mid += r.mid;
        sum_best += blife.unwrap_or(0.0).min(r.mid);

        w.write_record([
            r.timestamp_raw.as_str(),
            r.market_slug.as_str(),
            r.outcome.as_str(),
            &fmt(Some(r.mid)),
            &fmt(b1),
            &fmt(b24),
            &fmt(blife),
            &fmt(a1),
            &fmt(a24),
            &fmt(alife),
            &usd(notional(true)),
            &usd(notional(false)),
            &after.len().to_string(),
        ])?;
    }
    w.flush()?;
    drop(w);
    // Rename is atomic within a filesystem: a reader sees either the previous
    // complete file or this one, never a half-written one.
    std::fs::rename(&tmp, &out)
        .with_context(|| format!("promoting {} to {}", tmp.display(), out.display()))?;

    println!("wrote {}", out.display());
    if !truncated.is_empty() {
        // Named, not counted: a reader has to be able to tell which rows carry a
        // weaker lower bound than the rest.
        println!(
            "\n{} market(s) hit the API's {OFFSET_CAP}-trade offset cap — their tape is INCOMPLETE,\nso reachability on their rows is a weaker lower bound than elsewhere:",
            truncated.len()
        );
        for cid in &truncated {
            println!("  {cid}");
        }
    }
    println!(
        "{reachable}/{known} rows ever saw a bid at or above the midpoint they were scored against"
    );
    if sum_mid > 0.0 {
        println!(
            "summed midpoints {sum_mid:.3} vs summed best reachable bids {sum_best:.3} — \
             {:.0}% of the price we scored against",
            100.0 * sum_best / sum_mid
        );
    }
    Ok(())
}

/// Empty for "no observed counterparty" — a missing price is not a zero price.
fn fmt(v: Option<f64>) -> String {
    v.map(|v| format!("{v:.4}")).unwrap_or_default()
}

/// Dollars, with negative zero flattened so an empty sum reads as `0.00`.
fn usd(v: f64) -> String {
    format!("{:.2}", if v == 0.0 { 0.0 } else { v })
}

fn default_dir() -> PathBuf {
    // repo/tools/fillcheck/ -> repo/predictions/
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(|repo| repo.join("predictions"))
        .unwrap_or_else(|| PathBuf::from("predictions"))
}

fn read_predictions(path: &Path) -> Result<Vec<Row>> {
    let mut rdr =
        csv::Reader::from_path(path).with_context(|| format!("opening {}", path.display()))?;
    let mut out = Vec::new();
    for rec in rdr.deserialize::<HashMap<String, String>>() {
        let rec = rec?;
        let get = |k: &str| rec.get(k).map(String::as_str).unwrap_or_default();
        let raw = get("timestamp").to_string();
        let at = raw
            .parse::<DateTime<Utc>>()
            .with_context(|| format!("bad timestamp {raw}"))?;
        out.push(Row {
            timestamp_raw: raw,
            at,
            market_slug: get("market_slug").to_string(),
            condition_id: get("condition_id").to_string(),
            outcome: get("outcome").to_string(),
            mid: get("market_price").parse().unwrap_or(f64::NAN),
        });
    }
    Ok(out)
}

/// Page the public trade feed for one market, folding every trade into
/// YES-equivalent units.
/// One page of the trade tape, retried on transport failure.
///
/// A settlement day walks ~83 markets of several thousand trades each, which is
/// tens of thousands of requests, and a single reset used to kill the whole run:
/// on 2026-08-01 it died at market 39 of 83 with `Tls Error: Connection reset by
/// peer` and did no work at all. The tape is immutable history, so a retry can
/// only return the same bytes — there is no correctness cost, only patience.
///
/// A non-2xx status is *not* retried: that is the API answering, and repeating
/// it is how you turn a bad request into a rate-limit.
fn get_page(url: &str) -> Result<attohttpc::Response> {
    let mut last = None;
    for attempt in 0..4 {
        match attohttpc::get(url).send() {
            Ok(r) => return Ok(r),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_secs(1 << attempt));
            }
        }
    }
    Err(anyhow::Error::from(last.expect("loop ran at least once")))
        .with_context(|| format!("4 attempts failed for {url}"))
}

/// The Data API refuses `offset` past ~10,000 with a **400**, not an empty page.
/// Probed 2026-08-01 on a live market: offset 9,500 and 10,000 return 200,
/// 10,500 returns 400. Gamma has the same shape of cap at a different number
/// (`wiki/recipes/polymarket-api.md`), so treat it as a house style.
///
/// A market busier than this cannot be walked to the end, and that is a fact
/// about the data, not an error to swallow — `fetch_market` reports it and the
/// caller names the affected markets. `fills.csv` is already documented as a
/// LOWER bound on reachability; a truncated tape simply makes it a weaker one,
/// and the one thing we must not do is fail to say which rows it applies to.
const OFFSET_CAP: usize = 10_000;

/// Trades for one market, and whether the tape was truncated by the cap.
fn fetch_market(condition_id: &str) -> Result<(Vec<Fill>, bool)> {
    let mut out = Vec::new();
    let mut truncated = false;
    for page in 0..MAX_PAGES {
        let offset = page * PAGE;
        if offset > OFFSET_CAP {
            truncated = true;
            break;
        }
        let url = format!("{DATA_API}?market={condition_id}&limit={PAGE}&offset={offset}");
        let resp = get_page(&url)?;
        if !resp.is_success() {
            bail!("data-api returned {} for {url}", resp.status());
        }
        let batch: Vec<RawTrade> = resp.json()?;
        let n = batch.len();
        for t in batch {
            let yes = t.outcome.eq_ignore_ascii_case("yes");
            let price = if yes { t.price } else { 1.0 - t.price };
            let sold = if yes {
                t.side.eq_ignore_ascii_case("sell")
            } else {
                t.side.eq_ignore_ascii_case("buy")
            };
            out.push(Fill { at: t.timestamp, price, taker_sold: sold, size: t.size });
        }
        if n < PAGE {
            break;
        }
    }
    Ok((out, truncated))
}
