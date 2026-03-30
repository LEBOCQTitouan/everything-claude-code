//! Project initialization — gitignore + untrack flow.

use super::InstallContext;
use crate::config::gitignore as app_gitignore;
use ecc_domain::ansi;
use ecc_domain::config::gitignore;
use std::path::Path;

/// Initialize ECC in a project directory (gitignore + untrack).
pub fn init_project(
    ctx: &InstallContext,
    project_dir: &Path,
    no_gitignore: bool,
    dry_run: bool,
) -> bool {
    let colored = ctx.env.var("NO_COLOR").is_none();
    let prefix = if dry_run { "[DRY RUN] " } else { "" };

    ctx.terminal.stdout_write(&format!(
        "\n{}{}\n\n",
        prefix,
        ansi::bold("ECC Init", colored)
    ));

    if no_gitignore {
        ctx.terminal
            .stdout_write("Skipping .gitignore (--no-gitignore)\n");
        return true;
    }

    if dry_run {
        init_project_dry_run(ctx, project_dir);
    } else {
        init_project_apply(ctx, project_dir);
    }

    true
}

fn init_project_dry_run(ctx: &InstallContext, project_dir: &Path) {
    let existing_content = ctx
        .fs
        .read_to_string(&project_dir.join(".gitignore"))
        .unwrap_or_default();
    let existing_patterns = gitignore::parse_gitignore_patterns(&existing_content);

    let missing: Vec<&str> = gitignore::ECC_GITIGNORE_ENTRIES
        .iter()
        .filter(|e| !existing_patterns.contains(e.pattern))
        .map(|e| e.pattern)
        .collect();

    if missing.is_empty() {
        ctx.terminal
            .stdout_write("All gitignore entries already present.\n");
    } else {
        ctx.terminal
            .stdout_write(&format!("Would add {} gitignore entries:\n", missing.len()));
        for pattern in &missing {
            ctx.terminal.stdout_write(&format!("  {pattern}\n"));
        }
    }

    let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
    if !tracked.is_empty() {
        ctx.terminal
            .stdout_write(&format!("Would untrack {} file(s):\n", tracked.len()));
        for file in &tracked {
            ctx.terminal.stdout_write(&format!("  {file}\n"));
        }
    }
}

fn init_project_apply(ctx: &InstallContext, project_dir: &Path) {
    let result = app_gitignore::ensure_gitignore_entries(ctx.fs, ctx.shell, project_dir, None);

    if result.skipped {
        ctx.terminal
            .stdout_write("Not a git repository — skipping .gitignore.\n");
        return;
    }

    if result.added.is_empty() {
        ctx.terminal
            .stdout_write("All gitignore entries already present.\n");
    } else {
        ctx.terminal.stdout_write(&format!(
            "Added {} gitignore entries:\n",
            result.added.len()
        ));
        for pattern in &result.added {
            ctx.terminal.stdout_write(&format!("  {pattern}\n"));
        }
    }

    let tracked = app_gitignore::find_tracked_ecc_files(ctx.shell, ctx.fs, project_dir);
    for file in &tracked {
        if let Err(err) =
            ctx.shell
                .run_command_in_dir("git", &["rm", "--cached", file], project_dir)
        {
            tracing::warn!("git rm --cached failed: {err}");
        }
        ctx.terminal.stdout_write(&format!("Untracked: {file}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn no_color_env() -> MockEnvironment {
        MockEnvironment::new().with_var("NO_COLOR", "1")
    }

    #[test]
    fn init_project_adds_gitignore() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Added"));
    }

    #[test]
    fn init_project_no_gitignore_flag() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new();
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), true, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Skipping .gitignore"));
    }

    #[test]
    fn init_project_dry_run_test() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, true));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("DRY RUN"));
        assert!(output.contains("Would add"));
    }

    #[test]
    fn init_project_not_git_repo() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let env = no_color_env();
        let terminal = BufferedTerminal::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: "not a git repo".into(),
                exit_code: 128,
            },
        );
        let ctx = InstallContext {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &terminal,
        };
        assert!(init_project(&ctx, Path::new("/project"), false, false));
        let output = terminal.stdout_output().join("");
        assert!(output.contains("Not a git repository"));
    }
}
