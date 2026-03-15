use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Self-contained test environment with isolated HOME and project directories.
///
/// Each test gets its own pair of TempDirs, enabling safe parallel execution.
/// Dropped automatically on test completion (even on panic).
#[allow(dead_code)]
pub struct EccTestEnv {
    /// Simulated $HOME directory (contains .claude/)
    pub home: TempDir,
    /// Simulated project directory
    pub project: TempDir,
}

#[allow(dead_code)]
impl EccTestEnv {
    /// Create a fresh, empty test environment.
    pub fn new() -> Self {
        Self {
            home: TempDir::new().expect("failed to create home TempDir"),
            project: TempDir::new().expect("failed to create project TempDir"),
        }
    }

    /// Build a `Command` for the `ecc` binary with isolated env vars.
    ///
    /// Sets HOME, ECC_ROOT (pointing to real workspace assets), and NO_COLOR.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("ecc").expect("ecc binary not found");
        cmd.env("HOME", self.home.path());
        cmd.env("ECC_ROOT", Self::ecc_root());
        cmd.env("NO_COLOR", "1");
        // Prevent tests from picking up the real user's config
        cmd.env("XDG_CONFIG_HOME", self.home.path().join(".config"));
        cmd
    }

    /// Path to ~/.claude/ inside the test HOME.
    pub fn claude_dir(&self) -> PathBuf {
        self.home.path().join(".claude")
    }

    /// Read and parse ~/.claude/settings.json.
    pub fn settings_json(&self) -> serde_json::Value {
        let path = self.claude_dir().join("settings.json");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
    }

    /// Check whether a file exists relative to ~/.claude/.
    pub fn file_exists(&self, rel: &str) -> bool {
        self.claude_dir().join(rel).exists()
    }

    /// Read a file relative to ~/.claude/.
    pub fn read_file(&self, rel: &str) -> String {
        let path = self.claude_dir().join(rel);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
    }

    /// Seed a file relative to ~/.claude/ (creates parent directories).
    pub fn write_file(&self, rel: &str, content: &str) {
        let path = self.claude_dir().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("failed to mkdir {}: {e}", parent.display()));
        }
        std::fs::write(&path, content)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    }

    /// Run `ecc install --no-interactive --force` with optional extra args.
    pub fn install(&self, extra_args: &[&str]) -> assert_cmd::assert::Assert {
        let mut cmd = self.cmd();
        cmd.args(["install", "--no-interactive", "--force"]);
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.assert()
    }

    /// Path to the real workspace root (where assets like agents/, commands/ live).
    pub fn ecc_root() -> PathBuf {
        workspace_root()
    }
}

/// Locate the workspace root by finding the directory containing the top-level Cargo.toml.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // crates/ecc-integration-tests -> go up 2 levels to workspace root
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to find workspace root")
        .to_path_buf()
}
