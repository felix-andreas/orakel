// barrier-touch/ladder-rv — backtest + live tooling for Polymarket "Hit Price"
// one-touch barrier ladders (CLOB pipeline copied from temp-truncation/runningmax).
//
// Subcommands (data_dir is a working directory OUTSIDE git; frozen to R2 as a tarball):
//   discover <data_dir> <slug,slug,...>       Gamma events -> events/<slug>.json + legs.csv
//   candles  <data_dir> <KEY> <from> <to>     1-min candles -> candles/<KEY>/<date>.json
//                                             KEY: BTCUSDT ETHUSDT (Binance) | USOILSPOT SPY NVDA WTIU6 (Pyth)
//   vol      <data_dir>                       OVX+VIX (CBOE) + DVOL BTC/ETH (Deribit) -> vol/
//   clob     <data_dir> <fidelity> [board,..] CLOB prices-history per leg -> clob<f>/<board>/<condition_id>.json
//   analyze  <data_dir>                       gates 0-3 (+ violations if clob10 present) -> out/ + stdout
//   tape     <data_dir> <board> [board...]    Data-API trades -> tape/<board>/<condition_id>.json
//   wash     <data_dir> <board> [board...]    gate 4 wash checks on tape
//   live     <data_dir> <slug,slug,...>       books + model -> prediction rows
//
// Fine print baked in (verified against market descriptions, 2026-07-23):
//   - crypto monthlies resolve on BINANCE <ASSET>/USDT 1-min candles (not Pyth);
//     original legs = calendar month in ET; re-added legs = "from creation of this market".
//   - equity weeklies resolve on Pyth Equity.US.<T>/USD, RTH 13:30-20:00 UTC (EDT) only.
//   - WTI monthlies resolve on Pyth active-month CL futures, session = 18:00 ET prior
//     day -> 17:00 ET, business days only; active month rolls 3 sessions before LTD.
//     Expired per-contract Pyth feeds are DELISTED from Benchmarks -> backtests use the
//     continuous Commodities.USOILSPOT CFD feed as proxy (basis error measured in
//     `analyze` on the WTIU6 overlap; borderline legs flagged by |margin|).
//   - all summer-2026 windows -> fixed EDT (UTC-4). Holidays: May 25, Jun 19, Jul 3.

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Datelike, DateTime, Duration, NaiveDate, Utc, Weekday};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// ---------- small math ----------

fn erf(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26, |err| < 1.5e-7
    let s = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-x * x).exp();
    s * y
}

fn ncdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn sd(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return f64::NAN;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn quantile(v: &mut Vec<f64>, q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(((v.len() - 1) as f64) * q).round() as usize]
}

/// One-touch probability under driftless GBM: 2*N(-|ln(B/S)|/(sigma*sqrt(tau))).
fn touch_prob(s: f64, b: f64, dir: char, sigma: f64, tau_yrs: f64) -> f64 {
    let beyond = (dir == 'H' && s >= b) || (dir == 'L' && s <= b);
    if beyond {
        return 1.0;
    }
    if tau_yrs <= 0.0 || sigma <= 0.0 {
        return 0.0;
    }
    let d = (b / s).ln().abs() / (sigma * tau_yrs.sqrt());
    (2.0 * ncdf(-d)).min(1.0)
}

// ---------- HTTP (copied from runningmax) ----------

fn mk_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("orakel-ladderrv/0.1")
        .build()
        .expect("client")
}

fn get_text(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    let mut last = anyhow!("no attempt");
    for (i, backoff) in [1u64, 3, 8].iter().enumerate() {
        match client.get(url).send() {
            Ok(r) if r.status().is_success() => return Ok(r.text()?),
            Ok(r) => last = anyhow!("status {} on {url}", r.status()),
            Err(e) => last = anyhow!("{e} on {url}"),
        }
        if i < 2 {
            std::thread::sleep(std::time::Duration::from_secs(*backoff));
        }
    }
    Err(last)
}

/// Fetch (url, out_path) pairs with a small thread pool; skips existing files.
fn fetch_all(jobs: Vec<(String, PathBuf)>, threads: usize) -> (usize, usize, usize) {
    let total = jobs.len();
    let skipped = jobs.iter().filter(|(_, p)| p.exists()).count();
    let queue = Arc::new(Mutex::new(
        jobs.into_iter().filter(|(_, p)| !p.exists()).collect::<Vec<_>>(),
    ));
    let fails = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];
    for _ in 0..threads {
        let queue = Arc::clone(&queue);
        let fails = Arc::clone(&fails);
        handles.push(std::thread::spawn(move || {
            let client = mk_client();
            loop {
                let job = queue.lock().unwrap().pop();
                let Some((url, path)) = job else { break };
                match get_text(&client, &url) {
                    Ok(body) => {
                        if let Some(par) = path.parent() {
                            fs::create_dir_all(par).ok();
                        }
                        fs::write(&path, body).ok();
                        if url.contains("benchmarks.pyth.network") {
                            std::thread::sleep(std::time::Duration::from_millis(600));
                        }
                    }
                    Err(e) => {
                        eprintln!("FAIL {e:#}");
                        *fails.lock().unwrap() += 1;
                    }
                }
            }
        }));
    }
    for h in handles {
        h.join().ok();
    }
    let f = *fails.lock().unwrap();
    (total - skipped - f, skipped, f)
}

// ---------- assets, sessions, calendars ----------

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum Class {
    Crypto,
    Equity,
    Wti,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Asset {
    Btc,
    Eth,
    Wti,
    Spy,
    Nvda,
    Gold,
    Silver,
}

impl Asset {
    fn from_slug(slug: &str) -> Result<Asset> {
        if slug.contains("bitcoin") {
            Ok(Asset::Btc)
        } else if slug.contains("ethereum") {
            Ok(Asset::Eth)
        } else if slug.contains("-wti-") || slug.starts_with("wti-") {
            Ok(Asset::Wti)
        } else if slug.contains("-spy-") {
            Ok(Asset::Spy)
        } else if slug.contains("-nvda-") {
            Ok(Asset::Nvda)
        } else if slug.contains("xauusd") {
            Ok(Asset::Gold)
        } else if slug.contains("xagusd") {
            Ok(Asset::Silver)
        } else {
            bail!("cannot detect asset from slug {slug}")
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Asset::Btc => "btc",
            Asset::Eth => "eth",
            Asset::Wti => "wti",
            Asset::Spy => "spy",
            Asset::Nvda => "nvda",
            Asset::Gold => "gold",
            Asset::Silver => "silver",
        }
    }
    fn from_name(n: &str) -> Result<Asset> {
        Ok(match n {
            "btc" => Asset::Btc,
            "eth" => Asset::Eth,
            "wti" => Asset::Wti,
            "spy" => Asset::Spy,
            "nvda" => Asset::Nvda,
            "gold" => Asset::Gold,
            "silver" => Asset::Silver,
            _ => bail!("unknown asset {n}"),
        })
    }
    fn class(&self) -> Class {
        match self {
            Asset::Btc | Asset::Eth => Class::Crypto,
            Asset::Spy | Asset::Nvda => Class::Equity,
            // Metals (XAUUSD/XAGUSD) resolve on a COMEX-style session: 6pm ET Sun ->
            // 5pm ET Fri with a daily 5-6pm ET break == WTI's 22:00Z->21:00Z model.
            Asset::Wti | Asset::Gold | Asset::Silver => Class::Wti,
        }
    }
    /// Candle store key for backtests (WTI: continuous USOILSPOT proxy).
    fn candle_key(&self) -> &'static str {
        match self {
            Asset::Btc => "BTCUSDT",
            Asset::Eth => "ETHUSDT",
            Asset::Wti => "USOILSPOT",
            Asset::Spy => "SPY",
            Asset::Nvda => "NVDA",
            // Continuous Pyth metals feeds (not per-contract) — no delisting, no proxy.
            Asset::Gold => "XAUUSD",
            Asset::Silver => "XAGUSD",
        }
    }
    /// Candle store key for LIVE spot reads (WTI: the current active-month contract).
    /// U6 is active until ~2026-08-17 18:00 ET (3 sessions before U6 LTD 2026-08-20).
    fn live_key(&self) -> &'static str {
        match self {
            Asset::Wti => "WTIU6",
            _ => self.candle_key(),
        }
    }
}

fn key_url(key: &str) -> (&'static str, String) {
    match key {
        "BTCUSDT" => ("binance", "BTCUSDT".into()),
        "ETHUSDT" => ("binance", "ETHUSDT".into()),
        "USOILSPOT" => ("pyth", "Commodities.USOILSPOT".into()),
        "SPY" => ("pyth", "Equity.US.SPY/USD".into()),
        "NVDA" => ("pyth", "Equity.US.NVDA/USD".into()),
        "WTIU6" => ("pyth", "Commodities.WTIU6/USD".into()),
        "WTIV6" => ("pyth", "Commodities.WTIV6/USD".into()),
        "XAUUSD" => ("pyth", "Metal.XAU/USD".into()),
        "XAGUSD" => ("pyth", "Metal.XAG/USD".into()),
        _ => panic!("unknown candle key {key}"),
    }
}

fn is_holiday(d: NaiveDate) -> bool {
    d.year() == 2026
        && matches!((d.month(), d.day()), (5, 25) | (6, 19) | (7, 3))
}

fn is_bizday(d: NaiveDate) -> bool {
    !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) && !is_holiday(d)
}

fn ts(d: NaiveDate, h: u32, m: u32) -> i64 {
    d.and_hms_opt(h, m, 0).unwrap().and_utc().timestamp()
}

fn date_of(t: i64) -> NaiveDate {
    DateTime::from_timestamp(t, 0).unwrap().date_naive()
}

/// Sorted minute-start timestamps of all in-session minutes, 2026-04-01..2026-08-20.
struct SessionCal {
    mins: Vec<i64>,
}

impl SessionCal {
    fn build(class: Class) -> SessionCal {
        let from = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 20).unwrap();
        let mut mins = vec![];
        let mut d = from;
        while d <= to {
            match class {
                Class::Crypto => {
                    let t0 = ts(d, 0, 0);
                    for i in 0..1440 {
                        mins.push(t0 + i * 60);
                    }
                }
                Class::Equity => {
                    if is_bizday(d) {
                        let t0 = ts(d, 13, 30);
                        for i in 0..390 {
                            mins.push(t0 + i * 60);
                        }
                    }
                }
                Class::Wti => {
                    // session for business day d: (d-1) 22:00Z .. d 21:00Z (EDT)
                    if is_bizday(d) {
                        let t0 = ts(d - Duration::days(1), 22, 0);
                        for i in 0..1380 {
                            mins.push(t0 + i * 60);
                        }
                    }
                }
            }
            d += Duration::days(1);
        }
        mins.sort_unstable();
        SessionCal { mins }
    }
    fn is_open(&self, t: i64) -> bool {
        let m = t - t % 60;
        self.mins.binary_search(&m).is_ok()
    }
    /// Number of session minutes in [t0, t1).
    fn count(&self, t0: i64, t1: i64) -> i64 {
        if t1 <= t0 {
            return 0;
        }
        let a = self.mins.partition_point(|&m| m < t0);
        let b = self.mins.partition_point(|&m| m < t1);
        (b - a) as i64
    }
}

fn min_per_year(class: Class) -> f64 {
    match class {
        Class::Crypto => 365.25 * 1440.0,
        Class::Equity => 252.0 * 390.0,
        Class::Wti => 252.0 * 1380.0,
    }
}

fn day_minutes(class: Class) -> f64 {
    match class {
        Class::Crypto => 1440.0,
        Class::Equity => 390.0,
        Class::Wti => 1380.0,
    }
}

// ---------- boards & legs ----------

fn month_num(name: &str) -> Option<u32> {
    let months = [
        "january", "february", "march", "april", "may", "june", "july", "august", "september",
        "october", "november", "december",
    ];
    months.iter().position(|m| *m == name).map(|i| i as u32 + 1)
}

/// Board period [start, end) in epoch seconds, from the event slug.
/// Monthly crypto: calendar month in ET (04:00Z..04:00Z, EDT).
/// Monthly WTI: first session open (prior day 22:00Z) .. last bizday 21:00Z.
/// Weekly equity: Mon 00:00Z .. Fri 20:00Z (RTH filter narrows further).
fn board_period(slug: &str, asset: Asset) -> Result<(i64, i64)> {
    let toks: Vec<&str> = slug.split('-').collect();
    if let Some(i) = toks.iter().position(|t| *t == "week") {
        // ...-week-of-<month>-<day>-<year>
        let m = month_num(toks[i + 2]).context("weekly month")?;
        let d: u32 = toks[i + 3].parse()?;
        let y: i32 = toks[i + 4].parse()?;
        let mon = NaiveDate::from_ymd_opt(y, m, d).context("weekly date")?;
        anyhow::ensure!(mon.weekday() == Weekday::Mon, "week-of date {mon} not a Monday");
        // The weekly window follows the asset's own session clock, not a calendar week.
        // Equity weeklies run Mon 00:00Z (RTH filter applies) -> Fri 20:00Z (16:00 ET).
        // WTI/metals weeklies run on the CME/COMEX clock: the Monday session opens
        // Sunday 22:00Z (6pm ET) and the week ends Friday 21:00Z (5pm ET) — Gamma's
        // endDate confirms 21:00Z on these boards. Holidays inside the week are handled
        // by SessionCal, which simply has no minutes for them.
        return Ok(match asset.class() {
            Class::Equity => (ts(mon, 0, 0), ts(mon + Duration::days(4), 20, 0)),
            Class::Wti => (
                ts(mon - Duration::days(1), 22, 0),
                ts(mon + Duration::days(4), 21, 0),
            ),
            Class::Crypto => (ts(mon, 0, 0), ts(mon + Duration::days(5), 0, 0)),
        });
    }
    if let Some(i) = toks.iter().position(|t| *t == "in") {
        let m = month_num(toks[i + 1]).context("monthly month")?;
        let y: i32 = toks[i + 2].parse()?;
        let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let next = if m == 12 {
            NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap()
        };
        return Ok(match asset.class() {
            Class::Crypto => (ts(first, 4, 0), ts(next, 4, 0)),
            Class::Wti => {
                let mut fb = first;
                while !is_bizday(fb) {
                    fb += Duration::days(1);
                }
                let mut lb = next - Duration::days(1);
                while !is_bizday(lb) {
                    lb -= Duration::days(1);
                }
                (ts(fb - Duration::days(1), 22, 0), ts(lb, 21, 0))
            }
            Class::Equity => (ts(first, 0, 0), ts(next, 0, 0)),
        });
    }
    bail!("cannot parse period from slug {slug}")
}

#[derive(Clone, Debug)]
struct Leg {
    board: String,
    asset: Asset,
    tier: String, // weekly | monthly
    market_slug: String,
    condition_id: String,
    token_yes: String,
    dir: char, // 'H' | 'L'
    barrier: f64,
    ws: i64, // touch window start (period start, or leg creation for re-added legs)
    we: i64, // touch window end
    leg_start: i64,
    creation_clause: bool,
    closed: bool,
    winner: i8, // 1/0 resolved, -1 open
    volume: f64,
    closed_time: i64,
}

fn parse_iso(s: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.timestamp())
}

fn extract_legs(slug: &str, ev: &serde_json::Value) -> Result<Vec<Leg>> {
    let asset = Asset::from_slug(slug)?;
    let (ps, pe) = board_period(slug, asset)?;
    let tier = if slug.contains("week-of") { "weekly" } else { "monthly" };
    let mut out = vec![];
    for m in ev["markets"].as_array().context("no markets")? {
        let git = m["groupItemTitle"].as_str().unwrap_or_default();
        let mut chars = git.chars();
        let arrow = chars.next().unwrap_or('?');
        let dir = match arrow {
            '↑' => 'H',
            '↓' => 'L',
            _ => bail!("no direction arrow in {git:?}"),
        };
        let num: String = git
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let barrier: f64 = num.parse().with_context(|| format!("barrier from {git:?}"))?;
        let desc = m["description"].as_str().unwrap_or_default();
        let creation_clause =
            desc.contains("after market creation") || desc.contains("creation of this market");
        let leg_start = parse_iso(m["startDate"].as_str().unwrap_or_default()).unwrap_or(0);
        // Uniform creation-start rule: even "calendar month" legs resolve from listing
        // in practice (BTC June ↑72.5k: pre-listing touch on Jun 1 04:00-14:40Z was NOT
        // counted by the resolver). ws = max(period start, leg listing).
        let ws = leg_start.max(ps);
        let tokens: Vec<String> =
            serde_json::from_str(m["clobTokenIds"].as_str().unwrap_or("[]")).unwrap_or_default();
        let prices: Vec<String> =
            serde_json::from_str(m["outcomePrices"].as_str().unwrap_or("[]")).unwrap_or_default();
        let closed = m["closed"].as_bool().unwrap_or(false);
        let winner = if !closed {
            -1
        } else if prices.first().map(|p| p == "1").unwrap_or(false) {
            1
        } else {
            0
        };
        out.push(Leg {
            board: slug.to_string(),
            asset,
            tier: tier.into(),
            market_slug: m["slug"].as_str().unwrap_or_default().into(),
            condition_id: m["conditionId"].as_str().unwrap_or_default().into(),
            token_yes: tokens.first().cloned().unwrap_or_default(),
            dir,
            barrier,
            ws,
            we: pe,
            leg_start,
            creation_clause,
            closed,
            winner,
            volume: m["volumeNum"].as_f64().unwrap_or(0.0),
            closed_time: parse_iso(m["closedTime"].as_str().unwrap_or_default()).unwrap_or(0),
        });
    }
    Ok(out)
}

fn legs_csv(data: &Path) -> PathBuf {
    data.join("legs.csv")
}

fn save_legs(data: &Path, legs: &[Leg]) -> Result<()> {
    let mut rows = vec![
        "board,asset,tier,market_slug,condition_id,token_yes,dir,barrier,ws,we,leg_start,creation_clause,closed,winner,volume,closed_time"
            .to_string(),
    ];
    for l in legs {
        rows.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            l.board,
            l.asset.name(),
            l.tier,
            l.market_slug,
            l.condition_id,
            l.token_yes,
            l.dir,
            l.barrier,
            l.ws,
            l.we,
            l.leg_start,
            l.creation_clause as u8,
            l.closed as u8,
            l.winner,
            l.volume,
            l.closed_time
        ));
    }
    fs::write(legs_csv(data), rows.join("\n") + "\n")?;
    Ok(())
}

fn load_legs(data: &Path) -> Result<Vec<Leg>> {
    let txt = fs::read_to_string(legs_csv(data)).context("legs.csv (run discover first)")?;
    let mut out = vec![];
    for line in txt.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 16 {
            continue;
        }
        out.push(Leg {
            board: f[0].into(),
            asset: Asset::from_name(f[1])?,
            tier: f[2].into(),
            market_slug: f[3].into(),
            condition_id: f[4].into(),
            token_yes: f[5].into(),
            dir: f[6].chars().next().unwrap(),
            barrier: f[7].parse()?,
            ws: f[8].parse()?,
            we: f[9].parse()?,
            leg_start: f[10].parse()?,
            creation_clause: f[11] == "1",
            closed: f[12] == "1",
            winner: f[13].parse()?,
            volume: f[14].parse()?,
            closed_time: f[15].parse()?,
        });
    }
    Ok(out)
}

// ---------- discover ----------

fn cmd_discover(data: &Path, slugs: &[String]) -> Result<()> {
    let jobs: Vec<(String, PathBuf)> = slugs
        .iter()
        .map(|s| {
            (
                format!("https://gamma-api.polymarket.com/events?slug={s}"),
                data.join("events").join(format!("{s}.json")),
            )
        })
        .collect();
    let (fetched, skipped, failed) = fetch_all(jobs, 6);
    println!("discover: fetched {fetched}, cached {skipped}, failed {failed}");
    let mut legs = vec![];
    for entry in fs::read_dir(data.join("events"))? {
        let p = entry?.path();
        let slug = p.file_stem().unwrap().to_string_lossy().to_string();
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
        let Some(ev) = v.as_array().and_then(|a| a.first()) else {
            eprintln!("  {slug}: event not found");
            continue;
        };
        let mut ls = extract_legs(&slug, ev)?;
        println!(
            "  {slug}: {} legs ({} closed, {} creation-clause)",
            ls.len(),
            ls.iter().filter(|l| l.closed).count(),
            ls.iter().filter(|l| l.creation_clause).count()
        );
        legs.append(&mut ls);
    }
    legs.sort_by(|a, b| (a.board.clone(), a.dir, a.barrier as i64).cmp(&(b.board.clone(), b.dir, b.barrier as i64)));
    save_legs(data, &legs)?;
    println!("legs.csv: {} legs total", legs.len());
    Ok(())
}

// ---------- candles ----------

#[derive(Clone, Copy, Debug)]
struct Candle {
    t: i64,
    h: f64,
    l: f64,
    c: f64,
}

fn cmd_candles(data: &Path, key: &str, from: NaiveDate, to: NaiveDate) -> Result<()> {
    let (provider, sym) = key_url(key);
    let dir = data.join("candles").join(key);
    let today = Utc::now().date_naive();
    let mut jobs = vec![];
    let mut d = from;
    while d <= to {
        let t0 = ts(d, 0, 0);
        if d == today {
            // partial day: always refetch
            let _ = fs::remove_file(dir.join(format!("{d}_a.json")));
            let _ = fs::remove_file(dir.join(format!("{d}_b.json")));
        }
        match provider {
            "pyth" => {
                let url = format!(
                    "https://benchmarks.pyth.network/v1/shims/tradingview/history?symbol={sym}&resolution=1&from={t0}&to={}",
                    t0 + 86399
                );
                jobs.push((url, dir.join(format!("{d}_a.json"))));
            }
            "binance" => {
                let u1 = format!(
                    "https://data-api.binance.vision/api/v3/klines?symbol={sym}&interval=1m&startTime={}&limit=1000",
                    t0 * 1000
                );
                let u2 = format!(
                    "https://data-api.binance.vision/api/v3/klines?symbol={sym}&interval=1m&startTime={}&limit=440",
                    (t0 + 60000) * 1000
                );
                jobs.push((u1, dir.join(format!("{d}_a.json"))));
                jobs.push((u2, dir.join(format!("{d}_b.json"))));
            }
            _ => unreachable!(),
        }
        d += Duration::days(1);
    }
    // Pyth benchmarks rate-limits aggressively: single thread + pause between fetches.
    let threads = if provider == "pyth" { 1 } else { 4 };
    let (fetched, skipped, failed) = fetch_all(jobs, threads);
    println!("candles {key}: fetched {fetched}, cached {skipped}, failed {failed}");
    Ok(())
}

/// Load every candle file for a key, sorted+dedup by t. Sniffs Pyth vs Binance format.
fn load_candles(data: &Path, key: &str) -> Vec<Candle> {
    let dir = data.join("candles").join(key);
    let mut out: BTreeMap<i64, Candle> = BTreeMap::new();
    let Ok(rd) = fs::read_dir(&dir) else { return vec![] };
    for entry in rd.flatten() {
        let Ok(txt) = fs::read_to_string(entry.path()) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
        if let Some(ts_arr) = v["t"].as_array() {
            // Pyth TV shim: {s,t[],o[],h[],l[],c[]}
            let (h, l, c) = (
                v["h"].as_array().cloned().unwrap_or_default(),
                v["l"].as_array().cloned().unwrap_or_default(),
                v["c"].as_array().cloned().unwrap_or_default(),
            );
            for i in 0..ts_arr.len() {
                let t = ts_arr[i].as_i64().unwrap_or(0);
                let (Some(h), Some(l), Some(c)) =
                    (h.get(i).and_then(|x| x.as_f64()), l.get(i).and_then(|x| x.as_f64()), c.get(i).and_then(|x| x.as_f64()))
                else {
                    continue;
                };
                out.insert(t, Candle { t, h, l, c });
            }
        } else if let Some(rows) = v.as_array() {
            // Binance klines: [[openTimeMs, o, h, l, c, ...], ...]
            for r in rows {
                let Some(r) = r.as_array() else { continue };
                let t = r[0].as_i64().unwrap_or(0) / 1000;
                let h: f64 = r[2].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let l: f64 = r[3].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let c: f64 = r[4].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                out.insert(t, Candle { t, h, l, c });
            }
        }
    }
    out.into_values().collect()
}

struct CandleDb {
    by_key: HashMap<String, Vec<Candle>>,
}

impl CandleDb {
    fn load(data: &Path, keys: &[&str]) -> CandleDb {
        let mut by_key = HashMap::new();
        for k in keys {
            let v = load_candles(data, k);
            println!("  candles[{k}]: {} rows", v.len());
            by_key.insert(k.to_string(), v);
        }
        CandleDb { by_key }
    }
    fn slice(&self, key: &str, t0: i64, t1: i64) -> &[Candle] {
        let Some(v) = self.by_key.get(key) else { return &[] };
        let a = v.partition_point(|c| c.t < t0);
        let b = v.partition_point(|c| c.t < t1);
        &v[a..b]
    }
    /// Last in-session close at or before t (within max_age).
    fn spot_at(&self, key: &str, cal: &SessionCal, t: i64, max_age: i64) -> Option<f64> {
        let Some(v) = self.by_key.get(key) else { return None };
        let idx = v.partition_point(|c| c.t <= t);
        v[..idx]
            .iter()
            .rev()
            .take(10000)
            .find(|c| cal.is_open(c.t) && t - c.t <= max_age)
            .map(|c| c.c)
    }
}

/// Realized vol from 5-min closes over trailing `lookback_s`, annualized on session time.
fn realized_vol(db: &CandleDb, key: &str, cal: &SessionCal, class: Class, t: i64, lookback_s: i64) -> Option<f64> {
    let candles = db.slice(key, t - lookback_s, t);
    let mut buckets: BTreeMap<i64, f64> = BTreeMap::new();
    for c in candles {
        if class == Class::Crypto || cal.is_open(c.t) {
            buckets.insert(c.t - c.t % 300, c.c);
        }
    }
    if buckets.len() < 100 {
        return None;
    }
    let closes: Vec<f64> = buckets.values().cloned().collect();
    let mut ssq = 0.0;
    for w in closes.windows(2) {
        let r = (w[1] / w[0]).ln();
        ssq += r * r;
    }
    let minutes = (closes.len() as f64 - 1.0) * 5.0;
    Some((ssq * min_per_year(class) / minutes).sqrt())
}

// ---------- vol anchors ----------

fn cmd_vol(data: &Path) -> Result<()> {
    let dir = data.join("vol");
    fs::create_dir_all(&dir)?;
    let client = mk_client();
    for (name, url) in [
        ("ovx.csv", "https://cdn.cboe.com/api/global/us_indices/daily_prices/OVX_History.csv"),
        ("vix.csv", "https://cdn.cboe.com/api/global/us_indices/daily_prices/VIX_History.csv"),
        ("gvz.csv", "https://cdn.cboe.com/api/global/us_indices/daily_prices/GVZ_History.csv"),
        ("vxslv.csv", "https://cdn.cboe.com/api/global/us_indices/daily_prices/VXSLV_History.csv"),
    ] {
        let txt = get_text(&client, url)?;
        fs::write(dir.join(name), &txt)?;
        println!("vol: {name} {} lines", txt.lines().count());
    }
    let now_ms = Utc::now().timestamp_millis();
    for cur in ["BTC", "ETH"] {
        let url = format!(
            "https://www.deribit.com/api/v2/public/get_volatility_index_data?currency={cur}&start_timestamp={}&end_timestamp={now_ms}&resolution=1D",
            now_ms - 120 * 86400 * 1000
        );
        let txt = get_text(&client, &url)?;
        fs::write(dir.join(format!("dvol_{}.json", cur.to_lowercase())), &txt)?;
        println!("vol: dvol_{cur} saved");
    }
    Ok(())
}

/// asset -> BTreeMap<date, annualized IV fraction>
fn load_iv(data: &Path) -> HashMap<&'static str, BTreeMap<NaiveDate, f64>> {
    let dir = data.join("vol");
    let mut out: HashMap<&'static str, BTreeMap<NaiveDate, f64>> = HashMap::new();
    for (name, tag) in [
        ("ovx.csv", "wti"),
        ("vix.csv", "spy"),
        ("gvz.csv", "gold"),
        ("vxslv.csv", "silver"),
    ] {
        let mut m = BTreeMap::new();
        if let Ok(txt) = fs::read_to_string(dir.join(name)) {
            for line in txt.lines().skip(1) {
                let f: Vec<&str> = line.split(',').collect();
                if f.len() < 2 {
                    continue;
                }
                let Ok(d) = NaiveDate::parse_from_str(f[0], "%m/%d/%Y") else { continue };
                if let Ok(v) = f[f.len() - 1].parse::<f64>() {
                    m.insert(d, v / 100.0);
                }
            }
        }
        out.insert(tag, m);
    }
    for (fname, tag) in [("dvol_btc.json", "btc"), ("dvol_eth.json", "eth")] {
        let mut m = BTreeMap::new();
        if let Ok(txt) = fs::read_to_string(dir.join(fname)) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                for row in v["result"]["data"].as_array().unwrap_or(&vec![]) {
                    let r = row.as_array().unwrap();
                    let d = date_of(r[0].as_i64().unwrap_or(0) / 1000);
                    if let Some(close) = r[4].as_f64() {
                        m.insert(d, close / 100.0);
                    }
                }
            }
        }
        out.insert(tag, m);
    }
    out
}

fn iv_at(iv: &HashMap<&'static str, BTreeMap<NaiveDate, f64>>, asset: Asset, t: i64) -> Option<f64> {
    let m = iv.get(asset.name())?;
    m.range(..=date_of(t)).next_back().map(|(_, v)| *v)
}

// ---------- clob prices ----------

fn cmd_clob(data: &Path, fidelity: u32, filter: Option<Vec<String>>) -> Result<()> {
    let legs = load_legs(data)?;
    let mut jobs = vec![];
    for l in &legs {
        if let Some(f) = &filter {
            if !f.contains(&l.board) {
                continue;
            }
        }
        let start = l.ws - 3 * 86400;
        let url = format!(
            "https://clob.polymarket.com/prices-history?market={}&startTs={start}&fidelity={fidelity}",
            l.token_yes
        );
        jobs.push((
            url,
            data.join(format!("clob{fidelity}")).join(&l.board).join(format!("{}.json", l.condition_id)),
        ));
    }
    let (fetched, skipped, failed) = fetch_all(jobs, 6);
    println!("clob f{fidelity}: fetched {fetched}, cached {skipped}, failed {failed}");
    Ok(())
}

fn load_series(data: &Path, fidelity: u32, leg: &Leg) -> Option<Vec<(i64, f64)>> {
    let p = data
        .join(format!("clob{fidelity}"))
        .join(&leg.board)
        .join(format!("{}.json", leg.condition_id));
    let txt = fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&txt).ok()?;
    let h = v["history"].as_array()?;
    let mut out = vec![];
    for pt in h {
        out.push((pt["t"].as_i64()?, pt["p"].as_f64()?));
    }
    if out.is_empty() { None } else { Some(out) }
}

fn price_at(series: &[(i64, f64)], t: i64, max_age_s: i64) -> Option<f64> {
    let idx = series.partition_point(|(ts, _)| *ts <= t);
    series[..idx].last().filter(|(ts, _)| t - ts <= max_age_s).map(|(_, p)| *p)
}

// ---------- touch computation (gate 0 core) ----------

/// First in-session candle in [ws, we) that touches the barrier.
fn first_touch(db: &CandleDb, cal: &SessionCal, key: &str, leg: &Leg) -> Option<i64> {
    for c in db.slice(key, leg.ws, leg.we) {
        if leg.asset.class() != Class::Crypto && !cal.is_open(c.t) {
            continue;
        }
        let hit = match leg.dir {
            'H' => c.h >= leg.barrier,
            _ => c.l <= leg.barrier,
        };
        if hit {
            return Some(c.t);
        }
    }
    None
}

/// Extreme (max high / min low) of in-session candles over the window.
fn window_extreme(db: &CandleDb, cal: &SessionCal, key: &str, leg: &Leg) -> Option<f64> {
    let mut ext: Option<f64> = None;
    for c in db.slice(key, leg.ws, leg.we) {
        if leg.asset.class() != Class::Crypto && !cal.is_open(c.t) {
            continue;
        }
        ext = Some(match (leg.dir, ext) {
            ('H', None) => c.h,
            ('H', Some(e)) => e.max(c.h),
            (_, None) => c.l,
            (_, Some(e)) => e.min(c.l),
        });
    }
    ext
}

// ---------- analyze (gates 0-3) ----------

fn cmd_analyze(data: &Path) -> Result<()> {
    let legs = load_legs(data)?;
    let out_dir = data.join("out");
    fs::create_dir_all(&out_dir)?;
    let cals: BTreeMap<Class, SessionCal> = [Class::Crypto, Class::Equity, Class::Wti]
        .into_iter()
        .map(|c| (c, SessionCal::build(c)))
        .collect();
    println!("loading candles...");
    let db = CandleDb::load(
        data,
        &["BTCUSDT", "ETHUSDT", "USOILSPOT", "SPY", "NVDA", "WTIU6", "WTIV6", "XAUUSD", "XAGUSD"],
    );
    let iv = load_iv(data);

    // ---- Gate 0: resolution reproduction ----
    let mut g0_rows =
        vec!["board,market_slug,dir,barrier,ws,winner,touch,first_touch,extreme,margin,match,proxy".to_string()];
    let mut per_board: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut mismatches = vec![];
    let mut mismatch_ids: Vec<String> = vec![];
    let mut touch_of: HashMap<String, Option<i64>> = HashMap::new(); // condition_id -> first touch
    // Boards where every leg is closed: only these enter gates 1-3. A live board's
    // closed legs are the touched ones only -> survivorship poison.
    let fully_closed: Vec<String> = {
        let mut m: BTreeMap<String, bool> = BTreeMap::new();
        for l in &legs {
            *m.entry(l.board.clone()).or_insert(true) &= l.closed;
        }
        m.into_iter().filter(|(_, v)| *v).map(|(k, _)| k).collect()
    };
    for l in &legs {
        let cal = &cals[&l.asset.class()];
        let key = l.asset.candle_key();
        let ft = first_touch(&db, cal, key, l);
        touch_of.insert(l.condition_id.clone(), ft);
        if !l.closed {
            continue;
        }
        let ext = window_extreme(&db, cal, key, l);
        let touch = ft.is_some();
        let ok = (l.winner == 1) == touch;
        let margin = ext.map(|e| if l.dir == 'H' { e - l.barrier } else { l.barrier - e });
        let e = per_board.entry(l.board.clone()).or_default();
        e.1 += 1;
        if ok {
            e.0 += 1;
        } else {
            mismatch_ids.push(l.condition_id.clone());
            mismatches.push(format!(
                "  MISMATCH {} {} {}{} winner={} touch={} extreme={:?} margin={:?} ft={:?}",
                l.board, l.market_slug, l.dir, l.barrier, l.winner, touch, ext, margin, ft
            ));
        }
        g0_rows.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            l.board,
            l.market_slug,
            l.dir,
            l.barrier,
            l.ws,
            l.winner,
            touch as u8,
            ft.unwrap_or(0),
            ext.map(|e| format!("{e:.4}")).unwrap_or_default(),
            margin.map(|m| format!("{m:.4}")).unwrap_or_default(),
            ok as u8,
            (l.asset == Asset::Wti) as u8
        ));
    }
    fs::write(out_dir.join("gate0.csv"), g0_rows.join("\n") + "\n")?;
    println!("\n== Gate 0: candle reproduction of resolved outcomes ==");
    let (mut tot_ok, mut tot_n) = (0, 0);
    for (b, (ok, n)) in &per_board {
        tot_ok += ok;
        tot_n += n;
        println!("  {b}: {ok}/{n}");
    }
    println!("  TOTAL: {tot_ok}/{tot_n} ({:.2}%)", 100.0 * tot_ok as f64 / tot_n.max(1) as f64);
    for m in &mismatches {
        println!("{m}");
    }

    // ---- WTI proxy error: USOILSPOT vs WTIU6 on common in-session minutes ----
    {
        let cal = &cals[&Class::Wti];
        let spot = db.by_key.get("USOILSPOT").cloned().unwrap_or_default();
        let fut: HashMap<i64, Candle> =
            db.by_key.get("WTIU6").cloned().unwrap_or_default().into_iter().map(|c| (c.t, c)).collect();
        let mut dh = vec![];
        let mut dl = vec![];
        let u6_active = ts(NaiveDate::from_ymd_opt(2026, 7, 16).unwrap(), 22, 0);
        let mut dh_act = vec![];
        for c in &spot {
            if !cal.is_open(c.t) {
                continue;
            }
            if let Some(f) = fut.get(&c.t) {
                dh.push((c.h - f.h).abs());
                dl.push((c.l - f.l).abs());
                if c.t >= u6_active {
                    dh_act.push((c.h - f.h).abs());
                }
            }
        }
        if !dh.is_empty() {
            println!("\n== WTI proxy error (USOILSPOT vs WTIU6, {} common minutes) ==", dh.len());
            println!(
                "  all overlap:  |Δhigh| p50 {:.3} p95 {:.3} max {:.3} | |Δlow| p50 {:.3} p95 {:.3} max {:.3}",
                quantile(&mut dh.clone(), 0.5),
                quantile(&mut dh.clone(), 0.95),
                dh.iter().cloned().fold(0.0, f64::max),
                quantile(&mut dl.clone(), 0.5),
                quantile(&mut dl.clone(), 0.95),
                dl.iter().cloned().fold(0.0, f64::max),
            );
            println!(
                "  U6-active only (>=Jul16 22Z, n={}): |Δhigh| p50 {:.3} p95 {:.3} max {:.3}",
                dh_act.len(),
                quantile(&mut dh_act.clone(), 0.5),
                quantile(&mut dh_act.clone(), 0.95),
                dh_act.iter().cloned().fold(0.0, f64::max),
            );
        }
    }

    // ---- vol anchors vs realized ----
    println!("\n== vol anchors (annualized) ==");
    for (asset, key) in [
        (Asset::Btc, "BTCUSDT"),
        (Asset::Eth, "ETHUSDT"),
        (Asset::Wti, "USOILSPOT"),
        (Asset::Spy, "SPY"),
        (Asset::Nvda, "NVDA"),
    ] {
        let cal = &cals[&asset.class()];
        let t = Utc::now().timestamp();
        let rv = realized_vol(&db, key, cal, asset.class(), t, 14 * 86400);
        let ivv = iv_at(&iv, asset, t);
        println!(
            "  {}: RV14d {} | IV {}",
            asset.name(),
            rv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            ivv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into())
        );
    }

    // ---- Gate 1: window-open calibration ----
    let series60: HashMap<String, Vec<(i64, f64)>> = legs
        .iter()
        .filter_map(|l| load_series(data, 60, l).map(|s| (l.condition_id.clone(), s)))
        .collect();
    println!("  clob60 series loaded: {}/{}", series60.len(), legs.len());
    let mut g1_rows = vec!["board,market_slug,dir,barrier,open_mid,q_rv_open,winner".to_string()];
    let mut bins: BTreeMap<usize, (usize, f64, f64)> = BTreeMap::new(); // bin -> (n, sum_mid, sum_win)
    let bin_edges = [0.0, 0.02, 0.05, 0.10, 0.20, 0.35, 0.50, 0.65, 0.80, 0.95, 1.01];
    let (mut brier_mkt_open, mut brier_rv_open) = (vec![], vec![]);
    for l in legs.iter().filter(|l| l.closed && fully_closed.contains(&l.board)) {
        let Some(s) = series60.get(&l.condition_id) else { continue };
        let Some(m0) = price_at(s, l.ws + 3 * 3600, 76 * 3600) else { continue };
        let cal = &cals[&l.asset.class()];
        let key = l.asset.candle_key();
        let q0 = db
            .spot_at(key, cal, l.ws, 4 * 86400)
            .and_then(|s0| {
                realized_vol(&db, key, cal, l.asset.class(), l.ws, 14 * 86400).map(|rv| {
                    let tau = cal.count(l.ws, l.we) as f64 / min_per_year(l.asset.class());
                    touch_prob(s0, l.barrier, l.dir, rv, tau)
                })
            });
        let win = (l.winner == 1) as u8 as f64;
        let bi = bin_edges.iter().position(|e| m0 < *e).unwrap_or(10).saturating_sub(1);
        let b = bins.entry(bi).or_default();
        b.0 += 1;
        b.1 += m0;
        b.2 += win;
        brier_mkt_open.push((m0 - win) * (m0 - win));
        if let Some(q) = q0 {
            brier_rv_open.push((q - win) * (q - win));
        }
        g1_rows.push(format!(
            "{},{},{},{},{:.4},{},{}",
            l.board,
            l.market_slug,
            l.dir,
            l.barrier,
            m0,
            q0.map(|q| format!("{q:.4}")).unwrap_or_default(),
            l.winner
        ));
    }
    fs::write(out_dir.join("gate1_open.csv"), g1_rows.join("\n") + "\n")?;
    println!("\n== Gate 1: window-open calibration (closed legs with open mid) ==");
    println!("  {:>12} {:>5} {:>9} {:>9}", "bin", "n", "avg_mid", "hit_rate");
    for (bi, (n, sm, sw)) in &bins {
        println!(
            "  {:>12} {:>5} {:>9.3} {:>9.3}",
            format!("{:.0}-{:.0}c", bin_edges[*bi] * 100.0, bin_edges[bi + 1] * 100.0),
            n,
            sm / *n as f64,
            sw / *n as f64
        );
    }
    println!(
        "  Brier at open: market {:.4} (n={}) | RV model {:.4} (n={})",
        mean(&brier_mkt_open),
        brier_mkt_open.len(),
        mean(&brier_rv_open),
        brier_rv_open.len()
    );

    // ---- Gate 2: daily-checkpoint sim, instant + t+24h delayed ----
    let mut cp_rows = vec![
        "board,asset,tier,market_slug,dir,barrier,t,spot,rv,iv,tau_yrs,q_rv,q_iv,mid,mid24,zone_excl,winner"
            .to_string(),
    ];
    #[derive(Clone)]
    struct Trade {
        board: String,
        asset: Asset,
        tier: String,
        market_slug: String,
        t: i64,
        side: i8, // +1 buy YES, -1 sell YES
        fill: f64,
        q: f64,
        won: f64,
        net: f64,
        delayed: bool,
        iv_model: bool,
    }
    let mut trades: Vec<Trade> = vec![];
    let cost = 0.015;
    let need_edge = 0.04;
    for l in legs.iter().filter(|l| {
        l.closed && fully_closed.contains(&l.board) && !mismatch_ids.contains(&l.condition_id)
    }) {
        let Some(s60) = series60.get(&l.condition_id) else { continue };
        let cal = &cals[&l.asset.class()];
        let key = l.asset.candle_key();
        let ft = touch_of.get(&l.condition_id).cloned().flatten();
        let won = (l.winner == 1) as u8 as f64;
        // daily checkpoints at 12:00Z inside [ws, we)
        let mut d = date_of(l.ws);
        loop {
            let t = ts(d, 12, 0);
            d += Duration::days(1);
            if t < l.ws {
                continue;
            }
            if t >= l.we {
                break;
            }
            if let Some(ft) = ft {
                if ft <= t {
                    break; // leg already resolved YES by candle evidence
                }
            }
            let Some(mid) = price_at(s60, t, 5400) else { continue };
            let Some(spot) = db.spot_at(key, cal, t, 4 * 86400) else { continue };
            let Some(rv) = realized_vol(&db, key, cal, l.asset.class(), t, 14 * 86400) else { continue };
            let ivv = iv_at(&iv, l.asset, t);
            let tau = cal.count(t, l.we) as f64 / min_per_year(l.asset.class());
            let q_rv = touch_prob(spot, l.barrier, l.dir, rv, tau);
            let q_iv = ivv.map(|v| touch_prob(spot, l.barrier, l.dir, v, tau));
            let tau1 = day_minutes(l.asset.class()) / min_per_year(l.asset.class());
            let zone = (l.barrier / spot).ln().abs() < rv * tau1.sqrt();
            let mid24 = price_at(s60, t + 86400, 5400).filter(|_| t + 86400 < l.we);
            cp_rows.push(format!(
                "{},{},{},{},{},{},{},{:.4},{:.4},{},{:.6},{:.4},{},{:.4},{},{},{}",
                l.board,
                l.asset.name(),
                l.tier,
                l.market_slug,
                l.dir,
                l.barrier,
                t,
                spot,
                rv,
                ivv.map(|v| format!("{v:.4}")).unwrap_or_default(),
                tau,
                q_rv,
                q_iv.map(|v| format!("{v:.4}")).unwrap_or_default(),
                mid,
                mid24.map(|v| format!("{v:.4}")).unwrap_or_default(),
                zone as u8,
                l.winner
            ));
            if zone {
                continue;
            }
            // model variants: RV primary, IV secondary
            for (q, iv_model) in [(Some(q_rv), false), (q_iv, true)] {
                let Some(q) = q else { continue };
                for (fill_mid, delayed) in [(Some(mid), false), (mid24, true)] {
                    let Some(m) = fill_mid else { continue };
                    if !(0.03..=0.50).contains(&m) {
                        continue;
                    }
                    let (side, fill, net) = if q > m + need_edge {
                        (1i8, m + cost, won - (m + cost))
                    } else if q < m - need_edge {
                        (-1i8, m - cost, (m - cost) - won)
                    } else {
                        continue;
                    };
                    trades.push(Trade {
                        board: l.board.clone(),
                        asset: l.asset,
                        tier: l.tier.clone(),
                        market_slug: l.market_slug.clone(),
                        t,
                        side,
                        fill,
                        q,
                        won,
                        net,
                        delayed,
                        iv_model,
                    });
                }
            }
        }
    }
    fs::write(out_dir.join("gate2_checkpoints.csv"), cp_rows.join("\n") + "\n")?;
    let mut tr_rows =
        vec!["board,asset,tier,market_slug,t,side,fill,q,won,net,delayed,iv_model".to_string()];
    for t in &trades {
        tr_rows.push(format!(
            "{},{},{},{},{},{},{:.4},{:.4},{},{:.4},{},{}",
            t.board,
            t.asset.name(),
            t.tier,
            t.market_slug,
            t.t,
            t.side,
            t.fill,
            t.q,
            t.won,
            t.net,
            t.delayed as u8,
            t.iv_model as u8
        ));
    }
    fs::write(out_dir.join("gate2_trades.csv"), tr_rows.join("\n") + "\n")?;

    let july_split = ts(NaiveDate::from_ymd_opt(2026, 7, 3).unwrap(), 0, 0);
    let summarize = |name: &str, sel: &dyn Fn(&Trade) -> bool| {
        for (label, delayed) in [("instant", false), ("t+24h ", true)] {
            let nets: Vec<f64> =
                trades.iter().filter(|t| t.delayed == delayed && sel(t)).map(|t| t.net).collect();
            if nets.is_empty() {
                println!("  {name} {label}: no trades");
                continue;
            }
            let h1: Vec<f64> = trades
                .iter()
                .filter(|t| t.delayed == delayed && sel(t) && t.t < july_split)
                .map(|t| t.net)
                .collect();
            let h2: Vec<f64> = trades
                .iter()
                .filter(|t| t.delayed == delayed && sel(t) && t.t >= july_split)
                .map(|t| t.net)
                .collect();
            println!(
                "  {name} {label}: n={:4} avg {:+.4} (se {:.4}) | pre-Jul3 n={} avg {} | post n={} avg {}",
                nets.len(),
                mean(&nets),
                sd(&nets) / (nets.len() as f64).sqrt(),
                h1.len(),
                if h1.is_empty() { "-".into() } else { format!("{:+.4}", mean(&h1)) },
                h2.len(),
                if h2.is_empty() { "-".into() } else { format!("{:+.4}", mean(&h2)) },
            );
        }
    };
    println!("\n== Gate 2: daily 12:00Z checkpoint sim (cost {cost}, edge>{need_edge}, mid 3-50c, zone-excluded) ==");
    println!(" RV model (primary):");
    summarize("ALL       ", &|t: &Trade| !t.iv_model);
    summarize("buys      ", &|t: &Trade| !t.iv_model && t.side == 1);
    summarize("sells     ", &|t: &Trade| !t.iv_model && t.side == -1);
    summarize("crypto    ", &|t: &Trade| !t.iv_model && t.asset.class() == Class::Crypto);
    summarize("wti       ", &|t: &Trade| !t.iv_model && t.asset == Asset::Wti);
    summarize("equity    ", &|t: &Trade| !t.iv_model && t.asset.class() == Class::Equity);
    summarize("weekly    ", &|t: &Trade| !t.iv_model && t.tier == "weekly");
    summarize("monthly   ", &|t: &Trade| !t.iv_model && t.tier == "monthly");
    println!(" RV sells by class:");
    summarize("sell/wti  ", &|t: &Trade| !t.iv_model && t.side == -1 && t.asset == Asset::Wti);
    summarize("sell/cryp ", &|t: &Trade| {
        !t.iv_model && t.side == -1 && t.asset.class() == Class::Crypto
    });
    summarize("sell/equi ", &|t: &Trade| {
        !t.iv_model && t.side == -1 && t.asset.class() == Class::Equity
    });
    println!(" RV buys by class:");
    summarize("buy/wti   ", &|t: &Trade| !t.iv_model && t.side == 1 && t.asset == Asset::Wti);
    summarize("buy/cryp  ", &|t: &Trade| {
        !t.iv_model && t.side == 1 && t.asset.class() == Class::Crypto
    });
    summarize("buy/equi  ", &|t: &Trade| {
        !t.iv_model && t.side == 1 && t.asset.class() == Class::Equity
    });
    println!(" IV model (secondary, where anchor exists):");
    summarize("ALL       ", &|t: &Trade| t.iv_model);
    summarize("buys      ", &|t: &Trade| t.iv_model && t.side == 1);
    summarize("sells     ", &|t: &Trade| t.iv_model && t.side == -1);

    // ---- Gate 3: jump-premium attribution on delayed RV sells ----
    let sells: Vec<&Trade> =
        trades.iter().filter(|t| t.delayed && !t.iv_model && t.side == -1).collect();
    let prem: f64 = sells.iter().filter(|t| t.won == 0.0).map(|t| t.net).sum();
    let losses: f64 = -sells.iter().filter(|t| t.won == 1.0).map(|t| t.net).sum::<f64>();
    println!("\n== Gate 3: delayed-sim SELL attribution ==");
    println!(
        "  sells n={} | premium collected (no-touch wins) {:.2} | losses to touches {:.2} | ratio {:.2}",
        sells.len(),
        prem,
        losses,
        losses / prem.max(1e-9)
    );

    // ---- Brier/log-loss, market vs model, all checkpoints ----
    {
        let mut bm = vec![];
        let mut bq = vec![];
        let mut bqiv = vec![];
        for line in cp_rows.iter().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            let (mid, q_rv, q_iv, win): (f64, f64, Option<f64>, f64) = (
                f[13].parse().unwrap_or(f64::NAN),
                f[11].parse().unwrap_or(f64::NAN),
                f[12].parse().ok(),
                f[16].parse().unwrap_or(f64::NAN),
            );
            bm.push((mid - win) * (mid - win));
            bq.push((q_rv - win) * (q_rv - win));
            if let Some(q) = q_iv {
                bqiv.push((q - win) * (q - win));
            }
        }
        println!("\n== checkpoint Brier (all closed legs x daily checkpoints) ==");
        println!(
            "  market {:.4} | RV model {:.4} (n={}) | IV model {:.4} (n={})",
            mean(&bm),
            mean(&bq),
            bq.len(),
            mean(&bqiv),
            bqiv.len()
        );
    }

    // ---- Gate 1b: monotonicity-violation lifetimes (weekly boards, clob10) ----
    let mut life_all: Vec<f64> = vec![];
    let mut viol_rows = vec!["board,dir,strike_deep,strike_shallow,start,minutes".to_string()];
    let weekly_boards: Vec<String> = {
        let mut b: Vec<String> = legs
            .iter()
            .filter(|l| l.tier == "weekly" && l.closed)
            .map(|l| l.board.clone())
            .collect();
        b.sort();
        b.dedup();
        b
    };
    for board in &weekly_boards {
        let bl: Vec<&Leg> = legs.iter().filter(|l| &l.board == board).collect();
        for dir in ['H', 'L'] {
            // adjacent pairs among same-window-start legs
            let mut group: Vec<&&Leg> = bl.iter().filter(|l| l.dir == dir && !l.creation_clause).collect();
            group.sort_by(|a, b| a.barrier.partial_cmp(&b.barrier).unwrap());
            if dir == 'L' {
                group.reverse(); // deeper = lower strike for LOW
            }
            for w in group.windows(2) {
                let (shallow, deep) = (w[0], w[1]);
                let (Some(ss), Some(sd)) = (
                    load_series(data, 10, shallow),
                    load_series(data, 10, deep),
                ) else {
                    continue;
                };
                // 10-min grid over the touch window
                let mut run_start: Option<i64> = None;
                let mut t = shallow.ws;
                while t < shallow.we {
                    let (ps, pd) = (price_at(&ss, t, 1800), price_at(&sd, t, 1800));
                    let viol = match (ps, pd) {
                        (Some(ps), Some(pd)) => pd > ps + 0.01,
                        _ => false,
                    };
                    match (viol, run_start) {
                        (true, None) => run_start = Some(t),
                        (false, Some(s)) => {
                            let mins = (t - s) as f64 / 60.0;
                            life_all.push(mins);
                            viol_rows.push(format!(
                                "{board},{dir},{},{},{s},{mins:.0}",
                                deep.barrier, shallow.barrier
                            ));
                            run_start = None;
                        }
                        _ => {}
                    }
                    t += 600;
                }
                if let Some(s) = run_start {
                    let mins = (shallow.we - s) as f64 / 60.0;
                    life_all.push(mins);
                    viol_rows.push(format!(
                        "{board},{dir},{},{},{s},{mins:.0}",
                        deep.barrier, shallow.barrier
                    ));
                }
            }
        }
    }
    fs::write(out_dir.join("gate1b_violations.csv"), viol_rows.join("\n") + "\n")?;
    if !life_all.is_empty() {
        println!(
            "\n== Gate 1b: >=1c monotonicity-violation lifetimes on {} weekly boards ==",
            weekly_boards.len()
        );
        println!(
            "  n={} p50 {:.0} min p95 {:.0} min max {:.0} min",
            life_all.len(),
            quantile(&mut life_all.clone(), 0.5),
            quantile(&mut life_all.clone(), 0.95),
            life_all.iter().cloned().fold(0.0, f64::max)
        );
    } else {
        println!("\n== Gate 1b: no clob10 data or no violations found ==");
    }

    println!("\nCSV outputs in {}", out_dir.display());
    Ok(())
}

// ---------- tape + wash (gate 4, copied from runningmax) ----------

fn cmd_tape(data: &Path, boards: &[String]) -> Result<()> {
    let legs = load_legs(data)?;
    let client = mk_client();
    for board in boards {
        for l in legs.iter().filter(|l| &l.board == board) {
            let out = data.join("tape").join(board).join(format!("{}.json", l.condition_id));
            if out.exists() {
                continue;
            }
            let mut trades = vec![];
            let mut offset = 0;
            loop {
                let url = format!(
                    "https://data-api.polymarket.com/trades?market={}&limit=500&offset={offset}",
                    l.condition_id
                );
                let v: serde_json::Value = serde_json::from_str(&get_text(&client, &url)?)?;
                let arr = v.as_array().cloned().unwrap_or_default();
                let n = arr.len();
                trades.extend(arr);
                if n < 500 || offset > 20000 {
                    break;
                }
                offset += 500;
            }
            fs::create_dir_all(out.parent().unwrap())?;
            fs::write(&out, serde_json::to_string(&trades)?)?;
            println!("tape {board} {}{}: {} trades", l.dir, l.barrier, trades.len());
        }
    }
    Ok(())
}

fn cmd_wash(data: &Path, boards: &[String]) -> Result<()> {
    let legs = load_legs(data)?;
    for board in boards {
        let mut notional = 0.0;
        let mut shares = 0.0;
        let mut n_trades = 0usize;
        let mut by_wallet: HashMap<String, f64> = HashMap::new();
        let mut wash_notional = 0.0;
        let mut headline = 0.0;
        for l in legs.iter().filter(|l| &l.board == board) {
            headline += l.volume;
            let p = data.join("tape").join(board).join(format!("{}.json", l.condition_id));
            let Ok(txt) = fs::read_to_string(p) else { continue };
            let Ok(tape) = serde_json::from_str::<Vec<serde_json::Value>>(&txt) else { continue };
            let mut per_wallet: HashMap<String, Vec<(i64, String, f64, f64)>> = HashMap::new();
            for t in &tape {
                let px = t["price"].as_f64().unwrap_or(0.0);
                let sz = t["size"].as_f64().unwrap_or(0.0);
                let w = t["proxyWallet"].as_str().unwrap_or("").to_string();
                let side = t["side"].as_str().unwrap_or("").to_string();
                let tss = t["timestamp"].as_i64().unwrap_or(0);
                notional += px * sz;
                shares += sz;
                n_trades += 1;
                *by_wallet.entry(w.clone()).or_default() += px * sz;
                per_wallet.entry(w).or_default().push((tss, side, px, sz));
            }
            for (_, mut tsv) in per_wallet {
                tsv.sort_by_key(|r| r.0);
                for i in 0..tsv.len() {
                    for j in i + 1..tsv.len() {
                        if tsv[j].0 - tsv[i].0 > 600 {
                            break;
                        }
                        if tsv[i].1 != tsv[j].1 {
                            let ratio = tsv[i].3 / tsv[j].3;
                            if ratio > 0.8 && ratio < 1.25 {
                                wash_notional += tsv[i].2 * tsv[i].3 + tsv[j].2 * tsv[j].3;
                            }
                        }
                    }
                }
            }
        }
        let mut ws: Vec<f64> = by_wallet.values().cloned().collect();
        ws.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let top10: f64 = ws.iter().take(10).sum();
        println!("== Gate 4 wash check: {board} ==");
        println!(
            "  tape: {n_trades} trades, {shares:.0} shares, ${notional:.0} taker notional | headline volumeNum sum ${headline:.0} (ratio {:.2})",
            headline / notional.max(1.0)
        );
        println!(
            "  wallets: {} distinct; top-10 share {:.1}%; top-1 {:.1}%",
            ws.len(),
            100.0 * top10 / notional.max(1.0),
            100.0 * ws.first().unwrap_or(&0.0) / notional.max(1.0)
        );
        println!(
            "  same-wallet paired buy+sell (<=10min, size within 25%): ${wash_notional:.0} = {:.1}% of notional",
            100.0 * wash_notional / notional.max(1.0)
        );
    }
    Ok(())
}

// ---------- live ----------

fn cmd_live(data: &Path, slugs: &[String]) -> Result<()> {
    let client = mk_client();
    let now = Utc::now().timestamp();
    let cals: BTreeMap<Class, SessionCal> = [Class::Crypto, Class::Equity, Class::Wti]
        .into_iter()
        .map(|c| (c, SessionCal::build(c)))
        .collect();
    let db = CandleDb::load(
        data,
        &["BTCUSDT", "ETHUSDT", "USOILSPOT", "SPY", "NVDA", "WTIU6", "WTIV6", "XAUUSD", "XAGUSD"],
    );
    let iv = load_iv(data);
    let mut pred_rows = vec![
        "market_slug,condition_id,outcome,token_id,probability,market_midpoint,bid,ask,dir,barrier,note"
            .to_string(),
    ];
    println!("live @ {} UTC", Utc::now().format("%Y-%m-%d %H:%M"));
    for slug in slugs {
        let ev_txt = get_text(
            &client,
            &format!("https://gamma-api.polymarket.com/events?slug={slug}"),
        )?;
        let live_dir = data.join("events_live");
        fs::create_dir_all(&live_dir)?;
        fs::write(live_dir.join(format!("{slug}.json")), &ev_txt)?;
        let v: serde_json::Value = serde_json::from_str(&ev_txt)?;
        let Some(ev) = v.as_array().and_then(|a| a.first()) else {
            println!("  {slug}: NOT FOUND");
            continue;
        };
        let legs = extract_legs(slug, ev)?;
        let Some(l0) = legs.first() else { continue };
        let asset = l0.asset;
        let class = asset.class();
        let cal = &cals[&class];
        let key = asset.live_key();
        let spot = db.spot_at(key, cal, now, 4 * 86400);
        let rv = realized_vol(&db, key, cal, class, now, 14 * 86400);
        let ivv = iv_at(&iv, asset, now);
        println!(
            "\n== {slug} | spot[{key}] {} | RV14d {} | IV {} | window ends {} ==",
            spot.map(|s| format!("{s:.2}")).unwrap_or("?".into()),
            rv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            ivv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            DateTime::from_timestamp(l0.we, 0).unwrap().format("%m-%d %H:%MZ")
        );
        println!(
            "  {:>3} {:>9} {:>7} {:>7} {:>7} {:>9} {:>7} {:>7} {:>7}",
            "dir", "barrier", "bid", "ask", "mid", "tob$", "q_rv", "q_iv", "edge"
        );
        for l in &legs {
            if l.closed {
                continue;
            }
            let book_txt = get_text(
                &client,
                &format!("https://clob.polymarket.com/book?token_id={}", l.token_yes),
            )
            .unwrap_or_default();
            let b: serde_json::Value = serde_json::from_str(&book_txt).unwrap_or_default();
            let best = |side: &str| -> Option<(f64, f64)> {
                let arr = b[side].as_array()?;
                let last = arr.last()?;
                Some((
                    last["price"].as_str()?.parse().ok()?,
                    last["size"].as_str()?.parse().ok()?,
                ))
            };
            let bid = best("bids");
            let ask = best("asks");
            let mid = match (bid, ask) {
                (Some((bb, _)), Some((aa, _))) => Some(0.5 * (bb + aa)),
                _ => None,
            };
            let tob =
                bid.map(|(p, s)| p * s).unwrap_or(0.0) + ask.map(|(p, s)| p * s).unwrap_or(0.0);
            let tau_t0 = now.max(l.ws);
            let tau = cal.count(tau_t0, l.we) as f64 / min_per_year(class);
            let q_rv = match (spot, rv) {
                (Some(s), Some(v)) => Some(touch_prob(s, l.barrier, l.dir, v, tau)),
                _ => None,
            };
            let q_iv = match (spot, ivv) {
                (Some(s), Some(v)) => Some(touch_prob(s, l.barrier, l.dir, v, tau)),
                _ => None,
            };
            println!(
                "  {:>3} {:>9} {:>7} {:>7} {:>7} {:>9.0} {:>7} {:>7} {:>7}",
                l.dir,
                l.barrier,
                bid.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                ask.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                mid.map(|p| format!("{p:.3}")).unwrap_or_default(),
                tob,
                q_rv.map(|p| format!("{p:.3}")).unwrap_or_default(),
                q_iv.map(|p| format!("{p:.3}")).unwrap_or_default(),
                match (q_rv, mid) {
                    (Some(q), Some(m)) => format!("{:+.3}", q - m),
                    _ => String::new(),
                }
            );
            if let (Some(q), Some(m)) = (q_rv, mid) {
                pred_rows.push(format!(
                    "{},{},Yes,{},{:.4},{:.4},{},{},{},{},spot={:.2};rv={:.3};tau={:.5};ws={}",
                    l.market_slug,
                    l.condition_id,
                    l.token_yes,
                    q,
                    m,
                    bid.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                    ask.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                    l.dir,
                    l.barrier,
                    spot.unwrap_or(f64::NAN),
                    rv.unwrap_or(f64::NAN),
                    tau,
                    l.ws
                ));
            }
        }
    }
    let date = Utc::now().date_naive();
    let pred_path = data.join("out").join(format!("predictions_{date}.csv"));
    fs::create_dir_all(pred_path.parent().unwrap())?;
    fs::write(&pred_path, pred_rows.join("\n") + "\n")?;
    println!("\nprediction rows -> {}", pred_path.display());
    Ok(())
}

// ---------- main ----------

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: ladderrv <discover|candles|vol|clob|analyze|tape|wash|live> <data_dir> ...";
    if args.len() < 3 {
        bail!("{usage}");
    }
    let data = PathBuf::from(&args[2]);
    fs::create_dir_all(&data)?;
    let date = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").context("date YYYY-MM-DD");
    match args[1].as_str() {
        "discover" => {
            let slugs: Vec<String> = args[3].split(',').map(String::from).collect();
            cmd_discover(&data, &slugs)
        }
        "candles" => cmd_candles(&data, &args[3], date(&args[4])?, date(&args[5])?),
        "vol" => cmd_vol(&data),
        "clob" => {
            let fid: u32 = args[3].parse()?;
            let filter = args.get(4).map(|s| s.split(',').map(String::from).collect());
            cmd_clob(&data, fid, filter)
        }
        "analyze" => cmd_analyze(&data),
        "tape" => cmd_tape(&data, &args[3..].to_vec()),
        "wash" => cmd_wash(&data, &args[3..].to_vec()),
        "live" => {
            let slugs: Vec<String> = args[3].split(',').map(String::from).collect();
            cmd_live(&data, &slugs)
        }
        _ => bail!("{usage}"),
    }
}
