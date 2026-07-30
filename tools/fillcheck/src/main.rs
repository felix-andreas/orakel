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
    for (i, cid) in markets.iter().enumerate() {
        let fills = fetch_market(cid).with_context(|| format!("fetching trades for {cid}"))?;
        eprintln!("  [{:>3}/{}] {cid} — {} trades", i + 1, markets.len(), fills.len());
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
fn fetch_market(condition_id: &str) -> Result<Vec<Fill>> {
    let mut out = Vec::new();
    for page in 0..MAX_PAGES {
        let url = format!("{DATA_API}?market={condition_id}&limit={PAGE}&offset={}", page * PAGE);
        let resp = attohttpc::get(&url).send()?;
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
    Ok(out)
}
