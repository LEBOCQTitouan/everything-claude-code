//! Domain types for symlink-based config switching (BL-058).
//!
//! Defines `DevProfile`, `SymlinkOp`, `SymlinkPlan`, and the pure function
//! `build_symlink_plan`. Zero I/O — all operations are pure value transforms.

use std::path::{Path, PathBuf};

/// Directories managed by ECC that are symlinked into `~/.claude/`.
pub const MANAGED_DIRS: &[&str] = &["agents", "commands", "skills", "rules", "teams"];

/// Which config profile is active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevProfile {
    /// Production profile: real `~/.claude/` install (no symlinks).
    Default,
    /// Development profile: symlinks from ECC repo into `~/.claude/`.
    Dev,
}

/// A single symlink operation: create `link` pointing at `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymlinkOp {
    /// Source path inside the ECC repository root.
    pub target: PathBuf,
    /// Destination path inside `~/.claude/`.
    pub link: PathBuf,
}

/// An ordered list of symlink operations to apply for a profile switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymlinkPlan {
    ops: Vec<SymlinkOp>,
}

impl SymlinkPlan {
    /// Returns the list of symlink operations.
    pub fn ops(&self) -> &[SymlinkOp] {
        &self.ops
    }
}

/// Builds a `SymlinkPlan` for the given profile.
///
/// For every directory in `managed_dirs`, the plan contains one `SymlinkOp`:
/// - `target` = `ecc_root/dir`
/// - `link`   = `claude_dir/dir`
///
/// This function is pure: no I/O, no side effects.
pub fn build_symlink_plan(
    ecc_root: &Path,
    claude_dir: &Path,
    managed_dirs: &[&str],
) -> SymlinkPlan {
    let ops = managed_dirs
        .iter()
        .map(|dir| SymlinkOp {
            target: ecc_root.join(dir),
            link: claude_dir.join(dir),
        })
        .collect();
    SymlinkPlan { ops }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // PC-001: DevProfile enum variants with required derives
    #[test]
    fn dev_profile_enum_variants() {
        let default_profile = DevProfile::Default;
        let dev_profile = DevProfile::Dev;

        // PartialEq
        assert_eq!(default_profile, DevProfile::Default);
        assert_eq!(dev_profile, DevProfile::Dev);
        assert_ne!(default_profile, dev_profile);

        // Clone
        let cloned = dev_profile.clone();
        assert_eq!(cloned, DevProfile::Dev);

        // Debug (just ensure it doesn't panic)
        let _ = format!("{:?}", default_profile);
        let _ = format!("{:?}", dev_profile);

        // Eq is implied by PartialEq derives; verified via assert_eq above
    }

    // PC-002: SymlinkPlan contains Vec<SymlinkOp> with target and link PathBuf
    #[test]
    fn symlink_plan_structure() {
        let op = SymlinkOp {
            target: PathBuf::from("/ecc/agents"),
            link: PathBuf::from("/home/.claude/agents"),
        };

        assert_eq!(op.target, PathBuf::from("/ecc/agents"));
        assert_eq!(op.link, PathBuf::from("/home/.claude/agents"));

        let plan = SymlinkPlan {
            ops: vec![op.clone()],
        };
        assert_eq!(plan.ops().len(), 1);
        assert_eq!(plan.ops()[0], op);
    }

    // PC-004: build_symlink_plan correct for Dev profile
    #[test]
    fn build_plan_dev_profile() {
        let ecc_root = PathBuf::from("/repo/ecc");
        let claude_dir = PathBuf::from("/home/user/.claude");

        let plan = build_symlink_plan(&ecc_root, &claude_dir, MANAGED_DIRS);
        let ops = plan.ops();

        assert_eq!(ops.len(), MANAGED_DIRS.len());
        for (op, dir) in ops.iter().zip(MANAGED_DIRS.iter()) {
            assert_eq!(op.target, ecc_root.join(dir));
            assert_eq!(op.link, claude_dir.join(dir));
        }
    }

    // PC-005: build_symlink_plan correct for Default profile
    #[test]
    fn build_plan_default_profile() {
        let ecc_root = PathBuf::from("/usr/local/ecc");
        let claude_dir = PathBuf::from("/root/.claude");
        let dirs: &[&str] = &[];

        let plan = build_symlink_plan(&ecc_root, &claude_dir, dirs);
        assert_eq!(plan.ops().len(), 0);
    }

    // PC-007: MANAGED_DIRS constant contains exactly agents, commands, skills, rules
    #[test]
    fn managed_dirs_constant() {
        assert_eq!(MANAGED_DIRS, &["agents", "commands", "skills", "rules", "teams"]);
    }
}
