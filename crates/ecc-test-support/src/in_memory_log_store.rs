use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ecc_ports::log_store::{ExportFormat, LogEntry, LogQuery, LogStore, LogStoreError};

/// In-memory test double for [`LogStore`].
///
/// Supports seeding entries via [`InMemoryLogStore::seed`] and introspecting
/// the stored entries via [`InMemoryLogStore::entries`].
pub struct InMemoryLogStore {
    entries: Mutex<Vec<LogEntry>>,
}

impl InMemoryLogStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Seed the store with a set of entries.
    pub fn seed(&self, entries: Vec<LogEntry>) {
        let mut guard = self.entries.lock().expect("lock poisoned");
        guard.extend(entries);
    }

    /// Return a snapshot of all stored entries.
    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries.lock().expect("lock poisoned").clone()
    }
}

impl Default for InMemoryLogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LogStore for InMemoryLogStore {
    fn search(&self, query: &LogQuery) -> Result<Vec<LogEntry>, LogStoreError> {
        let guard = self.entries.lock().expect("lock poisoned");
        let results: Vec<LogEntry> = guard
            .iter()
            .filter(|e| matches_query(e, query))
            .cloned()
            .take(query.limit)
            .collect();
        Ok(results)
    }

    fn tail(&self, count: usize, session_id: Option<&str>) -> Result<Vec<LogEntry>, LogStoreError> {
        let guard = self.entries.lock().expect("lock poisoned");
        let filtered: Vec<&LogEntry> = guard
            .iter()
            .filter(|e| {
                session_id.map_or(true, |sid| e.session_id == sid)
            })
            .collect();
        let start = filtered.len().saturating_sub(count);
        Ok(filtered[start..].iter().map(|e| (*e).clone()).collect())
    }

    fn prune(&self, older_than: Duration) -> Result<u64, LogStoreError> {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(older_than);
        let cutoff_secs = cutoff.as_secs();

        let mut guard = self.entries.lock().expect("lock poisoned");
        let before = guard.len();
        guard.retain(|e| {
            parse_timestamp_secs(&e.timestamp).map_or(true, |ts| ts >= cutoff_secs)
        });
        let removed = before - guard.len();
        Ok(removed as u64)
    }

    fn export(&self, query: &LogQuery, format: ExportFormat) -> Result<String, LogStoreError> {
        let entries = self.search(query)?;
        match format {
            ExportFormat::Json => {
                let items: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        format!(
                            r#"{{"id":{id},"session_id":{session_id},"timestamp":{timestamp},"level":{level},"target":{target},"message":{message},"fields_json":{fields_json}}}"#,
                            id = e.id.map_or("null".to_string(), |v| v.to_string()),
                            session_id = json_str(&e.session_id),
                            timestamp = json_str(&e.timestamp),
                            level = json_str(&e.level),
                            target = json_str(&e.target),
                            message = json_str(&e.message),
                            fields_json = e.fields_json.clone(),
                        )
                    })
                    .collect();
                Ok(format!("[{}]", items.join(",")))
            }
            ExportFormat::Csv => {
                let mut rows =
                    vec!["id,session_id,timestamp,level,target,message,fields_json".to_string()];
                for e in &entries {
                    rows.push(format!(
                        "{},{},{},{},{},{},{}",
                        e.id.map_or("".to_string(), |v| v.to_string()),
                        csv_escape(&e.session_id),
                        csv_escape(&e.timestamp),
                        csv_escape(&e.level),
                        csv_escape(&e.target),
                        csv_escape(&e.message),
                        csv_escape(&e.fields_json),
                    ));
                }
                Ok(rows.join("\n"))
            }
        }
    }
}

fn matches_query(entry: &LogEntry, query: &LogQuery) -> bool {
    if let Some(ref text) = query.text {
        let text_lower = text.to_lowercase();
        if !entry.message.to_lowercase().contains(&text_lower)
            && !entry.fields_json.to_lowercase().contains(&text_lower)
        {
            return false;
        }
    }
    if let Some(ref sid) = query.session_id {
        if entry.session_id != *sid {
            return false;
        }
    }
    if let Some(since) = query.since {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(since);
        let cutoff_secs = cutoff.as_secs();
        let entry_secs = parse_timestamp_secs(&entry.timestamp).unwrap_or(0);
        if entry_secs < cutoff_secs {
            return false;
        }
    }
    if let Some(ref level) = query.level {
        if !entry.level.eq_ignore_ascii_case(level) {
            return false;
        }
    }
    true
}

/// Parse an ISO-8601 timestamp to Unix seconds (best-effort).
fn parse_timestamp_secs(ts: &str) -> Option<u64> {
    // Accept "2024-01-15T12:00:00Z" or "2024-01-15T12:00:00+00:00"
    // Simple heuristic: extract the numeric parts
    let ts = ts.trim_end_matches('Z').trim_end_matches("+00:00");
    let parts: Vec<&str> = ts.splitn(2, 'T').collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<u32> = parts[0]
        .split('-')
        .filter_map(|p| p.parse().ok())
        .collect();
    let time_parts: Vec<u32> = parts[1]
        .splitn(4, ':')
        .filter_map(|p| p.parse().ok())
        .collect();
    if date_parts.len() != 3 || time_parts.len() < 2 {
        return None;
    }
    // Rough epoch calculation (ignores leap years/seconds but good enough for tests)
    let year = date_parts[0] as u64;
    let month = date_parts[1] as u64;
    let day = date_parts[2] as u64;
    let hour = time_parts[0] as u64;
    let min = time_parts[1] as u64;
    let sec = time_parts.get(2).copied().unwrap_or(0) as u64;

    let days_since_epoch = (year - 1970) * 365 + (year - 1970) / 4 + month * 30 + day;
    Some(days_since_epoch * 86400 + hour * 3600 + min * 60 + sec)
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(session_id: &str, level: &str, message: &str) -> LogEntry {
        LogEntry {
            id: None,
            session_id: session_id.to_string(),
            timestamp: "2024-01-15T12:00:00Z".to_string(),
            level: level.to_string(),
            target: "ecc".to_string(),
            message: message.to_string(),
            fields_json: "{}".to_string(),
        }
    }

    #[test]
    fn search_returns_all_when_no_filters() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "hello"),
            make_entry("s2", "WARN", "world"),
        ]);
        let results = store.search(&LogQuery::default()).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_filters_by_session_id() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "hello"),
            make_entry("s2", "WARN", "world"),
        ]);
        let query = LogQuery {
            session_id: Some("s1".to_string()),
            ..LogQuery::default()
        };
        let results = store.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "s1");
    }

    #[test]
    fn search_filters_by_level() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "hello"),
            make_entry("s2", "WARN", "world"),
        ]);
        let query = LogQuery {
            level: Some("WARN".to_string()),
            ..LogQuery::default()
        };
        let results = store.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, "WARN");
    }

    #[test]
    fn search_filters_by_text() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "hello world"),
            make_entry("s2", "WARN", "goodbye"),
        ]);
        let query = LogQuery {
            text: Some("hello".to_string()),
            ..LogQuery::default()
        };
        let results = store.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "hello world");
    }

    #[test]
    fn search_limit_defaults_to_100() {
        let store = InMemoryLogStore::new();
        let entries: Vec<LogEntry> = (0..150)
            .map(|i| make_entry("s1", "INFO", &format!("msg {i}")))
            .collect();
        store.seed(entries);
        let results = store.search(&LogQuery::default()).unwrap();
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn tail_returns_last_n_entries() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "first"),
            make_entry("s1", "INFO", "second"),
            make_entry("s1", "INFO", "third"),
        ]);
        let results = store.tail(2, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].message, "second");
        assert_eq!(results[1].message, "third");
    }

    #[test]
    fn tail_filters_by_session() {
        let store = InMemoryLogStore::new();
        store.seed(vec![
            make_entry("s1", "INFO", "a"),
            make_entry("s2", "INFO", "b"),
            make_entry("s1", "INFO", "c"),
        ]);
        let results = store.tail(10, Some("s1")).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn export_json_contains_entries() {
        let store = InMemoryLogStore::new();
        store.seed(vec![make_entry("s1", "INFO", "hello")]);
        let output = store.export(&LogQuery::default(), ExportFormat::Json).unwrap();
        assert!(output.starts_with('['));
        assert!(output.ends_with(']'));
        assert!(output.contains("hello"));
    }

    #[test]
    fn export_csv_has_header() {
        let store = InMemoryLogStore::new();
        store.seed(vec![make_entry("s1", "INFO", "hello")]);
        let output = store.export(&LogQuery::default(), ExportFormat::Csv).unwrap();
        assert!(output.starts_with("id,session_id,timestamp,level,target,message,fields_json"));
    }
}
