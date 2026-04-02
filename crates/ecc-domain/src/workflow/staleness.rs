//! Staleness detection for workflow state — pure function, no I/O.

/// Default staleness threshold: 4 hours (14400 seconds).
pub const DEFAULT_STALENESS_THRESHOLD_SECS: u64 = 14400;

/// Check if a workflow is stale based on its `started_at` timestamp.
///
/// Both `started_at` and `now` are ISO 8601 UTC strings (e.g., "2026-04-01T10:00:00Z").
/// Returns `true` if the elapsed time >= `threshold_secs`.
pub fn is_stale(started_at: &str, now: &str, threshold_secs: u64) -> bool {
    let start = parse_iso8601_to_epoch(started_at);
    let current = parse_iso8601_to_epoch(now);
    match (start, current) {
        (Some(s), Some(c)) => c.saturating_sub(s) >= threshold_secs,
        _ => false, // unparseable timestamps → not stale (fail open)
    }
}

/// Parse a subset of ISO 8601 (YYYY-MM-DDTHH:MM:SSZ) to epoch seconds.
/// Handles the format used by `utc_now_iso8601()` in the codebase.
fn parse_iso8601_to_epoch(ts: &str) -> Option<u64> {
    // Format: "2026-04-01T10:00:00Z"
    let ts = ts.trim_end_matches('Z');
    let (date_part, time_part) = ts.split_once('T')?;
    let mut date_iter = date_part.split('-');
    let year: u64 = date_iter.next()?.parse().ok()?;
    let month: u64 = date_iter.next()?.parse().ok()?;
    let day: u64 = date_iter.next()?.parse().ok()?;

    let mut time_iter = time_part.split(':');
    let hour: u64 = time_iter.next()?.parse().ok()?;
    let min: u64 = time_iter.next()?.parse().ok()?;
    let sec: u64 = time_iter.next()?.parse().ok()?;

    // Days from epoch (1970-01-01) using a simplified calculation
    let days = days_from_epoch(year, month, day)?;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

/// Calculate days from Unix epoch (1970-01-01) to the given date.
fn days_from_epoch(year: u64, month: u64, day: u64) -> Option<u64> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    // Adjust for months before March (year starts in March for leap year calc)
    let (y, m) = if month <= 2 {
        (year - 1, month + 9)
    } else {
        (year, month - 3)
    };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe;
    // Epoch offset: 1970-01-01 is day 719468 in this calendar
    Some(days - 719468)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_when_threshold_exceeded() {
        // 5 hours elapsed, threshold is 4 hours → stale
        assert!(is_stale(
            "2026-04-01T06:00:00Z",
            "2026-04-01T11:00:00Z",
            14400
        ));
    }

    #[test]
    fn not_stale_within_threshold() {
        // 2 hours elapsed, threshold is 4 hours → not stale
        assert!(!is_stale(
            "2026-04-01T09:00:00Z",
            "2026-04-01T11:00:00Z",
            14400
        ));
    }

    #[test]
    fn stale_at_exact_threshold() {
        // Exactly 4 hours elapsed → stale (>= threshold)
        assert!(is_stale(
            "2026-04-01T07:00:00Z",
            "2026-04-01T11:00:00Z",
            14400
        ));
    }

    #[test]
    fn not_stale_for_unparseable_timestamps() {
        assert!(!is_stale("not-a-date", "2026-04-01T11:00:00Z", 14400));
        assert!(!is_stale("2026-04-01T11:00:00Z", "garbage", 14400));
    }

    #[test]
    fn parse_epoch_known_date() {
        // 2026-04-01T00:00:00Z should be a known epoch value
        let epoch = parse_iso8601_to_epoch("2026-04-01T00:00:00Z");
        assert!(epoch.is_some());
        // Verify it's in the right ballpark (after 2025, before 2027)
        let secs = epoch.unwrap();
        assert!(secs > 1_700_000_000, "2026 should be after ~1.7B epoch secs");
        assert!(secs < 1_900_000_000, "2026 should be before ~1.9B epoch secs");
    }
}
