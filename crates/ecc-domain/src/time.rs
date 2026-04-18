/// A simple date-time struct for formatting, avoiding external crate dependencies.
/// Fields are expected to be pre-validated calendar values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    /// Year in the proleptic Gregorian calendar.
    pub year: u16,
    /// Month (1-12).
    pub month: u8,
    /// Day of month (1-31).
    pub day: u8,
    /// Hour (0-23).
    pub hour: u8,
    /// Minute (0-59).
    pub minute: u8,
    /// Second (0-59).
    pub second: u8,
}

/// Format a `DateTime` as YYYY-MM-DD.
pub fn format_date(dt: &DateTime) -> String {
    format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day)
}

/// Format a `DateTime` as HH:MM.
pub fn format_time(dt: &DateTime) -> String {
    format!("{:02}:{:02}", dt.hour, dt.minute)
}

/// Format a `DateTime` as YYYY-MM-DD HH:MM:SS.
pub fn format_datetime(dt: &DateTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    )
}

/// Parse an ISO 8601 datetime string (YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM:SS) into a `DateTime`.
///
/// Returns `None` if the format is invalid or dates are out of range.
pub fn parse_datetime(s: &str) -> Option<DateTime> {
    // Accept both 'T' and space as separator
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }
    let bytes = s.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || (bytes[10] != b'T' && bytes[10] != b' ')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }

    let year = s[0..4].parse::<u16>().ok()?;
    let month = s[5..7].parse::<u8>().ok()?;
    let day = s[8..10].parse::<u8>().ok()?;
    let hour = s[11..13].parse::<u8>().ok()?;
    let minute = s[14..16].parse::<u8>().ok()?;
    let second = s[17..19].parse::<u8>().ok()?;

    if month == 0 || month > 12 || day == 0 || day > 31 || hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    Some(DateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

/// Returns true if `y` is a leap year in the proleptic Gregorian calendar.
///
/// A year is a leap year if divisible by 4, except century years which must be divisible by 400.
pub fn is_leap_year(y: u64) -> bool {
    (y.is_multiple_of(4) && !(y.is_multiple_of(100))) || y.is_multiple_of(400)
}

/// Convert days since 1970-01-01 to (year, month, day) using the Hinnant algorithm.
pub fn days_to_civil(days: u32) -> (u16, u8, u8) {
    // Algorithm from Howard Hinnant
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u16, m as u8, d as u8)
}

/// Build a `DateTime` from a Unix epoch timestamp (seconds since 1970-01-01 UTC) in UTC.
pub fn datetime_from_epoch(secs: u64) -> DateTime {
    let days = (secs / 86400) as u32;
    let time_of_day = (secs % 86400) as u32;

    let hour = (time_of_day / 3600) as u8;
    let minute = ((time_of_day % 3600) / 60) as u8;
    let second = (time_of_day % 60) as u8;

    let (year, month, day) = days_to_civil(days);

    DateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_leap_year ---

    #[test]
    fn leap_year_divisible_by_400() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2400));
    }

    #[test]
    fn not_leap_year_century_not_divisible_by_400() {
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn leap_year_divisible_by_4_not_century() {
        assert!(is_leap_year(2024));
    }

    #[test]
    fn not_leap_year_not_divisible_by_4() {
        assert!(!is_leap_year(2023));
    }

    fn dt(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> DateTime {
        DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    // --- format_date ---

    #[test]
    fn format_date_basic() {
        assert_eq!(format_date(&dt(2026, 3, 14, 0, 0, 0)), "2026-03-14");
    }

    #[test]
    fn format_date_pads_month_and_day() {
        assert_eq!(format_date(&dt(2024, 1, 5, 0, 0, 0)), "2024-01-05");
    }

    #[test]
    fn format_date_december() {
        assert_eq!(format_date(&dt(2025, 12, 31, 0, 0, 0)), "2025-12-31");
    }

    // --- format_time ---

    #[test]
    fn format_time_basic() {
        assert_eq!(format_time(&dt(0, 0, 0, 14, 30, 0)), "14:30");
    }

    #[test]
    fn format_time_midnight() {
        assert_eq!(format_time(&dt(0, 0, 0, 0, 0, 0)), "00:00");
    }

    #[test]
    fn format_time_pads_hours() {
        assert_eq!(format_time(&dt(0, 0, 0, 9, 5, 0)), "09:05");
    }

    // --- format_datetime ---

    #[test]
    fn format_datetime_basic() {
        assert_eq!(
            format_datetime(&dt(2026, 3, 14, 10, 30, 45)),
            "2026-03-14 10:30:45"
        );
    }

    #[test]
    fn format_datetime_midnight() {
        assert_eq!(
            format_datetime(&dt(2024, 1, 1, 0, 0, 0)),
            "2024-01-01 00:00:00"
        );
    }

    #[test]
    fn format_datetime_end_of_day() {
        assert_eq!(
            format_datetime(&dt(2025, 12, 31, 23, 59, 59)),
            "2025-12-31 23:59:59"
        );
    }

    // --- parse_datetime ---

    #[test]
    fn parse_datetime_with_t_separator() {
        assert_eq!(
            parse_datetime("2026-03-14T10:30:45"),
            Some(dt(2026, 3, 14, 10, 30, 45))
        );
    }

    #[test]
    fn parse_datetime_with_space_separator() {
        assert_eq!(
            parse_datetime("2026-03-14 10:30:45"),
            Some(dt(2026, 3, 14, 10, 30, 45))
        );
    }

    #[test]
    fn parse_datetime_invalid_format() {
        assert_eq!(parse_datetime("not-a-date"), None);
    }

    #[test]
    fn parse_datetime_too_short() {
        assert_eq!(parse_datetime("2026-03-14"), None);
    }

    #[test]
    fn parse_datetime_invalid_month() {
        assert_eq!(parse_datetime("2026-13-14 10:30:45"), None);
    }

    #[test]
    fn parse_datetime_zero_month() {
        assert_eq!(parse_datetime("2026-00-14 10:30:45"), None);
    }

    #[test]
    fn parse_datetime_invalid_hour() {
        assert_eq!(parse_datetime("2026-03-14 25:30:45"), None);
    }

    #[test]
    fn parse_datetime_roundtrip() {
        let original = dt(2025, 6, 15, 8, 45, 12);
        let formatted = format_datetime(&original);
        let parsed = parse_datetime(&formatted).unwrap();
        assert_eq!(parsed, original);
    }
}
