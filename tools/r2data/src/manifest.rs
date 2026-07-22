use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The small JSON committed to git where the data logically lives.
/// Invariant: a manifest is only ever written after the bytes it references
/// exist in R2 (see `push` in main.rs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Immutable content-addressed object key: `blobs/<sha256>`.
    pub key: String,
    pub sha256: String,
    pub bytes: u64,
    pub content_type: String,
    pub original_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// RFC3339 UTC timestamp of when the data was frozen.
    pub fetched_at: String,
    pub bucket: String,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("cannot read manifest {}", path.display()))?;
        serde_json::from_str(&text)
            .with_context(|| format!("{} is not a valid r2data manifest", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut text = serde_json::to_string_pretty(self).context("serializing manifest")?;
        text.push('\n');
        fs::write(path, text).with_context(|| format!("cannot write manifest {}", path.display()))
    }
}

/// MIME type from the file extension; `application/octet-stream` when unknown.
pub fn content_type_for(name: &str) -> String {
    let ext = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default();
    let content_type = match ext.as_str() {
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "json" => "application/json",
        "jsonl" | "ndjson" => "application/x-ndjson",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "xml" => "application/xml",
        "parquet" => "application/vnd.apache.parquet",
        "arrow" | "feather" => "application/vnd.apache.arrow.file",
        "gz" => "application/gzip",
        "zst" => "application/zstd",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "sqlite" | "db" => "application/vnd.sqlite3",
        _ => "application/octet-stream",
    };
    content_type.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest {
            key: "blobs/ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                .to_owned(),
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
            bytes: 3,
            content_type: "text/csv".to_owned(),
            original_name: "prices.csv".to_owned(),
            source_url: Some("https://example.com/prices".to_owned()),
            note: Some("frozen for the gbm-v1 backtest".to_owned()),
            fetched_at: "2026-07-22T12:00:00Z".to_owned(),
            bucket: "orakel".to_owned(),
        }
    }

    #[test]
    fn json_round_trip() {
        let manifest = sample();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, back);
    }

    #[test]
    fn optional_fields_are_omitted_when_none() {
        let mut manifest = sample();
        manifest.source_url = None;
        manifest.note = None;
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(!json.contains("source_url"));
        assert!(!json.contains("note"));
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, back);
    }

    #[test]
    fn save_and_load_round_trip() {
        let manifest = sample();
        let path = std::env::temp_dir().join(format!(
            "r2data-manifest-test-{}.r2.json",
            std::process::id()
        ));
        manifest.save(&path).unwrap();
        let back = Manifest::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(manifest, back);
    }

    #[test]
    fn content_type_guessing() {
        assert_eq!(content_type_for("prices.csv"), "text/csv");
        assert_eq!(content_type_for("PRICES.CSV"), "text/csv");
        assert_eq!(content_type_for("dump.json"), "application/json");
        assert_eq!(content_type_for("trades.parquet"), "application/vnd.apache.parquet");
        assert_eq!(content_type_for("archive.tar.gz"), "application/gzip");
        assert_eq!(content_type_for("weird.unknownext"), "application/octet-stream");
        assert_eq!(content_type_for("no_extension"), "application/octet-stream");
    }
}
