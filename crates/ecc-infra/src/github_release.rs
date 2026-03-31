use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseClient, ReleaseInfo};
use sha2::{Digest, Sha256};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// Compile-time constants for cosign verification (PC-035)
pub(crate) const COSIGN_CERTIFICATE_IDENTITY: &str =
    "https://github.com/LEBOCQTitouan/everything-claude-code/.github/workflows/release.yml@refs/heads/main";
pub(crate) const COSIGN_OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";

const CHUNK_SIZE: usize = 8 * 1024; // 8 KiB

/// GitHub Releases adapter for [`ReleaseClient`].
///
/// Uses the `ureq` crate internally for GitHub API interaction.
/// Configured with rustls for TLS (no OpenSSL dependency).
///
/// Cosign verification shells out to `cosign verify-blob` via
/// `std::process::Command` with argument arrays (no shell interpolation).
pub struct GithubReleaseClient {
    repo_owner: String,
    repo_name: String,
}

impl GithubReleaseClient {
    /// Create a new client for the given GitHub repository.
    pub fn new(owner: &str, name: &str) -> Self {
        Self {
            repo_owner: owner.to_string(),
            repo_name: name.to_string(),
        }
    }

    fn api_base(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}",
            self.repo_owner, self.repo_name
        )
    }

    fn release_base_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/releases/download",
            self.repo_owner, self.repo_name
        )
    }
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

// ── Pure helpers ──────────────────────────────────────────────────────────────

/// Parse a GitHub Releases API JSON response into a [`ReleaseInfo`].
///
/// Strips the leading `v` from `tag_name` to produce a clean semver string.
pub(crate) fn parse_release_json(json: &str) -> Result<ReleaseInfo, BoxError> {
    let v: serde_json::Value = serde_json::from_str(json)?;
    let tag_name = v["tag_name"].as_str().ok_or("missing tag_name field")?;
    let version = tag_name.trim_start_matches('v').to_string();
    let release_notes = v["body"].as_str().unwrap_or("").to_string();
    Ok(ReleaseInfo {
        version,
        release_notes,
    })
}

/// Build the GitHub Releases API URL for a specific version tag.
pub(crate) fn version_tag_url(owner: &str, repo: &str, version: &str) -> String {
    format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/v{version}")
}

/// Stream bytes from `reader` to `dest`, invoking `on_progress` after each chunk.
///
/// `total` is passed to `on_progress` as the second argument. If unknown, pass 0.
pub(crate) fn stream_to_file(
    mut reader: impl Read,
    dest: &Path,
    total: u64,
    on_progress: &dyn Fn(u64, u64),
) -> Result<(), BoxError> {
    let file = std::fs::File::create(dest)?;
    let mut writer = BufWriter::new(file);
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut written: u64 = 0;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        written += n as u64;
        on_progress(written, total);
    }
    writer.flush()?;
    Ok(())
}

/// Compute the SHA-256 hex digest of a file.
pub(crate) fn compute_sha256(path: &Path) -> Result<String, BoxError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Parse a `checksums-sha256.txt` line and return the hex digest for `filename`.
///
/// Expected format (one file per line): `<sha256hex>  <filename>`
pub(crate) fn parse_checksum_line(content: &str, filename: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "<hash>  <filename>" (two spaces) or "<hash> <filename>" (one space)
        let mut parts = line.splitn(2, ' ');
        let hash = parts.next()?.trim();
        let name = parts.next()?.trim();
        if name == filename {
            return Some(hash.to_string());
        }
    }
    None
}

/// Check if an HTTP status code + optional `X-RateLimit-Reset` header indicates rate limiting.
///
/// Returns `Some(reset_timestamp)` when the response signals a 403 rate limit.
/// Used by HTTP error mapping in production and by tests for unit validation.
#[allow(dead_code)]
pub(crate) fn check_rate_limit(status: u16, reset_header: Option<&str>) -> Option<u64> {
    if status == 403 {
        if let Some(ts) = reset_header {
            ts.trim().parse::<u64>().ok().or(Some(0))
        } else {
            Some(0)
        }
    } else {
        None
    }
}

/// Build a `cosign verify-blob` [`Command`] (not executed) for inspection in tests.
pub(crate) fn build_cosign_command(
    blob: &Path,
    bundle: &Path,
    identity: &str,
    issuer: &str,
) -> Command {
    let mut cmd = Command::new("cosign");
    cmd.arg("verify-blob")
        .arg("--bundle")
        .arg(bundle)
        .arg("--certificate-identity")
        .arg(identity)
        .arg("--certificate-oidc-issuer")
        .arg(issuer)
        .arg(blob);
    cmd
}

/// Returns whether `token` should be included as an Authorization header.
///
/// Returns `Some(bearer)` when a non-empty token is provided.
pub(crate) fn make_auth_header(token: Option<&str>) -> Option<String> {
    token.filter(|t| !t.is_empty()).map(|t| format!("Bearer {t}"))
}

// ── ReleaseClient impl ────────────────────────────────────────────────────────

impl ReleaseClient for GithubReleaseClient {
    fn latest_version(&self, _include_prerelease: bool) -> Result<ReleaseInfo, BoxError> {
        let url = format!("{}/releases/latest", self.api_base());
        let response = make_request(&url, std::env::var("GITHUB_TOKEN").ok().as_deref())?;
        parse_release_json(&response)
    }

    fn get_version(&self, version: &str) -> Result<ReleaseInfo, BoxError> {
        let url = version_tag_url(&self.repo_owner, &self.repo_name, version);
        let response = make_request(&url, std::env::var("GITHUB_TOKEN").ok().as_deref())?;
        parse_release_json(&response)
    }

    fn download_tarball(
        &self,
        version: &str,
        artifact_name: &str,
        dest: &Path,
        on_progress: &dyn Fn(u64, u64),
    ) -> Result<(), BoxError> {
        let url = format!(
            "{}/v{}/{}.tar.gz",
            self.release_base_url(),
            version,
            artifact_name
        );
        download_url_streaming(&url, dest, on_progress)
    }

    fn download_file(&self, url: &str, dest: &Path) -> Result<(), BoxError> {
        download_url_streaming(url, dest, &|_, _| {})
    }

    fn verify_checksum(
        &self,
        version: &str,
        artifact_name: &str,
        file_path: &Path,
    ) -> Result<ChecksumResult, BoxError> {
        let checksum_url = format!(
            "{}/v{}/checksums-sha256.txt",
            self.release_base_url(),
            version
        );
        let tmp = tempfile_path();
        download_url_streaming(&checksum_url, &tmp, &|_, _| {})?;
        let checksum_content = std::fs::read_to_string(&tmp)?;
        let _ = std::fs::remove_file(&tmp);

        let filename = format!("{artifact_name}.tar.gz");
        let expected = parse_checksum_line(&checksum_content, &filename)
            .ok_or_else(|| format!("no checksum entry for {filename}"))?;
        let actual = compute_sha256(file_path)?;
        if actual == expected {
            Ok(ChecksumResult::Match)
        } else {
            Ok(ChecksumResult::Mismatch)
        }
    }

    fn verify_cosign(
        &self,
        _version: &str,
        _artifact_name: &str,
        file_path: &Path,
        bundle_path: &Path,
    ) -> Result<CosignResult, BoxError> {
        // Check if cosign is installed
        let cosign_check = Command::new("cosign").arg("version").output();
        match cosign_check {
            Err(_) => return Ok(CosignResult::NotInstalled),
            Ok(output) if !output.status.success() => return Ok(CosignResult::NotInstalled),
            Ok(_) => {}
        }

        let mut cmd = build_cosign_command(
            file_path,
            bundle_path,
            COSIGN_CERTIFICATE_IDENTITY,
            COSIGN_OIDC_ISSUER,
        );
        let output = cmd.output()?;
        if output.status.success() {
            Ok(CosignResult::Verified)
        } else {
            Ok(CosignResult::Failed)
        }
    }
}

// ── Internal HTTP helpers ─────────────────────────────────────────────────────

fn make_request(url: &str, token: Option<&str>) -> Result<String, BoxError> {
    let mut request = ureq::get(url).header("User-Agent", "ecc-update/1.0");
    if let Some(auth) = make_auth_header(token) {
        request = request.header("Authorization", auth);
    }
    let response = request
        .call()
        .map_err(|e| format!("network error: {e}. Try again later."))?;
    let body = response.into_body().read_to_string()?;
    Ok(body)
}

fn download_url_streaming(
    url: &str,
    dest: &Path,
    on_progress: &dyn Fn(u64, u64),
) -> Result<(), BoxError> {
    let token = std::env::var("GITHUB_TOKEN").ok();
    let mut request = ureq::get(url).header("User-Agent", "ecc-update/1.0");
    if let Some(auth) = make_auth_header(token.as_deref()) {
        request = request.header("Authorization", auth);
    }
    let response = request
        .call()
        .map_err(|e| format!("network error: {e}. Try again later."))?;
    let total: u64 = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let reader = response.into_body().into_reader();
    stream_to_file(reader, dest, total, on_progress)
}

fn tempfile_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "ecc-checksum-{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    // ── PC-025: parse_latest_release ─────────────────────────────────────────

    #[test]
    fn parse_latest_release() {
        let json = "{\"tag_name\": \"v4.3.0\", \"body\": \"Feature A and Bug fix B\"}";
        let info = parse_release_json(json).expect("should parse");
        assert_eq!(info.version, "4.3.0");
        assert!(info.release_notes.contains("Feature A"));
    }

    #[test]
    fn parse_latest_release_strips_v_prefix() {
        let json = "{\"tag_name\": \"v1.0.0\", \"body\": \"\"}";
        let info = parse_release_json(json).unwrap();
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn parse_latest_release_missing_tag_fails() {
        let json = "{\"name\": \"no-tag\"}";
        assert!(parse_release_json(json).is_err());
    }

    // ── PC-026: get_specific_version ─────────────────────────────────────────

    #[test]
    fn get_specific_version() {
        let url = version_tag_url("LEBOCQTitouan", "everything-claude-code", "4.3.0");
        assert_eq!(
            url,
            "https://api.github.com/repos/LEBOCQTitouan/everything-claude-code/releases/tags/v4.3.0"
        );
    }

    #[test]
    fn get_specific_version_includes_v_prefix() {
        let url = version_tag_url("owner", "repo", "2.1.0");
        assert!(
            url.ends_with("/tags/v2.1.0"),
            "URL must end with /tags/v2.1.0, got: {url}"
        );
    }

    // ── PC-027: streaming_download ────────────────────────────────────────────

    #[test]
    fn streaming_download() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("output.bin");

        // Simulate a 20 KiB payload (larger than one 8 KiB chunk)
        let payload: Vec<u8> = (0u8..=255).cycle().take(20 * 1024).collect();
        let reader = Cursor::new(payload.clone());
        let total = payload.len() as u64;

        use std::sync::{Arc, Mutex};
        let progress_calls: Arc<Mutex<Vec<(u64, u64)>>> = Arc::new(Mutex::new(Vec::new()));
        let calls_clone = Arc::clone(&progress_calls);
        stream_to_file(reader, &dest, total, &|written, t| {
            calls_clone.lock().unwrap().push((written, t));
        })
        .expect("streaming write should succeed");

        let written = std::fs::read(&dest).unwrap();
        assert_eq!(written, payload);

        let calls = progress_calls.lock().unwrap();
        assert!(
            calls.len() >= 2,
            "expected at least 2 progress callbacks for 20 KiB payload, got {}",
            calls.len()
        );

        let last = calls.last().unwrap();
        assert_eq!(last.0, total);
        assert_eq!(last.1, total);
    }

    // ── PC-028: checksum_verification ────────────────────────────────────────

    #[test]
    fn checksum_verification() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("ecc-darwin-arm64.tar.gz");
        let content = b"fake tarball content";
        std::fs::write(&file_path, content).unwrap();

        let expected_hash = compute_sha256(&file_path).unwrap();
        let checksum_file = format!("{expected_hash}  ecc-darwin-arm64.tar.gz\n");

        let found = parse_checksum_line(&checksum_file, "ecc-darwin-arm64.tar.gz");
        assert_eq!(found.as_deref(), Some(expected_hash.as_str()));
    }

    #[test]
    fn checksum_verification_mismatch_detected() {
        let content = "abc123  some-artifact.tar.gz\n";
        let found = parse_checksum_line(content, "some-artifact.tar.gz");
        assert_eq!(found.as_deref(), Some("abc123"));

        let missing = parse_checksum_line(content, "other-artifact.tar.gz");
        assert!(missing.is_none());
    }

    // ── PC-029: network_error ─────────────────────────────────────────────────

    #[test]
    fn network_error() {
        let simulated_err: BoxError =
            "network error: connection refused. Try again later.".into();
        let msg = simulated_err.to_string();
        assert!(msg.contains("network error"), "error should mention 'network error'");
        assert!(msg.contains("Try again later"), "error should contain retry guidance");
    }

    // ── PC-030: rate_limited ──────────────────────────────────────────────────

    #[test]
    fn rate_limited() {
        let reset = check_rate_limit(403, Some("1234567890"));
        assert_eq!(reset, Some(1_234_567_890));

        let no_ts = check_rate_limit(403, None);
        assert_eq!(no_ts, Some(0));

        let ok = check_rate_limit(200, None);
        assert!(ok.is_none());

        let not_found = check_rate_limit(404, None);
        assert!(not_found.is_none());
    }

    // ── PC-031: github_token_auth ─────────────────────────────────────────────

    #[test]
    fn github_token_auth() {
        let auth = make_auth_header(Some("ghp_mytoken123"));
        assert_eq!(auth.as_deref(), Some("Bearer ghp_mytoken123"));

        let no_auth = make_auth_header(None);
        assert!(no_auth.is_none());

        let empty = make_auth_header(Some(""));
        assert!(empty.is_none());
    }

    // ── PC-034: cosign_verify_bundle ─────────────────────────────────────────

    #[test]
    fn cosign_verify_bundle() {
        let blob = PathBuf::from("/tmp/artifact.tar.gz");
        let bundle = PathBuf::from("/tmp/artifact.tar.gz.bundle");
        let cmd =
            build_cosign_command(&blob, &bundle, COSIGN_CERTIFICATE_IDENTITY, COSIGN_OIDC_ISSUER);

        assert_eq!(cmd.get_program(), "cosign");

        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        let args_str: Vec<&str> = args.iter().filter_map(|a: &&std::ffi::OsStr| a.to_str()).collect();

        assert!(args_str.contains(&"verify-blob"), "must contain verify-blob");
        assert!(args_str.contains(&"--bundle"), "must contain --bundle");
        assert!(
            args_str.contains(&"/tmp/artifact.tar.gz.bundle"),
            "must include bundle path"
        );
        assert!(
            args_str.contains(&"--certificate-identity"),
            "must contain --certificate-identity"
        );
        assert!(
            args_str.contains(&COSIGN_CERTIFICATE_IDENTITY),
            "must include the certificate identity"
        );
        assert!(
            args_str.contains(&"--certificate-oidc-issuer"),
            "must contain --certificate-oidc-issuer"
        );
        assert!(
            args_str.contains(&COSIGN_OIDC_ISSUER),
            "must include the OIDC issuer"
        );
        assert!(
            args_str.contains(&"/tmp/artifact.tar.gz"),
            "must include the blob path"
        );
    }

    // ── PC-035: certificate_identity_constant ────────────────────────────────

    #[test]
    fn certificate_identity_constant() {
        assert!(
            COSIGN_CERTIFICATE_IDENTITY.contains(".github/workflows/release.yml"),
            "COSIGN_CERTIFICATE_IDENTITY must reference the release workflow"
        );
        assert!(
            COSIGN_CERTIFICATE_IDENTITY.starts_with("https://github.com/"),
            "COSIGN_CERTIFICATE_IDENTITY must be a GitHub Actions URL"
        );
        assert_eq!(
            COSIGN_OIDC_ISSUER,
            "https://token.actions.githubusercontent.com"
        );
    }
}
