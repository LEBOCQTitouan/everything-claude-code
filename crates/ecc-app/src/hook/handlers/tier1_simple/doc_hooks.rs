use crate::hook::{HookPorts, HookResult};
use std::path::Path;

use super::helpers::{extract_file_path, scan_exports};

/// doc-file-warning: warn about non-standard documentation files.
pub fn doc_file_warning(stdin: &str) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Only check .md and .txt files
    if !file_path.ends_with(".md") && !file_path.ends_with(".txt") {
        return HookResult::passthrough(stdin);
    }

    // Allow standard doc files
    let basename = Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let standard_files = [
        "README.md",
        "CLAUDE.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "CHANGELOG.md",
        "LICENSE.md",
        "SKILL.md",
    ];
    let basename_upper = basename.to_uppercase();
    if standard_files
        .iter()
        .any(|s| s.to_uppercase() == basename_upper)
    {
        return HookResult::passthrough(stdin);
    }

    // Allow paths in known directories
    let normalized = file_path.replace('\\', "/");
    if normalized.contains(".claude/plans/")
        || normalized.contains("/docs/")
        || normalized.starts_with("docs/")
        || normalized.contains("/skills/")
        || normalized.starts_with("skills/")
        || normalized.contains("/.history/")
    {
        return HookResult::passthrough(stdin);
    }

    let msg = format!(
        "[Hook] WARNING: Non-standard documentation file detected\n\
         [Hook] File: {}\n\
         [Hook] Consider consolidating into README.md or docs/ directory\n",
        file_path
    );
    HookResult::warn(stdin, &msg)
}

/// doc-coverage-reminder: remind about undocumented exports.
pub fn doc_coverage_reminder(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let source_exts = ["ts", "tsx", "js", "jsx", "py", "go", "rs", "java"];
    if !source_exts.contains(&ext.as_str()) {
        return HookResult::passthrough(stdin);
    }

    let skip_patterns = [
        "/node_modules/",
        "/dist/",
        "/build/",
        "/.",
        "/vendor/",
        "/__pycache__/",
    ];
    if skip_patterns.iter().any(|p| file_path.contains(p)) {
        return HookResult::passthrough(stdin);
    }

    let path = Path::new(&file_path);
    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let (total, undocumented) = scan_exports(&content, &ext);
    if total > 0 && undocumented > 0 {
        let basename = Path::new(&file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let msg = format!(
            "[DocCoverage] {}: {}/{} exported items lack doc comments. \
             Run /doc-generate --comments-only to add them.\n",
            basename, undocumented, total
        );
        return HookResult::warn(stdin, &msg);
    }

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
        }
    }

    // --- doc_file_warning ---

    #[test]
    fn doc_file_warning_warns_for_non_standard_md() {
        let stdin = r#"{"tool_input":{"file_path":"scratch.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.contains("Non-standard documentation"));
        assert!(result.stderr.contains("scratch.md"));
    }

    #[test]
    fn doc_file_warning_passthrough_for_readme() {
        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_claude_md() {
        let stdin = r#"{"tool_input":{"file_path":"CLAUDE.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_docs_dir() {
        let stdin = r#"{"tool_input":{"file_path":"docs/guide.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_non_doc_extension() {
        let stdin = r#"{"tool_input":{"file_path":"src/main.rs"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_when_file_path_absent() {
        let result = doc_file_warning("{}");
        assert!(result.stderr.is_empty());
    }

    // --- doc_coverage_reminder ---

    #[test]
    fn doc_coverage_reminder_warns_on_undocumented_exports() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/lib.rs", "pub fn alpha() {}\npub fn beta() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.contains("DocCoverage"));
        assert!(result.stderr.contains("lib.rs"));
    }

    #[test]
    fn doc_coverage_reminder_passthrough_for_non_source_extension() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_coverage_reminder_passthrough_when_file_missing() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/missing.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_coverage_reminder_passthrough_when_all_documented() {
        let fs =
            InMemoryFileSystem::new().with_file("src/lib.rs", "/// Documented\npub fn foo() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }
}
