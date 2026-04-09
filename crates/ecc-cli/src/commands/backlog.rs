//! CLI for deterministic backlog management.

use clap::{Args, Subcommand};
use ecc_infra::fs_backlog::FsBacklogRepository;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::os_worktree::OsWorktreeManager;
use ecc_infra::system_clock::SystemClock;
use std::path::PathBuf;

#[derive(Args)]
pub struct BacklogArgs {
    #[command(subcommand)]
    pub action: BacklogAction,

    /// Path to the backlog directory
    #[arg(long, default_value = "docs/backlog")]
    pub dir: PathBuf,
}

#[derive(Subcommand)]
pub enum BacklogAction {
    /// Print the next sequential BL-NNN ID
    NextId,

    /// Check for duplicate backlog entries by title similarity
    CheckDuplicates {
        /// Title to check for duplicates
        query: String,

        /// Comma-separated tags to boost matching score
        #[arg(long)]
        tags: Option<String>,
    },

    /// Regenerate BACKLOG.md index from BL-*.md files
    Reindex {
        /// Print generated content without writing to file
        #[arg(long)]
        dry_run: bool,
    },

    /// List open backlog entries, optionally filtering out in-progress items
    List {
        /// Only show entries not claimed by active worktrees or locks
        #[arg(long)]
        available: bool,
        /// Show all open items regardless of claims
        #[arg(long)]
        show_all: bool,
    },
}

pub fn run(args: BacklogArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let repo = FsBacklogRepository::new(&fs);
    let worktree_mgr = OsWorktreeManager;
    let clock = SystemClock;
    let dir = &args.dir;

    // Determine project root (parent of backlog dir, fallback to cwd)
    let project_dir = dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match args.action {
        BacklogAction::NextId => {
            let id = ecc_app::backlog::next_id(&repo, dir).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{id}");
        }
        BacklogAction::CheckDuplicates { query, tags } => {
            let tag_list: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let candidates = ecc_app::backlog::check_duplicates(&repo, dir, &query, &tag_list)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let json = serde_json::to_string_pretty(&candidates)?;
            println!("{json}");
        }
        BacklogAction::Reindex { dry_run } => {
            let output = ecc_app::backlog::reindex(
                &repo,
                &repo,
                &repo,
                &worktree_mgr,
                &clock,
                dir,
                &project_dir,
                dry_run,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let Some(content) = output {
                print!("{content}");
            }
        }
        BacklogAction::List {
            available,
            show_all,
        } => {
            let entries = if available || show_all {
                ecc_app::backlog::list_available(
                    &repo,
                    &repo,
                    &worktree_mgr,
                    &clock,
                    dir,
                    &project_dir,
                    show_all,
                )
                .map_err(|e| anyhow::anyhow!("{e}"))?
            } else {
                ecc_app::backlog::list_available(
                    &repo,
                    &repo,
                    &worktree_mgr,
                    &clock,
                    dir,
                    &project_dir,
                    false,
                )
                .map_err(|e| anyhow::anyhow!("{e}"))?
            };
            let json = serde_json::to_string_pretty(&entries)?;
            println!("{json}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ecc_domain::backlog::entry::BacklogError;
    use ecc_domain::backlog::entry::{BacklogEntry, BacklogStatus};
    use ecc_domain::backlog::lock::LockFile;
    use ecc_ports::backlog::{BacklogEntryStore, BacklogLockStore};
    use ecc_ports::clock::Clock;
    use ecc_ports::worktree::{WorktreeError, WorktreeInfo, WorktreeManager};
    use std::path::{Path, PathBuf};

    // --- Minimal in-memory stubs ---

    struct StubEntries(Vec<BacklogEntry>);

    impl BacklogEntryStore for StubEntries {
        fn load_entries(&self, _dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError> {
            Ok(self.0.clone())
        }
        fn load_entry(&self, _dir: &Path, _id: &str) -> Result<BacklogEntry, BacklogError> {
            Err(BacklogError::Io {
                path: "stub".into(),
                message: "not found".into(),
            })
        }
        fn save_entry(
            &self,
            _dir: &Path,
            _entry: &BacklogEntry,
            _body: &str,
        ) -> Result<(), BacklogError> {
            Ok(())
        }
        fn next_id(&self, _dir: &Path) -> Result<String, BacklogError> {
            Ok("BL-001".into())
        }
    }

    struct StubLocks;

    impl BacklogLockStore for StubLocks {
        fn load_lock(&self, _dir: &Path, _id: &str) -> Result<Option<LockFile>, BacklogError> {
            Ok(None)
        }
        fn save_lock(&self, _dir: &Path, _id: &str, _lock: &LockFile) -> Result<(), BacklogError> {
            Ok(())
        }
        fn remove_lock(&self, _dir: &Path, _id: &str) -> Result<(), BacklogError> {
            Ok(())
        }
        fn list_locks(&self, _dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError> {
            Ok(vec![])
        }
    }

    struct StubWorktree;

    impl WorktreeManager for StubWorktree {
        fn has_uncommitted_changes(&self, _path: &Path) -> Result<bool, WorktreeError> {
            Ok(false)
        }
        fn has_untracked_files(&self, _path: &Path) -> Result<bool, WorktreeError> {
            Ok(false)
        }
        fn unmerged_commit_count(&self, _path: &Path, _branch: &str) -> Result<u64, WorktreeError> {
            Ok(0)
        }
        fn has_stash(&self, _path: &Path) -> Result<bool, WorktreeError> {
            Ok(false)
        }
        fn is_pushed_to_remote(&self, _path: &Path, _branch: &str) -> Result<bool, WorktreeError> {
            Ok(true)
        }
        fn remove_worktree(&self, _root: &Path, _path: &Path) -> Result<(), WorktreeError> {
            Ok(())
        }
        fn delete_branch(&self, _root: &Path, _branch: &str) -> Result<(), WorktreeError> {
            Ok(())
        }
        fn list_worktrees(&self, _root: &Path) -> Result<Vec<WorktreeInfo>, WorktreeError> {
            Ok(vec![])
        }
    }

    struct StubClock;

    impl Clock for StubClock {
        fn now_iso8601(&self) -> String {
            "2026-01-01T00:00:00Z".into()
        }
        fn now_epoch_secs(&self) -> u64 {
            0
        }
    }

    fn make_entry(id: &str, status: BacklogStatus) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {id}"),
            status,
            created: "2026-01-01".to_string(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags: vec![],
        }
    }

    #[test]
    fn list_available_json_output() {
        let entries = StubEntries(vec![
            make_entry("BL-001", BacklogStatus::Open),
            make_entry("BL-002", BacklogStatus::Open),
        ]);
        let locks = StubLocks;
        let wm = StubWorktree;
        let clock = StubClock;
        let dir = PathBuf::from("docs/backlog");
        let project_dir = PathBuf::from(".");

        let result = ecc_app::backlog::list_available(
            &entries,
            &locks,
            &wm,
            &clock,
            &dir,
            &project_dir,
            false,
        )
        .expect("list_available should succeed");

        let json = serde_json::to_string_pretty(&result).expect("serialize should succeed");
        let parsed: Vec<serde_json::Value> =
            serde_json::from_str(&json).expect("json should be a valid array");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], "BL-001");
        assert_eq!(parsed[1]["id"], "BL-002");
    }

    #[test]
    fn list_show_all() {
        let entries = StubEntries(vec![
            make_entry("BL-001", BacklogStatus::Open),
            make_entry("BL-002", BacklogStatus::Open),
            make_entry("BL-003", BacklogStatus::Open),
        ]);
        let locks = StubLocks;
        let wm = StubWorktree;
        let clock = StubClock;
        let dir = PathBuf::from("docs/backlog");
        let project_dir = PathBuf::from(".");

        let result = ecc_app::backlog::list_available(
            &entries,
            &locks,
            &wm,
            &clock,
            &dir,
            &project_dir,
            true, // show_all = true
        )
        .expect("list_available should succeed with show_all");

        assert_eq!(result.len(), 3);
    }
}
