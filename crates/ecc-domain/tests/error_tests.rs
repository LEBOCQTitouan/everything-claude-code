//! PC-023: WorkflowError::MissingJustification display test.

use ecc_domain::workflow::error::WorkflowError;

mod error {
    use super::*;

    #[test]
    fn missing_justification_message() {
        let err = WorkflowError::MissingJustification;
        let msg = err.to_string();
        assert!(
            msg.contains("justification must be non-empty"),
            "expected 'justification must be non-empty' in error message, got: {msg}"
        );
    }
}
