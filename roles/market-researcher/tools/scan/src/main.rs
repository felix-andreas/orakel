// Landscape scan: page Gamma /events (open, active), flatten nested markets to CSV,
// print a category / horizon / volume summary. Read-only, no keys.
//
// Usage: scan <out.csv> [pages=15] [order=volume24hr]
//   pages of 100 events each; order is any Gamma sort key (volume24hr, volumeNum,
//   liquidityNum, startDate, endDate, ...).

use std::collections::BTreeMap;
use std::io::Write;

fn f(v: &serde_json::Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

fn s(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn csv_escape(x: &str) -> String {
    if x.contains(',') || x.contains('"') || x.contains('\n') {
        format!("\"{}\"", x.replace('"', "\"\""))
    } else {
        x.to_string()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_path = args.get(1).map(String::as_str).unwrap_or("scan.csv");
    let pages: usize = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(15);
    let order = args.get(3).map(String::as_str).unwrap_or("volume24hr");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("client");

    let mut rows: Vec<BTreeMap<&'static str, String>> = Vec::new();
    let mut seen_events = std::collections::HashSet::new();

    for page in 0..pages {
        let offset = (page * 100).to_string();
        let resp: serde_json::Value = client
            .get("https://gamma-api.polymarket.com/events")
            .query(&[
                ("closed", "false"),
                ("active", "true"),
                ("archived", "false"),
                ("order", order),
                ("ascending", "false"),
                ("limit", "100"),
                ("offset", &offset),
            ])
            .send()
            .expect("request")
            .json()
            .expect("json");
        let events = resp.as_array().cloned().unwrap_or_default();
        let n = events.len();
        for ev in events {
            let ev_slug = s(&ev["slug"]);
            if !seen_events.insert(ev_slug.clone()) {
                continue; // ordering can shift between pages
            }
            let tags: Vec<String> = ev["tags"]
                .as_array()
                .map(|a| a.iter().map(|t| s(&t["label"])).collect())
                .unwrap_or_default();
            let markets = ev["markets"].as_array().cloned().unwrap_or_default();
            let n_markets = markets.len();
            for m in &markets {
                if m["closed"].as_bool() == Some(true) {
                    continue;
                }
                let prices: Vec<f64> = m["outcomePrices"]
                    .as_str()
                    .and_then(|p| serde_json::from_str::<Vec<String>>(p).ok())
                    .map(|v| v.iter().filter_map(|x| x.parse().ok()).collect())
                    .unwrap_or_default();
                let mut row = BTreeMap::new();
                row.insert("event_slug", ev_slug.clone());
                row.insert("event_title", s(&ev["title"]));
                row.insert("tags", tags.join("|"));
                row.insert("neg_risk", ev["negRisk"].as_bool().unwrap_or(false).to_string());
                row.insert("n_markets", n_markets.to_string());
                row.insert("question", s(&m["question"]));
                row.insert("market_slug", s(&m["slug"]));
                row.insert("condition_id", s(&m["conditionId"]));
                row.insert("end_date", s(&m["endDateIso"]));
                row.insert("start_date", s(&m["startDateIso"]));
                row.insert("volume_num", format!("{:.2}", f(&m["volumeNum"])));
                row.insert("volume_24hr", format!("{:.2}", f(&m["volume24hr"])));
                row.insert("volume_1wk", format!("{:.2}", f(&m["volume1wk"])));
                row.insert("liquidity_num", format!("{:.2}", f(&m["liquidityNum"])));
                row.insert("spread", format!("{:.4}", f(&m["spread"])));
                row.insert("best_bid", format!("{:.4}", f(&m["bestBid"])));
                row.insert("best_ask", format!("{:.4}", f(&m["bestAsk"])));
                row.insert(
                    "yes_price",
                    prices.first().map(|p| format!("{:.4}", p)).unwrap_or_default(),
                );
                rows.push(row);
            }
        }
        eprintln!("page {page}: {n} events, total {} market rows", rows.len());
        if n < 100 {
            break;
        }
    }

    // CSV out
    let cols = [
        "event_slug", "event_title", "tags", "neg_risk", "n_markets", "question",
        "market_slug", "condition_id", "start_date", "end_date", "volume_num",
        "volume_24hr", "volume_1wk", "liquidity_num", "spread", "best_bid", "best_ask",
        "yes_price",
    ];
    let mut out = std::fs::File::create(out_path).expect("create csv");
    writeln!(out, "{}", cols.join(",")).unwrap();
    for r in &rows {
        let line: Vec<String> = cols
            .iter()
            .map(|c| csv_escape(r.get(c).map(String::as_str).unwrap_or("")))
            .collect();
        writeln!(out, "{}", line.join(",")).unwrap();
    }

    // Summary: tag counts, horizon buckets, volume buckets
    let today = {
        // days since epoch for a YYYY-MM-DD without chrono: rough parse
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now / 86400
    };
    let days = |d: &str| -> Option<i64> {
        let mut it = d.split('-');
        let y: i64 = it.next()?.parse().ok()?;
        let m: i64 = it.next()?.parse().ok()?;
        let dd: i64 = it.next()?.parse().ok()?;
        // days since epoch (civil algorithm, Howard Hinnant)
        let y2 = if m <= 2 { y - 1 } else { y };
        let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
        let yoe = y2 - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + dd - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Some(era * 146097 + doe - 719468 - today)
    };

    let mut tag_count: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    let mut horizon: BTreeMap<&str, usize> = BTreeMap::new();
    let mut volbucket: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &rows {
        for t in r["tags"].split('|').take(3) {
            if t.is_empty() {
                continue;
            }
            let e = tag_count.entry(t.to_string()).or_insert((0, 0.0));
            e.0 += 1;
            e.1 += r["volume_num"].parse::<f64>().unwrap_or(0.0);
        }
        let h = days(&r["end_date"]).unwrap_or(9999);
        *horizon
            .entry(match h {
                i64::MIN..=0 => "0 <=today",
                1..=7 => "1 <=7d",
                8..=30 => "2 <=30d",
                31..=90 => "3 <=90d",
                _ => "4 >90d",
            })
            .or_insert(0) += 1;
        let v = r["volume_num"].parse::<f64>().unwrap_or(0.0);
        *volbucket
            .entry(if v < 10_000.0 {
                "0 <$10k"
            } else if v < 100_000.0 {
                "1 $10k-100k"
            } else if v < 1_000_000.0 {
                "2 $100k-1M"
            } else {
                "3 >$1M"
            })
            .or_insert(0) += 1;
    }
    println!("== {} market rows -> {} ==", rows.len(), out_path);
    println!("-- horizon (end_date) --");
    for (k, v) in &horizon {
        println!("{k}: {v}");
    }
    println!("-- volume --");
    for (k, v) in &volbucket {
        println!("{k}: {v}");
    }
    let mut tags: Vec<_> = tag_count.into_iter().collect();
    tags.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    println!("-- top tags (first 3 per event) : n_markets, total volume --");
    for (t, (n, v)) in tags.iter().take(40) {
        println!("{t}: {n}, ${:.0}", v);
    }
}
