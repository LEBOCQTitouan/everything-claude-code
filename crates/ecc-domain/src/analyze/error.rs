//! Domain errors for the analyze module.

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("invalid top value: {0} (must be > 0)")]
    InvalidTopN(usize),
}
