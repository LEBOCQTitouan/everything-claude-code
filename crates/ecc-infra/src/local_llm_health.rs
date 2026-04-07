/// Check if the Ollama API is available at the given base URL.
pub fn check_ollama_health(_base_url: &str, _timeout_ms: u64) -> bool {
    todo!("not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_returns_false_on_connection_refused() {
        let result = check_ollama_health("http://127.0.0.1:1", 1000);
        assert!(!result);
    }

    #[test]
    fn health_check_returns_false_on_invalid_url() {
        let result = check_ollama_health("not-a-url", 1000);
        assert!(!result);
    }
}
