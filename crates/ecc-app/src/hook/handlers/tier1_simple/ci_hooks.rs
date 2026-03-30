use crate::hook::{HookPorts, HookResult};
use super::helpers::extract_file_path;

/// Protected branch names (exact match only).
const PROTECTED_BRANCHES: &[&str] = &["main", "master", "production"];

/// pre:edit-write:workflow-branch-guard -- block workflow edits on protected branches.
pub fn pre_edit_write_workflow_branch_guard(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    todo!("implement workflow branch guard")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::shell::CommandOutput;
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

    fn make_stdin(file_path: &str) -> String {
        format!(r#"{{"tool_input":{{"file_path":"{}"}}}}"#, file_path)
    }

    fn git_output(branch: &str) -> CommandOutput {
        CommandOutput {
            stdout: format!("{}\n", branch),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    #[test]
    fn blocks_on_main() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("main"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }

    #[test]
    fn blocks_on_master() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("master"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/release.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }

    #[test]
    fn blocks_on_production() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("production"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/cd.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }

    #[test]
    fn passthrough_feature_branch() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("feature/update-ci"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_non_workflow() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("main"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/CODEOWNERS");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_detached_head() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("HEAD"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_non_git_repo() {
        let fs = InMemoryFileSystem::new();
        // No mock registered -- run_command returns ShellError::NotFound
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_superstring_main_feature() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("main-feature"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_superstring_production_hotfix() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("production-hotfix"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_shell_error() {
        let fs = InMemoryFileSystem::new();
        // No mock registered -- run_command returns ShellError::NotFound
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = make_stdin(".github/workflows/ci.yml");
        let result = pre_edit_write_workflow_branch_guard(&stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn passthrough_empty_file_path() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("main"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // stdin with no file_path key
        let stdin = r#"{"tool_input":{"content":"something"}}"#;
        let result = pre_edit_write_workflow_branch_guard(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn multiedit_file_path_extraction() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--abbrev-ref", "HEAD"],
            git_output("main"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // MultiEdit uses same tool_input.file_path field
        let stdin = r#"{"tool_input":{"file_path":".github/workflows/ci.yml","edits":[]}}"#;
        let result = pre_edit_write_workflow_branch_guard(stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }
}
