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

use worker::{Cache, Env, Fetch, Headers, Method, Request, RequestInit, Response, Result};

const REPO: &str = "felix-andreas/orakel";
const BRANCH: &str = "main";
const TTL_SECS: u32 = 60;

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
async fn gh_get(url: &str, tok: &str) -> Result<(u16, String)> {
    match gh_get_once(url, tok).await {
        Ok((status, body)) if !(500..=599).contains(&status) => Ok((status, body)),
        _ => gh_get_once(url, tok).await,
    }
}

async fn gh_get_once(url: &str, tok: &str) -> Result<(u16, String)> {
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
            let _ = h.set("cache-control", &format!("public, max-age={TTL_SECS}"));
            let _ = h.set("x-orakel-upstream-status", &status.to_string());
            let _ = cache.put(url, cacheable).await; // cache failure ≠ error
        }
    }
    Ok((status, body))
}

/// A repo file at request time. A failed read yields empty text and
/// `live: false` — the page renders the error, never a stale copy.
pub async fn repo_text(env: &Env, path: &str) -> Fetched {
    let dead = || Fetched { text: String::new(), live: false };
    let Some(tok) = token(env) else { return dead() };
    let url = format!("https://api.github.com/repos/{REPO}/contents/{path}?ref={BRANCH}");
    match gh_get(&url, &tok).await {
        Ok((200, body)) => Fetched { text: body, live: true },
        // The repository answered and the file is not there. That is data.
        Ok((404, _)) => Fetched { text: String::new(), live: true },
        _ => dead(),
    }
}

/// Commit timestamp of `main`'s HEAD (RFC3339), for the "last updated"
/// indicator — and, on failure, the reason every other read on the page is
/// failing too. Same 60s cache as every other read, so it costs one request a
/// minute across the whole dashboard.
pub async fn head_commit_date(env: &Env) -> std::result::Result<String, String> {
    let Some(tok) = token(env) else {
        return Err(NO_TOKEN.to_string());
    };
    let url = format!("https://api.github.com/repos/{REPO}/commits/{BRANCH}");
    let (status, body) = gh_get(&url, &tok)
        .await
        .map_err(|_| UNREACHABLE.to_string())?;
    if status != 200 {
        return Err(explain(status));
    }
    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| "GitHub returned a response the dashboard could not parse.".to_string())?;
    v["commit"]["committer"]["date"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "GitHub's commit response carried no timestamp.".to_string())
}

/// All blob paths of `main` via one recursive Trees API call.
/// `None` ⇒ the listing could not be read. Callers render nothing plus the
/// error banner rather than implying the repo is empty.
pub async fn repo_tree(env: &Env) -> Option<Vec<String>> {
    let tok = token(env)?;
    let url = format!("https://api.github.com/repos/{REPO}/git/trees/{BRANCH}?recursive=1");
    let (status, body) = gh_get(&url, &tok).await.ok()?;
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
