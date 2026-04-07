use std::time::Duration;

/// Check if the Ollama API is available at the given base URL.
/// Returns `true` if GET `{base_url}/api/tags` responds with HTTP 200 within the timeout.
/// Returns `false` on any error (connection refused, timeout, HTTP error, invalid URL).
pub fn check_ollama_health(base_url: &str, timeout_ms: u64) -> bool {
    let url = format!("{base_url}/api/tags");
    let timeout = Duration::from_millis(timeout_ms);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into();
    match agent.get(&url).call() {
        Ok(resp) => resp.status() == 200,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_returns_false_on_connection_refused() {
        // Port 1 on localhost is never open
        let result = check_ollama_health("http://127.0.0.1:1", 1000);
        assert!(!result);
    }

    #[test]
    fn health_check_returns_false_on_invalid_url() {
        let result = check_ollama_health("not-a-url", 1000);
        assert!(!result);
    }
}
