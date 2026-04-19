//! Noise-path predicate for the cartography bounded context.
//!
//! Classifies repository paths as noise (workflow metadata, docs) or signal
//! (source code changes). Zero I/O — pure function.

/// Returns `true` when `path` refers to a noise location that should be
/// excluded from cartography delta files.
///
/// Matching rules:
/// - Normalize path separators `\` → `/`.
/// - Apply ASCII lowercase before comparison.
/// - Noise if the lowercased path starts with any prefix in [`NOISE_PREFIXES`].
pub fn is_noise_path(_path: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

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
