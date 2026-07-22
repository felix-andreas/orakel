mod config;
mod hash;
mod manifest;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use s3::Bucket;

use config::R2Config;
use manifest::Manifest;

/// Hard cap: r2data freezes datasets, it is not a bulk-archive pipe.
const MAX_PUSH_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Parser)]
#[command(
    name = "r2data",
    version,
    about = "Freeze datasets to Cloudflare R2: git keeps the manifest, R2 keeps the bytes.",
    after_help = "Environment: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY \
                  (required), R2_BUCKET (default \"orakel\")."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Upload a file to blobs/<sha256> and write <file>.r2.json next to it.
    ///
    /// The upload happens (and is verified) BEFORE the manifest is written, so a
    /// committed manifest always references bytes that exist in R2. If the blob
    /// is already there the upload is skipped (content-addressed dedup).
    Push {
        /// File to freeze (max 200 MB).
        file: PathBuf,
        /// URL the data was fetched from, recorded in the manifest.
        #[arg(long)]
        source: Option<String>,
        /// Free-form note recorded in the manifest.
        #[arg(long)]
        note: Option<String>,
    },
    /// Download the object a manifest references and verify its sha256.
    Pull {
        /// Path to a <name>.r2.json manifest.
        manifest: PathBuf,
        /// Output path (default: original_name, next to the manifest).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// HEAD each manifest's object: report exists/missing and size match.
    ///
    /// Exits non-zero if any object is missing or its size disagrees.
    Verify {
        /// One or more <name>.r2.json manifests.
        #[arg(required = true)]
        manifests: Vec<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Push { file, source, note } => push(&file, source, note),
        Command::Pull { manifest, out } => pull(&manifest, out),
        Command::Verify { manifests } => verify(&manifests),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn push(file: &Path, source_url: Option<String>, note: Option<String>) -> Result<()> {
    let config = R2Config::from_env()?;

    let size = fs::metadata(file)
        .with_context(|| format!("cannot stat {}", file.display()))?
        .len();
    check_push_size(size)?;

    let original_name = file
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("{} has no usable file name", file.display()))?
        .to_owned();
    let content_type = manifest::content_type_for(&original_name);

    let bytes = fs::read(file).with_context(|| format!("cannot read {}", file.display()))?;
    let sha256 = hash::sha256_hex(&bytes);
    let key = format!("blobs/{sha256}");

    let bucket = config.bucket()?;
    if object_exists(&bucket, &key)?.is_some() {
        eprintln!(
            "{key} already exists in bucket {} — skipping upload (content-addressed dedup)",
            config.bucket_name
        );
    } else {
        eprintln!(
            "uploading {} ({} bytes) to {}/{key} ...",
            file.display(),
            bytes.len(),
            config.bucket_name
        );
        let response = bucket
            .put_object_with_content_type(&key, &bytes, &content_type)
            .with_context(|| format!("uploading {key}"))?;
        let code = response.status_code();
        if !(200..300).contains(&code) {
            bail!(
                "upload of {key} failed with HTTP {code}: {}",
                response.as_str().unwrap_or("<non-utf8 body>")
            );
        }
        // Invariant check: the manifest may only be written once the bytes
        // verifiably exist, so a committed manifest can never dangle.
        if object_exists(&bucket, &key)?.is_none() {
            bail!("upload reported success but HEAD {key} finds nothing — not writing a manifest");
        }
    }

    let manifest = Manifest {
        key,
        sha256,
        bytes: bytes.len() as u64,
        content_type,
        original_name,
        source_url,
        note,
        fetched_at: humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
        bucket: config.bucket_name.clone(),
    };
    let manifest_path = manifest_path_for(file);
    manifest.save(&manifest_path)?;
    println!("{}", manifest_path.display());
    Ok(())
}

fn pull(manifest_path: &Path, out: Option<PathBuf>) -> Result<()> {
    let config = R2Config::from_env()?;
    let manifest = Manifest::load(manifest_path)?;

    let out_path = out.unwrap_or_else(|| {
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(&manifest.original_name)
    });

    // The manifest records which bucket holds the bytes; trust it over R2_BUCKET.
    let bucket = config.bucket_named(&manifest.bucket)?;
    let response = bucket
        .get_object(&manifest.key)
        .with_context(|| format!("downloading {}", manifest.key))?;
    let code = response.status_code();
    if code == 404 {
        bail!(
            "object {} is MISSING from bucket {} — dangling manifest {}",
            manifest.key,
            manifest.bucket,
            manifest_path.display()
        );
    }
    if !(200..300).contains(&code) {
        bail!("GET {} failed with HTTP {code}", manifest.key);
    }

    let bytes = response.bytes();
    let actual_sha256 = hash::sha256_hex(bytes);
    if actual_sha256 != manifest.sha256 {
        bail!(
            "sha256 MISMATCH for {}: manifest says {}, downloaded bytes hash to {} — \
             refusing to write {}",
            manifest.key,
            manifest.sha256,
            actual_sha256,
            out_path.display()
        );
    }

    fs::write(&out_path, bytes).with_context(|| format!("cannot write {}", out_path.display()))?;
    eprintln!(
        "verified sha256 {} ({} bytes)",
        manifest.sha256,
        bytes.len()
    );
    println!("{}", out_path.display());
    Ok(())
}

fn verify(manifests: &[PathBuf]) -> Result<()> {
    let config = R2Config::from_env()?;
    let mut failures = 0usize;
    for path in manifests {
        match verify_one(&config, path) {
            Ok(line) => println!("OK      {line}"),
            Err(err) => {
                failures += 1;
                println!("FAIL    {}: {err:#}", path.display());
            }
        }
    }
    if failures > 0 {
        bail!("{failures} of {} manifest(s) failed verification", manifests.len());
    }
    Ok(())
}

fn verify_one(config: &R2Config, path: &Path) -> Result<String> {
    let manifest = Manifest::load(path)?;
    let bucket = config.bucket_named(&manifest.bucket)?;
    match object_exists(&bucket, &manifest.key)? {
        None => bail!(
            "object {} MISSING from bucket {}",
            manifest.key,
            manifest.bucket
        ),
        Some(Some(remote_bytes)) if remote_bytes != manifest.bytes => bail!(
            "SIZE MISMATCH for {}: manifest says {} bytes, R2 reports {}",
            manifest.key,
            manifest.bytes,
            remote_bytes
        ),
        Some(Some(remote_bytes)) => Ok(format!(
            "{}: {} exists in {} ({remote_bytes} bytes)",
            path.display(),
            manifest.key,
            manifest.bucket
        )),
        Some(None) => Ok(format!(
            "{}: {} exists in {} (size unreported by HEAD)",
            path.display(),
            manifest.key,
            manifest.bucket
        )),
    }
}

/// HEAD the object. `Ok(None)` = 404, `Ok(Some(content_length))` = exists.
/// Any other status (auth failure, throttle, ...) is an error — never mistake
/// "can't tell" for "doesn't exist".
fn object_exists(bucket: &Bucket, key: &str) -> Result<Option<Option<u64>>> {
    let (head, code) = bucket
        .head_object(key)
        .with_context(|| format!("HEAD {key}"))?;
    match code {
        200 => Ok(Some(head.content_length.and_then(|n| u64::try_from(n).ok()))),
        404 => Ok(None),
        _ => bail!("HEAD {key} returned unexpected HTTP {code} (check credentials/bucket)"),
    }
}

fn check_push_size(bytes: u64) -> Result<()> {
    if bytes > MAX_PUSH_BYTES {
        bail!(
            "file is {bytes} bytes, over the 200 MB ({MAX_PUSH_BYTES} bytes) push limit — \
             r2data freezes datasets, not bulk archives; trim, split, or compress the file"
        );
    }
    Ok(())
}

/// `data/foo.csv` -> `data/foo.csv.r2.json` (always next to the file).
fn manifest_path_for(file: &Path) -> PathBuf {
    let mut os = file.as_os_str().to_owned();
    os.push(".r2.json");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_guard_allows_up_to_200mb() {
        assert!(check_push_size(0).is_ok());
        assert!(check_push_size(MAX_PUSH_BYTES).is_ok());
    }

    #[test]
    fn size_guard_rejects_over_200mb() {
        let err = check_push_size(MAX_PUSH_BYTES + 1).unwrap_err().to_string();
        assert!(err.contains("200 MB"), "got: {err}");
    }

    #[test]
    fn size_guard_rejects_real_sparse_file_over_200mb() {
        // A sparse file: 200 MB + 1 byte of apparent size, ~0 bytes on disk.
        let path = std::env::temp_dir().join(format!("r2data-sparse-{}.bin", std::process::id()));
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_PUSH_BYTES + 1).unwrap();
        let size = fs::metadata(&path).unwrap().len();
        let result = check_push_size(size);
        fs::remove_file(&path).ok();
        assert!(result.is_err());
    }

    #[test]
    fn manifest_path_is_next_to_the_file() {
        assert_eq!(
            manifest_path_for(Path::new("data/foo.csv")),
            PathBuf::from("data/foo.csv.r2.json")
        );
        assert_eq!(
            manifest_path_for(Path::new("bare")),
            PathBuf::from("bare.r2.json")
        );
    }
}
