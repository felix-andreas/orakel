//! Live GitHub reads with Cache API caching and embedded-snapshot fallback.
//!
//! Every repo read goes through here: file bodies via the Contents API
//! (`Accept: application/vnd.github.raw+json`), directory knowledge via ONE
//! recursive Trees API call (every listing — runs, inboxes, wiki, ideas — is
//! derived from it locally, keeping the GitHub request count small).
//! Responses are cached ~60s in the Workers Cache API keyed on the API URL,
//! so a click-around costs a handful of GitHub requests, not dozens.
//!
//! Resilience: the caller passes the build-time embedded fallback. No
//! GITHUB_TOKEN secret, or a failed fetch → `live: false` and the embedded
//! text is served (pages render a "stale" notice). A 404 is a *live* answer
//! (the file genuinely doesn't exist → empty text), not staleness.

use worker::{Cache, Env, Fetch, Headers, Method, Request, RequestInit, Response, Result};

const REPO: &str = "felix-andreas/orakel";
const BRANCH: &str = "main";
const TTL_SECS: u32 = 60;

pub struct Fetched {
    pub text: String,
    pub live: bool,
}

fn token(env: &Env) -> Option<String> {
    env.secret("GITHUB_TOKEN")
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
}

/// GET a GitHub API URL with auth → (upstream status, body). Cached ~60s via
/// the Cache API keyed on the URL; the upstream status is tucked into a
/// response header so 404s are cached too and don't re-fetch every request.
async fn gh_get(url: &str, tok: &str) -> Result<(u16, String)> {
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

/// A repo file at request time, embedded fallback when live is unavailable.
pub async fn repo_text(env: &Env, path: &str, embedded: &str) -> Fetched {
    let Some(tok) = token(env) else {
        return Fetched { text: embedded.to_string(), live: false };
    };
    let url = format!("https://api.github.com/repos/{REPO}/contents/{path}?ref={BRANCH}");
    match gh_get(&url, &tok).await {
        Ok((200, body)) => Fetched { text: body, live: true },
        Ok((404, _)) => Fetched { text: String::new(), live: true },
        _ => Fetched { text: embedded.to_string(), live: false },
    }
}

/// All blob paths of `main` via one recursive Trees API call.
/// `None` ⇒ live listing unavailable (no token / fetch failed).
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
