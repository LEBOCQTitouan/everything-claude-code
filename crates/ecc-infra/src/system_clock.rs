//! Production adapter for [`ecc_ports::clock::Clock`].
//!
//! Uses `std::time::SystemTime` for the real wall clock.

use ecc_ports::clock::Clock;
use std::time::SystemTime;

/// Clock adapter that returns real system time.
///
/// # Pattern
///
/// Adapter \[Hexagonal Architecture\] — implements `ecc_ports::clock::Clock`
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_iso8601(&self) -> String {
        let secs = self.now_epoch_secs();
        epoch_to_iso8601(secs)
    }

    fn now_epoch_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Convert epoch seconds to an ISO 8601 UTC string.
fn epoch_to_iso8601(epoch_secs: u64) -> String {
    // Civil time calculation from epoch seconds
    let secs_in_day = epoch_secs % 86400;
    let hour = secs_in_day / 3600;
    let min = (secs_in_day % 3600) / 60;
    let sec = secs_in_day % 60;

    let days = epoch_secs / 86400;
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}
