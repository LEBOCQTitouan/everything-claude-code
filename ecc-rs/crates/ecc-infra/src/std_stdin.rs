use ecc_ports::stdin::{StdinError, StdinReader};
use std::io::Read;

/// Production stdin reader for hook runtime.
pub struct StdStdinReader;

impl StdinReader for StdStdinReader {
    fn read_json(&self) -> Result<serde_json::Value, StdinError> {
        let raw = self.read_raw()?;
        serde_json::from_str(&raw).map_err(|e| StdinError::InvalidJson(e.to_string()))
    }

    fn read_raw(&self) -> Result<String, StdinError> {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| StdinError::Io(e.to_string()))?;
        Ok(buf)
    }
}
