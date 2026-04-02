//! Mock implementation of [`ecc_ports::clock::Clock`] for testing.

use ecc_ports::clock::Clock;

/// Fixed-time clock for deterministic tests.
pub struct MockClock {
    iso8601: String,
    epoch_secs: u64,
}

impl MockClock {
    /// Create a mock clock fixed at the given time.
    pub fn fixed(iso8601: impl Into<String>, epoch_secs: u64) -> Self {
        Self {
            iso8601: iso8601.into(),
            epoch_secs,
        }
    }
}

impl Clock for MockClock {
    fn now_iso8601(&self) -> String {
        self.iso8601.clone()
    }

    fn now_epoch_secs(&self) -> u64 {
        self.epoch_secs
    }
}
