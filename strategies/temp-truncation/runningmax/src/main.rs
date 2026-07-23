// temp-truncation/runningmax — backtest + live tooling.
//
// Subcommands (all paths absolute; data_dir is the working data directory, NOT in git):
//   discover <data_dir> <from> <to> <city,city,...>   fetch resolved Gamma events per city-day -> events/, legs.csv
//   prices   <data_dir> <fidelity> [fam,fam,...]      fetch CLOB prices-history per leg -> prices{fid}/
//   obs      <data_dir> <from> <to>                   fetch IEM ASOS obs per station + HKO daily max -> obs/
//   tape     <data_dir> <fam> [fam...]                fetch Data-API trades per leg -> tape/   (fam = city_YYYY-MM-DD)
//   analyze  <data_dir>                               gates 1/2/3 + resolution validation -> out/ + stdout
//   wash     <data_dir> <fam> [fam...]                gate 4 wash checks on tape families
//   live     <data_dir> <date> <city,city,...>        today's books + obs + model -> prediction rows
//
// Conventions: all obs stored in UTC; "units" = display units of the market
// (°C for C-cities, °F for US cities, real °C with floor-buckets for HK).

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const CHECKPOINTS: [i64; 4] = [10, 12, 14, 16];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Unit {
    C,       // whole-degree rounding (Wunderground displays round(°C))
    F,       // whole-degree °F rounding, 2°F buckets
    HkFloor, // HKO one-decimal °C, bucket CONTAINS the value => floor semantics
}

struct City {
    slug: &'static str,
    station: &'static str, // IEM ASOS id (for HK: VHHH used as intraday PROXY only)
    off: i64,              // local UTC offset hours, summer 2026
    unit: Unit,
}

const CITIES: &[City] = &[
    City { slug: "london", station: "EGLC", off: 1, unit: Unit::C },
    City { slug: "paris", station: "LFPB", off: 2, unit: Unit::C },
    City { slug: "seoul", station: "RKSI", off: 9, unit: Unit::C },
    City { slug: "hong-kong", station: "VHHH", off: 8, unit: Unit::HkFloor },
    City { slug: "los-angeles", station: "KLAX", off: -7, unit: Unit::F },
    City { slug: "nyc", station: "KLGA", off: -4, unit: Unit::F },
    City { slug: "chicago", station: "KORD", off: -5, unit: Unit::F },
];

fn city(slug: &str) -> Result<&'static City> {
    CITIES.iter().find(|c| c.slug == slug).ok_or_else(|| anyhow!("unknown city {slug}"))
}

fn kernel_sigma(u: Unit) -> f64 {
    match u {
        Unit::C | Unit::HkFloor => 0.45,
        Unit::F => 0.8,
    }
}

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

fn norm_cdf(x: f64, mean: f64, sigma: f64) -> f64 {
    0.5 * (1.0 + erf((x - mean) / (sigma * std::f64::consts::SQRT_2)))
}

fn median(v: &mut Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
}

// ---------- HTTP ----------

fn mk_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("orakel-runningmax/0.1")
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

// ---------- slugs / buckets ----------

fn family_slug(city: &str, date: NaiveDate) -> String {
    let month = date.format("%B").to_string().to_lowercase();
    format!("highest-temperature-in-{city}-on-{month}-{}-{}", date.day(), date.year())
}

/// Parse groupItemTitle into (lo, hi) in display units; -999/999 = open end.
fn parse_bucket(title: &str) -> Result<(i64, i64)> {
    let re_range = regex::Regex::new(r"^(\d+)-(\d+)°F$").unwrap();
    let re_single = regex::Regex::new(r"^(\d+)°(C|F)$").unwrap();
    let re_below = regex::Regex::new(r"^(\d+)°(C|F) or below$").unwrap();
    let re_above = regex::Regex::new(r"^(\d+)°(C|F) or higher$").unwrap();
    if let Some(c) = re_range.captures(title) {
        return Ok((c[1].parse()?, c[2].parse()?));
    }
    if let Some(c) = re_single.captures(title) {
        let v: i64 = c[1].parse()?;
        return Ok((v, v));
    }
    if let Some(c) = re_below.captures(title) {
        return Ok((-999, c[1].parse()?));
    }
    if let Some(c) = re_above.captures(title) {
        return Ok((c[1].parse()?, 999));
    }
    bail!("unparseable bucket title {title:?}")
}

fn bucket_fname(lo: i64, hi: i64) -> String {
    format!("{}_{}", lo, hi).replace('-', "m")
}

/// Real-valued interval of station-unit values that resolve into this bucket.
fn bucket_interval(unit: Unit, lo: i64, hi: i64) -> (f64, f64) {
    let (l, h) = match unit {
        Unit::C | Unit::F => (lo as f64 - 0.5, hi as f64 + 0.5),
        Unit::HkFloor => (lo as f64, hi as f64 + 1.0),
    };
    let l = if lo == -999 { f64::NEG_INFINITY } else { l };
    let h = if hi == 999 { f64::INFINITY } else { h };
    (l, h)
}

fn display(unit: Unit, v: f64) -> i64 {
    match unit {
        Unit::C | Unit::F => v.round() as i64,
        Unit::HkFloor => v.floor() as i64,
    }
}

fn bucket_contains(lo: i64, hi: i64, d: i64) -> bool {
    (lo == -999 || d >= lo) && (hi == 999 || d <= hi)
}

// ---------- legs ----------

#[derive(Clone, Debug)]
struct Leg {
    city: String,
    date: NaiveDate,
    market_slug: String,
    condition_id: String,
    token_id: String,
    lo: i64,
    hi: i64,
    title: String,
    winner: bool,
    volume: f64,
}

fn legs_csv(data: &Path) -> PathBuf {
    data.join("legs.csv")
}

fn load_legs(data: &Path) -> Result<BTreeMap<(String, NaiveDate), Vec<Leg>>> {
    let txt = fs::read_to_string(legs_csv(data)).context("legs.csv (run discover first)")?;
    let mut out: BTreeMap<(String, NaiveDate), Vec<Leg>> = BTreeMap::new();
    for line in txt.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 10 {
            continue;
        }
        let leg = Leg {
            city: f[0].into(),
            date: NaiveDate::parse_from_str(f[1], "%Y-%m-%d")?,
            market_slug: f[2].into(),
            condition_id: f[3].into(),
            token_id: f[4].into(),
            lo: f[5].parse()?,
            hi: f[6].parse()?,
            title: f[7].into(),
            winner: f[8] == "1",
            volume: f[9].parse().unwrap_or(0.0),
        };
        out.entry((leg.city.clone(), leg.date)).or_default().push(leg);
    }
    for legs in out.values_mut() {
        legs.sort_by_key(|l| (l.lo, l.hi));
    }
    Ok(out)
}

fn extract_legs_from_event(city: &str, date: NaiveDate, ev: &serde_json::Value) -> Result<Vec<Leg>> {
    let markets = ev["markets"].as_array().context("no markets")?;
    let mut legs = vec![];
    for m in markets {
        let title = m["groupItemTitle"].as_str().unwrap_or_default().to_string();
        let (lo, hi) = parse_bucket(&title)?;
        let prices: Vec<String> =
            serde_json::from_str(m["outcomePrices"].as_str().unwrap_or("[]")).unwrap_or_default();
        let tokens: Vec<String> =
            serde_json::from_str(m["clobTokenIds"].as_str().unwrap_or("[]")).unwrap_or_default();
        legs.push(Leg {
            city: city.into(),
            date,
            market_slug: m["slug"].as_str().unwrap_or_default().into(),
            condition_id: m["conditionId"].as_str().unwrap_or_default().into(),
            token_id: tokens.first().cloned().unwrap_or_default(),
            lo,
            hi,
            title,
            winner: prices.first().map(|p| p == "1").unwrap_or(false),
            volume: m["volumeNum"].as_f64().unwrap_or(0.0),
        });
    }
    Ok(legs)
}

// ---------- discover ----------

fn cmd_discover(data: &Path, from: NaiveDate, to: NaiveDate, cities: &[String]) -> Result<()> {
    let mut jobs = vec![];
    for c in cities {
        city(c)?;
        let mut d = from;
        while d <= to {
            let slug = family_slug(c, d);
            let url = format!("https://gamma-api.polymarket.com/events?slug={slug}");
            jobs.push((url, data.join("events").join(format!("{c}_{d}.json"))));
            d += Duration::days(1);
        }
    }
    let (fetched, skipped, failed) = fetch_all(jobs, 8);
    println!("discover: fetched {fetched}, cached {skipped}, failed {failed}");

    // Build legs.csv from every event file present.
    let mut rows = vec!["city,date,market_slug,condition_id,token_id,lo,hi,title,winner,volume"
        .to_string()];
    let mut n_fam = 0;
    let mut n_unresolved = 0;
    for entry in fs::read_dir(data.join("events"))? {
        let p = entry?.path();
        let name = p.file_stem().unwrap().to_string_lossy().to_string();
        let (c, ds) = name.rsplit_once('_').context("bad event filename")?;
        let d = NaiveDate::parse_from_str(ds, "%Y-%m-%d")?;
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p)?)?;
        let Some(ev) = v.as_array().and_then(|a| a.first()) else { continue };
        if !ev["closed"].as_bool().unwrap_or(false) {
            continue;
        }
        let legs = extract_legs_from_event(c, d, ev)?;
        let winners = legs.iter().filter(|l| l.winner).count();
        if winners != 1 {
            n_unresolved += 1;
            continue;
        }
        n_fam += 1;
        for l in &legs {
            rows.push(format!(
                "{},{},{},{},{},{},{},{},{},{}",
                l.city,
                l.date,
                l.market_slug,
                l.condition_id,
                l.token_id,
                l.lo,
                l.hi,
                l.title,
                if l.winner { 1 } else { 0 },
                l.volume
            ));
        }
    }
    fs::write(legs_csv(data), rows.join("\n") + "\n")?;
    println!("legs.csv: {n_fam} resolved families ({n_unresolved} closed-but-unresolved skipped)");
    Ok(())
}

// ---------- prices ----------

fn local_midnight_utc_ts(c: &City, date: NaiveDate) -> i64 {
    (date.and_hms_opt(0, 0, 0).unwrap() - Duration::hours(c.off)).and_utc().timestamp()
}

fn cmd_prices(data: &Path, fidelity: u32, filter: Option<Vec<String>>) -> Result<()> {
    let fams = load_legs(data)?;
    let mut jobs = vec![];
    for ((cslug, date), legs) in &fams {
        let fam_id = format!("{cslug}_{date}");
        if let Some(f) = &filter {
            if !f.contains(&fam_id) {
                continue;
            }
        }
        let c = city(cslug)?;
        let start = local_midnight_utc_ts(c, *date)
            - if fidelity <= 2 { 2 * 3600 } else { 3 * 86400 };
        for l in legs {
            let url = format!(
                "https://clob.polymarket.com/prices-history?market={}&startTs={start}&fidelity={fidelity}",
                l.token_id
            );
            jobs.push((
                url,
                data.join(format!("prices{fidelity}"))
                    .join(&fam_id)
                    .join(format!("{}.json", bucket_fname(l.lo, l.hi))),
            ));
        }
    }
    let (fetched, skipped, failed) = fetch_all(jobs, 8);
    println!("prices f{fidelity}: fetched {fetched}, cached {skipped}, failed {failed}");
    Ok(())
}

fn load_series(data: &Path, fidelity: u32, leg: &Leg) -> Option<Vec<(i64, f64)>> {
    let p = data
        .join(format!("prices{fidelity}"))
        .join(format!("{}_{}", leg.city, leg.date))
        .join(format!("{}.json", bucket_fname(leg.lo, leg.hi)));
    let txt = fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&txt).ok()?;
    let h = v["history"].as_array()?;
    let mut out = vec![];
    for pt in h {
        out.push((pt["t"].as_i64()?, pt["p"].as_f64()?));
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn price_at(series: &[(i64, f64)], t: i64, max_age_s: i64) -> Option<f64> {
    let mut best = None;
    for (ts, p) in series {
        if *ts <= t && t - *ts <= max_age_s {
            best = Some(*p);
        }
        if *ts > t {
            break;
        }
    }
    best
}

// ---------- obs ----------

fn cmd_obs(data: &Path, from: NaiveDate, to: NaiveDate) -> Result<()> {
    let mut jobs = vec![];
    for c in CITIES {
        let url = format!(
            "https://mesonet.agron.iastate.edu/cgi-bin/request/asos.py?station={}&data=tmpc,tmpf&year1={}&month1={}&day1={}&year2={}&month2={}&day2={}&tz=Etc/UTC&format=onlycomma&missing=M&trace=T&report_type=3,4",
            c.station, from.year(), from.month(), from.day(), to.year(), to.month(), to.day()
        );
        jobs.push((url, data.join("obs").join(format!("{}.csv", c.station))));
    }
    jobs.push((
        "https://data.weather.gov.hk/weatherAPI/opendata/opendata.php?dataType=CLMMAXT&year=2026&rformat=json&station=HKO".into(),
        data.join("obs").join("hko_daily.json"),
    ));
    let (fetched, skipped, failed) = fetch_all(jobs, 4);
    println!("obs: fetched {fetched}, cached {skipped}, failed {failed}");
    Ok(())
}

/// station -> sorted Vec<(utc_ts, tmpc, tmpf)>
fn load_obs_file(path: &Path) -> Result<Vec<(i64, Option<f64>, Option<f64>)>> {
    let txt = fs::read_to_string(path)?;
    let mut out = vec![];
    for line in txt.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 4 {
            continue;
        }
        let t = NaiveDateTime::parse_from_str(f[1], "%Y-%m-%d %H:%M")?.and_utc().timestamp();
        let tmpc = f[2].parse::<f64>().ok();
        let tmpf = f[3].parse::<f64>().ok();
        out.push((t, tmpc, tmpf));
    }
    out.sort_by_key(|r| r.0);
    Ok(out)
}

struct ObsDb {
    /// (city_slug, local_date) -> sorted Vec<(utc_ts, value_in_station_units)>
    days: HashMap<(String, NaiveDate), Vec<(i64, f64)>>,
    /// HKO finalized daily max, by date
    hko: HashMap<NaiveDate, f64>,
}

fn load_obs_db(data: &Path) -> Result<ObsDb> {
    let mut days: HashMap<(String, NaiveDate), Vec<(i64, f64)>> = HashMap::new();
    for c in CITIES {
        let p = data.join("obs").join(format!("{}.csv", c.station));
        if !p.exists() {
            continue;
        }
        for (t, tmpc, tmpf) in load_obs_file(&p)? {
            let val = match c.unit {
                Unit::C | Unit::HkFloor => tmpc,
                Unit::F => tmpf,
            };
            let Some(val) = val else { continue };
            let local = chrono::DateTime::from_timestamp(t, 0).unwrap().naive_utc()
                + Duration::hours(c.off);
            days.entry((c.slug.to_string(), local.date())).or_default().push((t, val));
        }
    }
    for v in days.values_mut() {
        v.sort_by_key(|r| r.0);
    }
    let mut hko = HashMap::new();
    let hp = data.join("obs").join("hko_daily.json");
    if hp.exists() {
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(hp)?)?;
        for row in v["data"].as_array().unwrap_or(&vec![]) {
            let r = row.as_array().unwrap();
            let (y, m, d): (i32, u32, u32) = (
                r[0].as_str().unwrap_or("0").parse()?,
                r[1].as_str().unwrap_or("0").parse()?,
                r[2].as_str().unwrap_or("0").parse()?,
            );
            if let Ok(val) = r[3].as_str().unwrap_or("x").parse::<f64>() {
                hko.insert(NaiveDate::from_ymd_opt(y, m, d).context("hko date")?, val);
            }
        }
    }
    Ok(ObsDb { days, hko })
}

impl ObsDb {
    fn runmax_at(&self, cslug: &str, date: NaiveDate, t_utc: i64) -> Option<f64> {
        let day = self.days.get(&(cslug.to_string(), date))?;
        let mut m: Option<f64> = None;
        for (t, v) in day {
            if *t > t_utc {
                break;
            }
            m = Some(m.map_or(*v, |x: f64| x.max(*v)));
        }
        m
    }

    /// Day's final max in station units + whether the day looks complete (obs past 22:00 local).
    fn final_max(&self, c: &City, date: NaiveDate) -> Option<(f64, bool)> {
        let day = self.days.get(&(c.slug.to_string(), date))?;
        let max = day.iter().map(|r| r.1).fold(f64::NEG_INFINITY, f64::max);
        let last_local =
            chrono::DateTime::from_timestamp(day.last()?.0, 0)?.naive_utc() + Duration::hours(c.off);
        Some((max, last_local.time().hour() >= 22))
    }
}

/// residuals[(city, local_hour)] -> Vec<(date, final_units - runmax_units_at_hour)>
/// For HK the "final" is the finalized HKO daily max and runmax is the VHHH proxy,
/// so the residual folds in the HKO-VHHH station bias.
fn build_residuals(db: &ObsDb) -> HashMap<(String, i64), Vec<(NaiveDate, f64)>> {
    let mut out: HashMap<(String, i64), Vec<(NaiveDate, f64)>> = HashMap::new();
    for c in CITIES {
        let dates: Vec<NaiveDate> = db
            .days
            .keys()
            .filter(|(s, _)| s == c.slug)
            .map(|(_, d)| *d)
            .collect();
        for date in dates {
            let fin = match c.unit {
                Unit::HkFloor => match db.hko.get(&date) {
                    Some(v) => *v,
                    None => continue,
                },
                _ => match db.final_max(c, date) {
                    Some((v, true)) => v,
                    _ => continue,
                },
            };
            for h in 0..24i64 {
                let t = (date.and_hms_opt(0, 0, 0).unwrap() + Duration::hours(h - c.off))
                    .and_utc()
                    .timestamp();
                if let Some(rm) = db.runmax_at(c.slug, date, t) {
                    out.entry((c.slug.to_string(), h)).or_default().push((date, fin - rm));
                }
            }
        }
    }
    out
}

/// Truncated kernel model: P(final bucket) given running max now + historical residuals.
fn model_probs(
    c: &City,
    buckets: &[(i64, i64)],
    rm_now: f64,
    resid: &[(NaiveDate, f64)],
    exclude: Option<NaiveDate>,
) -> Option<Vec<f64>> {
    let sigma = kernel_sigma(c.unit);
    let rs: Vec<f64> = resid
        .iter()
        .filter(|(d, _)| Some(*d) != exclude)
        .map(|(_, r)| *r)
        .collect();
    if rs.len() < 15 {
        return None;
    }
    let mut p = vec![0.0f64; buckets.len()];
    for r in &rs {
        // final >= rm_now always (monotone running max); clamp negative residuals
        // (possible for HK where runmax is a proxy station) at 0.
        let mean = rm_now + r.max(0.0);
        for (i, (lo, hi)) in buckets.iter().enumerate() {
            let (a, b) = bucket_interval(c.unit, *lo, *hi);
            p[i] += norm_cdf(b, mean, sigma) - norm_cdf(a, mean, sigma);
        }
    }
    let n = rs.len() as f64;
    for v in p.iter_mut() {
        *v /= n;
    }
    // Truncation: mass in buckets strictly below the running max's display value is
    // impossible; move it to the bucket containing display(rm_now).
    let d_now = display(c.unit, rm_now);
    let idx_now = buckets.iter().position(|(lo, hi)| bucket_contains(*lo, *hi, d_now));
    if let Some(idx_now) = idx_now {
        let mut moved = 0.0;
        for (i, (_, hi)) in buckets.iter().enumerate() {
            if *hi != 999 && *hi < d_now {
                moved += p[i];
                p[i] = 0.0;
            }
        }
        p[idx_now] += moved;
    }
    // floor + renormalize
    for v in p.iter_mut() {
        *v = v.max(0.002);
    }
    let s: f64 = p.iter().sum();
    for v in p.iter_mut() {
        *v /= s;
    }
    Some(p)
}

// ---------- tape ----------

fn tape_dir(data: &Path, fam: &str) -> PathBuf {
    data.join("tape").join(fam)
}

fn cmd_tape(data: &Path, fams: &[String]) -> Result<()> {
    let all = load_legs(data)?;
    let client = mk_client();
    for fam in fams {
        let (cslug, ds) = fam.rsplit_once('_').context("fam = city_YYYY-MM-DD")?;
        let date = NaiveDate::parse_from_str(ds, "%Y-%m-%d")?;
        let legs = all
            .get(&(cslug.to_string(), date))
            .with_context(|| format!("family {fam} not in legs.csv"))?;
        for l in legs {
            let out = tape_dir(data, fam).join(format!("{}.json", bucket_fname(l.lo, l.hi)));
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
            println!("tape {fam} {}: {} trades", l.title, trades.len());
        }
    }
    Ok(())
}

fn load_tape(data: &Path, fam: &str, leg: &Leg) -> Option<Vec<serde_json::Value>> {
    let p = tape_dir(data, fam).join(format!("{}.json", bucket_fname(leg.lo, leg.hi)));
    serde_json::from_str(&fs::read_to_string(p).ok()?).ok()
}

// ---------- analyze ----------

fn devig(ps: &[f64]) -> Vec<f64> {
    let s: f64 = ps.iter().sum();
    if s <= 0.0 {
        return ps.to_vec();
    }
    ps.iter().map(|p| p / s).collect()
}

fn cmd_analyze(data: &Path) -> Result<()> {
    let fams = load_legs(data)?;
    let db = load_obs_db(data)?;
    let resid = build_residuals(&db);
    let out_dir = data.join("out");
    fs::create_dir_all(&out_dir)?;

    // ---------- Gate 0: resolution validation (station fine print) ----------
    let mut val_rows = vec!["city,date,winner_bucket,obs_final,obs_display,match".to_string()];
    let mut val_stats: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for ((cslug, date), legs) in &fams {
        let c = city(cslug)?;
        let win = legs.iter().find(|l| l.winner).unwrap();
        let fin = match c.unit {
            Unit::HkFloor => db.hko.get(date).copied().map(|v| (v, true)),
            _ => db.final_max(c, *date),
        };
        let Some((fin, complete)) = fin else { continue };
        if !complete {
            continue;
        }
        let d = display(c.unit, fin);
        let ok = bucket_contains(win.lo, win.hi, d);
        let e = val_stats.entry(cslug.clone()).or_default();
        e.1 += 1;
        if ok {
            e.0 += 1;
        }
        val_rows.push(format!("{cslug},{date},{},{fin:.1},{d},{}", win.title, ok as u8));
    }
    fs::write(out_dir.join("validation.csv"), val_rows.join("\n") + "\n")?;
    println!("== Gate 0: does station obs reproduce the resolved winner? ==");
    for (c, (ok, n)) in &val_stats {
        println!("  {c}: {ok}/{n} ({:.1}%)", 100.0 * *ok as f64 / *n as f64);
    }

    // ---------- Gate 1: window-open calibration ----------
    let mut g1_rows =
        vec!["city,date,open_ts,herfindahl,winner_open_devig,n_legs_priced".to_string()];
    let mut herfs = vec![];
    let mut wopens = vec![];
    let mut per_city: BTreeMap<String, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
    // ---------- Gate 3 accumulators ----------
    let mut g3_rows = vec![
        "city,date,hour,runmax,n_resid,p_mkt_winner,p_model_winner,ll_mkt,ll_model".to_string(),
    ];
    let mut g3_stats: BTreeMap<i64, Vec<(f64, f64)>> = BTreeMap::new(); // hour -> (ll_mkt, ll_model)
    let mut trades_rows = vec![
        "city,date,hour,bucket,side,p_mkt,p_model,won,pnl_gross,pnl_net".to_string(),
    ];
    let mut trade_pnl: BTreeMap<i64, Vec<f64>> = BTreeMap::new(); // hour -> net pnl per trade
    // Robustness sim: model frozen at h:00 obs, but trades executed against the
    // de-vigged mid 15 minutes LATER (book has had time to reprice on the same obs).
    let mut trades2_rows = trades_rows.clone();
    let mut trade2_pnl: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    let mut n_no_series = 0;

    for ((cslug, date), legs) in &fams {
        let c = city(cslug)?;
        let series: Vec<Option<Vec<(i64, f64)>>> =
            legs.iter().map(|l| load_series(data, 10, l)).collect();
        let widx = legs.iter().position(|l| l.winner).unwrap();
        if series[widx].is_none() {
            n_no_series += 1;
            continue;
        }
        // Gate 1: family open = earliest first-timestamp across legs
        let t_open = series
            .iter()
            .flatten()
            .map(|s| s[0].0)
            .min()
            .unwrap();
        let opens: Vec<f64> = series
            .iter()
            .map(|s| {
                s.as_ref()
                    .and_then(|s| s.iter().find(|(t, _)| *t <= t_open + 12 * 3600))
                    .map(|(_, p)| *p)
                    .unwrap_or(0.005)
            })
            .collect();
        let n_priced = series.iter().flatten().filter(|s| s[0].0 <= t_open + 12 * 3600).count();
        let dv = devig(&opens);
        let herf: f64 = dv.iter().map(|p| p * p).sum();
        herfs.push(herf);
        wopens.push(dv[widx]);
        let e = per_city.entry(cslug.clone()).or_default();
        e.0.push(herf);
        e.1.push(dv[widx]);
        g1_rows.push(format!("{cslug},{date},{t_open},{herf:.4},{:.4},{n_priced}", dv[widx]));

        // Gate 3: checkpoints
        let buckets: Vec<(i64, i64)> = legs.iter().map(|l| (l.lo, l.hi)).collect();
        for h in CHECKPOINTS {
            let t = (date.and_hms_opt(0, 0, 0).unwrap() + Duration::hours(h - c.off))
                .and_utc()
                .timestamp();
            let Some(rm) = db.runmax_at(cslug, *date, t) else { continue };
            let mkt_raw: Vec<f64> = series
                .iter()
                .map(|s| s.as_ref().and_then(|s| price_at(s, t, 5400)).unwrap_or(0.005))
                .collect();
            if series[widx].as_ref().and_then(|s| price_at(s, t, 5400)).is_none() {
                continue;
            }
            let mkt = devig(&mkt_raw);
            let rvec = resid.get(&(cslug.clone(), h)).map(|v| v.as_slice()).unwrap_or(&[]);
            let Some(model) = model_probs(c, &buckets, rm, rvec, Some(*date)) else { continue };
            let (pm, pq) = (mkt[widx].max(1e-4), model[widx].max(1e-4));
            g3_rows.push(format!(
                "{cslug},{date},{h},{rm:.1},{},{pm:.4},{pq:.4},{:.4},{:.4}",
                rvec.len().saturating_sub(1),
                -pm.ln(),
                -pq.ln()
            ));
            g3_stats.entry(h).or_default().push((-pm.ln(), -pq.ln()));
            // trading sim: trade legs where model and market disagree by >3c
            for (i, l) in legs.iter().enumerate() {
                let (pm_i, pq_i) = (mkt[i], model[i]);
                let edge = pq_i - pm_i;
                let won = l.winner as u8 as f64;
                let cost = 0.015;
                if edge > 0.03 && pm_i >= 0.02 && pm_i <= 0.95 {
                    let g = won - pm_i;
                    let n = won - (pm_i + cost);
                    trades_rows.push(format!(
                        "{cslug},{date},{h},{},BUY,{pm_i:.3},{pq_i:.3},{won},{g:.3},{n:.3}",
                        l.title
                    ));
                    trade_pnl.entry(h).or_default().push(n);
                } else if edge < -0.03 && pm_i >= 0.03 && pm_i <= 0.95 {
                    let g = pm_i - won;
                    let n = (pm_i - cost) - won;
                    trades_rows.push(format!(
                        "{cslug},{date},{h},{},SELL,{pm_i:.3},{pq_i:.3},{won},{g:.3},{n:.3}",
                        l.title
                    ));
                    trade_pnl.entry(h).or_default().push(n);
                }
            }
            // sim2: same model (obs cutoff h:00), execution vs mid at h:15
            let t2 = t + 900;
            if series[widx].as_ref().and_then(|s| price_at(s, t2, 5400)).is_some() {
                let mkt2_raw: Vec<f64> = series
                    .iter()
                    .map(|s| s.as_ref().and_then(|s| price_at(s, t2, 5400)).unwrap_or(0.005))
                    .collect();
                let mkt2 = devig(&mkt2_raw);
                for (i, l) in legs.iter().enumerate() {
                    let (pm_i, pq_i) = (mkt2[i], model[i]);
                    let edge = pq_i - pm_i;
                    let won = l.winner as u8 as f64;
                    let cost = 0.015;
                    if edge > 0.03 && pm_i >= 0.02 && pm_i <= 0.95 {
                        let n = won - (pm_i + cost);
                        trades2_rows.push(format!(
                            "{cslug},{date},{h},{},BUY,{pm_i:.3},{pq_i:.3},{won},{:.3},{n:.3}",
                            l.title,
                            won - pm_i
                        ));
                        trade2_pnl.entry(h).or_default().push(n);
                    } else if edge < -0.03 && pm_i >= 0.03 && pm_i <= 0.95 {
                        let n = (pm_i - cost) - won;
                        trades2_rows.push(format!(
                            "{cslug},{date},{h},{},SELL,{pm_i:.3},{pq_i:.3},{won},{:.3},{n:.3}",
                            l.title,
                            pm_i - won
                        ));
                        trade2_pnl.entry(h).or_default().push(n);
                    }
                }
            }
        }
    }
    fs::write(out_dir.join("gate1_families.csv"), g1_rows.join("\n") + "\n")?;
    fs::write(out_dir.join("gate3_checkpoints.csv"), g3_rows.join("\n") + "\n")?;
    fs::write(out_dir.join("gate3_trades.csv"), trades_rows.join("\n") + "\n")?;
    fs::write(out_dir.join("gate3_trades_delayed.csv"), trades2_rows.join("\n") + "\n")?;

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!("\n== Gate 1: window-open calibration ({} families, {n_no_series} skipped w/o winner series) ==", herfs.len());
    println!(
        "  mean winner open (devig): {:.4}   mean Herfindahl: {:.4}   diff: {:+.4}",
        mean(&wopens),
        mean(&herfs),
        mean(&wopens) - mean(&herfs)
    );
    for (c, (h, w)) in &per_city {
        println!(
            "  {c}: n={} winner_open {:.3} vs herf {:.3} ({:+.3})",
            h.len(),
            mean(w),
            mean(h),
            mean(w) - mean(h)
        );
    }

    println!("\n== Gate 3: truncated model vs de-vigged market at local checkpoints ==");
    for h in CHECKPOINTS {
        if let Some(v) = g3_stats.get(&h) {
            let lm = mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
            let lq = mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
            let wins = v.iter().filter(|(m, q)| q < m).count();
            let pnl = trade_pnl.get(&h).cloned().unwrap_or_default();
            let pnl2 = trade2_pnl.get(&h).cloned().unwrap_or_default();
            println!(
                "  {h:02}h local: n={:3}  LL market {lm:.4}  LL model {lq:.4}  (model better: {wins}/{})  trades={} avg_net_pnl={:+.4} | delayed-exec trades={} avg_net_pnl={:+.4}",
                v.len(),
                v.len(),
                pnl.len(),
                mean(&pnl),
                pnl2.len(),
                mean(&pnl2)
            );
        }
    }

    // ---------- Gate 2: dead-leg collapse on the 1-minute subset ----------
    let mut g2_rows = vec![
        "city,date,bucket,death_utc,pre_death_mid,collapse_min,avg_mid_2h,tape_sell_notional,tape_sell_shares,tape_sell_trades"
            .to_string(),
    ];
    let mut collapse_mins = vec![];
    let mut avg_mids = vec![];
    let mut sell_notionals = vec![];
    let p1_dir = data.join("prices1");
    if p1_dir.exists() {
        for entry in fs::read_dir(&p1_dir)? {
            let fam = entry?.file_name().to_string_lossy().to_string();
            let (cslug, ds) = fam.rsplit_once('_').unwrap();
            let date = NaiveDate::parse_from_str(ds, "%Y-%m-%d")?;
            let c = city(cslug)?;
            if c.unit == Unit::HkFloor {
                continue; // VHHH is only a proxy; death timing would be unreliable
            }
            let Some(legs) = fams.get(&(cslug.to_string(), date)) else { continue };
            let Some(day) = db.days.get(&(cslug.to_string(), date)) else { continue };
            for l in legs {
                if l.hi == 999 {
                    continue;
                }
                // death = first obs where display(runmax) > hi
                let mut rm = f64::NEG_INFINITY;
                let mut death: Option<i64> = None;
                for (t, v) in day {
                    rm = rm.max(*v);
                    if display(c.unit, rm) > l.hi {
                        death = Some(*t);
                        break;
                    }
                }
                let Some(death) = death else { continue };
                let Some(s1) = load_series(data, 1, l) else { continue };
                let pre = price_at(&s1, death, 7200);
                let collapse = s1
                    .iter()
                    .find(|(t, p)| *t >= death && *p <= 0.015)
                    .map(|(t, _)| (*t - death) as f64 / 60.0);
                let window: Vec<f64> = s1
                    .iter()
                    .filter(|(t, _)| *t >= death && *t <= death + 7200)
                    .map(|(_, p)| *p)
                    .collect();
                let avg_mid = if window.is_empty() { f64::NAN } else { mean(&window) };
                // tape: post-death SELL prints >= 2c (someone actually got out above dust)
                let (mut tn, mut tsh, mut ttr) = (0.0, 0.0, 0);
                if let Some(tape) = load_tape(data, &fam, l) {
                    for tr in &tape {
                        let ts = tr["timestamp"].as_i64().unwrap_or(0);
                        let px = tr["price"].as_f64().unwrap_or(0.0);
                        let sz = tr["size"].as_f64().unwrap_or(0.0);
                        let side = tr["side"].as_str().unwrap_or("");
                        if ts > death && side == "SELL" && px >= 0.02 {
                            tn += px * sz;
                            tsh += sz;
                            ttr += 1;
                        }
                    }
                }
                if let Some(cm) = collapse {
                    collapse_mins.push(cm);
                }
                if avg_mid.is_finite() {
                    avg_mids.push(avg_mid);
                }
                sell_notionals.push(tn);
                g2_rows.push(format!(
                    "{cslug},{date},{},{death},{},{},{avg_mid:.4},{tn:.2},{tsh:.1},{ttr}",
                    l.title,
                    pre.map(|p| format!("{p:.4}")).unwrap_or_default(),
                    collapse.map(|m| format!("{m:.1}")).unwrap_or_default(),
                ));
            }
        }
        fs::write(out_dir.join("gate2_deadlegs.csv"), g2_rows.join("\n") + "\n")?;
        println!("\n== Gate 2: dead-leg collapse (1-min subset, {} dead legs) ==", sell_notionals.len());
        println!(
            "  median collapse-to-<=1.5c: {:.0} min   mean avg-mid death+2h: {:.3}   median: {:.3}",
            median(&mut collapse_mins.clone()),
            mean(&avg_mids),
            median(&mut avg_mids.clone())
        );
        println!(
            "  post-death SELL prints >=2c: total ${:.0} across {} legs (mean ${:.0}/leg, median ${:.0})",
            sell_notionals.iter().sum::<f64>(),
            sell_notionals.len(),
            mean(&sell_notionals),
            median(&mut sell_notionals.clone())
        );
    } else {
        println!("\n== Gate 2 skipped: no prices1/ subset pulled ==");
    }
    println!("\nCSV outputs in {}", out_dir.display());
    Ok(())
}

// ---------- wash (gate 4) ----------

fn cmd_wash(data: &Path, fams: &[String]) -> Result<()> {
    let all = load_legs(data)?;
    for fam in fams {
        let (cslug, ds) = fam.rsplit_once('_').context("fam")?;
        let date = NaiveDate::parse_from_str(ds, "%Y-%m-%d")?;
        let legs = all.get(&(cslug.to_string(), date)).context("family not found")?;
        let mut notional = 0.0;
        let mut shares = 0.0;
        let mut n_trades = 0usize;
        let mut by_wallet: HashMap<String, f64> = HashMap::new();
        let mut wash_notional = 0.0;
        for l in legs {
            let Some(tape) = load_tape(data, fam, l) else { continue };
            let mut per_wallet: HashMap<String, Vec<(i64, String, f64, f64)>> = HashMap::new();
            for t in &tape {
                let px = t["price"].as_f64().unwrap_or(0.0);
                let sz = t["size"].as_f64().unwrap_or(0.0);
                let w = t["proxyWallet"].as_str().unwrap_or("").to_string();
                let side = t["side"].as_str().unwrap_or("").to_string();
                let ts = t["timestamp"].as_i64().unwrap_or(0);
                notional += px * sz;
                shares += sz;
                n_trades += 1;
                *by_wallet.entry(w.clone()).or_default() += px * sz;
                per_wallet.entry(w).or_default().push((ts, side, px, sz));
            }
            // same-wallet buy/sell pairs within 10 min, size ratio within 25%
            for (_, mut ts) in per_wallet {
                ts.sort_by_key(|r| r.0);
                for i in 0..ts.len() {
                    for j in i + 1..ts.len() {
                        if ts[j].0 - ts[i].0 > 600 {
                            break;
                        }
                        if ts[i].1 != ts[j].1 {
                            let ratio = ts[i].3 / ts[j].3;
                            if ratio > 0.8 && ratio < 1.25 {
                                wash_notional += ts[i].2 * ts[i].3 + ts[j].2 * ts[j].3;
                            }
                        }
                    }
                }
            }
        }
        let mut ws: Vec<f64> = by_wallet.values().cloned().collect();
        ws.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let top10: f64 = ws.iter().take(10).sum();
        let headline: f64 = legs.iter().map(|l| l.volume).sum();
        println!("== Gate 4 wash check: {fam} ==");
        println!(
            "  tape: {n_trades} trades, {shares:.0} shares, ${notional:.0} taker notional | headline volumeNum sum ${headline:.0} (ratio {:.2})",
            headline / notional.max(1.0)
        );
        println!(
            "  wallets: {} distinct; top-10 share of notional {:.1}%; top-1 {:.1}%",
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

fn cmd_live(data: &Path, date: NaiveDate, cities: &[String]) -> Result<()> {
    let client = mk_client();
    let db = load_obs_db(data)?;
    let resid = build_residuals(&db);
    let now = Utc::now();
    println!("live @ {} UTC", now.format("%Y-%m-%d %H:%M"));
    let mut pred_rows =
        vec!["market_slug,condition_id,outcome,token_id,probability,market_midpoint,bucket,model_note".to_string()];
    for cslug in cities {
        let c = city(cslug)?;
        let slug = family_slug(cslug, date);
        let ev_txt = get_text(
            &client,
            &format!("https://gamma-api.polymarket.com/events?slug={slug}"),
        )?;
        let live_dir = data.join("events_live");
        fs::create_dir_all(&live_dir)?;
        fs::write(live_dir.join(format!("{cslug}_{date}.json")), &ev_txt)?;
        let v: serde_json::Value = serde_json::from_str(&ev_txt)?;
        let Some(ev) = v.as_array().and_then(|a| a.first()) else {
            println!("  {slug}: NOT FOUND");
            continue;
        };
        let legs = extract_legs_from_event(cslug, date, ev)?;

        // live obs for today (IEM; HK uses VHHH proxy + HKO-residuals)
        let obs_url = format!(
            "https://mesonet.agron.iastate.edu/cgi-bin/request/asos.py?station={}&data=tmpc,tmpf&year1={}&month1={}&day1={}&year2={}&month2={}&day2={}&tz=Etc/UTC&format=onlycomma&missing=M&trace=T&report_type=3,4",
            c.station, date.year(), date.month(), date.day(),
            (date + Duration::days(1)).year(), (date + Duration::days(1)).month(), (date + Duration::days(1)).day()
        );
        let obs_txt = get_text(&client, &obs_url)?;
        let obs_live = data.join("obs_live");
        fs::create_dir_all(&obs_live)?;
        fs::write(obs_live.join(format!("{}_{date}.csv", c.station)), &obs_txt)?;
        let mut rm = f64::NEG_INFINITY;
        let mut last_obs = String::new();
        for line in obs_txt.lines().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() < 4 {
                continue;
            }
            let t = NaiveDateTime::parse_from_str(f[1], "%Y-%m-%d %H:%M")?;
            let local = t + Duration::hours(c.off);
            if local.date() != date {
                continue;
            }
            let val = match c.unit {
                Unit::C | Unit::HkFloor => f[2].parse::<f64>().ok(),
                Unit::F => f[3].parse::<f64>().ok(),
            };
            if let Some(v) = val {
                rm = rm.max(v);
                last_obs = format!("{} UTC {v}", f[1]);
            }
        }
        let local_now = now.naive_utc() + Duration::hours(c.off);
        let h_now = (local_now.time().hour() as i64).clamp(0, 23);
        let buckets: Vec<(i64, i64)> = legs.iter().map(|l| (l.lo, l.hi)).collect();
        let rvec = resid.get(&(cslug.clone(), h_now)).map(|v| v.as_slice()).unwrap_or(&[]);
        let model = if rm.is_finite() {
            model_probs(c, &buckets, rm, rvec, None)
        } else {
            None
        };
        println!(
            "\n== {slug} | local {} (h={h_now}) | runmax[{}] = {} | last obs: {last_obs} | n_resid={} ==",
            local_now.format("%H:%M"),
            c.station,
            if rm.is_finite() { format!("{rm:.1}") } else { "?".into() },
            rvec.len()
        );
        println!(
            "  {:>16} {:>7} {:>7} {:>7} {:>9} {:>7} {:>7}",
            "bucket", "bid", "ask", "mid", "tob$", "model", "edge"
        );
        for (i, l) in legs.iter().enumerate() {
            let book_txt = get_text(
                &client,
                &format!("https://clob.polymarket.com/book?token_id={}", l.token_id),
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
                (Some((b, _)), Some((a, _))) => Some(0.5 * (b + a)),
                _ => None,
            };
            let tob = bid.map(|(p, s)| p * s).unwrap_or(0.0) + ask.map(|(p, s)| p * s).unwrap_or(0.0);
            let mp = model.as_ref().map(|m| m[i]);
            println!(
                "  {:>16} {:>7} {:>7} {:>7} {:>9.0} {:>7} {:>7}",
                l.title,
                bid.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                ask.map(|(p, _)| format!("{p:.3}")).unwrap_or_default(),
                mid.map(|p| format!("{p:.3}")).unwrap_or_default(),
                tob,
                mp.map(|p| format!("{p:.3}")).unwrap_or_default(),
                match (mp, mid) {
                    (Some(q), Some(m)) => format!("{:+.3}", q - m),
                    _ => String::new(),
                }
            );
            if let (Some(q), Some(m)) = (mp, mid) {
                pred_rows.push(format!(
                    "{},{},Yes,{},{:.4},{:.4},{},runmax={rm:.1}@h{h_now}",
                    l.market_slug, l.condition_id, l.token_id, q, m, l.title
                ));
            }
        }
    }
    let pred_path = data.join("out").join(format!("predictions_{date}.csv"));
    fs::create_dir_all(pred_path.parent().unwrap())?;
    fs::write(&pred_path, pred_rows.join("\n") + "\n")?;
    println!("\nprediction rows -> {}", pred_path.display());
    Ok(())
}

// ---------- main ----------

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: runningmax <discover|prices|obs|tape|analyze|wash|live> ...";
    if args.len() < 3 {
        bail!("{usage}");
    }
    let data = PathBuf::from(&args[2]);
    fs::create_dir_all(&data)?;
    let date = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").context("date YYYY-MM-DD");
    match args[1].as_str() {
        "discover" => {
            let cities: Vec<String> = args[5].split(',').map(String::from).collect();
            cmd_discover(&data, date(&args[3])?, date(&args[4])?, &cities)
        }
        "prices" => {
            let fid: u32 = args[3].parse()?;
            let filter = args.get(4).map(|s| s.split(',').map(String::from).collect());
            cmd_prices(&data, fid, filter)
        }
        "obs" => cmd_obs(&data, date(&args[3])?, date(&args[4])?),
        "tape" => cmd_tape(&data, &args[3..].to_vec()),
        "analyze" => cmd_analyze(&data),
        "wash" => cmd_wash(&data, &args[3..].to_vec()),
        "live" => {
            let cities: Vec<String> = args[4].split(',').map(String::from).collect();
            cmd_live(&data, date(&args[3])?, &cities)
        }
        _ => bail!("{usage}"),
    }
}
