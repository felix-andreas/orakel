use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use s3::creds::Credentials;
use s3::{Bucket, Region};

pub const DEFAULT_BUCKET: &str = "orakel";

/// Big-blob transfers can take a while; the rust-s3 default (60 s) is too tight
/// for a 200 MB upload on a slow link.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub struct R2Config {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
}

/// Manual impl so a stray `{:?}` can never leak the secret key into logs.
impl std::fmt::Debug for R2Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("R2Config")
            .field("account_id", &self.account_id)
            .field("access_key_id", &"<redacted>")
            .field("secret_access_key", &"<redacted>")
            .field("bucket_name", &self.bucket_name)
            .finish()
    }
}

impl R2Config {
    /// Read config from the process environment. Fails fast with a message
    /// listing every missing variable. Empty values count as missing.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
    }

    /// Same as `from_env` but with an injectable lookup, so the error path is
    /// unit-testable without mutating process-global env vars.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let mut missing = Vec::new();
        let mut require = |name: &'static str| {
            get(name).unwrap_or_else(|| {
                missing.push(name);
                String::new()
            })
        };
        let account_id = require("R2_ACCOUNT_ID");
        let access_key_id = require("R2_ACCESS_KEY_ID");
        let secret_access_key = require("R2_SECRET_ACCESS_KEY");
        if !missing.is_empty() {
            return Err(anyhow!(
                "missing environment variable(s): {} — R2 credentials are required for \
                 push/pull/verify (R2_BUCKET is optional, default \"{DEFAULT_BUCKET}\")",
                missing.join(", ")
            ));
        }
        let bucket_name = get("R2_BUCKET").unwrap_or_else(|| DEFAULT_BUCKET.to_owned());
        Ok(Self {
            account_id,
            access_key_id,
            secret_access_key,
            bucket_name,
        })
    }

    /// S3 client for the bucket named in the environment (`R2_BUCKET`).
    pub fn bucket(&self) -> Result<Box<Bucket>> {
        self.bucket_named(&self.bucket_name)
    }

    /// S3 client for an explicit bucket name (pull/verify use the bucket
    /// recorded in the manifest, which is where the bytes actually live).
    pub fn bucket_named(&self, name: &str) -> Result<Box<Bucket>> {
        let region = Region::Custom {
            region: "auto".to_owned(),
            endpoint: format!("https://{}.r2.cloudflarestorage.com", self.account_id),
        };
        let credentials = Credentials::new(
            Some(&self.access_key_id),
            Some(&self.secret_access_key),
            None,
            None,
            None,
        )
        .context("building R2 credentials")?;
        let bucket = Bucket::new(name, region, credentials)
            .context("configuring R2 bucket client")?
            .with_path_style();
        let bucket = bucket
            .with_request_timeout(REQUEST_TIMEOUT)
            .context("setting request timeout")?;
        Ok(bucket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name: &str| map.get(name).cloned()
    }

    #[test]
    fn missing_everything_lists_all_required_vars() {
        let err = R2Config::from_lookup(lookup(&[])).unwrap_err().to_string();
        assert!(err.contains("R2_ACCOUNT_ID"), "got: {err}");
        assert!(err.contains("R2_ACCESS_KEY_ID"), "got: {err}");
        assert!(err.contains("R2_SECRET_ACCESS_KEY"), "got: {err}");
    }

    #[test]
    fn missing_one_var_lists_only_that_var() {
        let err = R2Config::from_lookup(lookup(&[
            ("R2_ACCOUNT_ID", "acct"),
            ("R2_ACCESS_KEY_ID", "key"),
        ]))
        .unwrap_err()
        .to_string();
        assert!(err.contains("R2_SECRET_ACCESS_KEY"), "got: {err}");
        assert!(!err.contains("R2_ACCOUNT_ID,"), "got: {err}");
        assert!(!err.contains("R2_ACCESS_KEY_ID,"), "got: {err}");
    }

    #[test]
    fn bucket_defaults_to_orakel() {
        let config = R2Config::from_lookup(lookup(&[
            ("R2_ACCOUNT_ID", "acct"),
            ("R2_ACCESS_KEY_ID", "key"),
            ("R2_SECRET_ACCESS_KEY", "secret"),
        ]))
        .unwrap();
        assert_eq!(config.bucket_name, "orakel");
        assert_eq!(config.account_id, "acct");
    }

    #[test]
    fn bucket_env_var_overrides_default() {
        let config = R2Config::from_lookup(lookup(&[
            ("R2_ACCOUNT_ID", "acct"),
            ("R2_ACCESS_KEY_ID", "key"),
            ("R2_SECRET_ACCESS_KEY", "secret"),
            ("R2_BUCKET", "other-bucket"),
        ]))
        .unwrap();
        assert_eq!(config.bucket_name, "other-bucket");
    }

    #[test]
    fn bucket_client_builds_without_network() {
        let config = R2Config {
            account_id: "acct".to_owned(),
            access_key_id: "key".to_owned(),
            secret_access_key: "secret".to_owned(),
            bucket_name: "orakel".to_owned(),
        };
        let bucket = config.bucket().unwrap();
        assert!(bucket.host().contains("acct.r2.cloudflarestorage.com"), "host: {}", bucket.host());
    }
}
