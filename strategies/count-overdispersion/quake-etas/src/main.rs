// count-overdispersion/quake-etas — weekly USGS earthquake-count bucket ladders on Polymarket.
//
// Subcommands (data_dir lives OUTSIDE git; frozen to R2 as a tarball):
//   boards   <dir>            parse raw/events/*.json -> boards.json (window in UTC, lattice, legs)
//   gate0    <dir>            recount from the USGS catalogue vs how each board resolved
//   revision <dir> [n]        emit the ComCat ids to pull version history for (threshold-adjacent)
//   revfit   <dir>            fit the magnitude-revision layer from raw/detail/*.json
//   gate3    <dir>            fee + book + fundability: where does the edge actually live?
//   etas     <dir>            fit ETAS by MLE, integrate the posterior, simulate every board
//   live     <dir> <slug,..>  books for open boards -> prediction rows
//
// Fine print baked in (read off the boards' own descriptions, 2026-07-25):
//   - window is stated in ET; 2026 EDT runs Mar 8 .. Nov 1 (UTC-4), otherwise UTC-5.
//   - windows are NOT all 7 days: e.g. "July 14 .. July 19" is 6 days. Parse, never assume.
//   - "If a qualifying earthquake has been recorded on the final day, this market may remain
//     open for 24 hours to allow for revisions to the recorded magnitude." -> the resolving
//     vintage is the catalogue ~24-48h after window close, NOT today's catalogue.
//   - feeSchedule {rate: 0.05, exponent: 1, takerOnly: true, rebateRate: 0.25} (weather_fees).

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------- small stats ----------------

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}
fn sd(v: &[f64]) -> f64 {
    if v.len() < 2 { return f64::NAN; }
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}
fn se(v: &[f64]) -> f64 { sd(v) / (v.len() as f64).sqrt() }

// ---------------- time ----------------

/// 2026 US DST: Mar 8 .. Nov 1. ET offset in hours (negative).
fn et_offset_hours(y: i32, m: u32, d: u32) -> i64 {
    // second Sunday in March .. first Sunday in November
    let dst_start = nth_weekday(y, 3, chrono::Weekday::Sun, 2);
    let dst_end = nth_weekday(y, 11, chrono::Weekday::Sun, 1);
    let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
    if date >= dst_start && date < dst_end { -4 } else { -5 }
}

fn nth_weekday(y: i32, m: u32, wd: chrono::Weekday, n: u32) -> NaiveDate {
    let mut d = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
    let mut c = 0;
    loop {
        if d.weekday() == wd {
            c += 1;
            if c == n { return d; }
        }
        d = d.succ_opt().unwrap();
    }
}

fn et_to_utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> i64 {
    let off = et_offset_hours(y, mo, d);
    let naive = NaiveDate::from_ymd_opt(y, mo, d).unwrap().and_hms_opt(h, mi, 0).unwrap();
    Utc.from_utc_datetime(&naive).timestamp() - off * 3600
}

fn month_num(s: &str) -> Option<u32> {
    let s = s.to_lowercase();
    let months = ["january","february","march","april","may","june","july","august","september","october","november","december"];
    months.iter().position(|m| m.starts_with(&s[..s.len().min(3)]) && s.len() >= 3).map(|i| i as u32 + 1)
}

fn iso(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0).unwrap().format("%Y-%m-%dT%H:%MZ").to_string()
}

fn parse_iso(s: &str) -> Option<i64> {
    let s = s.trim();
    let core = s.split('.').next().unwrap().trim_end_matches('Z');
    NaiveDateTime::parse_from_str(core, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|n| Utc.from_utc_datetime(&n).timestamp())
}

// ---------------- board model ----------------

#[derive(Clone, Debug)]
struct Leg {
    label: String,
    condition_id: String,
    token_yes: String,
    volume: f64,
    lo: i32,
    hi: i32, // inclusive; 9999 = open top
    won: Option<bool>,
}

#[derive(Clone, Debug)]
struct Board {
    slug: String,
    threshold: f64,
    win_start: i64,
    win_end: i64,
    created: i64,
    closed: bool,
    volume: f64,
    legs: Vec<Leg>,
}

impl Board {
    fn days(&self) -> f64 { (self.win_end - self.win_start) as f64 / 86400.0 }
    fn family(&self) -> String { format!("M{:.1}+", self.threshold) }
    /// index of the leg containing count n
    fn leg_of(&self, n: i32) -> Option<usize> {
        self.legs.iter().position(|l| n >= l.lo && n <= l.hi)
    }
    fn winner(&self) -> Option<usize> { self.legs.iter().position(|l| l.won == Some(true)) }
}

fn jstr(v: &serde_json::Value, k: &str) -> String {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
}
fn jnum(v: &serde_json::Value, k: &str) -> f64 {
    v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

/// "between July 14, 2026, 12:00 AM ET, and July 19, 2026, 11:59 PM ET"
fn parse_window(desc: &str) -> Option<(i64, i64)> {
    let d = desc.replace('\u{a0}', " ");
    let start = d.find("between ")? + 8;
    let rest = &d[start..];
    let and_pos = rest.find(" and ")?;
    let a = &rest[..and_pos];
    let b_full = &rest[and_pos + 5..];
    let b_end = b_full.find(" ET").map(|i| i + 3)?;
    let b = &b_full[..b_end];
    let end = parse_et_stamp(b)?;
    // Some boards omit the year on the START stamp ("between December 22, 12:00 AM ET,
    // and December 28, 2025, ..."). Borrow the year from the end stamp.
    let start = match parse_et_stamp(a) {
        Some(s) => s,
        None => {
            let yr = DateTime::<Utc>::from_timestamp(end, 0)?.year();
            let parts: Vec<&str> = a.splitn(2, ',').collect();
            if parts.len() < 2 { return None; }
            parse_et_stamp(&format!("{}, {},{}", parts[0], yr, parts[1]))?
        }
    };
    Some((start, end))
}

/// "July 14, 2026, 12:00 AM ET"
fn parse_et_stamp(s: &str) -> Option<i64> {
    let s = s.trim().trim_end_matches(',').trim();
    let parts: Vec<&str> = s.split(',').map(|x| x.trim()).collect();
    if parts.len() < 3 { return None; }
    let md: Vec<&str> = parts[0].split_whitespace().collect();
    let mo = month_num(md.first()?)?;
    let day: u32 = md.get(1)?.trim_end_matches(',').parse().ok()?;
    let year: i32 = parts[1].parse().ok()?;
    let tp: Vec<&str> = parts[2].split_whitespace().collect();
    let hm: Vec<&str> = tp.first()?.split(':').collect();
    let mut h: u32 = hm.first()?.parse().ok()?;
    let mi: u32 = hm.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
    let ampm = tp.get(1).map(|x| x.to_uppercase()).unwrap_or_default();
    if ampm == "PM" && h != 12 { h += 12; }
    if ampm == "AM" && h == 12 { h = 0; }
    Some(et_to_utc(year, mo, day, h, mi))
}

fn parse_lattice(label: &str) -> Option<(i32, i32)> {
    let t = label.trim().replace('≤', "<=").replace('≥', ">=");
    if let Some(r) = t.strip_prefix("<=") { return r.trim().parse().ok().map(|n| (0, n)); }
    if let Some(r) = t.strip_prefix('<') { return r.trim().parse::<i32>().ok().map(|n| (0, n - 1)); }
    if let Some(r) = t.strip_prefix(">=") { return r.trim().parse().ok().map(|n| (n, 9999)); }
    if let Some(r) = t.strip_prefix('>') { return r.trim().parse::<i32>().ok().map(|n| (n + 1, 9999)); }
    t.parse().ok().map(|n| (n, n))
}

fn load_boards(dir: &Path) -> Result<Vec<Board>> {
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> = fs::read_dir(dir.join("raw/events"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    files.sort();
    for f in files {
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&f)?)?;
        let arr = v.as_array().ok_or_else(|| anyhow!("not array {:?}", f))?;
        if arr.is_empty() { continue; }
        let e = &arr[0];
        let slug = jstr(e, "slug");
        let desc = jstr(e, "description");
        let threshold = if slug.contains("5pt5") || slug.contains("5-5") { 5.5 } else { 6.5 };
        let (ws, we) = match parse_window(&desc) {
            Some(x) => x,
            None => { eprintln!("WARN unparsed window: {}", slug); continue; }
        };
        let created = parse_iso(&jstr(e, "startDate")).unwrap_or(0);
        let mut legs = Vec::new();
        for m in e.get("markets").and_then(|x| x.as_array()).unwrap_or(&vec![]) {
            let label = jstr(m, "groupItemTitle");
            let (lo, hi) = match parse_lattice(&label) {
                Some(x) => x,
                None => { eprintln!("WARN lattice {} {}", slug, label); continue; }
            };
            let toks: Vec<String> = serde_json::from_str(&jstr(m, "clobTokenIds")).unwrap_or_default();
            let outs: Vec<String> = serde_json::from_str(&jstr(m, "outcomes")).unwrap_or_default();
            let prices: Vec<String> = serde_json::from_str(&jstr(m, "outcomePrices")).unwrap_or_default();
            if toks.len() < 2 || outs.len() < 2 { continue; }
            let yi = outs.iter().position(|o| o == "Yes").unwrap_or(0);
            let won = if m.get("closed").and_then(|x| x.as_bool()).unwrap_or(false) && prices.len() > yi {
                prices[yi].parse::<f64>().ok().map(|p| p > 0.5)
            } else { None };
            legs.push(Leg {
                label,
                condition_id: jstr(m, "conditionId"),
                token_yes: toks[yi].clone(),
                volume: jnum(m, "volumeNum"),
                lo, hi, won,
            });
        }
        legs.sort_by_key(|l| l.lo);
        out.push(Board {
            slug,
            threshold,
            win_start: ws,
            win_end: we + 60, // "11:59 PM" means through 23:59:59
            created,
            closed: e.get("closed").and_then(|x| x.as_bool()).unwrap_or(false),
            volume: jnum(e, "volume"),
            legs,
        });
    }
    out.sort_by_key(|b| b.win_start);
    Ok(out)
}

// ---------------- USGS catalogue ----------------

#[derive(Clone, Debug)]
struct Quake { t: i64, mag: f64, id: String, is_eq: bool }

fn load_catalogue(dir: &Path) -> Result<Vec<Quake>> {
    let txt = fs::read_to_string(dir.join("usgs_m45.csv")).context("usgs_m45.csv")?;
    let mut out = Vec::new();
    let mut lines = txt.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let (it, imag, iid, ityp) = (
        header.iter().position(|h| *h == "time").unwrap(),
        header.iter().position(|h| *h == "mag").unwrap(),
        header.iter().position(|h| *h == "id").unwrap(),
        header.iter().position(|h| *h == "type").unwrap(),
    );
    for line in lines {
        let f = split_csv(line);
        if f.len() <= ityp { continue; }
        let t = match parse_iso(&f[it]) { Some(t) => t, None => continue };
        let mag: f64 = match f[imag].parse() { Ok(m) => m, Err(_) => continue };
        out.push(Quake { t, mag, id: f[iid].clone(), is_eq: f[ityp] == "earthquake" });
    }
    out.sort_by_key(|q| q.t);
    Ok(out)
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    for c in line.chars() {
        match c {
            '"' => q = !q,
            ',' if !q => { out.push(std::mem::take(&mut cur)); }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn count_in(cat: &[Quake], t0: i64, t1: i64, thr: f64, eq_only: bool) -> i32 {
    cat.iter().filter(|q| q.t >= t0 && q.t <= t1 && q.mag >= thr - 1e-9 && (!eq_only || q.is_eq)).count() as i32
}

// ---------------- CLOB price series ----------------

fn load_series(dir: &Path, board: &str, token: &str) -> Vec<(i64, f64)> {
    let p = dir.join("raw/clob").join(format!("{}__{}.json", board, token));
    let txt = match fs::read_to_string(&p) { Ok(t) => t, Err(_) => return vec![] };
    let v: serde_json::Value = match serde_json::from_str(&txt) { Ok(v) => v, Err(_) => return vec![] };
    let mut out: Vec<(i64, f64)> = v.get("history").and_then(|h| h.as_array()).map(|a| {
        a.iter().filter_map(|r| Some((r.get("t")?.as_i64()?, r.get("p")?.as_f64()?))).collect()
    }).unwrap_or_default();
    out.sort_by_key(|x| x.0);
    out
}

/// last observation at or before `t`, if it is not staler than `max_stale` seconds
fn price_at(s: &[(i64, f64)], t: i64, max_stale: i64) -> Option<f64> {
    let mut best: Option<(i64, f64)> = None;
    for &(ts, p) in s {
        if ts <= t { best = Some((ts, p)); } else { break; }
    }
    best.filter(|(ts, _)| t - ts <= max_stale).map(|(_, p)| p)
}

fn total_variation(s: &[(i64, f64)], t0: i64, t1: i64) -> f64 {
    let mut tv = 0.0;
    let mut prev: Option<f64> = None;
    for &(ts, p) in s {
        if ts < t0 || ts > t1 { continue; }
        if let Some(q) = prev { tv += (p - q).abs(); }
        prev = Some(p);
    }
    tv
}

// ---------------- count models ----------------

/// Times (sorted) of catalogue events at or above `thr`.
fn times_above(cat: &[Quake], thr: f64, eq_only: bool) -> Vec<i64> {
    cat.iter().filter(|q| q.mag >= thr - 1e-9 && (!eq_only || q.is_eq)).map(|q| q.t).collect()
}

fn count_between(ts: &[i64], t0: i64, t1: i64) -> i32 {
    let a = ts.partition_point(|&x| x < t0);
    let b = ts.partition_point(|&x| x <= t1);
    (b - a) as i32
}

/// Empirical marginal of the count in a window of `days` days, from sliding daily windows
/// that END strictly before `before` (out-of-sample by construction).
fn empirical_counts(ts: &[i64], days: f64, before: i64, from: i64) -> Vec<i32> {
    let w = (days * 86400.0) as i64;
    let mut out = Vec::new();
    let mut t = from;
    while t + w < before {
        out.push(count_between(ts, t, t + w));
        t += 86400;
    }
    out
}

/// Turn a sample of counts into a distribution over the board's lattice (Laplace-smoothed).
fn lattice_dist(board: &Board, sample: &[i32]) -> Vec<f64> {
    let k = board.legs.len();
    let mut c = vec![0.5f64; k]; // Laplace
    for &n in sample {
        if let Some(i) = board.leg_of(n) { c[i] += 1.0; }
    }
    let s: f64 = c.iter().sum();
    c.iter().map(|x| x / s).collect()
}

fn poisson_lattice(board: &Board, lambda: f64) -> Vec<f64> {
    let mut pmf = Vec::new();
    let mut p = (-lambda).exp();
    for n in 0..200 {
        pmf.push(p);
        p *= lambda / (n as f64 + 1.0);
    }
    let mut out = vec![0.0; board.legs.len()];
    for (n, pn) in pmf.iter().enumerate() {
        if let Some(i) = board.leg_of(n as i32) { out[i] += pn; }
    }
    let s: f64 = out.iter().sum();
    out.iter().map(|x| x / s.max(1e-12)).collect()
}

fn devig(p: &[Option<f64>]) -> Option<Vec<f64>> {
    if p.iter().any(|x| x.is_none()) { return None; }
    let v: Vec<f64> = p.iter().map(|x| x.unwrap().clamp(0.0005, 0.9995)).collect();
    let s: f64 = v.iter().sum();
    if s <= 0.0 { return None; }
    Some(v.iter().map(|x| x / s).collect())
}

fn fee(p: f64) -> f64 { 0.05 * p * (1.0 - p) }

// ---------------- commands ----------------

fn cmd_boards(dir: &Path) -> Result<()> {
    let boards = load_boards(dir)?;
    println!("{} boards parsed\n", boards.len());
    println!("{:<58} {:>6} {:>18} {:>18} {:>5} {:>4} {:>10} {}", "slug", "thr", "window start UTC", "window end UTC", "days", "legs", "volume", "lattice");
    for b in &boards {
        let lat: Vec<String> = b.legs.iter().map(|l| l.label.clone()).collect();
        println!("{:<58} {:>6.1} {:>18} {:>18} {:>5.2} {:>4} {:>10.0} {}",
            &b.slug[..b.slug.len().min(58)], b.threshold, iso(b.win_start), iso(b.win_end),
            b.days(), b.legs.len(), b.volume, lat.join(","));
    }
    Ok(())
}

fn cmd_gate0(dir: &Path) -> Result<()> {
    let boards = load_boards(dir)?;
    let cat = load_catalogue(dir)?;
    println!("GATE 0 — reproduce every resolved board from the USGS catalogue (today's vintage)\n");
    let mut per_family: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut misses: Vec<(String, i32, i32, f64)> = Vec::new();
    println!("{:<58} {:>5} {:>7} {:>7} {:>7} {:>6} {:>8}", "board", "thr", "winner", "count", "count*", "match", "knife");
    for b in &boards {
        if !b.closed { continue; }
        let w = match b.winner() { Some(w) => w, None => continue };
        let n_eq = count_in(&cat, b.win_start, b.win_end, b.threshold, true);
        let n_all = count_in(&cat, b.win_start, b.win_end, b.threshold, false);
        let wl = &b.legs[w];
        let ok = n_eq >= wl.lo && n_eq <= wl.hi;
        let e = per_family.entry(b.family()).or_insert((0, 0));
        e.1 += 1;
        if ok { e.0 += 1; }
        // knife-edge: events within +-0.05 of the threshold in this window
        let knife = cat.iter().filter(|q| q.t >= b.win_start && q.t <= b.win_end && q.is_eq
            && (q.mag - b.threshold).abs() < 0.051).count();
        if !ok { misses.push((b.slug.clone(), n_eq, wl.lo, b.threshold)); }
        println!("{:<58} {:>5.1} {:>7} {:>7} {:>7} {:>6} {:>8}",
            &b.slug[..b.slug.len().min(58)], b.threshold, wl.label, n_eq, n_all,
            if ok { "OK" } else { "MISS" }, knife);
    }
    println!();
    for (f, (ok, n)) in &per_family {
        println!("{:<8} reproduced {}/{} ({:.0}%)", f, ok, n, 100.0 * *ok as f64 / *n as f64);
    }
    println!("\nMisses:");
    for (s, n, lo, thr) in &misses {
        println!("  {} recount={} winner_leg_lo={} thr={}", s, n, lo, thr);
    }
    // threshold-adjacency census
    for thr in [5.5f64, 6.5] {
        let exact = cat.iter().filter(|q| q.is_eq && q.t > et_to_utc(2024,1,1,0,0) && (q.mag - thr).abs() < 1e-9).count();
        let near = cat.iter().filter(|q| q.is_eq && q.t > et_to_utc(2024,1,1,0,0) && (q.mag - thr).abs() <= 0.1001).count();
        let weeks = (Utc::now().timestamp() - et_to_utc(2024,1,1,0,0)) as f64 / (7.0*86400.0);
        println!("\nM{:.1}: exactly-at-threshold {:.2}/week, within +-0.10 {:.2}/week (since 2024)", thr, exact as f64/weeks, near as f64/weeks);
    }
    Ok(())
}

/// Emit ComCat ids near the thresholds so their version history can be pulled.
fn cmd_revision(dir: &Path, n: usize) -> Result<()> {
    let cat = load_catalogue(dir)?;
    let since = et_to_utc(2022, 1, 1, 0, 0);
    let mut ids: Vec<&Quake> = cat.iter()
        .filter(|q| q.is_eq && q.t > since
            && ((q.mag - 5.5).abs() <= 0.25 || (q.mag - 6.5).abs() <= 0.35))
        .collect();
    // spread the sample over time
    let step = (ids.len() / n.max(1)).max(1);
    ids = ids.into_iter().step_by(step).collect();
    for q in ids.iter().take(n) { println!("{} {} {}", q.id, q.mag, q.t); }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { bail!("usage: quakeetas <cmd> <data_dir> [...]"); }
    let dir = PathBuf::from(&args[2]);
    match args[1].as_str() {
        "boards" => cmd_boards(&dir),
        "gate0" => cmd_gate0(&dir),
        "revision" => cmd_revision(&dir, args.get(3).and_then(|x| x.parse().ok()).unwrap_or(400)),
        "gate3" => crate::gate3::run(&dir),
        "models" => crate::models::run(&dir, args.get(3).and_then(|x| x.parse().ok()).unwrap_or(6),
                                       args.get(4).map(|s| s.as_str()).unwrap_or("open")),
        "ceiling" => crate::models::ceiling(&dir),
        "etas" => crate::etas::run(&dir, &args[3..]),
        "revfit" => crate::revfit::run(&dir),
        "live" => crate::live::run(&dir, args.get(3).map(|s| s.as_str()).unwrap_or("")),
        other => bail!("unknown command {}", other),
    }
}

// =====================================================================================
// GATE 3 — after the 0.05 taker fee and a book-quality gate, does tradeable edge remain
// in FUNDABLE legs (>=3c)?  This is the gate that decides the trial, so it is run before
// the ETAS engine exists, against a model that cannot be accused of being fitted: the
// out-of-sample empirical marginal (which carries the whole overdispersion claim) and a
// prior-activity-conditioned version of it (the "crude benchmark" of the idea file).
// =====================================================================================
mod gate3 {
    use super::*;

    pub struct Row {
        pub board: String,
        pub family: String,
        pub leg: String,
        pub mid_open: f64,
        pub mid_dv: f64,     // de-vigged
        pub mid_fill: f64,   // mid at open+30h (delayed fill)
        pub model: f64,
        pub poisson: f64,
        pub won: bool,
        pub volume: f64,
        pub tv: f64,         // total variation of the leg's price over its life
        pub lo: i32,
        pub hi: i32,
    }

    pub fn build(dir: &Path) -> Result<Vec<Row>> {
        let boards = load_boards(dir)?;
        let cat = load_catalogue(dir)?;
        let cat_from = et_to_utc(1990, 1, 1, 0, 0);
        let ts55 = times_above(&cat, 5.5, true);
        let ts65 = times_above(&cat, 6.5, true);
        let mut rows = Vec::new();
        for b in &boards {
            if !b.closed || b.winner().is_none() { continue; }
            let ck = b.win_start + 6 * 3600;
            let fill_t = b.win_start + 30 * 3600;
            let series: Vec<Vec<(i64, f64)>> = b.legs.iter().map(|l| load_series(dir, &b.slug, &l.token_yes)).collect();
            let mids: Vec<Option<f64>> = series.iter().map(|s| price_at(s, ck, 12 * 3600)).collect();
            let fills: Vec<Option<f64>> = series.iter().map(|s| price_at(s, fill_t, 12 * 3600)).collect();
            let dv = match devig(&mids) { Some(d) => d, None => continue };
            // out-of-sample empirical model: only windows ending before this board opened
            let ts = if b.threshold < 6.0 { &ts55 } else { &ts65 };
            let sample = empirical_counts(ts, b.days(), b.win_start, cat_from);
            let model = lattice_dist(b, &sample);
            let lam = sample.iter().map(|&x| x as f64).sum::<f64>() / sample.len() as f64;
            let pois = poisson_lattice(b, lam);
            for (i, l) in b.legs.iter().enumerate() {
                rows.push(Row {
                    board: b.slug.clone(),
                    family: b.family(),
                    leg: l.label.clone(),
                    mid_open: mids[i].unwrap(),
                    mid_dv: dv[i],
                    mid_fill: fills[i].unwrap_or(mids[i].unwrap()),
                    model: model[i],
                    poisson: pois[i],
                    won: l.won == Some(true),
                    volume: l.volume,
                    tv: total_variation(&series[i], b.created, b.win_end),
                    lo: l.lo, hi: l.hi,
                });
            }
        }
        Ok(rows)
    }

    fn logloss(p: f64) -> f64 { -(p.clamp(1e-6, 1.0)).ln() }

    pub fn run(dir: &Path) -> Result<()> {
        let rows = build(dir)?;
        let boards: Vec<String> = {
            let mut v: Vec<String> = rows.iter().map(|r| r.board.clone()).collect();
            v.dedup(); v
        };
        println!("GATE 3 — fee/book/fundability. {} legs on {} resolved boards.\n", rows.len(), boards.len());

        // ---- 0. book state (phantom-midpoint re-verification, cheap) ----
        let dead = rows.iter().filter(|r| r.tv < 1e-9).count();
        let flat = rows.iter().filter(|r| r.tv < 0.02).count();
        let mut tvs: Vec<f64> = rows.iter().map(|r| r.tv).collect();
        tvs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("book state: DEAD {}/{} ({:.1}%)  near-flat(<2c) {} ({:.1}%)  median total variation {:.2}",
            dead, rows.len(), 100.0*dead as f64/rows.len() as f64, flat, 100.0*flat as f64/rows.len() as f64,
            tvs[tvs.len()/2]);

        // ---- 1. is the CROWD actually Poisson-shaped? ----
        println!("\n--- Is the market's implied distribution Poisson, or already overdispersed? ---");
        for fam in ["M5.5+", "M6.5+"] {
            let fr: Vec<&Row> = rows.iter().filter(|r| r.family == fam).collect();
            if fr.is_empty() { continue; }
            let nb = fr.iter().map(|r| r.board.clone()).collect::<std::collections::HashSet<_>>().len();
            let mkt_ll: f64 = mean(&fr.iter().filter(|r| r.won).map(|r| logloss(r.mid_dv)).collect::<Vec<_>>());
            let emp_ll: f64 = mean(&fr.iter().filter(|r| r.won).map(|r| logloss(r.model)).collect::<Vec<_>>());
            let poi_ll: f64 = mean(&fr.iter().filter(|r| r.won).map(|r| logloss(r.poisson)).collect::<Vec<_>>());
            println!("{}  n={} boards   log-loss: market {:.3}  empirical {:.3}  Poisson {:.3}", fam, nb, mkt_ll, emp_ll, poi_ll);
            // mean market price and mean model price per lattice position type
            println!("   {:<10} {:>8} {:>8} {:>8} {:>8} {:>7}", "leg", "mkt(dv)", "empirical", "Poisson", "realised", "n");
            let mut by_leg: BTreeMap<String, Vec<&Row>> = BTreeMap::new();
            for r in &fr { by_leg.entry(r.leg.clone()).or_default().push(r); }
            let mut keys: Vec<&String> = by_leg.keys().collect();
            keys.sort_by_key(|k| by_leg[*k][0].lo);
            for k in keys {
                let g = &by_leg[k];
                let m = mean(&g.iter().map(|r| r.mid_dv).collect::<Vec<_>>());
                let e = mean(&g.iter().map(|r| r.model).collect::<Vec<_>>());
                let p = mean(&g.iter().map(|r| r.poisson).collect::<Vec<_>>());
                let w = g.iter().filter(|r| r.won).count() as f64 / g.len() as f64;
                println!("   {:<10} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>7}", k, m, e, p, w, g.len());
            }
        }

        // ---- 1b. the one-line version: the implied Fano factor of each distribution ----
        println!("\n--- Implied Fano factor (var/mean) of the distribution each side is pricing ---");
        println!("    (Poisson = 1.00 by definition; the thesis says the market should be at 1.00)");
        for fam in ["M5.5+", "M6.5+"] {
            let mut boards_seen: BTreeMap<String, Vec<&Row>> = BTreeMap::new();
            for r in rows.iter().filter(|r| r.family == fam) { boards_seen.entry(r.board.clone()).or_default().push(r); }
            let mut fanos = [Vec::new(), Vec::new(), Vec::new()];
            for legs in boards_seen.values() {
                for (k, get) in [0usize, 1, 2].iter().zip([
                    (|r: &Row| r.mid_dv) as fn(&Row) -> f64,
                    |r: &Row| r.model,
                    |r: &Row| r.poisson]) {
                    let (mut m1, mut m2) = (0.0, 0.0);
                    for r in legs.iter() {
                        // representative count of a bucket: exact legs -> lo; open legs -> lo + 0.5
                        let x = if r.hi >= 9999 { r.lo as f64 + 0.5 } else if r.lo == 0 && r.hi > 0 { r.hi as f64 - 0.5 } else { r.lo as f64 };
                        let p = get(r);
                        m1 += p * x; m2 += p * x * x;
                    }
                    let v = m2 - m1 * m1;
                    if m1 > 0.0 { fanos[*k].push(v / m1); }
                }
            }
            println!("  {}  market(de-vig) {:.3}   empirical {:.3}   Poisson {:.3}   (n={} boards)",
                fam, mean(&fanos[0]), mean(&fanos[1]), mean(&fanos[2]), fanos[0].len());
        }

        // ---- 2. WHERE does the edge live: by de-vigged price bucket ----
        println!("\n--- Where does the model/market disagreement live?  (by market price) ---");
        println!("{:<14} {:>5} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}", "price band", "n", "mkt", "model", "realised", "raw edge", "fee@fill", "net EV");
        let bands: Vec<(f64, f64, &str)> = vec![
            (0.00, 0.01, "<1c"), (0.01, 0.03, "1-3c"), (0.03, 0.05, "3-5c"), (0.05, 0.10, "5-10c"),
            (0.10, 0.20, "10-20c"), (0.20, 0.40, "20-40c"), (0.40, 0.70, "40-70c"), (0.70, 1.01, ">70c")];
        for (lo, hi, name) in &bands {
            let g: Vec<&Row> = rows.iter().filter(|r| r.mid_open >= *lo && r.mid_open < *hi).collect();
            if g.is_empty() { continue; }
            let m = mean(&g.iter().map(|r| r.mid_dv).collect::<Vec<_>>());
            let md = mean(&g.iter().map(|r| r.model).collect::<Vec<_>>());
            let w = g.iter().filter(|r| r.won).count() as f64 / g.len() as f64;
            let raw = mean(&g.iter().map(|r| r.model - r.mid_dv).collect::<Vec<_>>());
            let f = mean(&g.iter().map(|r| fee(r.mid_open)).collect::<Vec<_>>());
            // realised net EV of the model's directional trade at the open mid, fee only
            let ev: Vec<f64> = g.iter().map(|r| {
                let dirn = if r.model > r.mid_dv { 1.0 } else { -1.0 };
                let px = r.mid_open;
                let payoff = if r.won { 1.0 } else { 0.0 };
                dirn * (payoff - px) - fee(px)
            }).collect();
            println!("{:<14} {:>5} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4}", name, g.len(), m, md, w, raw, f, mean(&ev));
        }

        // ---- 3. THE GATE: fundable (>=3c) vs unfundable wings, full cost stack ----
        println!("\n--- GATE 3 proper: trade rule = |model - devig mid| > thresh, fill at open+30h with 2c adverse, taker fee ---");
        for min_px in [0.0f64, 0.03] {
            for edge_thr in [0.02f64, 0.03, 0.05] {
                let mut pnl: Vec<f64> = Vec::new();
                let mut n_trades = 0;
                for r in &rows {
                    if r.mid_open < min_px || r.mid_open > 1.0 - min_px { continue; }
                    let e = r.model - r.mid_dv;
                    if e.abs() < edge_thr { continue; }
                    let dirn = if e > 0.0 { 1.0 } else { -1.0 };
                    // delayed fill at open+30h, 2c adverse
                    let px = (r.mid_fill + dirn * 0.02).clamp(0.005, 0.995);
                    let payoff = if r.won { 1.0 } else { 0.0 };
                    pnl.push(dirn * (payoff - px) - fee(px));
                    n_trades += 1;
                }
                if n_trades == 0 { println!("  price>={:.2} edge>{:.2}: no trades", min_px, edge_thr); continue; }
                println!("  price>={:.2}c edge>{:.0}c: n={:>4}  net {:+.4}/share (se {:.4})  t={:+.2}",
                    min_px*100.0, edge_thr*100.0, n_trades, mean(&pnl), se(&pnl), mean(&pnl)/se(&pnl));
            }
        }

        // fundable vs unfundable split at a fixed rule
        println!("\n  fundable/unfundable split (edge>3c rule, delayed fill +2c adverse, fee):");
        for (lo, hi, name) in [(0.0, 0.03, "wings <3c"), (0.03, 1.0, "fundable >=3c")] {
            let mut pnl = Vec::new();
            let mut gross = Vec::new();
            for r in &rows {
                if r.mid_open < lo || r.mid_open >= hi { continue; }
                let e = r.model - r.mid_dv;
                if e.abs() < 0.03 { continue; }
                let dirn = if e > 0.0 { 1.0 } else { -1.0 };
                let px = (r.mid_fill + dirn * 0.02).clamp(0.005, 0.995);
                let payoff = if r.won { 1.0 } else { 0.0 };
                pnl.push(dirn * (payoff - px) - fee(px));
                gross.push(dirn * (payoff - r.mid_open));
            }
            if pnl.is_empty() { println!("    {:<16} no trades", name); continue; }
            println!("    {:<16} n={:>4}  gross {:+.4}  net {:+.4}/share (se {:.4}) t={:+.2}",
                name, pnl.len(), mean(&gross), mean(&pnl), se(&pnl), mean(&pnl)/se(&pnl));
        }
        Ok(())
    }
}

// =====================================================================================
// MODELS — out-of-sample log-loss of every candidate distribution against the market's
// own de-vigged prices, board by board. This is the test the idea's headline number
// (+0.110 log-loss at window-open, crude benchmark, n=22) has to survive.
// =====================================================================================
mod models {
    use super::*;

    fn ll(p: f64) -> f64 { -(p.clamp(1e-6, 1.0)).ln() }

    /// negative binomial on the lattice, matched to (mean, var); falls back to Poisson.
    fn nb_lattice(board: &Board, m: f64, v: f64) -> Vec<f64> {
        if v <= m * 1.001 { return poisson_lattice(board, m); }
        let r = m * m / (v - m);
        let p = r / (r + m); // P(X=0) = p^r
        let mut pmf = Vec::with_capacity(300);
        let mut cur = p.powf(r);
        for n in 0..300 {
            pmf.push(cur);
            cur *= (r + n as f64) / (n as f64 + 1.0) * (1.0 - p);
        }
        let mut out = vec![0.0; board.legs.len()];
        for (n, pn) in pmf.iter().enumerate() {
            if let Some(i) = board.leg_of(n as i32) { out[i] += pn; }
        }
        let s: f64 = out.iter().sum();
        out.iter().map(|x| x / s.max(1e-12)).collect()
    }

    /// How much of next week's count is predictable from the past at all?  This bounds
    /// anything ETAS can win by conditioning on state.
    pub fn ceiling(dir: &Path) -> Result<()> {
        let cat = load_catalogue(dir)?;
        println!("PREDICTABILITY CEILING — global weekly counts, 1990-2026\n");
        for thr in [5.5f64, 6.5] {
            let ts = times_above(&cat, thr, true);
            let t0 = et_to_utc(1990, 1, 8, 0, 0);
            let t1 = Utc::now().timestamp() - 7 * 86400;
            let (mut prior, mut next) = (Vec::new(), Vec::new());
            let mut t = t0;
            while t < t1 {
                prior.push(count_between(&ts, t - 7 * 86400, t) as f64);
                next.push(count_between(&ts, t, t + 7 * 86400) as f64);
                t += 7 * 86400; // non-overlapping
            }
            let mp = mean(&prior); let mn = mean(&next);
            let cov: f64 = prior.iter().zip(next.iter()).map(|(a, b)| (a - mp) * (b - mn)).sum::<f64>() / (prior.len() - 1) as f64;
            let r = cov / (sd(&prior) * sd(&next));
            let vn: f64 = sd(&next).powi(2);
            println!("M{:.1}+  n={} weeks  mean {:.3}  var {:.3}  Fano {:.2}  lag-1 corr {:+.3}  R^2 {:.4}",
                thr, next.len(), mn, vn, vn / mn, r, r * r);
            // how much of the OVERdispersion is week-to-week persistence vs within-week burst?
            println!("     var explained by last week's count: {:.3} of {:.3}  -> residual Fano {:.2}",
                r * r * vn, vn, (vn * (1.0 - r * r)) / mn);
        }
        Ok(())
    }

    pub fn run(dir: &Path, ck_hours: i64, anchor: &str) -> Result<()> {
        let boards = load_boards(dir)?;
        let cat = load_catalogue(dir)?;
        let ts55 = times_above(&cat, 5.5, true);
        let ts65 = times_above(&cat, 6.5, true);
        let cat_from = et_to_utc(1990, 1, 1, 0, 0);

        let names = ["market_raw", "market_devig", "poisson_long", "poisson_recent2y",
                     "emp_all(1990-)", "emp_10y", "nb_recent5y", "cond_prior7d", "cond_elapsed"];
        // per family -> per model -> per board log-loss of the WINNING leg
        let mut acc: BTreeMap<String, Vec<Vec<f64>>> = BTreeMap::new();
        let mut overround: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        let mut nboards: BTreeMap<String, usize> = BTreeMap::new();
        let mut per_board: Vec<(String, String, f64, f64, f64)> = Vec::new();

        for b in &boards {
            if !b.closed { continue; }
            let w = match b.winner() { Some(w) => w, None => continue };
            let ck = if anchor == "create" { b.created + ck_hours * 3600 } else { b.win_start + ck_hours * 3600 };
            let series: Vec<Vec<(i64, f64)>> = b.legs.iter().map(|l| load_series(dir, &b.slug, &l.token_yes)).collect();
            let mids: Vec<Option<f64>> = series.iter().map(|s| price_at(s, ck, 12 * 3600)).collect();
            if mids.iter().any(|x| x.is_none()) { continue; }
            let raw: Vec<f64> = mids.iter().map(|x| x.unwrap().clamp(0.001, 0.999)).collect();
            let sum: f64 = raw.iter().sum();
            let dv: Vec<f64> = raw.iter().map(|x| x / sum).collect();

            let ts = if b.threshold < 6.0 { &ts55 } else { &ts65 };
            let days = b.days();
            let s_all = empirical_counts(ts, days, b.win_start, cat_from);
            let s_10y = empirical_counts(ts, days, b.win_start, b.win_start - 10 * 365 * 86400);
            let s_5y = empirical_counts(ts, days, b.win_start, b.win_start - 5 * 365 * 86400);
            let s_2y = empirical_counts(ts, days, b.win_start, b.win_start - 2 * 365 * 86400);
            let lam_long = mean(&s_all.iter().map(|&x| x as f64).collect::<Vec<_>>());
            let lam_2y = mean(&s_2y.iter().map(|&x| x as f64).collect::<Vec<_>>());
            let m5 = mean(&s_5y.iter().map(|&x| x as f64).collect::<Vec<_>>());
            let v5 = { let v: Vec<f64> = s_5y.iter().map(|&x| x as f64).collect(); sd(&v).powi(2) };

            // conditional on the count in the 7 days BEFORE the window (all history)
            let prior_obs = count_between(ts, b.win_start - 7 * 86400, b.win_start);
            let wlen = (days * 86400.0) as i64;
            let mut cond7: Vec<i32> = Vec::new();
            let mut cond_el: Vec<i32> = Vec::new();
            let elapsed_obs = count_between(ts, b.win_start, ck);
            let mut t = cat_from + 7 * 86400;
            while t + wlen < b.win_start {
                let pr = count_between(ts, t - 7 * 86400, t);
                let tot = count_between(ts, t, t + wlen);
                let el = count_between(ts, t, t + ck_hours * 3600);
                // nearest-neighbour band on prior activity
                let band = if b.threshold < 6.0 { 2 } else { 0 };
                if (pr - prior_obs).abs() <= band { cond7.push(tot); }
                if el == elapsed_obs { cond_el.push(tot); }
                t += 86400;
            }
            if cond7.len() < 50 { cond7 = s_all.clone(); }
            if cond_el.len() < 50 { cond_el = s_all.clone(); }

            let dists: Vec<Vec<f64>> = vec![
                raw.clone(),
                dv.clone(),
                poisson_lattice(b, lam_long),
                poisson_lattice(b, lam_2y),
                lattice_dist(b, &s_all),
                lattice_dist(b, &s_10y),
                nb_lattice(b, m5, v5),
                lattice_dist(b, &cond7),
                lattice_dist(b, &cond_el),
            ];
            let fam = b.family();
            let e = acc.entry(fam.clone()).or_insert_with(|| vec![Vec::new(); names.len()]);
            for (i, d) in dists.iter().enumerate() { e[i].push(ll(d[w])); }
            overround.entry(fam.clone()).or_default().push(sum);
            *nboards.entry(fam.clone()).or_insert(0) += 1;
            per_board.push((fam, b.slug.clone(), ll(dv[w]), ll(dists[4][w]), ll(dists[7][w])));
        }

        println!("MODEL COMPARISON at window-open + {}h  (log-loss of the winning leg, per board)\n", ck_hours);
        for (fam, m) in &acc {
            let n = nboards[fam];
            let ov = &overround[fam];
            println!("=== {}  n={} boards   mean leg-sum (overround) {:.4}  -> ln(sum) = {:.4}",
                fam, n, mean(ov), mean(ov).ln());
            println!("{:<20} {:>8} {:>10} {:>8} {:>8} {:>8}", "model", "log-loss", "vs mkt_dv", "se(diff)", "t", "wins");
            let base = &m[1];
            for (i, name) in names.iter().enumerate() {
                let diffs: Vec<f64> = base.iter().zip(m[i].iter()).map(|(a, b)| a - b).collect();
                let wins = diffs.iter().filter(|d| **d > 0.0).count();
                println!("{:<20} {:>8.3} {:>+10.3} {:>8.3} {:>8.2} {:>5}/{}",
                    name, mean(&m[i]), mean(&diffs), se(&diffs), mean(&diffs) / se(&diffs).max(1e-9), wins, n);
            }
            println!();
        }
        Ok(())
    }
}

// =====================================================================================
// MAGNITUDE-REVISION LAYER — the market resolves on the magnitudes USGS was *reporting*
// ~24-48h after the window closed, not on today's final catalogue. Reconstructed from
// ComCat origin products (each carries updateTime + its own magnitude).
// =====================================================================================
mod revfit {
    use super::*;

    /// (reported at event+lag) - (current preferred). Empirical, threshold-adjacent events.
    pub fn deltas(dir: &Path, lag_h: f64) -> Vec<f64> {
        let mut out = Vec::new();
        let d = dir.join("raw/detail");
        let rd = match fs::read_dir(&d) { Ok(r) => r, Err(_) => return out };
        for e in rd.filter_map(|e| e.ok()) {
            let txt = match fs::read_to_string(e.path()) { Ok(t) => t, Err(_) => continue };
            let v: serde_json::Value = match serde_json::from_str(&txt) { Ok(v) => v, Err(_) => continue };
            let props = match v.get("properties") { Some(p) => p, None => continue };
            let final_mag = match props.get("mag").and_then(|x| x.as_f64()) { Some(m) => m, None => continue };
            let t_ms = match props.get("time").and_then(|x| x.as_i64()) { Some(t) => t, None => continue };
            let cutoff = t_ms + (lag_h * 3.6e6) as i64;
            let origins = match props.get("products").and_then(|p| p.get("origin")).and_then(|o| o.as_array()) {
                Some(o) => o, None => continue };
            let mut best: Option<(f64, i64, f64)> = None; // (weight, updateTime, mag)
            for o in origins {
                let ut = o.get("updateTime").and_then(|x| x.as_i64()).unwrap_or(0);
                if ut > cutoff { continue; }
                let w = o.get("preferredWeight").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let m = o.get("properties").and_then(|p| p.get("magnitude"))
                    .and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok());
                let m = match m { Some(m) => m, None => continue };
                if best.is_none() || (w, ut) > (best.unwrap().0, best.unwrap().1) { best = Some((w, ut, m)); }
            }
            if let Some((_, _, m)) = best { out.push(m - final_mag); }
        }
        out
    }

    pub fn run(dir: &Path) -> Result<()> {
        println!("MAGNITUDE-REVISION LAYER — reported-at-resolution minus final magnitude\n");
        for lag in [24.0f64, 48.0, 168.0] {
            let d = deltas(dir, lag);
            if d.is_empty() { println!("lag {}h: no data", lag); continue; }
            let nz = d.iter().filter(|x| x.abs() > 0.001).count();
            let mut a: Vec<f64> = d.clone();
            a.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let big = d.iter().filter(|x| x.abs() >= 0.1).count();
            let down = d.iter().filter(|x| **x < -0.001).count();
            let up = d.iter().filter(|x| **x > 0.001).count();
            println!("lag {:>4}h  n={}  mean {:+.4}  sd {:.4}  |d|>0 {:.1}%  |d|>=0.1 {:.1}%  (down {} / up {})  p05 {:+.2} p50 {:+.2} p95 {:+.2}",
                lag, d.len(), mean(&d), sd(&d), 100.0*nz as f64/d.len() as f64,
                100.0*big as f64/d.len() as f64, down, up,
                a[a.len()/20], a[a.len()/2], a[a.len()*19/20]);
        }
        Ok(())
    }
}

// =====================================================================================
// ETAS — Epidemic-Type Aftershock Sequence, temporal, fitted to the GLOBAL catalogue.
//   lambda(t) = mu + sum_{t_i<t} K * 10^{a(M_i-M0)} * (t - t_i + c)^{-p}
//   magnitudes ~ Gutenberg-Richter above M0, binned to 0.1 as USGS reports them,
//   then perturbed by the revision layer before thresholding.
// Superposing many regional ETAS processes is not itself ETAS; a temporal fit to the
// global catalogue is an aggregate approximation and is labelled as such.
// =====================================================================================
mod etas {
    use super::*;

    const M0: f64 = 5.0;
    const DAY: f64 = 86400.0;

    #[derive(Clone, Copy, Debug)]
    pub struct P { pub mu: f64, pub k: f64, pub a: f64, pub c: f64, pub p: f64, pub b: f64 }

    #[derive(Clone, Copy)]
    pub struct Ev { pub t: f64, pub m: f64 } // t in days

    // ---- fast PRNG (xoshiro256**) ----
    pub struct Rng(u64, u64, u64, u64);
    impl Rng {
        pub fn new(seed: u64) -> Rng {
            let mut s = seed.wrapping_add(0x9E3779B97F4A7C15);
            let mut nxt = || {
                s = s.wrapping_add(0x9E3779B97F4A7C15);
                let mut z = s;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
                z ^ (z >> 31)
            };
            Rng(nxt(), nxt(), nxt(), nxt())
        }
        #[inline]
        pub fn next_u64(&mut self) -> u64 {
            let r = self.1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
            let t = self.1 << 17;
            self.2 ^= self.0; self.3 ^= self.1; self.1 ^= self.2; self.0 ^= self.3; self.2 ^= t;
            self.3 = self.3.rotate_left(45);
            r
        }
        #[inline]
        pub fn f64(&mut self) -> f64 { (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0) }
        #[inline]
        pub fn open(&mut self) -> f64 { let u = self.f64(); if u <= 0.0 { 1e-17 } else { u } }
        pub fn poisson(&mut self, lam: f64) -> u32 {
            if lam <= 0.0 { return 0; }
            if lam < 30.0 {
                let l = (-lam).exp();
                let (mut k, mut prod) = (0u32, self.f64());
                while prod > l { k += 1; prod *= self.f64(); if k > 400 { break; } }
                return k;
            }
            // normal approximation with continuity correction (only used for big background rates)
            let z = {
                let (u1, u2) = (self.open(), self.f64());
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            };
            (lam + z * lam.sqrt() + 0.5).max(0.0) as u32
        }
    }

    // ---- Omori kernel ----
    /// integral of (s+c)^-p from 0 to x
    #[inline]
    fn gint(x: f64, c: f64, p: f64) -> f64 {
        if x <= 0.0 { return 0.0; }
        if (p - 1.0).abs() < 1e-8 { ((x + c) / c).ln() } else { ((x + c).powf(1.0 - p) - c.powf(1.0 - p)) / (1.0 - p) }
    }
    /// inverse of gint (draw an offspring time in (0, xmax] from the Omori density)
    #[inline]
    fn ginv(y: f64, c: f64, p: f64) -> f64 {
        if (p - 1.0).abs() < 1e-8 { c * (y.exp() - 1.0) }
        else { (y * (1.0 - p) + c.powf(1.0 - p)).powf(1.0 / (1.0 - p)) - c }
    }

    #[inline]
    fn productivity(m: f64, pr: &P) -> f64 { pr.k * 10f64.powf(pr.a * (m - M0)) }

    // ---- log-likelihood on [t0,t1], history truncated to `hist` days back ----
    pub fn loglik(evs: &[Ev], t0: f64, t1: f64, pr: &P, hist: f64) -> f64 {
        if pr.mu <= 0.0 || pr.k <= 0.0 || pr.c <= 1e-6 || pr.p <= 0.05 || pr.a < 0.0 || pr.a > 3.0 { return f64::NEG_INFINITY; }
        let pr = *pr; let pr = &pr;
        let i0 = evs.partition_point(|e| e.t < t0);
        let i1 = evs.partition_point(|e| e.t <= t1);
        let nt = 4usize;
        let chunk = (i1 - i0).div_ceil(nt);
        let ll: f64 = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..nt).map(|ci| {
                let (lo, hi) = (i0 + ci * chunk, (i0 + (ci + 1) * chunk).min(i1));
                sc.spawn(move || {
                    let mut acc = 0.0;
                    for j in lo..hi {
                        let tj = evs[j].t;
                        let mut lam = pr.mu;
                        let mut i = j;
                        while i > 0 {
                            i -= 1;
                            let dt = tj - evs[i].t;
                            if dt > hist { break; }
                            if dt <= 0.0 { continue; }
                            lam += productivity(evs[i].m, pr) * (dt + pr.c).powf(-pr.p);
                        }
                        acc += lam.ln();
                    }
                    acc
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).sum()
        });
        // compensator
        let mut comp = pr.mu * (t1 - t0);
        for e in evs.iter() {
            if e.t >= t1 { break; }
            let lo = (t0 - e.t).max(0.0);
            let hi = t1 - e.t;
            if hi <= 0.0 { continue; }
            if lo > hist { continue; }
            comp += productivity(e.m, pr) * (gint(hi.min(hist), pr.c, pr.p) - gint(lo, pr.c, pr.p));
        }
        ll - comp
    }

    fn pack(pr: &P) -> [f64; 5] { [pr.mu.ln(), pr.k.ln(), pr.a, pr.c.ln(), pr.p] }
    fn unpack(x: &[f64], b: f64) -> P { P { mu: x[0].exp(), k: x[1].exp(), a: x[2], c: x[3].exp(), p: x[4], b } }

    /// Nelder-Mead
    pub fn fit(evs: &[Ev], t0: f64, t1: f64, start: P, hist: f64, iters: usize) -> P {
        let n = 5;
        let f = |x: &[f64]| -loglik(evs, t0, t1, &unpack(x, start.b), hist);
        let mut simplex: Vec<Vec<f64>> = Vec::new();
        let s0 = pack(&start);
        simplex.push(s0.to_vec());
        for i in 0..n {
            let mut v = s0.to_vec();
            v[i] += if i == 4 { 0.08 } else { 0.25 };
            simplex.push(v);
        }
        let mut fv: Vec<f64> = simplex.iter().map(|x| f(x)).collect();
        for _ in 0..iters {
            let mut idx: Vec<usize> = (0..=n).collect();
            idx.sort_by(|&a, &b| fv[a].partial_cmp(&fv[b]).unwrap());
            let (best, worst, second) = (idx[0], idx[n], idx[n - 1]);
            if (fv[worst] - fv[best]).abs() < 1e-7 { break; }
            let mut cent = vec![0.0; n];
            for &i in idx.iter().take(n) { for j in 0..n { cent[j] += simplex[i][j] / n as f64; } }
            let refl: Vec<f64> = (0..n).map(|j| cent[j] + 1.0 * (cent[j] - simplex[worst][j])).collect();
            let fr = f(&refl);
            if fr < fv[best] {
                let exp: Vec<f64> = (0..n).map(|j| cent[j] + 2.0 * (cent[j] - simplex[worst][j])).collect();
                let fe = f(&exp);
                if fe < fr { simplex[worst] = exp; fv[worst] = fe; } else { simplex[worst] = refl; fv[worst] = fr; }
            } else if fr < fv[second] {
                simplex[worst] = refl; fv[worst] = fr;
            } else {
                let con: Vec<f64> = (0..n).map(|j| cent[j] + 0.5 * (simplex[worst][j] - cent[j])).collect();
                let fc = f(&con);
                if fc < fv[worst] { simplex[worst] = con; fv[worst] = fc; }
                else {
                    for &i in idx.iter().skip(1) {
                        let nv: Vec<f64> = (0..n).map(|j| simplex[best][j] + 0.5 * (simplex[i][j] - simplex[best][j])).collect();
                        simplex[i] = nv; fv[i] = f(&simplex[i]);
                    }
                }
            }
        }
        let bi = (0..=n).min_by(|&a, &b| fv[a].partial_cmp(&fv[b]).unwrap()).unwrap();
        unpack(&simplex[bi], start.b)
    }

    /// Aki-Utsu b-value for magnitudes binned to `dm`.
    pub fn bvalue(mags: &[f64], mc: f64, dm: f64) -> f64 {
        let v: Vec<f64> = mags.iter().cloned().filter(|m| *m >= mc - 1e-9).collect();
        std::f64::consts::E.log10() / (mean(&v) - (mc - dm / 2.0))
    }

    #[inline]
    fn draw_mag(rng: &mut Rng, b: f64, mmax: f64) -> f64 {
        // truncated Gutenberg-Richter on [M0, mmax], continuous
        let beta = b * std::f64::consts::LN_10;
        let umax = 1.0 - (-beta * (mmax - M0)).exp();
        let u = rng.f64() * umax;
        M0 - (1.0 - u).ln() / beta
    }

    /// Simulate ONE window and return the count of events whose *reported* magnitude
    /// (0.1-binned, revision-perturbed) is >= `thr`.
    #[allow(clippy::too_many_arguments)]
    pub fn sim_window(hist: &[Ev], w0: f64, w1: f64, pr: &P, rng: &mut Rng,
                      rev: &[f64], mmax: f64, thr: f64, wbuf: &mut Vec<f64>, stack: &mut Vec<Ev>) -> i32 {
        wbuf.clear(); stack.clear();
        let mut count = 0i32;
        let mut emit = |m: f64, rng: &mut Rng, count: &mut i32| {
            let d = if rev.is_empty() { 0.0 } else { rev[(rng.next_u64() as usize) % rev.len()] };
            // USGS reports to 0.1; the resolving vintage carries the revision delta
            let rep = ((m + d) * 10.0).round() / 10.0;
            if rep >= thr - 1e-9 { *count += 1; }
        };
        // 1. background
        let nbg = rng.poisson(pr.mu * (w1 - w0));
        for _ in 0..nbg {
            let t = w0 + rng.f64() * (w1 - w0);
            let m = draw_mag(rng, pr.b, mmax);
            emit(m, rng, &mut count);
            stack.push(Ev { t, m });
        }
        // 2. offspring of pre-window history: total expected count, then sample parents
        let mut tot = 0.0;
        wbuf.reserve(hist.len());
        for e in hist {
            let w = productivity(e.m, pr) * (gint(w1 - e.t, pr.c, pr.p) - gint(w0 - e.t, pr.c, pr.p));
            tot += w;
            wbuf.push(tot);
        }
        let nh = rng.poisson(tot);
        for _ in 0..nh {
            let u = rng.f64() * tot;
            let i = wbuf.partition_point(|&x| x < u).min(hist.len() - 1);
            let e = hist[i];
            let (g0, g1) = (gint(w0 - e.t, pr.c, pr.p), gint(w1 - e.t, pr.c, pr.p));
            let t = e.t + ginv(g0 + rng.f64() * (g1 - g0), pr.c, pr.p);
            let m = draw_mag(rng, pr.b, mmax);
            emit(m, rng, &mut count);
            stack.push(Ev { t: t.clamp(w0, w1), m });
        }
        // 3. recursive offspring of everything born inside the window
        let mut guard = 0;
        while let Some(e) = stack.pop() {
            guard += 1;
            if guard > 200_000 { break; }
            let lam = productivity(e.m, pr) * gint(w1 - e.t, pr.c, pr.p);
            let n = rng.poisson(lam);
            for _ in 0..n {
                let t = e.t + ginv(rng.f64() * gint(w1 - e.t, pr.c, pr.p), pr.c, pr.p);
                let m = draw_mag(rng, pr.b, mmax);
                emit(m, rng, &mut count);
                if t < w1 { stack.push(Ev { t, m }); }
            }
        }
        count
    }


    // ---------- driver ----------

    fn to_evs(cat: &[Quake], mc: f64) -> Vec<Ev> {
        cat.iter().filter(|q| q.is_eq && q.mag >= mc - 1e-9)
            .map(|q| Ev { t: q.t as f64 / DAY, m: q.mag }).collect()
    }

    pub fn run(dir: &Path, args: &[String]) -> Result<()> {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("fit");
        let cat = load_catalogue(dir)?;
        let evs = to_evs(&cat, M0);
        let mags: Vec<f64> = evs.iter().map(|e| e.m).collect();
        let b = bvalue(&mags, M0, 0.1);
        let t_start = et_to_utc(1990, 1, 1, 0, 0) as f64 / DAY;
        let oos = sub == "physics2015";
        let t_fitend = et_to_utc(if oos {2015} else {2025}, if oos {1} else {12}, 1, 0, 0) as f64 / DAY;
        let hist = 200.0;

        println!("ETAS  M0={:.1}  catalogue {} events  b(Aki-Utsu, dm=0.1) = {:.4}", M0, evs.len(), b);
        let start = P { mu: 3.0, k: 0.02, a: 0.9, c: 0.02, p: 1.1, b };
        let t_fit0 = t_start + 220.0;
        let cache = dir.join(if oos {"etas_mle_pre2015.json"} else {"etas_mle.json"});
        let pr = if let Ok(t) = fs::read_to_string(&cache) {
            let v: serde_json::Value = serde_json::from_str(&t)?;
            eprintln!("(cached MLE)");
            P { mu: jnum(&v,"mu"), k: jnum(&v,"k"), a: jnum(&v,"a"), c: jnum(&v,"c"), p: jnum(&v,"p"), b }
        } else {
            let pr = fit(&evs, t_fit0, t_fitend, start, hist, 400);
            fs::write(&cache, format!("{{\"mu\":{},\"k\":{},\"a\":{},\"c\":{},\"p\":{},\"b\":{}}}", pr.mu,pr.k,pr.a,pr.c,pr.p,pr.b))?;
            pr
        };
        let llv = loglik(&evs, t_fit0, t_fitend, &pr, hist);
        // branching ratio: expected direct offspring of an M0 event, GR-averaged
        let beta = b * std::f64::consts::LN_10;
        let alpha = pr.a * std::f64::consts::LN_10;
        let nbr = if beta > alpha { pr.k * gint(365.0, pr.c, pr.p) * beta / (beta - alpha) } else { f64::INFINITY };
        let nbr7 = if beta > alpha { pr.k * gint(7.0, pr.c, pr.p) * beta / (beta - alpha) } else { f64::INFINITY };
        println!("MLE (1990-08 .. {}): mu=", if oos {"2015-01-01"} else {"2025-12-01"});
        println!("  mu={:.4}/day  K={:.5}  alpha={:.3}  c={:.5}d  p={:.4}  logL={:.1}",
            pr.mu, pr.k, pr.a, pr.c, pr.p, llv);
        println!("branching ratio over 365d n = {:.3};  over a 7d window n7 = {:.3} (subcritical => simulation stable)", nbr, nbr7);

        // ---- posterior: finite-difference Hessian at the MLE -> MVN draws ----
        let post = posterior(&evs, t_fit0, t_fitend, &pr, hist, 240);
        println!("posterior: {} draws; mu {:.3}+-{:.3}  K {:.4}+-{:.4}  a {:.3}+-{:.3}  c {:.4}+-{:.4}  p {:.3}+-{:.3}",
            post.len(),
            mean(&post.iter().map(|p| p.mu).collect::<Vec<_>>()), sd(&post.iter().map(|p| p.mu).collect::<Vec<_>>()),
            mean(&post.iter().map(|p| p.k).collect::<Vec<_>>()), sd(&post.iter().map(|p| p.k).collect::<Vec<_>>()),
            mean(&post.iter().map(|p| p.a).collect::<Vec<_>>()), sd(&post.iter().map(|p| p.a).collect::<Vec<_>>()),
            mean(&post.iter().map(|p| p.c).collect::<Vec<_>>()), sd(&post.iter().map(|p| p.c).collect::<Vec<_>>()),
            mean(&post.iter().map(|p| p.p).collect::<Vec<_>>()), sd(&post.iter().map(|p| p.p).collect::<Vec<_>>()));

        let rev = revfit::deltas(dir, 36.0);
        println!("revision layer: {} sampled events, mean {:+.4}, sd {:.4}", rev.len(), mean(&rev), sd(&rev));

        match sub {
            "validate" => validate(&evs, &post, &rev),
            "physics" | "physics2015" => physics(&evs, &post, &rev, dir),
            "score" => score_boards(dir, &evs, &post, &rev),
            _ => Ok(()),
        }
    }

    fn posterior(evs: &[Ev], t0: f64, t1: f64, pr: &P, hist: f64, n: usize) -> Vec<P> {
        let x0 = pack(pr);
        let f = |x: &[f64]| loglik(evs, t0, t1, &unpack(x, pr.b), hist);
        let f0 = f(&x0);
        let h = [0.02, 0.02, 0.01, 0.03, 0.005];
        // diagonal curvature only (the full 5x5 needs 40 extra likelihood evals and the
        // off-diagonals do not change the conclusion at this effect size)
        let mut sdv = [0.0f64; 5];
        for i in 0..5 {
            let (mut xp, mut xm) = (x0, x0);
            xp[i] += h[i]; xm[i] -= h[i];
            let d2 = (f(&xp) - 2.0 * f0 + f(&xm)) / (h[i] * h[i]);
            sdv[i] = if d2 < 0.0 { (-1.0 / d2).sqrt() } else { 0.0 };
        }
        let mut rng = Rng::new(20260725);
        (0..n).map(|_| {
            let mut x = x0;
            for i in 0..5 {
                let (u1, u2) = (rng.open(), rng.f64());
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                x[i] += z * sdv[i];
            }
            unpack(&x, pr.b)
        }).collect()
    }

    /// Does the fitted process reproduce the observed weekly count moments?
    fn validate(evs: &[Ev], post: &[P], rev: &[f64]) -> Result<()> {
        println!("\n--- VALIDATION: simulated vs observed weekly count moments ---");
        let mut rng = Rng::new(7);
        let (mut wb, mut st) = (Vec::new(), Vec::new());
        for thr in [5.5f64, 6.5] {
            let mut counts: Vec<f64> = Vec::new();
            // start each simulated week from a real historical state
            let t_lo = et_to_utc(2015, 1, 1, 0, 0) as f64 / DAY;
            let t_hi = et_to_utc(2025, 12, 1, 0, 0) as f64 / DAY;
            for i in 0..40_000 {
                let w0 = t_lo + rng.f64() * (t_hi - t_lo);
                let lo = evs.partition_point(|e| e.t < w0 - 200.0);
                let hi = evs.partition_point(|e| e.t < w0);
                let p = post[i % post.len()];
                counts.push(sim_window(&evs[lo..hi], w0, w0 + 7.0, &p, &mut rng, rev, 9.5, thr, &mut wb, &mut st) as f64);
            }
            let m = mean(&counts); let v = sd(&counts).powi(2);
            println!("M{:.1}+ simulated: mean {:.3}  var {:.3}  Fano {:.2}   (n=40,000 weeks)", thr, m, v, v / m);
        }
        Ok(())
    }

    /// GATE 1 — on the physics alone, out-of-sample: does ETAS beat the empirical marginal?
    fn physics(evs: &[Ev], post: &[P], rev: &[f64], dir: &Path) -> Result<()> {
        println!("\n--- GATE 1: predictive log-loss on the catalogue itself, 2015-2026, weekly ---");
        let cat = load_catalogue(dir)?;
        let mut rng = Rng::new(11);
        let (mut wb, mut st) = (Vec::new(), Vec::new());
        for thr in [5.5f64, 6.5] {
            let ts = times_above(&cat, thr, true);
            let mut t = et_to_utc(2015, 1, 5, 0, 0);
            let tend = Utc::now().timestamp() - 7 * 86400;
            let (mut ll_etas, mut ll_emp, mut ll_poi, mut ll_nb) = (vec![], vec![], vec![], vec![]);
            // static models fitted on 1990..2015 only
            let train = empirical_counts(&ts, 7.0, et_to_utc(2015, 1, 1, 0, 0), et_to_utc(1990, 1, 1, 0, 0));
            let tf: Vec<f64> = train.iter().map(|&x| x as f64).collect();
            let (mt, vt) = (mean(&tf), sd(&tf).powi(2));
            let mut emp = vec![0.0f64; 80];
            for &c in &train { if (c as usize) < 80 { emp[c as usize] += 1.0; } }
            let tot: f64 = emp.iter().sum();
            let emp: Vec<f64> = emp.iter().map(|x| (x + 0.5) / (tot + 40.0)).collect();
            let mut poi = vec![0.0f64; 80];
            { let mut p = (-mt).exp(); for n in 0..80 { poi[n] = p; p *= mt / (n as f64 + 1.0); } }
            let mut nb = vec![0.0f64; 80];
            { let r = mt * mt / (vt - mt).max(1e-6); let pp = r / (r + mt);
              let mut cur = pp.powf(r);
              for n in 0..80 { nb[n] = cur; cur *= (r + n as f64) / (n as f64 + 1.0) * (1.0 - pp); } }
            while t < tend {
                let w0 = t as f64 / DAY;
                let actual = count_between(&ts, t, t + 7 * 86400) as usize;
                let lo = evs.partition_point(|e| e.t < w0 - 200.0);
                let hi = evs.partition_point(|e| e.t < w0);
                let nsim = 4000;
                let mut histgram = vec![0.0f64; 80];
                for i in 0..nsim {
                    let p = post[i % post.len()];
                    let c = sim_window(&evs[lo..hi], w0, w0 + 7.0, &p, &mut rng, rev, 9.5, thr, &mut wb, &mut st) as usize;
                    if c < 80 { histgram[c] += 1.0; }
                }
                let pe = (histgram[actual.min(79)] + 0.5) / (nsim as f64 + 40.0);
                ll_etas.push(-pe.ln());
                ll_emp.push(-emp[actual.min(79)].max(1e-9).ln());
                ll_poi.push(-poi[actual.min(79)].max(1e-9).ln());
                ll_nb.push(-nb[actual.min(79)].max(1e-9).ln());
                t += 7 * 86400;
            }
            let d_emp: Vec<f64> = ll_emp.iter().zip(ll_etas.iter()).map(|(a, b)| a - b).collect();
            println!("M{:.1}+  n={} weeks   ETAS {:.4}  empirical {:.4}  NB {:.4}  Poisson {:.4}",
                thr, ll_etas.len(), mean(&ll_etas), mean(&ll_emp), mean(&ll_nb), mean(&ll_poi));
            println!("       ETAS - empirical = {:+.4} log-loss (se {:.4}, t={:+.2}, ETAS wins {}/{})",
                mean(&d_emp), se(&d_emp), mean(&d_emp) / se(&d_emp),
                d_emp.iter().filter(|x| **x > 0.0).count(), d_emp.len());
        }
        Ok(())
    }

    /// GATE 2 — ETAS vs the market at window-open, out-of-sample, on the real boards.
    fn score_boards(dir: &Path, evs: &[Ev], post: &[P], rev: &[f64]) -> Result<()> {
        println!("\n--- GATE 2: ETAS vs the market at window-open (+6h), resolved boards ---");
        let boards = load_boards(dir)?;
        let cat = load_catalogue(dir)?;
        let ts55 = times_above(&cat, 5.5, true);
        let ts65 = times_above(&cat, 6.5, true);
        let cat_from = et_to_utc(1990, 1, 1, 0, 0);
        let mut rng = Rng::new(99);
        let (mut wb, mut st) = (Vec::new(), Vec::new());
        let mut per_fam: BTreeMap<String, (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> = BTreeMap::new();
        let mut rows: Vec<(String, String, f64, f64, f64, Vec<f64>, Vec<f64>, usize)> = Vec::new();

        for b in &boards {
            if !b.closed { continue; }
            let w = match b.winner() { Some(w) => w, None => continue };
            let ck = b.win_start + 6 * 3600;
            let series: Vec<Vec<(i64, f64)>> = b.legs.iter().map(|l| load_series(dir, &b.slug, &l.token_yes)).collect();
            let mids: Vec<Option<f64>> = series.iter().map(|s| price_at(s, ck, 12 * 3600)).collect();
            if mids.iter().any(|x| x.is_none()) { continue; }
            let dv = devig(&mids).unwrap();
            let w0 = b.win_start as f64 / DAY;
            let wlen = b.days();
            let lo = evs.partition_point(|e| e.t < w0 - 200.0);
            let hi = evs.partition_point(|e| e.t < w0);
            let nsim = 200_000usize;
            let mut hg = vec![0.0f64; 120];
            for i in 0..nsim {
                let p = post[i % post.len()];
                let c = sim_window(&evs[lo..hi], w0, w0 + wlen, &p, &mut rng, rev, 9.5, b.threshold, &mut wb, &mut st) as usize;
                if c < 120 { hg[c] += 1.0; }
            }
            let mut md = vec![0.0f64; b.legs.len()];
            for (n, cnt) in hg.iter().enumerate() {
                if let Some(i) = b.leg_of(n as i32) { md[i] += cnt; }
            }
            let s: f64 = md.iter().sum();
            let md: Vec<f64> = md.iter().map(|x| (x + 0.5) / (s + 0.5 * b.legs.len() as f64)).collect();
            let ts = if b.threshold < 6.0 { &ts55 } else { &ts65 };
            let sample = empirical_counts(ts, wlen, b.win_start, cat_from);
            let emp = lattice_dist(b, &sample);
            let e = per_fam.entry(b.family()).or_insert((vec![], vec![], vec![], vec![]));
            e.0.push(-dv[w].max(1e-6).ln());
            e.1.push(-md[w].max(1e-6).ln());
            e.2.push(-emp[w].max(1e-6).ln());
            e.3.push(mids.iter().map(|x| x.unwrap()).sum::<f64>());
            rows.push((b.family(), b.slug.clone(), -dv[w].ln(), -md[w].ln(), -emp[w].ln(), dv.clone(), md.clone(), w));
        }
        for (fam, (mk, et, em, ov)) in &per_fam {
            let d1: Vec<f64> = mk.iter().zip(et.iter()).map(|(a, b)| a - b).collect();
            let d2: Vec<f64> = mk.iter().zip(em.iter()).map(|(a, b)| a - b).collect();
            println!("\n{}  n={} boards  (mean leg-sum {:.4})", fam, mk.len(), mean(ov));
            println!("  market(de-vig) {:.4}   ETAS {:.4}   empirical {:.4}", mean(mk), mean(et), mean(em));
            println!("  ETAS - market      = {:+.4}  (se {:.4}, t={:+.2}, ETAS wins {}/{})",
                mean(&d1), se(&d1), mean(&d1) / se(&d1), d1.iter().filter(|x| **x > 0.0).count(), d1.len());
            println!("  empirical - market = {:+.4}  (se {:.4}, t={:+.2})", mean(&d2), se(&d2), mean(&d2) / se(&d2));
        }
        // per-leg mean model vs market
        println!("\n  per-leg mean de-vigged market price vs ETAS (M6.5+ lattice):");
        let mut agg: BTreeMap<String, (f64, f64, f64, usize)> = BTreeMap::new();
        for (fam, _s, _a, _b, _c, dv, md, w) in &rows {
            if fam != "M6.5+" { continue; }
            for (i, lbl) in ["0", "1", "2", "3", "4", "5", ">5"].iter().enumerate() {
                if i >= dv.len() { break; }
                let e = agg.entry(lbl.to_string()).or_insert((0.0, 0.0, 0.0, 0));
                e.0 += dv[i]; e.1 += md[i]; e.3 += 1;
                if *w == i { e.2 += 1.0; }
            }
        }
        println!("  {:<6} {:>9} {:>9} {:>9}", "leg", "market", "ETAS", "realised");
        for (k, (a, b, c, n)) in &agg {
            println!("  {:<6} {:>9.4} {:>9.4} {:>9.4}", k, a / *n as f64, b / *n as f64, c / *n as f64);
        }
        // gate-3 style net PnL with the ETAS signal
        println!("\n  GATE 3 with the ETAS signal (edge>3c, delayed fill +2c adverse, 0.05 taker fee):");
        for (lo, hi, name) in [(0.0f64, 0.03f64, "wings <3c"), (0.03, 1.0, "fundable >=3c"), (0.0, 1.0, "all legs")] {
            let mut pnl = Vec::new();
            for (_fam, slug, _a, _b, _c, dv, md, w) in &rows {
                let bd = boards.iter().find(|x| &x.slug == slug).unwrap();
                for i in 0..dv.len() {
                    let series = load_series(dir, slug, &bd.legs[i].token_yes);
                    let mid = match price_at(&series, bd.win_start + 6 * 3600, 12 * 3600) { Some(m) => m, None => continue };
                    if mid < lo || mid >= hi { continue; }
                    let edge = md[i] - dv[i];
                    if edge.abs() < 0.03 { continue; }
                    let dirn = if edge > 0.0 { 1.0 } else { -1.0 };
                    let fillmid = price_at(&series, bd.win_start + 30 * 3600, 12 * 3600).unwrap_or(mid);
                    let px = (fillmid + dirn * 0.02).clamp(0.005, 0.995);
                    let payoff = if i == *w { 1.0 } else { 0.0 };
                    pnl.push(dirn * (payoff - px) - fee(px));
                }
            }
            if pnl.is_empty() { println!("    {:<16} no trades", name); continue; }
            println!("    {:<16} n={:>4}  net {:+.4}/share (se {:.4}) t={:+.2}", name, pnl.len(), mean(&pnl), se(&pnl), mean(&pnl) / se(&pnl));
        }
        Ok(())
    }
}

mod live { use super::*; pub fn run(_d: &Path, _s: &str) -> Result<()> { bail!("live not built yet") } }
