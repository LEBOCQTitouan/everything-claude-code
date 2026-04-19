use ecc_domain::worktree::WorktreeName;

use crate::output::WorkflowOutput;

pub fn run(concern: &str, feature: &str) -> WorkflowOutput {
    // Use parent PID (the Claude Code session process) instead of our own
    // short-lived subprocess PID. This ensures GC's `kill -0 <pid>` check
    // correctly identifies active worktrees as non-stale.
    let pid = std::os::unix::process::parent_id();
    match WorktreeName::generate(concern, feature, pid) {
        Ok(name) => WorkflowOutput::pass(name.as_str().to_owned()),
        Err(e) => WorkflowOutput::block(format!("Invalid worktree name: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_pass_output() {
        let output = run("dev", "my feature");
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "expected pass, got: {:?}",
            output.status
        );
        let msg = &output.message;
        assert!(
            msg.starts_with("ecc-session-"),
            "expected ecc-session- prefix, got: {msg}"
        );
        assert!(
            msg.contains("my-feature"),
            "expected slug in name, got: {msg}"
        );
    }

    #[test]
    fn passes_for_safe_feature_name() {
        // generate() slugifies the feature; any safe ASCII input produces a pass result.
        let output = run("dev", "valid-feature");
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "expected pass, got: {:?}",
            output.status
        );
    }
}
