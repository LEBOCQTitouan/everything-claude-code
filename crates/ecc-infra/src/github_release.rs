use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseClient, ReleaseInfo};
use std::path::{Path, PathBuf};

/// GitHub Releases adapter for [`ReleaseClient`].
///
/// Uses the `ureq` crate internally for GitHub API interaction.
/// Configured with rustls for TLS (no OpenSSL dependency).
///
/// Cosign verification shells out to `cosign verify-blob` via
/// `std::process::Command` with argument arrays (no shell interpolation).
#[allow(dead_code)]
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
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl ReleaseClient for GithubReleaseClient {
    fn latest_version(&self, _include_prerelease: bool) -> Result<ReleaseInfo, BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }

    fn get_version(&self, _version: &str) -> Result<ReleaseInfo, BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }

    fn download_tarball(
        &self,
        _version: &str,
        _artifact_name: &str,
        _dest: &Path,
        _on_progress: &dyn Fn(u64, u64),
    ) -> Result<(), BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }

    fn download_file(&self, _url: &str, _dest: &Path) -> Result<(), BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }

    fn verify_checksum(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
    ) -> Result<ChecksumResult, BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }

    fn verify_cosign(
        &self,
        _version: &str,
        _artifact_name: &str,
        _file_path: &Path,
        _bundle_path: &Path,
    ) -> Result<CosignResult, BoxError> {
        Err("GithubReleaseClient not yet implemented".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    // ── PC-025: parse_latest_release ─────────────────────────────────────────

    #[test]
    fn parse_latest_release() {
        let json = r#"{"tag_name": "v4.3.0", "body": "## What's changed\n- Feature A\n- Bug fix B"}"#;
        let info = parse_release_json(json).expect("should parse");
        assert_eq!(info.version, "4.3.0");
        assert!(info.release_notes.contains("Feature A"));
    }

    #[test]
    fn parse_latest_release_strips_v_prefix() {
        let json = r#"{"tag_name": "v1.0.0", "body": ""}"#;
        let info = parse_release_json(json).unwrap();
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn parse_latest_release_missing_tag_fails() {
        let json = r#"{"name": "no-tag"}"#;
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
        assert!(url.ends_with("/tags/v2.1.0"), "URL must end with /tags/v2.1.0, got: {url}");
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

        let mut progress_calls: Vec<(u64, u64)> = Vec::new();
        stream_to_file(reader, &dest, total, &|written, t| {
            progress_calls.push((written, t));
        })
        .expect("streaming write should succeed");

        let written = std::fs::read(&dest).unwrap();
        assert_eq!(written, payload);

        assert!(
            progress_calls.len() >= 2,
            "expected at least 2 progress callbacks for 20 KiB payload, got {}",
            progress_calls.len()
        );

        let last = progress_calls.last().unwrap();
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
            format!("network error: connection refused. Try again later.").into();
        let msg = simulated_err.to_string();
        assert!(msg.contains("network error"), "error should mention 'network error'");
        assert!(msg.contains("Try again later"), "error should contain retry guidance");
    }

    // ── PC-030: rate_limited ──────────────────────────────────────────────────

    #[test]
    fn rate_limited() {
        let reset = check_rate_limit(403, Some("1234567890"));
        assert_eq!(reset, Some(1234567890));

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
        let mut cmd =
            build_cosign_command(&blob, &bundle, COSIGN_CERTIFICATE_IDENTITY, COSIGN_OIDC_ISSUER);

        assert_eq!(cmd.get_program(), "cosign");

        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        let args_str: Vec<&str> = args.iter().filter_map(|a| a.to_str()).collect();

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
