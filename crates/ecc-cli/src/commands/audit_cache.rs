//! CLI subcommands for `ecc audit cache check` and `ecc audit cache clear`.

use clap::{Args, Subcommand};
use ecc_ports::cache_store::CacheStore;

#[derive(Args)]
pub struct AuditCacheArgs {
    #[command(subcommand)]
    pub action: AuditCacheAction,
}

#[derive(Subcommand)]
pub enum AuditCacheAction {
    /// Check cache status for a domain
    Check {
        /// Domain name to check
        domain: String,
    },
    /// Clear all cache entries
    Clear,
}

/// Execute the check action against the given store; returns the output string.
pub fn run_check(_store: &dyn CacheStore, _domain: &str) -> anyhow::Result<String> {
    todo!("not yet implemented")
}

/// Execute the clear action against the given store; returns a confirmation string.
pub fn run_clear(_store: &dyn CacheStore) -> anyhow::Result<String> {
    todo!("not yet implemented")
}

pub fn run(args: AuditCacheArgs) -> anyhow::Result<()> {
    use ecc_infra::file_cache_store::FileCacheStore;

    let cache_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".ecc")
        .join("cache");
    let store = FileCacheStore::new(cache_dir);

    match args.action {
        AuditCacheAction::Check { domain } => {
            let output = run_check(&store, &domain)?;
            println!("{output}");
        }
        AuditCacheAction::Clear => {
            let output = run_clear(&store)?;
            println!("{output}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::in_memory_cache_store::InMemoryCacheStore;

    #[test]
    fn audit_cache_check_miss_prints_miss() {
        let store = InMemoryCacheStore::new();
        let output = run_check(&store, "my-domain").unwrap();
        assert!(
            output.contains("miss"),
            "expected 'miss' in output, got: {output}"
        );
        assert!(output.contains("my-domain"));
    }

    #[test]
    fn audit_cache_check_hit_prints_metadata() {
        let store = InMemoryCacheStore::new();
        store
            .write("my-domain", "findings", 3600, "deadbeef")
            .unwrap();
        let output = run_check(&store, "my-domain").unwrap();
        assert!(
            output.contains("hit"),
            "expected 'hit' in output, got: {output}"
        );
        assert!(output.contains("deadbeef"), "expected hash in output");
        assert!(output.contains("3600"), "expected ttl in output");
    }

    #[test]
    fn audit_cache_clear_removes_entries() {
        let store = InMemoryCacheStore::new();
        store.write("domain-a", "v1", 3600, "h1").unwrap();
        store.write("domain-b", "v2", 3600, "h2").unwrap();

        let output = run_clear(&store).unwrap();
        assert!(
            output.contains("cleared"),
            "expected 'cleared' in output, got: {output}"
        );

        assert!(store.check("domain-a").unwrap().is_none());
        assert!(store.check("domain-b").unwrap().is_none());
    }
}
