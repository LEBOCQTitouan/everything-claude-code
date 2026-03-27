use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowOutput {
    pub status: Status,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    // Block is used by later subcommands (e.g., phase-gate); allow until then
    #[allow(dead_code)]
    Block,
    Warn,
}

impl WorkflowOutput {
    pub fn pass(message: impl Into<String>) -> Self {
        Self {
            status: Status::Pass,
            message: message.into(),
        }
    }

    // Used by later subcommands (e.g., phase-gate); allow until then
    #[allow(dead_code)]
    pub fn block(message: impl Into<String>) -> Self {
        Self {
            status: Status::Block,
            message: message.into(),
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            status: Status::Warn,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorkflowOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{json}")
    }
}
