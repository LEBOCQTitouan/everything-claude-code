/// Port for reading structured data from stdin (used by hook runtime).
pub trait StdinReader: Send + Sync {
    fn read_json(&self) -> Result<serde_json::Value, StdinError>;
    fn read_raw(&self) -> Result<String, StdinError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StdinError {
    #[error("stdin not available")]
    NotAvailable,

    #[error("invalid JSON on stdin: {0}")]
    InvalidJson(String),

    #[error("I/O error reading stdin: {0}")]
    Io(String),
}
