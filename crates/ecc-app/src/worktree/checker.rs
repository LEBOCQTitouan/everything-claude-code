//! `LivenessChecker` — single source of truth for worktree liveness decisions.
//!
//! Wraps the pure domain predicate [`ecc_domain::worktree::liveness::is_live`]
//! with I/O via the [`FileSystem`] and [`ShellExecutor`] ports.
//!
//! ## SOLID-002 remediation
//!
//! Both `gc` and `status` share this struct so liveness logic lives in exactly
//! one place (PC-043, PC-078).

use ecc_domain::worktree::liveness::{LivenessRecord, is_live};
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::path::Path;

/// Outcome of a single liveness check for a worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivenessVerdict {
    /// Fresh heartbeat AND live PID.
    Live,
    /// Heartbeat exists but `last_seen_unix_ts` is older than `ttl_secs`.
    Stale,
    /// Heartbeat exists + PID reaped (`kill -0` returned non-zero).
    Dead,
    /// No `.ecc-session` file found.
    MissingFile,
    /// `.ecc-session` exists but could not be parsed.
    Malformed,
}

/// Single-responsibility struct for worktree liveness decisions.
///
/// Both `gc` and `status` construct one of these with the same TTL to ensure
/// a consistent verdict. PC-078 enforces that `LivenessVerdict` is defined here
/// and nowhere else.
pub struct LivenessChecker<'a> {
    /// Filesystem port — reads `.ecc-session`.
    pub fs: &'a dyn FileSystem,
    /// Shell executor port — runs `kill -0 <pid>`.
    pub shell: &'a dyn ShellExecutor,
    /// Clock source — returns current Unix timestamp in seconds.
    pub now_fn: Box<dyn Fn() -> u64 + Send + Sync + 'a>,
    /// Heartbeat TTL in seconds. Heartbeats older than this → `Stale`.
    pub ttl_secs: u64,
}

impl<'a> LivenessChecker<'a> {
    /// Determine the liveness of the session at `worktree_path`.
    ///
    /// Algorithm:
    /// 1. Read `<worktree_path>/.ecc-session` → `MissingFile` on error.
    /// 2. Parse JSON → `Malformed` on error.
    /// 3. Run `kill -0 <pid>` to check if the owning process is alive.
    /// 4. Delegate to [`ecc_domain::worktree::liveness::is_live`]:
    ///    - If live → `Live`
    ///    - Else if PID dead → `Dead`
    ///    - Else (heartbeat stale) → `Stale`
    pub fn check(&self, worktree_path: &Path) -> LivenessVerdict {
        let session_path = worktree_path.join(".ecc-session");
        let content = match self.fs.read_to_string(&session_path) {
            Ok(c) => c,
            Err(_) => return LivenessVerdict::MissingFile,
        };
        let record = match LivenessRecord::parse(&content) {
            Ok(r) => r,
            Err(_) => return LivenessVerdict::Malformed,
        };
        let now = (self.now_fn)();
        let pid_str = record.claude_code_pid.to_string();
        let pid_alive = self
            .shell
            .run_command("kill", &["-0", &pid_str])
            .map(|o| o.success())
            .unwrap_or(false);
        if is_live(&record, now, pid_alive, self.ttl_secs) {
            LivenessVerdict::Live
        } else if !pid_alive {
            LivenessVerdict::Dead
        } else {
            LivenessVerdict::Stale
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{InMemoryFileSystem, MockExecutor};

    const NOW: u64 = 1_735_689_600;
    const TTL: u64 = 3600;
    const PID: u32 = 12345;

    fn ok(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn err_out(code: i32) -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "no such process".to_owned(),
            exit_code: code,
        }
    }

    fn fresh_json() -> String {
        let ts = NOW - 60;
        format!(r#"{{"schema_version":1,"claude_code_pid":{PID},"last_seen_unix_ts":{ts}}}"#)
    }

    fn stale_json() -> String {
        let ts = NOW - (TTL + 1);
        format!(r#"{{"schema_version":1,"claude_code_pid":{PID},"last_seen_unix_ts":{ts}}}"#)
    }

    fn make_checker<'a>(
        fs: &'a dyn FileSystem,
        shell: &'a dyn ShellExecutor,
        now: u64,
        ttl: u64,
    ) -> LivenessChecker<'a> {
        LivenessChecker {
            fs,
            shell,
            now_fn: Box::new(move || now),
            ttl_secs: ttl,
        }
    }

    /// PC-043: `LivenessChecker::check` is the single source of truth used by
    /// both GC and status. This test verifies the struct compiles and its verdict
    /// is consistent across two independent instantiations with identical inputs.
    #[test]
    fn gc_and_status_use_same_checker() {
        let fs = InMemoryFileSystem::new().with_file("/wt/.ecc-session", &fresh_json());
        let shell = MockExecutor::new().on_args("kill", &["-0", &PID.to_string()], ok(""));

        let checker_gc = make_checker(&fs, &shell, NOW, TTL);
        let checker_status = make_checker(&fs, &shell, NOW, TTL);

        let verdict_gc = checker_gc.check(std::path::Path::new("/wt"));
        let verdict_status = checker_status.check(std::path::Path::new("/wt"));

        assert_eq!(
            verdict_gc, verdict_status,
            "gc and status must get identical verdict from same inputs"
        );
        assert_eq!(verdict_gc, LivenessVerdict::Live);
    }

    /// PC-074: `LivenessChecker` is `Send + Sync`; concurrent `check()` across
    /// threads returns consistent verdicts.
    #[test]
    fn checker_send_sync_concurrent_safe() {
        // Compile-time proof: LivenessChecker is Send + Sync.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LivenessChecker>();

        // Runtime proof: two threads calling check() on Arc<checker-equivalent
        // data> get the same verdict.
        use std::sync::Arc;

        let fs = Arc::new(InMemoryFileSystem::new().with_file("/wt/.ecc-session", &fresh_json()));
        let shell =
            Arc::new(MockExecutor::new().on_args("kill", &["-0", &PID.to_string()], ok("")));

        // We clone the Arc refs to move into threads; each thread builds its own
        // LivenessChecker (which borrows from the Arc data).
        let fs1 = Arc::clone(&fs);
        let shell1 = Arc::clone(&shell);
        let fs2 = Arc::clone(&fs);
        let shell2 = Arc::clone(&shell);

        let h1 = std::thread::spawn(move || {
            let checker = LivenessChecker {
                fs: fs1.as_ref(),
                shell: shell1.as_ref(),
                now_fn: Box::new(|| NOW),
                ttl_secs: TTL,
            };
            checker.check(std::path::Path::new("/wt"))
        });
        let h2 = std::thread::spawn(move || {
            let checker = LivenessChecker {
                fs: fs2.as_ref(),
                shell: shell2.as_ref(),
                now_fn: Box::new(|| NOW),
                ttl_secs: TTL,
            };
            checker.check(std::path::Path::new("/wt"))
        });

        let v1 = h1.join().expect("thread 1 panicked");
        let v2 = h2.join().expect("thread 2 panicked");
        assert_eq!(v1, v2, "concurrent check() must return identical verdicts");
        assert_eq!(v1, LivenessVerdict::Live);
    }

    #[test]
    fn missing_file_returns_missing() {
        let fs = InMemoryFileSystem::new(); // no .ecc-session
        let shell = MockExecutor::new();
        let checker = make_checker(&fs, &shell, NOW, TTL);
        assert_eq!(
            checker.check(Path::new("/wt")),
            LivenessVerdict::MissingFile
        );
    }

    #[test]
    fn malformed_json_returns_malformed() {
        let fs = InMemoryFileSystem::new().with_file("/wt/.ecc-session", "not json {{{");
        let shell = MockExecutor::new();
        let checker = make_checker(&fs, &shell, NOW, TTL);
        assert_eq!(checker.check(Path::new("/wt")), LivenessVerdict::Malformed);
    }

    #[test]
    fn dead_pid_returns_dead() {
        let fs = InMemoryFileSystem::new().with_file("/wt/.ecc-session", &fresh_json());
        let shell = MockExecutor::new().on_args("kill", &["-0", &PID.to_string()], err_out(1));
        let checker = make_checker(&fs, &shell, NOW, TTL);
        assert_eq!(checker.check(Path::new("/wt")), LivenessVerdict::Dead);
    }

    #[test]
    fn stale_heartbeat_pid_alive_returns_stale() {
        let fs = InMemoryFileSystem::new().with_file("/wt/.ecc-session", &stale_json());
        let shell = MockExecutor::new().on_args("kill", &["-0", &PID.to_string()], ok(""));
        let checker = make_checker(&fs, &shell, NOW, TTL);
        assert_eq!(checker.check(Path::new("/wt")), LivenessVerdict::Stale);
    }
}
