mod checks;
mod hooks_diff;

pub use checks::*;
pub use hooks_diff::*;

use ecc_domain::config::audit::{compute_audit_score, AuditReport};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Read a JSON file.
/// Returns `Ok(None)` if the file does not exist,
/// `Ok(Some(value))` on success, `Err(message)` if the file exists but is corrupt.
pub(super) fn read_json_safe(
    fs: &dyn FileSystem,
    path: &Path,
) -> Result<Option<serde_json::Value>, String> {
    let content = match fs.read_to_string(path) {
        Ok(c) => c,
        Err(ecc_ports::fs::FsError::NotFound(_)) => return Ok(None),
        Err(e) => return Err(format!("Cannot read {}: {}", path.display(), e)),
    };
    match serde_json::from_str(&content) {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(format!("Corrupt JSON in {}: {}", path.display(), e)),
    }
}

/// Full ECC config audit comparing installed artifacts against source.
pub fn audit_ecc_config(
    fs: &dyn FileSystem,
    ecc_root: &Path,
    claude_dir: &Path,
) -> ecc_domain::config::audit::ConfigAudit {
    let agents = audit_artifact_dir(
        fs,
        &ecc_root.join("agents"),
        &claude_dir.join("agents"),
        ".md",
    );

    let commands = audit_artifact_dir(
        fs,
        &ecc_root.join("commands"),
        &claude_dir.join("commands"),
        ".md",
    );

    let hooks_json_path = ecc_root.join("hooks").join("hooks.json");
    let settings_json_path = claude_dir.join("settings.json");
    let hooks = diff_hooks(fs, &settings_json_path, &hooks_json_path);

    let has_differences = !agents.outdated.is_empty()
        || !agents.missing.is_empty()
        || !commands.outdated.is_empty()
        || !commands.missing.is_empty()
        || !hooks.stale.is_empty()
        || !hooks.missing.is_empty();

    ecc_domain::config::audit::ConfigAudit {
        agents,
        commands,
        hooks,
        has_differences,
    }
}

/// Run all audit checks and compute a score and grade.
pub fn run_all_checks(
    fs: &dyn FileSystem,
    claude_dir: &Path,
    project_dir: &Path,
    ecc_root: &Path,
) -> AuditReport {
    let settings_path = claude_dir.join("settings.json");
    let agents_dir = ecc_root.join("agents");
    let commands_dir = ecc_root.join("commands");

    let checks = vec![
        check_deny_rules(fs, &settings_path),
        check_gitignore(fs, project_dir),
        check_hook_duplicates(fs, &settings_path),
        check_global_claude_md(fs, claude_dir),
        check_agent_skills(fs, &agents_dir),
        check_command_descriptions(fs, &commands_dir),
        check_project_claude_md(fs, project_dir),
        check_statusline(fs, claude_dir),
    ];

    let (score, grade) = compute_audit_score(&checks);

    AuditReport {
        checks,
        score,
        grade,
    }
}
