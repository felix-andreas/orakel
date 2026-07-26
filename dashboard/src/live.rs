//! Live GitHub reads with Cache API caching. **There is no fallback.**
//!
//! Every repo read goes through here: file bodies via the Contents API
//! (`Accept: application/vnd.github.raw+json`), directory knowledge via ONE
//! recursive Trees API call (every listing — runs, inboxes, wiki, ideas — is
//! derived from it locally, keeping the GitHub request count small).
//! Responses are cached ~60s in the Workers Cache API keyed on the API URL,
//! so a click-around costs a handful of GitHub requests, not dozens.
//!
//! Failure policy: `main` at request time is the only source of truth. Earlier
//! builds shipped a copy of the repo compiled into the Worker and served it
//! when GitHub was unreachable — which meant an outage looked like a working
//! dashboard showing quietly outdated numbers. A wrong number that looks right
//! is worse than a visible error, so a failed read now returns empty text with
//! `live: false` and the page says so, loudly, with the reason.
//!
//! A 404 is a *successful* answer ("that file does not exist" → empty text),
//! not a failure.

use std::cell::RefCell;

use worker::{Cache, Date, Env, Fetch, Headers, Method, Request, RequestInit, Response, Result};

const REPO: &str = "felix-andreas/orakel";
const BRANCH: &str = "main";

/// How stale the answer to "which commit is `main` at?" may be. This is the
/// dashboard's real freshness knob — everything else follows from it.
const TTL_HEAD: u32 = 60;

/// How long a file read may be cached. Content reads are pinned to a commit
/// SHA, so their URLs are immutable and a long TTL is not a staleness
/// trade-off: a new commit produces new URLs, and the old entries simply go
/// unused. Before this, every entry expired on a 60-second clock whether or
/// not anything had changed, so the first visitor each minute paid full price
/// for the whole page (measured: 2.9s against 0.87s warm).
const TTL_PINNED: u32 = 86_400;

/// Which commit `main` is at, plus when it was made.
#[derive(Clone)]
pub struct Head {
    pub sha: String,
    pub date: String,
}

// The SHA lookup is memoised for the isolate, not just cached in the Cache
// API, because every content read needs it first: a cache round trip per read
// would double the marginal cost of a read (~33ms each, ~20 reads a page).
// Staleness is bounded by TTL_HEAD exactly as before.
thread_local! {
    static HEAD_MEMO: RefCell<Option<(Head, f64)>> = const { RefCell::new(None) };
    /// Paths whose read failed during THIS request, so the banner can name them.
    /// "Part of this page could not be read" is not a diagnosis.
    static FAILED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Resolve HEAD once, at the start of the request.
///
/// Every read needs the commit SHA, so before this existed each read consulted
/// the memo independently — and a memo that lapsed *between* two reads of the
/// same page did two bad things. A failed refresh made that one read return
/// empty while its siblings succeeded, which is the "part of this page is
/// missing" banner appearing over a page that looks complete. And a refresh
/// that SUCCEEDED after `main` moved pinned the page's later reads to a
/// different commit than its earlier ones — a page silently mixing two
/// commits, which is the exact failure the fallback pack was removed to avoid.
///
/// Priming here means no read can trigger a refresh mid-page: a request takes
/// well under a second, the TTL is 60, so the whole page sees one SHA.
pub async fn begin_request(env: &Env) {
    FAILED.with(|f| f.borrow_mut().clear());
    let _ = head(env).await;
}

fn note_failure(path: &str) {
    FAILED.with(|f| {
        let mut f = f.borrow_mut();
        if !f.iter().any(|p| p == path) {
            f.push(path.to_string());
        }
    });
}

/// What failed during this request, for the banner.
pub fn failed_paths() -> Vec<String> {
    FAILED.with(|f| f.borrow().clone())
}

fn memo_get() -> Option<Head> {
    let now = Date::now().as_millis() as f64;
    HEAD_MEMO.with(|m| {
        let m = m.borrow();
        let (head, at) = m.as_ref()?;
        (now - at < TTL_HEAD as f64 * 1000.0).then(|| head.clone())
    })
}

/// Last known HEAD regardless of age. A refresh that fails should not make the
/// dashboard forget which commit it was serving a second ago — that turns a
/// blip in GitHub's availability into a blank page.
fn memo_stale() -> Option<Head> {
    HEAD_MEMO.with(|m| m.borrow().as_ref().map(|(h, _)| h.clone()))
}

fn memo_put(head: &Head) {
    let now = Date::now().as_millis() as f64;
    HEAD_MEMO.with(|m| *m.borrow_mut() = Some((head.clone(), now)));
}

pub struct Fetched {
    pub text: String,
    pub live: bool,
}

/// Why a live read failed, in words a human can act on. GitHub's status codes
/// map to distinct fixes, so name the fix rather than the number alone.
pub fn explain(status: u16) -> String {
    match status {
        401 => "GitHub rejected the token (401). The GITHUB_TOKEN worker secret is invalid or revoked.".into(),
        403 => "GitHub refused the request (403) — rate limit exhausted, or the token lacks access to the repository.".into(),
        404 => "GitHub returned 404 for the repository itself. Check the repo name and the token's scope.".into(),
        429 => "GitHub rate-limited the dashboard (429). It will recover on its own; reload in a minute.".into(),
        500..=599 => format!("GitHub returned a server error ({status}). This is upstream; reload shortly."),
        other => format!("GitHub returned an unexpected status ({other})."),
    }
}

/// No token configured — the one failure that is our own misconfiguration.
pub const NO_TOKEN: &str =
    "No GITHUB_TOKEN worker secret is set, so the dashboard cannot read the repository. \
     Set it with `wrangler secret put GITHUB_TOKEN`.";

/// The network never answered.
pub const UNREACHABLE: &str = "Could not reach the GitHub API at all (network or DNS failure).";

fn token(env: &Env) -> Option<String> {
    env.secret("GITHUB_TOKEN")
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
}

/// GET a GitHub API URL with auth → (upstream status, body). Cached ~60s via
/// the Cache API keyed on the URL; the upstream status is tucked into a
/// response header so 404s are cached too and don't re-fetch every request.
/// One attempt, plus one retry when the failure is the kind that a retry can
/// fix. With no fallback copy of the repo, a single transient blip would
/// otherwise blank a page — and a retry costs one subrequest and no staleness,
/// unlike serving an old copy. Definitive answers (200/401/403/404) are never
/// retried: repeating them is pointless and, for 403, actively hostile to the
/// rate limit.
async fn gh_get(url: &str, tok: &str, ttl: u32) -> Result<(u16, String)> {
    match gh_get_once(url, tok, ttl).await {
        Ok((status, body)) if !(500..=599).contains(&status) => Ok((status, body)),
        _ => gh_get_once(url, tok, ttl).await,
    }
}

async fn gh_get_once(url: &str, tok: &str, ttl: u32) -> Result<(u16, String)> {
    let cache = Cache::default();
    if let Ok(Some(mut hit)) = cache.get(url, true).await {
        let status = hit
            .headers()
            .get("x-orakel-upstream-status")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200);
        return Ok((status, hit.text().await?));
    }

    let headers = Headers::new();
    headers.set("authorization", &format!("Bearer {tok}"))?;
    headers.set("user-agent", "orakel-dashboard")?; // required by the GitHub API
    headers.set("accept", "application/vnd.github.raw+json")?;
    headers.set("x-github-api-version", "2022-11-28")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);
    let req = Request::new_with_init(url, &init)?;
    let mut resp = Fetch::Request(req).send().await?;
    let status = resp.status_code();
    let body = resp.text().await?;

    // Cache definitive answers only (success + not-found); transient errors
    // (403 rate limit, 5xx) must not stick for a minute.
    if status == 200 || status == 404 {
        if let Ok(mut cacheable) = Response::ok(body.clone()) {
            let h = cacheable.headers_mut();
            let _ = h.set("cache-control", &format!("public, max-age={ttl}"));
            let _ = h.set("x-orakel-upstream-status", &status.to_string());
            let _ = cache.put(url, cacheable).await; // cache failure ≠ error
        }
    }
    Ok((status, body))
}

/// Which commit `main` is at. Asked of GitHub at most once per `TTL_HEAD`
/// seconds per isolate; every content read is then pinned to the SHA it
/// returns. On failure this also carries the reason every other read on the
/// page is failing.
pub async fn head(env: &Env) -> std::result::Result<Head, String> {
    if let Some(h) = memo_get() {
        return Ok(h);
    }
    let stale = memo_stale();
    let recover = |e: String| stale.clone().ok_or(e);
    let Some(tok) = token(env) else {
        return recover(NO_TOKEN.to_string());
    };
    let url = format!("https://api.github.com/repos/{REPO}/commits/{BRANCH}");
    let Ok((status, body)) = gh_get(&url, &tok, TTL_HEAD).await else {
        return recover(UNREACHABLE.to_string());
    };
    if status != 200 {
        return recover(explain(status));
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else {
        return recover("GitHub returned a response the dashboard could not parse.".to_string());
    };
    let (Some(sha), Some(date)) = (
        v["sha"].as_str(),
        v["commit"]["committer"]["date"].as_str(),
    ) else {
        return recover("GitHub's commit response was missing its SHA or timestamp.".to_string());
    };
    let head = Head { sha: sha.to_string(), date: date.to_string() };
    memo_put(&head);
    Ok(head)
}

/// A repo file at request time. A failed read yields empty text and
/// `live: false` — the page renders the error, never a stale copy.
pub async fn repo_text(env: &Env, path: &str) -> Fetched {
    let dead = || {
        note_failure(path);
        Fetched { text: String::new(), live: false }
    };
    let (Some(tok), Ok(h)) = (token(env), head(env).await) else {
        return dead();
    };
    // Pinned to the SHA, not to `main`: the URL then names one immutable blob,
    // which is what lets it cache for a day instead of a minute.
    let url = format!(
        "https://api.github.com/repos/{REPO}/contents/{path}?ref={}",
        h.sha
    );
    match gh_get(&url, &tok, TTL_PINNED).await {
        Ok((200, body)) => Fetched { text: body, live: true },
        // The repository answered and the file is not there. That is data.
        Ok((404, _)) => Fetched { text: String::new(), live: true },
        _ => dead(),
    }
}

/// All blob paths via one recursive Trees API call, pinned to the same commit
/// as every content read on the page — so a listing can never disagree with
/// the files it points at. `None` ⇒ the listing could not be read; callers
/// render nothing plus the error banner rather than implying an empty repo.
pub async fn repo_tree(env: &Env) -> Option<Vec<String>> {
    let tok = token(env)?;
    let h = head(env).await.ok()?;
    let url = format!(
        "https://api.github.com/repos/{REPO}/git/trees/{}?recursive=1",
        h.sha
    );
    let (status, body) = gh_get(&url, &tok, TTL_PINNED).await.ok()?;
    if status != 200 {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    Some(
        v["tree"]
            .as_array()?
            .iter()
            .filter(|e| e["type"] == "blob")
            .filter_map(|e| e["path"].as_str().map(str::to_string))
            .collect(),
    )
}
