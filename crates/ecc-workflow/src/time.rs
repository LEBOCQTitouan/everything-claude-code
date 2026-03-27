/// Return the current UTC time formatted as ISO 8601: `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc_now_iso8601() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (year, month, day, hour, min, sec) = unix_secs_to_calendar(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Return the current UTC date as `YYYY-MM-DD`.
pub fn utc_today() -> String {
    let ts = utc_now_iso8601();
    ts[..10].to_string()
}

/// Return the current UTC time as `HH:MM`.
pub fn utc_hhmm() -> String {
    let ts = utc_now_iso8601();
    ts[11..16].to_string()
}

/// Convert a Unix epoch (seconds) to (year, month, day, hour, min, sec) in UTC.
///
/// Uses the Gregorian calendar algorithm from
/// <http://howardhinnant.github.io/date_algorithms.html> (civil_from_days).
pub fn unix_secs_to_calendar(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d, hour, min, sec)
}
