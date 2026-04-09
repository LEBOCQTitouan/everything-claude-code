//! Lock file value object — pure domain type for session claiming.
//!
//! Lock file format: line 1 = worktree name, line 2 = ISO 8601 timestamp.
//! No I/O: parsing and staleness logic are pure functions.

use super::entry::BacklogError;

/// Staleness threshold: 24 hours in seconds.
pub const LOCK_STALE_SECS: u64 = 24 * 3600;

/// A parsed lock file representing a session's claim on a backlog item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockFile {
    pub worktree_name: String,
    pub timestamp: String,
    epoch_secs: u64,
}

impl LockFile {
    /// Parse lock file content (line 1 = worktree name, line 2 = ISO 8601 timestamp).
    pub fn parse(content: &str) -> Result<Self, BacklogError> {
        let mut lines = content.lines();
        let worktree_name = lines
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| BacklogError::Io {
                path: String::new(),
                message: "lock file missing worktree name (line 1)".into(),
            })?
            .to_owned();

        let timestamp = lines
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| BacklogError::Io {
                path: String::new(),
                message: "lock file missing timestamp (line 2)".into(),
            })?
            .to_owned();

        let epoch_secs = parse_iso8601_to_epoch(&timestamp).ok_or_else(|| BacklogError::Io {
            path: String::new(),
            message: format!("invalid ISO 8601 timestamp: {timestamp}"),
        })?;

        Ok(Self {
            worktree_name,
            timestamp,
            epoch_secs,
        })
    }

    /// Format the lock file content for writing.
    pub fn format(&self) -> String {
        format!("{}\n{}\n", self.worktree_name, self.timestamp)
    }

    /// Check if this lock is stale (older than 24 hours).
    pub fn is_stale(&self, now_epoch_secs: u64) -> bool {
        now_epoch_secs.saturating_sub(self.epoch_secs) > LOCK_STALE_SECS
    }

    /// Create a new lock file from components.
    pub fn new(worktree_name: String, timestamp: String) -> Result<Self, BacklogError> {
        let epoch_secs = parse_iso8601_to_epoch(&timestamp).ok_or_else(|| BacklogError::Io {
            path: String::new(),
            message: format!("invalid ISO 8601 timestamp: {timestamp}"),
        })?;
        Ok(Self {
            worktree_name,
            timestamp,
            epoch_secs,
        })
    }
}

/// Parse a subset of ISO 8601 timestamps to Unix epoch seconds.
/// Supports: `2026-04-07T14:10:39Z` format.
fn parse_iso8601_to_epoch(ts: &str) -> Option<u64> {
    let ts = ts.trim();
    // Format: YYYY-MM-DDTHH:MM:SSZ
    if ts.len() < 20 || !ts.ends_with('Z') {
        return None;
    }
    let year: u64 = ts[0..4].parse().ok()?;
    let month: u64 = ts[5..7].parse().ok()?;
    let day: u64 = ts[8..10].parse().ok()?;
    let hour: u64 = ts[11..13].parse().ok()?;
    let min: u64 = ts[14..16].parse().ok()?;
    let sec: u64 = ts[17..19].parse().ok()?;

    // Simplified days-since-epoch (no leap second precision needed for 24h staleness)
    let mut days: u64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap(year) {
            days += 1;
        }
    }
    days += day - 1;

    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_lock() {
        let content = "ecc-session-20260407-142256-backlog-sync-11371\n2026-04-07T14:10:39Z\n";
        let lock = LockFile::parse(content).unwrap();
        assert_eq!(
            lock.worktree_name,
            "ecc-session-20260407-142256-backlog-sync-11371"
        );
        assert_eq!(lock.timestamp, "2026-04-07T14:10:39Z");
    }

    #[test]
    fn parse_invalid_lock() {
        // Empty content
        assert!(LockFile::parse("").is_err());
        // Missing timestamp
        assert!(LockFile::parse("worktree-name\n").is_err());
        // Whitespace only
        assert!(LockFile::parse("  \n  \n").is_err());
        // Invalid timestamp format
        assert!(LockFile::parse("name\nnot-a-timestamp\n").is_err());
    }

    #[test]
    fn format_roundtrip() {
        let lock = LockFile::new("my-worktree".into(), "2026-04-07T14:10:39Z".into()).unwrap();
        let formatted = lock.format();
        let parsed = LockFile::parse(&formatted).unwrap();
        assert_eq!(parsed.worktree_name, lock.worktree_name);
        assert_eq!(parsed.timestamp, lock.timestamp);
    }

    #[test]
    fn stale_after_24h() {
        let lock = LockFile::new("wt".into(), "2026-04-07T00:00:00Z".into()).unwrap();
        // 25 hours later
        let now = lock.epoch_secs + 25 * 3600;
        assert!(lock.is_stale(now));
    }

    #[test]
    fn fresh_within_24h() {
        let lock = LockFile::new("wt".into(), "2026-04-07T00:00:00Z".into()).unwrap();
        // 23 hours later
        let now = lock.epoch_secs + 23 * 3600;
        assert!(!lock.is_stale(now));
    }
}
