//! Clock port for injectable time sources.

/// Port for obtaining the current time.
///
/// Production code uses [`ecc_infra::system_clock::SystemClock`].
/// Tests use [`ecc_test_support::mock_clock::MockClock`] for deterministic time.
pub trait Clock: Send + Sync {
    /// Return the current time as an ISO 8601 UTC string (e.g., "2026-04-01T10:00:00Z").
    fn now_iso8601(&self) -> String;

    /// Return the current time as seconds since the Unix epoch.
    fn now_epoch_secs(&self) -> u64;
}
