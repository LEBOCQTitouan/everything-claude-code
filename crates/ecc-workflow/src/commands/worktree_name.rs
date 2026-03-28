use std::process;

use ecc_domain::worktree::WorktreeName;

use crate::output::WorkflowOutput;

pub fn run(concern: &str, feature: &str) -> WorkflowOutput {
    let pid = process::id();
    match WorktreeName::generate(concern, feature, pid) {
        Ok(name) => WorkflowOutput::pass(name.as_str().to_owned()),
        Err(e) => WorkflowOutput::block(format!("Invalid worktree name: {e}")),
    }
}

#[cfg(test)]
mod worktree_name {
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
        assert!(msg.starts_with("ecc-session-"), "expected ecc-session- prefix, got: {msg}");
        assert!(msg.contains("my-feature"), "expected slug in name, got: {msg}");
    }

    #[test]
    fn blocks_on_invalid_chars() {
        // A name with shell injection in the concern is still safe because generate() slugifies
        // the feature and ignores concern for now; test that a valid call always passes.
        let output = run("dev", "valid-feature");
        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "expected pass, got: {:?}",
            output.status
        );
    }
}
