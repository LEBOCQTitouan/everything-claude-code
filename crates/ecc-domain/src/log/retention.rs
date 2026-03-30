/// Type-safe retention duration for log pruning.
///
/// Wraps a number of days parsed from the `Nd` format (e.g., `"30d"`, `"7d"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionDuration {
    /// Number of days to retain logs.
    pub days: u32,
}

impl RetentionDuration {
    /// Parse a retention duration from a string in `Nd` format.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string is not a valid `Nd` pattern or if `N` is zero.
    pub fn from_str(s: &str) -> Result<Self, String> {
        todo!("implement RetentionDuration::from_str")
    }

    /// Returns the default retention of 30 days.
    pub fn default_30_days() -> Self {
        todo!("implement RetentionDuration::default_30_days")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_30d() {
        let r = RetentionDuration::from_str("30d").unwrap();
        assert_eq!(r.days, 30);
    }

    #[test]
    fn parses_7d() {
        let r = RetentionDuration::from_str("7d").unwrap();
        assert_eq!(r.days, 7);
    }

    #[test]
    fn parses_1d() {
        let r = RetentionDuration::from_str("1d").unwrap();
        assert_eq!(r.days, 1);
    }

    #[test]
    fn rejects_zero() {
        assert!(RetentionDuration::from_str("0d").is_err());
    }

    #[test]
    fn rejects_abc() {
        assert!(RetentionDuration::from_str("abc").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(RetentionDuration::from_str("").is_err());
    }

    #[test]
    fn rejects_missing_d_suffix() {
        assert!(RetentionDuration::from_str("30").is_err());
    }

    #[test]
    fn rejects_negative_string() {
        assert!(RetentionDuration::from_str("-7d").is_err());
    }

    #[test]
    fn default_30_days_returns_30() {
        let r = RetentionDuration::default_30_days();
        assert_eq!(r.days, 30);
    }

    #[test]
    fn large_value_accepted() {
        let r = RetentionDuration::from_str("365d").unwrap();
        assert_eq!(r.days, 365);
    }
}
