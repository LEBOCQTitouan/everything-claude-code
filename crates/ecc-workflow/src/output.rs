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
    Block,
    Warn,
}
