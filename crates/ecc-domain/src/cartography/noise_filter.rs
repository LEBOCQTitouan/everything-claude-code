//! Noise-path predicate for the cartography bounded context.
//!
//! Classifies repository paths as noise (workflow metadata, docs) or signal
//! (source code changes). Zero I/O — pure function.

/// Fixed noise exact matches — ordered alphabetically for review clarity.
const NOISE_EXACT: &[&str] = &[".claude/workflow", "cargo.lock"];

/// Fixed noise prefixes — ordered alphabetically for review clarity.
const NOISE_PREFIXES: &[&str] = &[
    ".claude/cartography/",
    ".claude/workflow/",
    ".claude/worktrees/",
    "docs/backlog/",
    "docs/cartography/",
    "docs/specs/",
];

/// Returns `true` when `path` refers to a noise location that should be
/// excluded from cartography delta files.
///
/// Matching rules:
/// - Normalize path separators `\` → `/`.
/// - Apply ASCII lowercase before comparison.
/// - Noise if the lowercased path starts with any prefix in [`NOISE_PREFIXES`].
pub fn is_noise_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    NOISE_EXACT.iter().any(|exact| normalized == *exact)
        || NOISE_PREFIXES
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_exact_matches_as_noise() {
        let exact_cases = ["Cargo.lock", ".claude/workflow"];
        for case in exact_cases {
            assert!(is_noise_path(case), "expected exact-match noise: {case}");
        }
    }

    #[test]
    fn crate_paths_are_signal() {
        let signal_cases = [
            "crates/ecc-domain/src/foo.rs",
            "crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs",
            "README.md",
            "docs/ARCHITECTURE.md",
            "docs/commands-reference.md",
            "docs/adr/0068-foo.md",
            ".github/workflows/ci.yml",
            "rust-toolchain.toml",
            "src/main.rs",
        ];
        for case in signal_cases {
            assert!(!is_noise_path(case), "expected signal: {case}");
        }
    }

    #[test]
    fn normalizes_case_and_separators() {
        // Case-insensitive
        assert!(is_noise_path(".CLAUDE/WORKFLOW/state.json"));
        assert!(is_noise_path(".Claude/Workflow/State.json"));
        // Separator normalization (Windows-style paths)
        assert!(is_noise_path(".claude\\workflow\\state.json"));
        assert!(is_noise_path("docs\\specs\\foo.md"));
        // Mixed separators + case
        assert!(is_noise_path(".CLAUDE\\cartography/pending.json"));
    }

    #[test]
    fn symlink_policy_by_path_only() {
        // Contract: is_noise_path classifies by the input string only.
        // Symlinks are NOT resolved — a symlink at `crates/foo/link.rs`
        // pointing to `.claude/workflow/state.json` is classified by its
        // symlink path (`crates/foo/link.rs` = signal), not its target.
        //
        // This is a pure function over &str; no FS access is possible.
        // The test is documentary and asserts the observable behavior.

        // A "symlink-like" path (just a regular string) is classified by the string itself.
        let symlink_path = "crates/ecc-domain/src/symlink_to_workflow.rs";
        assert!(
            !is_noise_path(symlink_path),
            "symlink path classified by its own path, not target"
        );

        // The target being in the noise set does not affect classification of the link path.
        let target = ".claude/workflow/state.json";
        assert!(
            is_noise_path(target),
            "target path classified directly (unrelated)"
        );
    }

    #[test]
    fn module_has_no_io_imports() {
        // Grep this module's own source for forbidden I/O imports.
        // Uses include_str! which is a compile-time read — not runtime I/O.
        //
        // Patterns are built at runtime to avoid the self-referential false-positive
        // where the test source itself contains the forbidden string literal.
        const SOURCE: &str = include_str!("noise_filter.rs");

        // Each tuple is ("prefix", "suffix") — joined at runtime so the combined
        // string never appears verbatim in this source file.
        let forbidden_parts: &[(&str, &str)] = &[
            ("use std::", "fs"),
            ("use std::", "net"),
            ("use std::", "process"),
            ("use std::io::", "Read"),
            ("use std::io::", "Write"),
            ("use ", "tokio"),
            ("use ", "async_std"),
            ("use ", "reqwest"),
        ];

        for (prefix, suffix) in forbidden_parts {
            let pat = format!("{prefix}{suffix}");
            assert!(
                !SOURCE.contains(&pat),
                "domain purity violation: noise_filter.rs contains `{pat}`"
            );
        }
    }

    #[test]
    fn bare_workflow_exact_noise() {
        // AC-001.8 edge case: `.claude/workflow` bare (no trailing slash)
        // must classify as noise via NOISE_EXACT, not be mis-classified as
        // signal because it doesn't match the `.claude/workflow/` prefix.
        assert!(is_noise_path(".claude/workflow"), "bare path must be noise");
        // Sanity: with trailing slash it's still noise (via prefix)
        assert!(
            is_noise_path(".claude/workflow/"),
            "trailing-slash variant still noise"
        );
        // And a subpath is noise via prefix
        assert!(
            is_noise_path(".claude/workflow/state.json"),
            "subpath noise"
        );
    }

    #[test]
    fn classifies_fixed_prefixes_as_noise() {
        let noise_cases = [
            ".claude/workflow/state.json",
            ".claude/workflow/implement-done.md",
            ".claude/cartography/pending-delta-session-123.json",
            ".claude/worktrees/ecc-session-foo/bar.rs",
            "docs/specs/2026-04-19-my-feature/spec.md",
            "docs/backlog/BL-001-foo.md",
            "docs/cartography/journeys/foo.md",
        ];
        for case in noise_cases {
            assert!(is_noise_path(case), "expected noise: {case}");
        }
    }
}
