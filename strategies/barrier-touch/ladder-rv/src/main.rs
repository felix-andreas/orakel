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
    // Full market closures inside the session calendar's span. August 2026 has none;
    // Sep 7 is Labor Day. Columbus Day (Oct 12) is NOT here — NYSE and CME energy both
    // trade that day; only the bond market closes.
    d.year() == 2026
        && matches!((d.month(), d.day()), (5, 25) | (6, 19) | (7, 3) | (9, 7))
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
        // Must extend past the far end of every board we price. It used to stop at
        // 2026-08-20 (CLU6's LTD), which silently truncated tau for any board running
        // later: the August monthly ends 2026-08-31 21:00Z, so 7 of its 21 sessions had
        // no minutes and sigma*sqrt(tau) came out 18% too small (2026-07-26).
        // Extending the calendar forward cannot change any past count. Stops at Oct 31
        // because ts() assumes EDT — revisit before the 2026-11-01 EST transition.
        let to = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
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
    let mut jobs = vec![];
    let mut d = from;
    while d <= to {
        let t0 = ts(d, 0, 0);
        // A day's file is COMPLETE only if it was written after that day ended. `d == today`
        // is the obvious case, but the dangerous one is YESTERDAY: a run at 07:00Z writes a
        // file covering 00:00-07:00Z, and every later run then reports it "cached" and keeps
        // the truncation forever. That is how day-4 logged RV14 48.8% against a true 51.7%,
        // and on 2026-07-28 it was found holding a 52-byte `no_data` SPY/NVDA file for the
        // whole of Monday's RTH session. Refetch whenever the file predates the day's end.
        let day_end = t0 + 86400;
        for suf in ["a", "b"] {
            let p = dir.join(format!("{d}_{suf}.json"));
            let complete = fs::metadata(&p)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|dur| dur.as_secs() as i64 >= day_end)
                .unwrap_or(false);
            if !complete {
                let _ = fs::remove_file(&p);
            }
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

/// The SMOOTH half of `realized_vol`: same 5-minute closes, but only consecutive pairs
/// that are genuinely 5 minutes apart, so close-to-open gaps (and data holes) are
/// excluded from both the numerator and the denominator. Pair it with `gap_sd` — the two
/// together reconstruct total variance without pretending a weekend is smooth.
fn realized_vol_intraday(
    db: &CandleDb,
    key: &str,
    cal: &SessionCal,
    class: Class,
    t: i64,
    lookback_s: i64,
) -> Option<f64> {
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
    let pts: Vec<(i64, f64)> = buckets.into_iter().collect();
    let (mut ssq, mut n) = (0.0, 0usize);
    for w in pts.windows(2) {
        if w[1].0 - w[0].0 != 300 {
            continue;
        }
        let r = (w[1].1 / w[0].1).ln();
        ssq += r * r;
        n += 1;
    }
    if n < 100 {
        return None;
    }
    Some((ssq * min_per_year(class) / (n as f64 * 5.0)).sqrt())
}

// ---------- close-to-open GAP variance ----------
//
// `realized_vol` walks consecutive in-session 5-minute closes, so a close->open return
// IS in its sum of squares — but it is charged only 5 minutes of the denominator and
// `tau` then re-spreads it smoothly over session time. For a FIRST-PASSAGE question that
// is the wrong shape: the gap is a lump that lands at a known instant, and a barrier can
// be jumped clean over.
//
// It cost us `will-wti-dip-to-85-in-july-2026` on 2026-07-26 (see
// results/legsum-null-and-stale-feed-2026-07-27.md). Measured over 2026-04-01..2026-07-27:
//
//   feed        weekend-gap rms   overnight rms   intraday (open->close) rms
//   USOILSPOT        3.78%            0.57%              3.56%
//   XAUUSD           0.74%            0.13%              1.43%
//   XAGUSD           1.21%            0.16%              3.08%
//   SPY              0.73%            0.57%              0.66%
//   NVDA             1.38%            1.42%              1.83%
//
// A WTI weekend gap carries as much variance as a whole trading session, and for the
// RTH-only equity feeds the overnight gap is ~85% of a session. Crypto never closes, so
// its gap variance is zero by construction.

/// Session boundaries in `[t0, t1)`: for each, the length of the CLOSED interval in
/// seconds. Derived from the calendar, so it needs no per-asset table.
fn session_breaks(cal: &SessionCal, t0: i64, t1: i64) -> Vec<i64> {
    let a = cal.mins.partition_point(|&m| m < t0);
    let b = cal.mins.partition_point(|&m| m < t1);
    let mut out = vec![];
    for w in cal.mins[a..b].windows(2) {
        if w[1] - w[0] > 60 {
            out.push(w[1] - w[0]);
        }
    }
    out
}

/// If `t` falls inside a closed interval, its length in seconds — i.e. "the market is
/// shut and the next thing that happens is an open, `n` seconds after the last print".
fn current_break(cal: &SessionCal, t: i64) -> Option<i64> {
    if cal.is_open(t) {
        return None;
    }
    let i = cal.mins.partition_point(|&m| m < t);
    let prev = if i == 0 { None } else { Some(cal.mins[i - 1]) };
    let next = cal.mins.get(i).copied();
    match (prev, next) {
        (Some(p), Some(n)) => Some(n - p),
        _ => None,
    }
}

/// Variance of a close-to-open gap of clock length `dt`, from the measured (short, long)
/// rms pair. The split is a step, not a curve: an overnight pause and a weekend are
/// different animals and there is no useful sample in between.
fn gap_var(dt: i64, sd: (f64, f64)) -> f64 {
    let s = if dt > 86400 { sd.1 } else { sd.0 };
    s * s
}

/// rms close-to-open log return over the trailing `lookback_s`, split into "short" breaks
/// (an overnight pause) and "long" ones (a weekend / holiday). Returns (short, long).
/// `None` for a class that never closes.
fn gap_sd(
    db: &CandleDb,
    key: &str,
    cal: &SessionCal,
    class: Class,
    t: i64,
    lookback_s: i64,
) -> Option<(f64, f64)> {
    if class == Class::Crypto {
        return Some((0.0, 0.0));
    }
    // last in-session close before each break, first in-session close after it
    let candles = db.slice(key, t - lookback_s, t);
    let mut open: Vec<(i64, f64)> = vec![];
    for c in candles {
        if cal.is_open(c.t) {
            open.push((c.t, c.c));
        }
    }
    if open.len() < 200 {
        return None;
    }
    let (mut sh, mut lg) = (vec![], vec![]);
    for w in open.windows(2) {
        let dt = w[1].0 - w[0].0;
        if dt <= 60 {
            continue;
        }
        let r = (w[1].1 / w[0].1).ln();
        // > 24h of clock closure == a weekend or a holiday bridge
        if dt > 86400 { lg.push(r) } else { sh.push(r) }
    }
    let rms = |v: &Vec<f64>| {
        if v.is_empty() {
            0.0
        } else {
            (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
        }
    };
    Some((rms(&sh), rms(&lg)))
}

/// First-passage probability with an explicit INITIAL JUMP before the diffusion starts.
///
/// Two situations need it and they compose:
///   * the resolving feed is shut right now, so the next thing that happens to the
///     barrier is a close-to-open gap (`gap_sd`), and
///   * the leg's window has not opened yet, so the level diffuses for `tau_pre` of
///     session time before the barrier is watched at all.
///
/// A path that jumps beyond the barrier touches AT the open — the venue reads the
/// candle, not our smooth model — so that mass counts in full.
///
/// The jump is `exp(jump_sd * Z)` with NO `-jump_sd^2/2` convexity term, because the rest
/// of this model is driftless in LOG price (`touch_prob` is `2N(-|ln(B/S)|/(sigma sqrt(tau)))`,
/// which assumes zero log-drift). Using the martingale-in-price convention here instead
/// would inject a `-jump_sd^2/2` log-drift that makes every DOWN leg likelier and every UP
/// leg less likely — a systematic tilt in exactly the direction that flatters a seller of
/// the up wing. `selftest` catches it: with the convexity term the equal-variance
/// jump-vs-diffusion inequality holds for H legs and is violated for L legs.
///
/// Integration: 201-node midpoint rule on the standard normal over +/-6 sd, which
/// reproduces `touch_prob` to <1e-4 when `jump_sd = 0` (see `selftest`).
fn touch_prob_jump(
    s: f64,
    b: f64,
    dir: char,
    sigma: f64,
    tau_yrs: f64,
    jump_sd: f64,
) -> f64 {
    if jump_sd <= 0.0 {
        return touch_prob(s, b, dir, sigma, tau_yrs);
    }
    let (mut num, mut den) = (0.0, 0.0);
    let (k, hi) = (201usize, 6.0f64);
    for i in 0..k {
        let z = -hi + 2.0 * hi * (i as f64) / (k as f64 - 1.0);
        let w = (-0.5 * z * z).exp();
        let sp = s * (jump_sd * z).exp();
        let beyond = (dir == 'H' && sp >= b) || (dir == 'L' && sp <= b);
        num += w * if beyond { 1.0 } else { touch_prob(sp, b, dir, sigma, tau_yrs) };
        den += w;
    }
    num / den
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

/// Which pricer produced a row. The ledger (`predictions/predictions.csv`) has no column
/// for this, and on 2026-07-27 the pricer changed mid-trial: rows up to and including
/// 2026-07-26 used plain `touch_prob` (first passage, no jump term), rows from 2026-07-27
/// use `touch_prob_jump` (explicit pre-window / closed-feed jump). The change was uniformly
/// DOWNWARD in q, i.e. it flatters a seller, and it landed on the very day the trial's
/// headline flipped — so the 07-31 scoring must be splittable by it or it cannot say
/// whether the fix helped. Stamp it on every row from here on; the historical mapping is
/// `memory/pricer-versions.csv`.
const PRICER_VERSION: &str = "touch_prob_jump";

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
        "market_slug,condition_id,outcome,token_id,probability,market_midpoint,bid,ask,dir,barrier,feed_age_h,feed_open,jump_sd,pricer,sigma_rv,sigma_iv,q_iv,q_blend,note"
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
        let rv_i = realized_vol_intraday(&db, key, cal, class, now, 14 * 86400);
        let gsd = gap_sd(&db, key, cal, class, now, 120 * 86400).unwrap_or((0.0, 0.0));
        let ivv = iv_at(&iv, asset, now);

        // How stale is the resolving feed right now? A prediction made while the feed is
        // shut carries NO information the last print did not already have, while the
        // book keeps trading. This is what lost `will-wti-dip-to-85-in-july-2026`:
        // 28.8h stale, the market moved 0.475 -> 0.715 over the closure, the model could
        // not move at all, and CLU6 opened -7.8% straight through the barrier.
        let last_print = db
            .by_key
            .get(key)
            .and_then(|v| v.iter().rev().find(|c| cal.is_open(c.t) && c.t <= now).map(|c| c.t));
        let age_h = last_print.map(|t| (now - t) as f64 / 3600.0).unwrap_or(f64::NAN);
        let cur_break = current_break(cal, now);
        let feed_open = cur_break.is_none();

        println!(
            "\n== {slug} | spot[{key}] {} ({} {:.1}h old) | RV14d {} (intraday {}) | gap sd {:.2}%/{:.2}% | IV {} | window ends {} ==",
            spot.map(|s| format!("{s:.2}")).unwrap_or("?".into()),
            if feed_open { "feed OPEN," } else { "feed SHUT," },
            age_h,
            rv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            rv_i.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            gsd.0 * 100.0,
            gsd.1 * 100.0,
            ivv.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
            DateTime::from_timestamp(l0.we, 0).unwrap().format("%m-%d %H:%MZ")
        );
        if let Some(dt) = cur_break {
            println!(
                "  !! STALE FEED: shut for {:.1}h, reopens in {:.1}h. Every q below is priced off a\n\
                 !! frozen spot; the book has been trading the whole time. Treat any disagreement\n\
                 !! with the market as OUR blind spot until the feed reopens (stale-feed gate).",
                age_h,
                (dt as f64 / 3600.0) - age_h
            );
        }
        println!(
            "  {:>3} {:>9} {:>7} {:>7} {:>7} {:>9} {:>7} {:>7} {:>7} {:>7}",
            "dir", "barrier", "bid", "ask", "mid", "tob$", "jump", "q_rv", "q_iv", "edge"
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

            // ---- variance that lands BEFORE the barrier is watched, as one jump ----
            // (a) the leg's window has not opened yet: the level free-diffuses for
            //     tau_pre of session time, plus any session breaks in between;
            // (b) the feed is shut right now: the next event the barrier sees is a
            //     close-to-open gap, and a path that gaps past the barrier touches AT
            //     the open. Both were missing before 2026-07-27.
            let sig_i = rv_i.or(rv).unwrap_or(0.0);
            let tau_pre = cal.count(now, l.ws.max(now)) as f64 / min_per_year(class);
            let mut jump_var = sig_i * sig_i * tau_pre;
            for dt in session_breaks(cal, now, l.ws.max(now)) {
                jump_var += gap_var(dt, gsd);
            }
            if let Some(dt) = cur_break {
                jump_var += gap_var(dt, gsd);
            }
            // remaining in-window breaks stay smooth (they are far away and many)
            let mut win_gap_var = 0.0;
            for dt in session_breaks(cal, tau_t0, l.we) {
                win_gap_var += gap_var(dt, gsd);
            }
            let bump = |v: f64| {
                if tau > 0.0 { (v * v + win_gap_var / tau).sqrt() } else { v }
            };
            let jump_sd = jump_var.sqrt();

            let q_rv = match (spot, rv_i.or(rv)) {
                (Some(s), Some(v)) => {
                    Some(touch_prob_jump(s, l.barrier, l.dir, bump(v), tau, jump_sd))
                }
                _ => None,
            };
            let q_iv = match (spot, ivv) {
                (Some(s), Some(v)) => {
                    Some(touch_prob_jump(s, l.barrier, l.dir, bump(v), tau, jump_sd))
                }
                _ => None,
            };
            // PRE-REGISTERED COMPARISON, not a live switch (results/prereg-rv-iv-blend-2026-07-28.md).
            // Recorded so 2026-07-31 can score RV-primary against IV-primary and a fixed
            // 50/50 sigma blend on the SAME legs. `sigma_blend` is fixed at w = 0.5 and is
            // never tuned; tuning it after seeing the outcome is what this file exists to
            // prevent. The live trade signal continues to read q_rv only.
            let sigma_rv = rv_i.or(rv).map(bump);
            let sigma_iv = ivv.map(bump);
            let q_blend = match (spot, sigma_rv, sigma_iv) {
                (Some(s), Some(a), Some(b)) => Some(touch_prob_jump(
                    s,
                    l.barrier,
                    l.dir,
                    0.5 * (a + b),
                    tau,
                    jump_sd,
                )),
                _ => None,
            };
            println!(
                "  {:>3} {:>9} {:>7} {:>7} {:>7} {:>9.0} {:>6.2}% {:>7} {:>7} {:>7}",
                l.dir,
                l.barrier,
                bid.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                ask.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                mid.map(|p| format!("{p:.3}")).unwrap_or_default(),
                tob,
                jump_sd * 100.0,
                q_rv.map(|p| format!("{p:.3}")).unwrap_or_default(),
                q_iv.map(|p| format!("{p:.3}")).unwrap_or_default(),
                match (q_rv, mid) {
                    (Some(q), Some(m)) => format!("{:+.3}", q - m),
                    _ => String::new(),
                }
            );
            if let (Some(q), Some(m)) = (q_rv, mid) {
                pred_rows.push(format!(
                    "{},{},Yes,{},{:.4},{:.4},{},{},{},{},{:.1},{},{:.4},{},{},{},{},{},spot={:.2};rv={:.3};rv_i={:.3};tau={:.5};ws={}",
                    l.market_slug,
                    l.condition_id,
                    l.token_yes,
                    q,
                    m,
                    bid.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                    ask.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                    l.dir,
                    l.barrier,
                    age_h,
                    feed_open as u8,
                    jump_sd,
                    PRICER_VERSION,
                    sigma_rv.map(|v| format!("{v:.4}")).unwrap_or_default(),
                    sigma_iv.map(|v| format!("{v:.4}")).unwrap_or_default(),
                    q_iv.map(|v| format!("{v:.4}")).unwrap_or_default(),
                    q_blend.map(|v| format!("{v:.4}")).unwrap_or_default(),
                    spot.unwrap_or(f64::NAN),
                    rv.unwrap_or(f64::NAN),
                    sig_i,
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

// ---------- roll-aware pricing (WTI active-month boards spanning a CME roll) ----------
//
// A "Hit Price" WTI board resolves on the ACTIVE MONTH contract, and the active month
// changes inside a monthly board's window. The resolving series therefore JUMPS by the
// calendar spread on a known date, and `touch_prob` (one spot, one sigma, no jump) is
// simply the wrong model for such a board. Derivation and numbers:
// results/august-roll-model-2026-07-26.md.
//
// Model. Primitive = ln V (the DEFERRED contract, the one that survives to the end of the
// board), driftless BM with vol sigma_v in session time. The front contract is linked to
// it by the log calendar spread k = ln(U/V), which is itself a function of the flat price
// (backwardation steepens when crude rallies):
//
//     ln U(t) = ln V(t) + k0 + beta * (ln V(t) - ln V0)
//
// beta is estimated by regressing d(ln U - ln V) on d(ln V); it also reproduces the
// observed vol ratio, sigma_u = (1 + beta) * sigma_v. The link is deterministic, so a
// barrier B on the U scale is the level B * (V0/U0)^... on the V scale:
//
//     B_front = V0 * (B / U0)^(1 / (1 + beta))
//
// The board is then one process with a barrier that STEPS at the roll: B_front while the
// front contract is active, B afterwards. For an up-barrier B_front < B, so the pre-roll
// leg is easier; for a down-barrier the post-roll leg is easier, and any path sitting
// between the two levels at the roll instant is absorbed AT the roll — the series jumps
// down onto the barrier. That atom is the whole story of the down wing.
//
// Numerics: absorbing heat kernel by images on a uniform log grid,
// p(y|x,tau) = phi(y-x) - phi(y+x-2b). The grid must reach 2b - lo or the image source
// is truncated and every touch probability comes out exactly half. Validated against
// 2*N(-|ln(B/S)|/(sigma*sqrt(tau))) to ~5e-4.

const ROLL_DX: f64 = 0.0004; // log-price grid step (~0.04%)

/// One diffusion step with an absorbing UP barrier at grid index `ib` (mass at or above
/// `ib` is killed). `dens` is a density on a uniform grid of spacing `dx`.
fn absorb_step(dens: &[f64], tau: f64, sigma: f64, dx: f64, ib: usize) -> Vec<f64> {
    let n = dens.len();
    let mut out = vec![0.0; n];
    if tau <= 0.0 {
        out[..ib.min(n)].copy_from_slice(&dens[..ib.min(n)]);
        return out;
    }
    let s = sigma * tau.sqrt();
    let m = (9.0 * s / dx).ceil() as usize;
    let mut ker = vec![0.0; 2 * m + 1];
    let mut ksum = 0.0;
    for (i, k) in ker.iter_mut().enumerate() {
        let z = (i as f64 - m as f64) * dx;
        *k = (-z * z / (2.0 * s * s)).exp();
        ksum += *k;
    }
    for k in ker.iter_mut() {
        *k /= ksum;
    }
    // image source: the density reflected about the barrier node
    let mut img = vec![0.0; n];
    for (j, v) in img.iter_mut().enumerate() {
        if 2 * ib >= j && 2 * ib - j < n {
            *v = dens[2 * ib - j];
        }
    }
    for j in 0..ib.min(n) {
        let mut acc = 0.0;
        for (i, k) in ker.iter().enumerate() {
            let src = j as isize + m as isize - i as isize;
            if src >= 0 && (src as usize) < n {
                acc += k * (dens[src as usize] - img[src as usize]);
            }
        }
        out[j] = acc.max(0.0);
    }
    out
}

/// P(the active-month series touches `barrier` during the board window), with the active
/// month switching from the front contract to the deferred one at the roll.
/// `tau_pre` = session time from now to window open (barrier not live yet),
/// `tau_front` = window open -> roll, `tau_back` = roll -> window close.
#[allow(clippy::too_many_arguments)]
fn touch_prob_roll(
    u0: f64,
    v0: f64,
    barrier: f64,
    dir: char,
    sigma_v: f64,
    beta: f64,
    tau_pre: f64,
    tau_front: f64,
    tau_back: f64,
) -> f64 {
    let b_front = v0 * (barrier / u0).powf(1.0 / (1.0 + beta));
    let sgn = if dir == 'H' { 1.0 } else { -1.0 };
    let (x0, bf, bb) = (sgn * v0.ln(), sgn * b_front.ln(), sgn * barrier.ln());
    let tot = sigma_v * (tau_pre + tau_front + tau_back).max(1e-12).sqrt();
    let lo = x0 - 11.0 * tot - 0.05;
    let (i_f, i_b) = (
        ((bf - lo) / ROLL_DX).round(),
        ((bb - lo) / ROLL_DX).round(),
    );
    if i_f <= 0.0 || i_b <= 0.0 {
        return 1.0; // already at or beyond the barrier
    }
    let (i_f, i_b) = (i_f as usize, i_b as usize);
    let n = 2 * i_f.max(i_b) + 1;
    // window has not opened yet: free diffusion, so the state at the open is lognormal
    let mut dens = vec![0.0; n];
    if tau_pre <= 0.0 {
        dens[(((x0 - lo) / ROLL_DX).round() as usize).min(n - 1)] = 1.0 / ROLL_DX;
    } else {
        let s0 = sigma_v * tau_pre.sqrt();
        for (i, d) in dens.iter_mut().enumerate() {
            let z = (lo + ROLL_DX * i as f64 - x0) / s0;
            *d = (-0.5 * z * z).exp() / (s0 * (2.0 * std::f64::consts::PI).sqrt());
        }
    }
    for d in dens.iter_mut().skip(i_f) {
        *d = 0.0; // window opens with the front-contract barrier live
    }
    let mut dens = absorb_step(&dens, tau_front, sigma_v, ROLL_DX, i_f);
    for d in dens.iter_mut().skip(i_b) {
        *d = 0.0; // THE ROLL: the series jumps onto the deferred contract
    }
    let dens = absorb_step(&dens, tau_back, sigma_v, ROLL_DX, i_b);
    (1.0 - dens.iter().sum::<f64>() * ROLL_DX).clamp(0.0, 1.0)
}

/// CLU6 -> CLV6: the active month changes at the start of the session for Tue 2026-08-18,
/// which opens 6pm ET Mon 2026-08-17. Derived from the board's own fine print (the next
/// contract is active for the final THREE sessions of the nearest one) and CLU6's LTD of
/// Thu 2026-08-20, which Pyth states outright ("PYTH WTI 20 AUGUST 2026").
const ROLL_U6_V6: &str = "2026-08-17T22:00:00Z";

fn cmd_roll(data: &Path, slug: &str, u0: f64, v0: f64, sigma_v: f64, beta: f64) -> Result<()> {
    let client = mk_client();
    let now = Utc::now().timestamp();
    let cal = SessionCal::build(Class::Wti);
    let roll = DateTime::parse_from_rfc3339(ROLL_U6_V6)?.timestamp();
    let ev_txt = get_text(
        &client,
        &format!("https://gamma-api.polymarket.com/events?slug={slug}"),
    )?;
    fs::create_dir_all(data.join("events_live"))?;
    fs::write(data.join("events_live").join(format!("{slug}.json")), &ev_txt)?;
    let v: serde_json::Value = serde_json::from_str(&ev_txt)?;
    let ev = v.as_array().and_then(|a| a.first()).context("board not found")?;
    let legs = extract_legs(slug, ev)?;
    let l0 = legs.first().context("no legs")?;
    let (ws, we) = (l0.ws, l0.we);
    let mpy = min_per_year(Class::Wti);
    let tau_pre = cal.count(now, ws) as f64 / mpy;
    let tau_front = cal.count(ws.max(now), roll.min(we)) as f64 / mpy;
    let tau_back = cal.count(roll.max(ws).max(now), we) as f64 / mpy;
    let sigma_u = (1.0 + beta) * sigma_v;
    println!(
        "roll model | {slug}\n  U0 {u0:.3}  V0 {v0:.3}  spread {:+.3} ({:.2}%)  sigma_v {:.1}%  \
         beta {beta:.2}  => sigma_u {:.1}%",
        u0 - v0,
        100.0 * (u0 - v0) / u0,
        sigma_v * 100.0,
        sigma_u * 100.0
    );
    println!(
        "  window {} -> {} | roll {} | tau: pre {:.6} front {:.6} back {:.6} ({} / {} / {} sessions)",
        DateTime::from_timestamp(ws, 0).unwrap().format("%m-%d %H:%MZ"),
        DateTime::from_timestamp(we, 0).unwrap().format("%m-%d %H:%MZ"),
        DateTime::from_timestamp(roll, 0).unwrap().format("%m-%d %H:%MZ"),
        tau_pre,
        tau_front,
        tau_back,
        (tau_pre * mpy / 1380.0).round(),
        (tau_front * mpy / 1380.0).round(),
        (tau_back * mpy / 1380.0).round()
    );
    println!(
        "  {:>3} {:>8} {:>9} {:>8} {:>8} {:>8} {:>8}",
        "dir", "barrier", "B_front", "naive", "roll", "diff", "mid"
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
        let best = |side: &str| -> Option<f64> {
            b[side].as_array()?.last()?["price"].as_str()?.parse().ok()
        };
        let mid = match (best("bids"), best("asks")) {
            (Some(bb), Some(aa)) => Some(0.5 * (bb + aa)),
            _ => None,
        };
        // what the current model would say: one spot (the front contract), no jump
        let naive = touch_prob(u0, l.barrier, l.dir, sigma_u, tau_front + tau_back);
        let q = touch_prob_roll(
            u0, v0, l.barrier, l.dir, sigma_v, beta, tau_pre, tau_front, tau_back,
        );
        println!(
            "  {:>3} {:>8.1} {:>9.2} {:>8.4} {:>8.4} {:>+8.4} {:>8}",
            l.dir,
            l.barrier,
            v0 * (l.barrier / u0).powf(1.0 / (1.0 + beta)),
            naive,
            q,
            naive - q,
            mid.map(|m| format!("{m:.4}")).unwrap_or_default()
        );
    }
    Ok(())
}

// ---------- main ----------

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: ladderrv <discover|candles|vol|clob|analyze|tape|wash|live|roll|gaps|selftest> \
                 <data_dir> ...\n  roll <board-slug> <u0> <v0> <sigma_v> <beta>";
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
        "roll" => cmd_roll(
            &data,
            &args[3],
            args[4].parse()?,
            args[5].parse()?,
            args[6].parse()?,
            args[7].parse()?,
        ),
        "selftest" => {
            // touch_prob_roll with beta=0, no pre-window time and one barrier must
            // reproduce the closed-form driftless one-touch probability.
            let (s, sig, tau) = (100.0, 0.5, 0.08);
            for (b, d) in [(105.0, 'H'), (110.0, 'H'), (130.0, 'H'), (95.0, 'L'), (70.0, 'L')] {
                let grid = touch_prob_roll(s, s, b, d, sig, 0.0, 0.0, tau, 0.0);
                let closed = touch_prob(s, b, d, sig, tau);
                println!("  {d}{b}: grid {grid:.5} closed {closed:.5} diff {:+.6}", grid - closed);
            }
            let split = touch_prob_roll(100.0, 100.0, 120.0, 'H', sig, 0.0, 0.0, 0.04, 0.04);
            let one = touch_prob(100.0, 120.0, 'H', sig, 0.08);
            println!("  split 0.04+0.04 vs one 0.08: {split:.5} vs {one:.5} diff {:+.6}", split - one);
            // touch_prob_jump with no jump must reproduce the closed form, and a jump of
            // sd j must equal free diffusion of the same variance when the barrier is far
            // enough that the "jumped past it" atom is negligible.
            for (b, d) in [(110.0, 'H'), (90.0, 'L')] {
                let z = touch_prob_jump(100.0, b, d, sig, tau, 0.0);
                let c = touch_prob(100.0, b, d, sig, tau);
                println!("  jump=0 {d}{b}: {z:.5} vs closed {c:.5} diff {:+.6}", z - c);
            }
            // The same variance delivered as a JUMP must give a strictly SMALLER touch
            // probability than delivering it as extra DIFFUSION time: a jump has no path,
            // so excursions inside the jump are not observed by the barrier, while a
            // diffusion of equal variance is watched continuously. (Checked, not assumed —
            // the first version of this comment asserted the opposite and was wrong.)
            for (b, d) in [(130.0, 'H'), (75.0, 'L')] {
                let j = 0.05;
                let jm = touch_prob_jump(100.0, b, d, sig, tau, j);
                let sm = touch_prob(100.0, b, d, sig, tau + j * j / (sig * sig));
                println!(
                    "  {d}{b} equal variance: JUMP {jm:.5} vs DIFFUSION {sm:.5} ({:+.2}%) {}",
                    100.0 * (jm / sm - 1.0),
                    if jm <= sm * (1.0 + 1e-4) { "ok" } else { "VIOLATED" }
                );
            }
            // The regime where the two forms genuinely separate: jump-dominated. With no
            // diffusion at all the barrier can only be crossed BY the jump, so the answer
            // must fall to N(-b/j) -- exactly HALF the reflection-principle value, because
            // reflection counts paths that touched and came back and a jump has no path.
            let j = 0.30;
            for (b, d) in [(130.0, 'H'), (75.0, 'L')] {
                let jm = touch_prob_jump(100.0, b, d, sig * 1e-6, 1e-12, j);
                let one = ncdf(-(b / 100.0f64).ln().abs() / j);
                println!(
                    "  {d}{b} jump-only: {jm:.5} vs N(-|ln(B/S)|/j) {one:.5} diff {:+.6} (reflection would be {:.5})",
                    jm - one,
                    2.0 * one
                );
            }
            Ok(())
        }
        "gaps" => {
            // Close-to-open gap statistics per feed: the input to the jump term.
            let cals: BTreeMap<Class, SessionCal> = [Class::Crypto, Class::Equity, Class::Wti]
                .into_iter()
                .map(|c| (c, SessionCal::build(c)))
                .collect();
            let keys = ["USOILSPOT", "WTIU6", "WTIV6", "XAUUSD", "XAGUSD", "SPY", "NVDA", "BTCUSDT"];
            let db = CandleDb::load(&data, &keys);
            let now = Utc::now().timestamp();
            println!(
                "\n  {:<12} {:>10} {:>12} {:>12} {:>14}",
                "feed", "class", "gap sd o/n", "gap sd wknd", "RV14 intraday"
            );
            for k in keys {
                let class = match k {
                    "SPY" | "NVDA" => Class::Equity,
                    "BTCUSDT" => Class::Crypto,
                    _ => Class::Wti,
                };
                let cal = &cals[&class];
                let g = gap_sd(&db, k, cal, class, now, 120 * 86400);
                let vi = realized_vol_intraday(&db, k, cal, class, now, 14 * 86400);
                println!(
                    "  {:<12} {:>10} {:>11} {:>12} {:>14}",
                    k,
                    format!("{class:?}"),
                    g.map(|x| format!("{:.2}%", x.0 * 100.0)).unwrap_or("n/a".into()),
                    g.map(|x| format!("{:.2}%", x.1 * 100.0)).unwrap_or("n/a".into()),
                    vi.map(|v| format!("{:.1}%", v * 100.0)).unwrap_or("n/a".into()),
                );
            }
            Ok(())
        }
        _ => bail!("{usage}"),
    }
}
