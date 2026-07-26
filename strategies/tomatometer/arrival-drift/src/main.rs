//! tomatometer/arrival-drift — exact arrival-process pricing of Rotten Tomatoes score ladders.
//!
//! The Tomatometer is `round(100 * liked / (liked + notLiked))`. Polymarket and Kalshi both
//! settle a ladder of "score >= s" legs at a wall-clock instant while critics are still filing.
//! At a checkpoint we observe `(L, N)`; by resolution `M` further reviews land of which `K` are
//! fresh, and the settled score is `round(100 * (L+K) / (N+M))`.
//!
//! We do NOT Monte-Carlo the terminal score. `M` has a few hundred plausible values and `K` is
//! binomial given `(M, p_late)`, so the exact terminal distribution over the integer score
//! lattice is a two-dimensional quadrature times an exact binomial pmf — a few million flops.
//! That matters because the whole thesis is a *lattice* claim: near a strike the answer turns on
//! one or two reviews, and Monte-Carlo noise of 1e-3 is the same size as the effect being priced.
//! `simcheck` runs a Monte-Carlo sampler over the same generative model to verify the quadrature.
//!
//! Subcommands:
//!   fit       <rt_paths.csv> <horizons_h...>   fit growth + late-pool-rate models, print params
//!   price     <L> <N> <hours> <strikes...>     ladder probabilities from a live state
//!   backtest  <bt_input.csv> <fit.json>        model vs market on resolved boards
//!   simcheck  <L> <N> <hours> <fit.json>       Monte-Carlo cross-check of the quadrature
//!   bands     <scored.csv>                     q* / q / Wilson lower bound per price band

use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::env;
use std::fs;

// ---------------------------------------------------------------- numerics

/// Lanczos log-gamma. Needed for exact binomial pmf in log space.
fn lgamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        // reflection
        std::f64::consts::PI.ln() - (std::f64::consts::PI * x).sin().ln() - lgamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = G[0];
        let t = x + 7.5;
        for (i, g) in G.iter().enumerate().skip(1) {
            a += g / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

fn ln_binom_coef(n: u32, k: u32) -> f64 {
    lgamma(n as f64 + 1.0) - lgamma(k as f64 + 1.0) - lgamma((n - k) as f64 + 1.0)
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}
fn expit(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}

/// Gauss-Hermite-style standard-normal quadrature: equally spaced z with normal weights.
/// Simple and adequate at 41 nodes over +/-4 sigma; the integrand is smooth in z.
fn normal_nodes(n: usize, half_width: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(n);
    let mut tot = 0.0;
    for i in 0..n {
        let z = -half_width + 2.0 * half_width * (i as f64) / ((n - 1) as f64);
        let w = (-0.5 * z * z).exp();
        out.push((z, w));
        tot += w;
    }
    for e in out.iter_mut() {
        e.1 /= tot;
    }
    out
}

/// xoshiro256++ — deterministic, dependency-free, for the Monte-Carlo cross-check only.
struct Rng(u64, u64, u64, u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut next = || {
            s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Rng(next(), next(), next(), next())
    }
    fn next_u64(&mut self) -> u64 {
        let r = self.0.wrapping_add(self.3).rotate_left(23).wrapping_add(self.0);
        let t = self.1 << 17;
        self.2 ^= self.0;
        self.3 ^= self.1;
        self.1 ^= self.2;
        self.0 ^= self.3;
        self.2 ^= t;
        self.3 = self.3.rotate_left(45);
        r
    }
    fn unif(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
    fn normal(&mut self) -> f64 {
        // Box-Muller
        let u1 = self.unif().max(1e-12);
        let u2 = self.unif();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------- csv

fn read_csv(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let txt = fs::read_to_string(path)?;
    let mut lines = txt.lines().filter(|l| !l.trim().is_empty());
    let hdr: Vec<String> = split_csv(lines.next().ok_or_else(|| anyhow!("empty csv"))?);
    let rows: Vec<Vec<String>> = lines.map(split_csv).collect();
    Ok((hdr, rows))
}

/// Minimal RFC-4180-ish splitter: handles double-quoted fields containing commas.
fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut inq = false;
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '"' => {
                if inq && it.peek() == Some(&'"') {
                    cur.push('"');
                    it.next();
                } else {
                    inq = !inq;
                }
            }
            ',' if !inq => {
                out.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn col(hdr: &[String], name: &str) -> Result<usize> {
    hdr.iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| anyhow!("missing column `{name}` (have: {})", hdr.join(",")))
}

// ---------------------------------------------------------------- model

/// Fitted parameters. Both regressions are OLS in transformed space; the residual sd carries
/// the film-level uncertainty that makes the terminal distribution wider than a plain binomial.
#[derive(Debug, Clone)]
struct Fit {
    // ln(N_T / N_t) = ga + gb*ln(N_t) + gc*ln(hours);  resid sd gtau
    ga: f64,
    gb: f64,
    gc: f64,
    gtau: f64,
    // logit(p_late) = la + lb*logit(p_hat) + lc*ln(N_t);  resid sd lsig
    la: f64,
    lb: f64,
    lc: f64,
    lsig: f64,
    n_growth: usize,
    n_late: usize,
    /// empirical growth ratios, used by the nonparametric arrival variant
    growth_emp: Vec<f64>,
}

impl Fit {
    fn to_json(&self) -> String {
        format!(
            "{{\"ga\":{},\"gb\":{},\"gc\":{},\"gtau\":{},\"la\":{},\"lb\":{},\"lc\":{},\"lsig\":{},\"n_growth\":{},\"n_late\":{}}}",
            self.ga, self.gb, self.gc, self.gtau, self.la, self.lb, self.lc, self.lsig,
            self.n_growth, self.n_late
        )
    }
    fn from_json(s: &str) -> Result<Fit> {
        let v: serde_json::Value = serde_json::from_str(s)?;
        let g = |k: &str| -> f64 { v[k].as_f64().unwrap_or(0.0) };
        Ok(Fit {
            ga: g("ga"),
            gb: g("gb"),
            gc: g("gc"),
            gtau: g("gtau"),
            la: g("la"),
            lb: g("lb"),
            lc: g("lc"),
            lsig: g("lsig"),
            n_growth: v["n_growth"].as_u64().unwrap_or(0) as usize,
            n_late: v["n_late"].as_u64().unwrap_or(0) as usize,
            growth_emp: vec![],
        })
    }
}

/// Ordinary least squares with intercept. `x` rows are predictor vectors (no intercept column).
fn ols(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, f64)> {
    let n = x.len();
    if n == 0 {
        return None;
    }
    let k = x[0].len() + 1;
    if n <= k {
        return None;
    }
    let mut a = vec![vec![0.0f64; k + 1]; k]; // augmented normal equations
    for i in 0..n {
        let mut xi = vec![1.0];
        xi.extend_from_slice(&x[i]);
        for r in 0..k {
            for c in 0..k {
                a[r][c] += xi[r] * xi[c];
            }
            a[r][k] += xi[r] * y[i];
        }
    }
    // Gaussian elimination with partial pivoting
    for c in 0..k {
        let mut piv = c;
        for r in c + 1..k {
            if a[r][c].abs() > a[piv][c].abs() {
                piv = r;
            }
        }
        if a[piv][c].abs() < 1e-12 {
            return None;
        }
        a.swap(c, piv);
        let d = a[c][c];
        for v in a[c].iter_mut() {
            *v /= d;
        }
        for r in 0..k {
            if r != c {
                let f = a[r][c];
                if f != 0.0 {
                    for j in 0..=k {
                        a[r][j] -= f * a[c][j];
                    }
                }
            }
        }
    }
    let beta: Vec<f64> = (0..k).map(|r| a[r][k]).collect();
    let mut ss = 0.0;
    for i in 0..n {
        let mut yh = beta[0];
        for j in 0..x[i].len() {
            yh += beta[j + 1] * x[i][j];
        }
        ss += (y[i] - yh).powi(2);
    }
    let sd = (ss / (n - k) as f64).sqrt();
    Some((beta, sd))
}

/// Exact terminal-score distribution over the integer lattice 0..=100.
///
/// `mode`:
///   "full" — fitted selection offset (the thesis)
///   "null" — zero-drift: late critics are fresh at the observed early rate (the gate-2 null)
///   "frozen" — point mass at the currently displayed score (what the crowd is claimed to do)
fn score_pmf(l: u32, n: u32, hours: f64, fit: &Fit, mode: &str) -> [f64; 101] {
    let mut pmf = [0.0f64; 101];
    if n == 0 {
        // No state observed: nothing to say. Caller must refuse.
        return pmf;
    }
    let p_hat = (l as f64 + 0.5) / (n as f64 + 1.0);
    let cur = (100.0 * l as f64 / n as f64).round().clamp(0.0, 100.0) as usize;
    if mode == "frozen" {
        pmf[cur] = 1.0;
        return pmf;
    }
    let lnn = (n as f64).ln();
    let lnh = hours.max(0.5).ln();
    let mu_g = fit.ga + fit.gb * lnn + fit.gc * lnh;
    let mu_l = if mode == "null" {
        logit(p_hat)
    } else {
        fit.la + fit.lb * logit(p_hat) + fit.lc * lnn
    };
    let zg = normal_nodes(25, 3.5);
    let zl = normal_nodes(25, 3.5);

    for (z1, w1) in zg.iter() {
        let g = (mu_g + fit.gtau * z1).exp().max(1.0); // N_T >= N_t
        let m = ((n as f64) * (g - 1.0)).round().max(0.0) as u32;
        let nt = n + m;
        for (z2, w2) in zl.iter() {
            let p_late = expit(mu_l + fit.lsig * z2);
            let w = w1 * w2;
            if m == 0 {
                pmf[cur] += w;
                continue;
            }
            // exact binomial pmf over K, restricted to a +/-8 sd window around the mean
            let mean = m as f64 * p_late;
            let sd = (m as f64 * p_late * (1.0 - p_late)).sqrt().max(1.0);
            let lo = ((mean - 8.0 * sd).floor().max(0.0)) as u32;
            let hi = ((mean + 8.0 * sd).ceil().min(m as f64)) as u32;
            let lp = p_late.clamp(1e-12, 1.0 - 1e-12).ln();
            let lq = (1.0 - p_late).clamp(1e-12, 1.0 - 1e-12).ln();
            let mut acc = 0.0;
            let mut buf: Vec<(usize, f64)> = Vec::with_capacity((hi - lo + 1) as usize);
            for k in lo..=hi {
                let lpmf = ln_binom_coef(m, k) + k as f64 * lp + (m - k) as f64 * lq;
                let pk = lpmf.exp();
                acc += pk;
                let s = (100.0 * (l + k) as f64 / nt as f64).round().clamp(0.0, 100.0) as usize;
                buf.push((s, pk));
            }
            if acc <= 0.0 {
                continue;
            }
            for (s, pk) in buf {
                pmf[s] += w * pk / acc;
            }
        }
    }
    let tot: f64 = pmf.iter().sum();
    if tot > 0.0 {
        for v in pmf.iter_mut() {
            *v /= tot;
        }
    }
    pmf
}

/// P(score >= s) for each strike, from a pmf.
fn ladder(pmf: &[f64; 101], strikes: &[i32]) -> Vec<f64> {
    strikes
        .iter()
        .map(|&s| {
            let s = s.clamp(0, 100) as usize;
            pmf[s..].iter().sum::<f64>()
        })
        .collect()
}

fn pmf_mean(pmf: &[f64; 101]) -> f64 {
    pmf.iter().enumerate().map(|(i, p)| i as f64 * p).sum()
}
fn pmf_quantile(pmf: &[f64; 101], q: f64) -> f64 {
    let mut c = 0.0;
    for (i, p) in pmf.iter().enumerate() {
        c += p;
        if c >= q {
            return i as f64;
        }
    }
    100.0
}
fn pmf_sd(pmf: &[f64; 101]) -> f64 {
    let m = pmf_mean(pmf);
    pmf.iter()
        .enumerate()
        .map(|(i, p)| p * (i as f64 - m).powi(2))
        .sum::<f64>()
        .sqrt()
}

// ---------------------------------------------------------------- fitting

#[derive(Debug, Clone)]
struct Obs {
    slug: String,
    ts: i64,
    liked: u32,
    not_liked: u32,
    score: i32,
}

fn load_paths(path: &str) -> Result<BTreeMap<String, Vec<Obs>>> {
    let (hdr, rows) = read_csv(path)?;
    let (c_slug, c_ts, c_l, c_d, c_s) = (
        col(&hdr, "rt_slug")?,
        col(&hdr, "capture_ts_utc")?,
        col(&hdr, "liked")?,
        col(&hdr, "not_liked")?,
        col(&hdr, "score")?,
    );
    let mut m: BTreeMap<String, Vec<Obs>> = BTreeMap::new();
    for r in rows {
        if r.len() <= c_s {
            continue;
        }
        let (l, d) = (r[c_l].trim().parse::<u32>(), r[c_d].trim().parse::<u32>());
        let (l, d) = match (l, d) {
            (Ok(a), Ok(b)) => (a, b),
            _ => continue,
        };
        if l + d == 0 {
            continue;
        }
        let ts = parse_ts(r[c_ts].trim());
        let ts = match ts {
            Some(t) => t,
            None => continue,
        };
        m.entry(r[c_slug].trim().to_string()).or_default().push(Obs {
            slug: r[c_slug].trim().to_string(),
            ts,
            liked: l,
            not_liked: d,
            score: r[c_s].trim().parse::<i32>().unwrap_or(-1),
        });
    }
    for v in m.values_mut() {
        v.sort_by_key(|o| o.ts);
        v.dedup_by_key(|o| o.ts);
    }
    Ok(m)
}

fn parse_ts(s: &str) -> Option<i64> {
    use chrono::{DateTime, NaiveDateTime, Utc};
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Some(d.timestamp());
    }
    if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(d.and_utc().timestamp());
    }
    if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S") {
        return Some(d.and_utc().timestamp());
    }
    if let Ok(v) = s.parse::<i64>() {
        return Some(v);
    }
    let _ = Utc::now();
    None
}

/// Build (checkpoint -> terminal) training pairs at a set of horizons, for every film.
/// `terminal_ts` is the film's board resolution instant; we use the last capture at or before it.
struct Pair {
    slug: String,
    hours: f64,
    l_t: u32,
    n_t: u32,
    n_bigt: u32,
    p_late: f64,
    growth: f64,
    final_score: i32,
    cur_score: i32,
}

fn build_pairs(
    paths: &BTreeMap<String, Vec<Obs>>,
    res: &BTreeMap<String, i64>,
    horizons: &[f64],
) -> Vec<Pair> {
    let mut out = Vec::new();
    for (slug, obs) in paths {
        let t_res = match res.get(slug) {
            Some(t) => *t,
            None => continue,
        };
        // terminal = last capture at or before resolution
        let term = match obs.iter().filter(|o| o.ts <= t_res).next_back() {
            Some(o) => o.clone(),
            None => continue,
        };
        let n_bigt = term.liked + term.not_liked;
        for &h in horizons {
            let cut = t_res - (h * 3600.0) as i64;
            let ck = match obs.iter().filter(|o| o.ts <= cut).next_back() {
                Some(o) => o.clone(),
                None => continue,
            };
            let n_t = ck.liked + ck.not_liked;
            if n_t == 0 || n_bigt <= n_t {
                continue; // no arrivals observed in the window -> uninformative for the late-rate fit
            }
            let dl = term.liked as i64 - ck.liked as i64;
            let dn = n_bigt as i64 - n_t as i64;
            if dn <= 0 {
                continue;
            }
            let p_late = (dl as f64 / dn as f64).clamp(0.0, 1.0);
            out.push(Pair {
                slug: slug.clone(),
                hours: h,
                l_t: ck.liked,
                n_t,
                n_bigt,
                p_late,
                growth: n_bigt as f64 / n_t as f64,
                final_score: term.score,
                cur_score: ck.score,
            });
        }
    }
    out
}

fn fit_from_pairs(pairs: &[Pair]) -> Result<Fit> {
    let mut xg = Vec::new();
    let mut yg = Vec::new();
    let mut xl = Vec::new();
    let mut yl = Vec::new();
    let mut growth_emp = Vec::new();
    for p in pairs {
        xg.push(vec![(p.n_t as f64).ln(), p.hours.max(0.5).ln()]);
        yg.push(p.growth.ln());
        growth_emp.push(p.growth);
        // the late-rate regression needs a non-degenerate p_late; clamp for the logit
        let ph = (p.l_t as f64 + 0.5) / (p.n_t as f64 + 1.0);
        let pl = p
            .p_late
            .clamp(1.0 / (2.0 * (p.n_bigt - p.n_t) as f64 + 2.0), 1.0 - 1.0 / (2.0 * (p.n_bigt - p.n_t) as f64 + 2.0));
        xl.push(vec![logit(ph), (p.n_t as f64).ln()]);
        yl.push(logit(pl));
    }
    let (bg, gtau) = ols(&xg, &yg).ok_or_else(|| anyhow!("growth regression failed (n={})", xg.len()))?;
    let (bl, lsig) = ols(&xl, &yl).ok_or_else(|| anyhow!("late-rate regression failed (n={})", xl.len()))?;
    Ok(Fit {
        ga: bg[0],
        gb: bg[1],
        gc: bg[2],
        gtau,
        la: bl[0],
        lb: bl[1],
        lc: bl[2],
        lsig,
        n_growth: xg.len(),
        n_late: xl.len(),
        growth_emp,
    })
}

// ---------------------------------------------------------------- scoring

fn log_loss(p: f64, y: bool) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    if y {
        -p.ln()
    } else {
        -(1.0 - p).ln()
    }
}
fn brier(p: f64, y: bool) -> f64 {
    let t = if y { 1.0 } else { 0.0 };
    (p - t).powi(2)
}

/// Wilson score interval lower bound at 95%.
fn wilson_lower(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let z = 1.959_963_984_540_054;
    let p = k as f64 / n as f64;
    let nn = n as f64;
    let denom = 1.0 + z * z / nn;
    let centre = p + z * z / (2.0 * nn);
    let half = z * ((p * (1.0 - p) / nn) + (z * z / (4.0 * nn * nn))).sqrt();
    ((centre - half) / denom).max(0.0)
}

fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len();
    if n == 0 {
        return (f64::NAN, f64::NAN);
    }
    let m = v.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, f64::NAN);
    }
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64;
    (m, (var / n as f64).sqrt())
}

// ---------------------------------------------------------------- main

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: arrivaldrift <fit|price|backtest|simcheck|bands> ...");
        std::process::exit(2);
    }
    match args[1].as_str() {
        "fit" => cmd_fit(&args[2..]),
        "price" => cmd_price(&args[2..]),
        "backtest" => cmd_backtest(&args[2..]),
        "simcheck" => cmd_simcheck(&args[2..]),
        "bands" => cmd_bands(&args[2..]),
        o => Err(anyhow!("unknown subcommand {o}")),
    }
}

/// fit <rt_paths.csv> <rt_slug_map.csv> <out_fit.json> [horizons_h,...]
fn cmd_fit(a: &[String]) -> Result<()> {
    if a.len() < 3 {
        return Err(anyhow!("fit <rt_paths.csv> <rt_slug_map.csv> <out.json> [h1,h2,...]"));
    }
    let paths = load_paths(&a[0])?;
    let (hdr, rows) = read_csv(&a[1])?;
    let (c_slug, c_res) = (col(&hdr, "rt_slug")?, col(&hdr, "resolution_ts")?);
    let mut res: BTreeMap<String, i64> = BTreeMap::new();
    for r in rows {
        if r.len() <= c_res.max(c_slug) {
            continue;
        }
        if let Some(t) = parse_ts(r[c_res].trim()) {
            res.insert(r[c_slug].trim().to_string(), t);
        }
    }
    let horizons: Vec<f64> = if a.len() > 3 {
        a[3].split(',').filter_map(|s| s.trim().parse().ok()).collect()
    } else {
        vec![96.0, 72.0, 48.0, 24.0]
    };
    let pairs = build_pairs(&paths, &res, &horizons);
    eprintln!(
        "films with paths: {}  films with resolution ts: {}  training pairs: {}",
        paths.len(),
        res.len(),
        pairs.len()
    );

    // descriptive drift table, per horizon
    println!("# drift by horizon (checkpoint score -> terminal score)");
    println!("horizon_h,n,mean_drift,median_drift,n_down,n_flat,n_up,mean_growth,median_growth,mean_p_late,mean_p_hat");
    for &h in &horizons {
        let sel: Vec<&Pair> = pairs.iter().filter(|p| (p.hours - h).abs() < 1e-6).collect();
        if sel.is_empty() {
            continue;
        }
        let mut d: Vec<f64> = sel
            .iter()
            .filter(|p| p.final_score >= 0 && p.cur_score >= 0)
            .map(|p| (p.final_score - p.cur_score) as f64)
            .collect();
        d.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let med = if d.is_empty() { f64::NAN } else { d[d.len() / 2] };
        let mean = d.iter().sum::<f64>() / d.len().max(1) as f64;
        let mut g: Vec<f64> = sel.iter().map(|p| p.growth).collect();
        g.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let gmed = g[g.len() / 2];
        let gmean = g.iter().sum::<f64>() / g.len() as f64;
        let pl = sel.iter().map(|p| p.p_late).sum::<f64>() / sel.len() as f64;
        let ph = sel
            .iter()
            .map(|p| p.l_t as f64 / p.n_t as f64)
            .sum::<f64>()
            / sel.len() as f64;
        println!(
            "{h},{},{:.3},{:.1},{},{},{},{:.3},{:.3},{:.4},{:.4}",
            d.len(),
            mean,
            med,
            d.iter().filter(|x| **x < 0.0).count(),
            d.iter().filter(|x| **x == 0.0).count(),
            d.iter().filter(|x| **x > 0.0).count(),
            gmean,
            gmed,
            pl,
            ph
        );
    }

    // n-conditioning: does the drift shrink with the observed denominator?
    println!();
    println!("# drift by checkpoint denominator (all horizons pooled)");
    println!("n_bucket,n,mean_drift,mean_p_hat,mean_p_late,mean_gap");
    for (lo, hi, name) in [
        (0u32, 60u32, "n<60"),
        (60, 80, "60<=n<80"),
        (80, 120, "80<=n<120"),
        (120, 200, "120<=n<200"),
        (200, u32::MAX, "n>=200"),
    ] {
        let sel: Vec<&Pair> = pairs
            .iter()
            .filter(|p| p.n_t >= lo && p.n_t < hi && p.final_score >= 0 && p.cur_score >= 0)
            .collect();
        if sel.is_empty() {
            continue;
        }
        let md = sel
            .iter()
            .map(|p| (p.final_score - p.cur_score) as f64)
            .sum::<f64>()
            / sel.len() as f64;
        let ph = sel.iter().map(|p| p.l_t as f64 / p.n_t as f64).sum::<f64>() / sel.len() as f64;
        let pl = sel.iter().map(|p| p.p_late).sum::<f64>() / sel.len() as f64;
        println!("{name},{},{:.3},{:.4},{:.4},{:.4}", sel.len(), md, ph, pl, pl - ph);
    }

    let fit = fit_from_pairs(&pairs)?;
    println!();
    println!("# fit");
    println!("ln(N_T/N_t) = {:.4} + {:.4}*ln(N_t) + {:.4}*ln(h)   resid_sd={:.4}  n={}", fit.ga, fit.gb, fit.gc, fit.gtau, fit.n_growth);
    println!("logit(p_late) = {:.4} + {:.4}*logit(p_hat) + {:.4}*ln(N_t)   resid_sd={:.4}  n={}", fit.la, fit.lb, fit.lc, fit.lsig, fit.n_late);
    fs::write(&a[2], fit.to_json())?;
    eprintln!("wrote {}", a[2]);
    Ok(())
}

/// price <L> <N> <hours> <fit.json> <strike,strike,...>
fn cmd_price(a: &[String]) -> Result<()> {
    if a.len() < 5 {
        return Err(anyhow!("price <L> <N> <hours> <fit.json> <strikes>"));
    }
    let l: u32 = a[0].parse()?;
    let n: u32 = a[1].parse()?;
    let h: f64 = a[2].parse()?;
    let fit = Fit::from_json(&fs::read_to_string(&a[3])?)?;
    let strikes: Vec<i32> = a[4].split(',').filter_map(|s| s.trim().parse().ok()).collect();
    for mode in ["frozen", "null", "full"] {
        let pmf = score_pmf(l, n, h, &fit, mode);
        let lad = ladder(&pmf, &strikes);
        println!(
            "{mode:>7}  mean={:.2} sd={:.2} p10={:.0} p50={:.0} p90={:.0}  ladder: {}",
            pmf_mean(&pmf),
            pmf_sd(&pmf),
            pmf_quantile(&pmf, 0.10),
            pmf_quantile(&pmf, 0.50),
            pmf_quantile(&pmf, 0.90),
            strikes
                .iter()
                .zip(lad.iter())
                .map(|(s, p)| format!("{s}+:{p:.4}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    Ok(())
}

/// simcheck <L> <N> <hours> <fit.json> [draws]
fn cmd_simcheck(a: &[String]) -> Result<()> {
    if a.len() < 4 {
        return Err(anyhow!("simcheck <L> <N> <hours> <fit.json> [draws]"));
    }
    let l: u32 = a[0].parse()?;
    let n: u32 = a[1].parse()?;
    let h: f64 = a[2].parse()?;
    let fit = Fit::from_json(&fs::read_to_string(&a[3])?)?;
    let draws: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(2_000_000);
    let mut rng = Rng::new(0x5EED_1234_ABCD);
    let p_hat = (l as f64 + 0.5) / (n as f64 + 1.0);
    let mu_g = fit.ga + fit.gb * (n as f64).ln() + fit.gc * h.max(0.5).ln();
    let mu_l = fit.la + fit.lb * logit(p_hat) + fit.lc * (n as f64).ln();
    let mut hist = [0.0f64; 101];
    for _ in 0..draws {
        let g = (mu_g + fit.gtau * rng.normal()).exp().max(1.0);
        let m = ((n as f64) * (g - 1.0)).round().max(0.0) as u32;
        let p = expit(mu_l + fit.lsig * rng.normal());
        // exact Bernoulli sum is fine: m is O(100s) and this is a one-off check
        let mut k = 0u32;
        for _ in 0..m {
            if rng.unif() < p {
                k += 1;
            }
        }
        let s = (100.0 * (l + k) as f64 / (n + m) as f64).round().clamp(0.0, 100.0) as usize;
        hist[s] += 1.0;
    }
    for v in hist.iter_mut() {
        *v /= draws as f64;
    }
    let exact = score_pmf(l, n, h, &fit, "full");
    let tv: f64 = (0..101).map(|i| (hist[i] - exact[i]).abs()).sum::<f64>() / 2.0;
    println!("monte-carlo draws={draws}");
    println!("exact  mean={:.4} sd={:.4}", pmf_mean(&exact), pmf_sd(&exact));
    println!("mc     mean={:.4} sd={:.4}", pmf_mean(&hist), pmf_sd(&hist));
    println!("total-variation distance exact vs mc = {tv:.5}");
    Ok(())
}

/// backtest <bt_input.csv> <fit.json> <out_rows.csv>
///
/// bt_input.csv columns:
///   venue,board,film,checkpoint_h,l_t,n_t,final_score,legs
/// where `legs` is `strike:price|strike:price|...` with strike semantics "score >= strike"
/// and price the venue's mid (already gated by the caller).
fn cmd_backtest(a: &[String]) -> Result<()> {
    if a.len() < 3 {
        return Err(anyhow!("backtest <bt_input.csv> <fit.json> <out_rows.csv>"));
    }
    let (hdr, rows) = read_csv(&a[0])?;
    let fit = Fit::from_json(&fs::read_to_string(&a[1])?)?;
    let (c_v, c_b, c_f, c_h, c_l, c_n, c_s, c_legs) = (
        col(&hdr, "venue")?,
        col(&hdr, "board")?,
        col(&hdr, "film")?,
        col(&hdr, "checkpoint_h")?,
        col(&hdr, "l_t")?,
        col(&hdr, "n_t")?,
        col(&hdr, "final_score")?,
        col(&hdr, "legs")?,
    );
    let c_spread = col(&hdr, "spreads").ok();

    let mut out = String::from(
        "venue,board,film,checkpoint_h,strike,l_t,n_t,cur_score,final_score,resolved_yes,market,p_full,p_null,p_frozen,spread\n",
    );
    let mut agg: BTreeMap<(String, i64), Vec<[f64; 4]>> = BTreeMap::new(); // key hours*10; ll: market, full, null, frozen

    for r in rows {
        if r.len() <= c_legs {
            continue;
        }
        let venue = r[c_v].trim().to_string();
        let h: f64 = match r[c_h].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let l: u32 = match r[c_l].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let n: u32 = match r[c_n].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let fs_: i32 = match r[c_s].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if n == 0 {
            continue;
        }
        let cur = (100.0 * l as f64 / n as f64).round() as i32;
        let pmf_full = score_pmf(l, n, h, &fit, "full");
        let pmf_null = score_pmf(l, n, h, &fit, "null");
        let pmf_frz = score_pmf(l, n, h, &fit, "frozen");
        let spreads: Vec<f64> = c_spread
            .map(|c| {
                r.get(c)
                    .map(|s| s.split('|').filter_map(|x| x.trim().parse().ok()).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        for (i, leg) in r[c_legs].split('|').enumerate() {
            let mut it = leg.split(':');
            let (s, p) = match (it.next(), it.next()) {
                (Some(s), Some(p)) => (s, p),
                _ => continue,
            };
            let (strike, price): (i32, f64) = match (s.trim().parse(), p.trim().parse()) {
                (Ok(a), Ok(b)) => (a, b),
                _ => continue,
            };
            if !(0.0..=1.0).contains(&price) {
                continue;
            }
            let y = fs_ >= strike;
            let pf = ladder(&pmf_full, &[strike])[0];
            let pn = ladder(&pmf_null, &[strike])[0];
            let pz = ladder(&pmf_frz, &[strike])[0];
            let sp = spreads.get(i).copied().unwrap_or(f64::NAN);
            out.push_str(&format!(
                "{venue},{},{},{h},{strike},{l},{n},{cur},{fs_},{},{price:.4},{pf:.4},{pn:.4},{pz:.4},{sp:.4}\n",
                r[c_b].trim(),
                r[c_f].trim(),
                if y { 1 } else { 0 }
            ));
            agg.entry((venue.clone(), (h * 10.0).round() as i64)).or_default().push([
                log_loss(price, y),
                log_loss(pf, y),
                log_loss(pn, y),
                log_loss(pz, y),
            ]);
        }
    }
    fs::write(&a[2], out)?;

    println!("venue,checkpoint_h,n_legs,ll_market,ll_full,ll_null,ll_frozen,paired_full_minus_market,se,t");
    for ((v, h10), rows) in &agg {
        let h = *h10 as f64 / 10.0;
        let n = rows.len();
        let m = |i: usize| rows.iter().map(|r| r[i]).sum::<f64>() / n as f64;
        let diff: Vec<f64> = rows.iter().map(|r| r[1] - r[0]).collect();
        let (dm, dse) = mean_se(&diff);
        println!(
            "{v},{h},{n},{:.4},{:.4},{:.4},{:.4},{:+.4},{:.4},{:+.2}",
            m(0),
            m(1),
            m(2),
            m(3),
            dm,
            dse,
            dm / dse
        );
    }
    Ok(())
}

/// bands <scored.csv> — q*, q, Wilson lower bound per price band on the side we would take.
/// Input columns: price,resolved_yes[,fee_rate]. `side` is inferred: we take YES when our model
/// is above the market and NO when below; the caller supplies `take_yes` (1/0).
fn cmd_bands(a: &[String]) -> Result<()> {
    if a.is_empty() {
        return Err(anyhow!("bands <scored.csv>"));
    }
    let (hdr, rows) = read_csv(&a[0])?;
    let (c_p, c_y, c_t) = (
        col(&hdr, "price")?,
        col(&hdr, "resolved_yes")?,
        col(&hdr, "take_yes")?,
    );
    let fee_rate: f64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.05);
    let bands = [
        (0.00, 0.10),
        (0.10, 0.30),
        (0.30, 0.50),
        (0.50, 0.70),
        (0.70, 0.90),
        (0.90, 0.97),
        (0.97, 1.01),
    ];
    println!("band,n,cost,q_star,q,q_lower95,losses_per_100_to_ruin,verdict");
    for (lo, hi) in bands {
        let mut n = 0usize;
        let mut k = 0usize;
        let mut cost_sum = 0.0;
        for r in &rows {
            if r.len() <= c_t {
                continue;
            }
            let p: f64 = match r[c_p].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y: i32 = match r[c_y].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let take_yes: i32 = match r[c_t].trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            // cost of the side we take, in that side's own price units
            let cost = if take_yes == 1 { p } else { 1.0 - p };
            if cost < lo || cost >= hi {
                continue;
            }
            let win = if take_yes == 1 { y == 1 } else { y == 0 };
            n += 1;
            if win {
                k += 1;
            }
            cost_sum += cost + fee_rate * p * (1.0 - p);
        }
        if n == 0 {
            continue;
        }
        let qstar = cost_sum / n as f64;
        let q = k as f64 / n as f64;
        let ql = wilson_lower(k, n);
        // losses per 100 trades that take the observed q down to q*
        let ruin = (q - qstar) * 100.0;
        println!(
            "{lo:.2}-{hi:.2},{n},{:.4},{qstar:.4},{q:.4},{ql:.4},{ruin:.2},{}",
            cost_sum / n as f64 - fee_rate * 0.0,
            if ql > qstar { "CLEARS" } else { "refuse" }
        );
    }
    Ok(())
}
