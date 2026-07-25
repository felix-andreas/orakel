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
use std::collections::{BTreeMap, HashMap};
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

// placeholder modules filled in later steps
mod revfit { use super::*; pub fn run(_d: &Path) -> Result<()> { bail!("revfit not built yet") } }
mod etas { use super::*; pub fn run(_d: &Path, _a: &[String]) -> Result<()> { bail!("etas not built yet") } }
mod live { use super::*; pub fn run(_d: &Path, _s: &str) -> Result<()> { bail!("live not built yet") } }
