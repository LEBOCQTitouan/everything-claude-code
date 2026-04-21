/// Returns `true` if `path` is under `~/.claude/projects/<hash>/memory/`.
///
/// These paths are always outside any git worktree — Claude's memory prune
/// writes there during active worktree sessions. The write-guard must never
/// block them; this function provides the allow-list predicate.
pub fn is_memory_root_path(path: &str) -> bool {
    is_claude_projects_memory(path.trim())
}

/// Core pattern check — separated for testability.
///
/// Matches: `/<anything>/.claude/projects/<hash>/memory[/<file>]`
fn is_claude_projects_memory(path: &str) -> bool {
    let marker = "/.claude/projects/";
    let Some(idx) = path.find(marker) else {
        return false;
    };
    let after_marker = &path[idx + marker.len()..];
    // Skip the hash component (first path segment after the marker).
    let Some(slash) = after_marker.find('/') else {
        return false;
    };
    let after_hash = &after_marker[slash + 1..];
    after_hash.starts_with("memory/") || after_hash == "memory"
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-120: write-guard allows memory root paths
    #[test]
    fn memory_root_allowed() {
        // Paths under ~/.claude/projects/<hash>/memory/ are OUTSIDE the worktree.
        // The write-guard must NOT block them even during an active worktree session.
        // This test asserts the allow-list predicate returns true for these paths,
        // so that callers can short-circuit before evaluating worktree membership.
        let memory_path = "/home/alice/.claude/projects/abc/memory/project_bl001_foo.md";
        assert!(
            is_memory_root_path(memory_path),
            "memory root path must be recognised by the allow-list"
        );

        // Variety of realistic paths
        let cases = [
            "/Users/dev/.claude/projects/some-hash-123/memory/MEMORY.md",
            "/root/.claude/projects/abc123def456/memory/project_bl099.md",
            "/home/ci/.claude/projects/x/memory/user_role.md",
            // bare memory dir (no trailing file)
            "/home/alice/.claude/projects/abc/memory",
        ];
        for case in cases {
            assert!(
                is_memory_root_path(case),
                "expected allow-list to match: {case}"
            );
        }
    }

    #[test]
    fn non_memory_paths_not_matched() {
        let non_cases = [
            "/home/alice/src/project/src/main.rs",
            "/repo/.claude/worktrees/session-123/file.rs",
            "/home/alice/.claude/projects/abc/specs/design.md",
            "/home/alice/.claude/projects/abc/memoryFAKE/file.md",
        ];
        for case in non_cases {
            assert!(
                !is_memory_root_path(case),
                "expected allow-list NOT to match: {case}"
            );
        }
    }
}
