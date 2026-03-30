//! Memory statistics aggregation types.

use crate::memory::tier::MemoryTier;
use std::collections::HashMap;

/// Aggregate statistics about the memory store.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    /// Entry counts per tier.
    pub counts_by_tier: HashMap<MemoryTier, usize>,
    /// Number of entries marked stale.
    pub stale_count: usize,
    /// Size of the database file in bytes.
    pub db_size_bytes: u64,
    /// ISO-8601 timestamp of the oldest entry, if any.
    pub oldest: Option<String>,
    /// ISO-8601 timestamp of the newest entry, if any.
    pub newest: Option<String>,
}

impl MemoryStats {
    /// Create a new MemoryStats value.
    pub fn new(
        counts_by_tier: HashMap<MemoryTier, usize>,
        stale_count: usize,
        db_size_bytes: u64,
        oldest: Option<String>,
        newest: Option<String>,
    ) -> Self {
        MemoryStats {
            counts_by_tier,
            stale_count,
            db_size_bytes,
            oldest,
            newest,
        }
    }

    /// Total number of entries across all tiers.
    pub fn total_count(&self) -> usize {
        self.counts_by_tier.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-019: MemoryStats struct holds counts_by_tier, stale_count, db_size_bytes, oldest, newest
    #[test]
    fn test_memory_stats_construction() {
        let mut counts = HashMap::new();
        counts.insert(MemoryTier::Working, 2);
        counts.insert(MemoryTier::Episodic, 10);
        counts.insert(MemoryTier::Semantic, 5);

        let stats = MemoryStats::new(
            counts.clone(),
            3,
            1024 * 1024,
            Some("2026-01-01T00:00:00Z".to_owned()),
            Some("2026-03-30T12:00:00Z".to_owned()),
        );

        assert_eq!(stats.counts_by_tier[&MemoryTier::Working], 2);
        assert_eq!(stats.counts_by_tier[&MemoryTier::Episodic], 10);
        assert_eq!(stats.counts_by_tier[&MemoryTier::Semantic], 5);
        assert_eq!(stats.stale_count, 3);
        assert_eq!(stats.db_size_bytes, 1024 * 1024);
        assert_eq!(stats.oldest, Some("2026-01-01T00:00:00Z".to_owned()));
        assert_eq!(stats.newest, Some("2026-03-30T12:00:00Z".to_owned()));
    }

    #[test]
    fn test_memory_stats_total_count() {
        let mut counts = HashMap::new();
        counts.insert(MemoryTier::Working, 2);
        counts.insert(MemoryTier::Episodic, 10);
        counts.insert(MemoryTier::Semantic, 5);

        let stats = MemoryStats::new(counts, 0, 0, None, None);
        assert_eq!(stats.total_count(), 17);
    }

    #[test]
    fn test_memory_stats_empty() {
        let stats = MemoryStats::new(HashMap::new(), 0, 0, None, None);
        assert_eq!(stats.total_count(), 0);
        assert!(stats.oldest.is_none());
        assert!(stats.newest.is_none());
    }

    #[test]
    fn test_memory_stats_clone() {
        let mut counts = HashMap::new();
        counts.insert(MemoryTier::Semantic, 7);
        let stats = MemoryStats::new(counts, 1, 512, None, None);
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn test_memory_stats_debug() {
        let stats = MemoryStats::new(HashMap::new(), 0, 0, None, None);
        let s = format!("{:?}", stats);
        assert!(s.contains("MemoryStats"));
    }
}
