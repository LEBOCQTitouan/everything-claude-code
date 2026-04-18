//! Test doubles for Everything Claude Code.
//!
//! Provides in-memory and mock implementations of [`ecc_ports`] traits
//! ([`InMemoryFileSystem`], [`MockExecutor`], [`MockEnvironment`],
//! [`BufferedTerminal`], [`ScriptedInput`]) enabling fully deterministic,
//! I/O-free testing of application use cases.

pub mod buffered_terminal;
pub mod in_memory_cache_store;
pub mod in_memory_config_store;
pub mod in_memory_cost_store;
pub mod in_memory_fs;
pub mod in_memory_lock;
pub mod in_memory_log_store;
pub mod in_memory_memory_store;
pub mod in_memory_metrics_store;
pub mod mock_clock;
pub mod mock_env;
pub mod mock_executor;
pub mod mock_extractor;
pub mod mock_git;
pub mod mock_release_client;
pub mod scripted_input;

pub use buffered_terminal::BufferedTerminal;
pub use in_memory_config_store::InMemoryConfigStore;
pub use in_memory_cost_store::InMemoryCostStore;
pub mod in_memory_bypass_store;
pub use in_memory_bypass_store::InMemoryBypassStore;
pub use in_memory_cache_store::InMemoryCacheStore;
pub use in_memory_fs::InMemoryFileSystem;
pub use in_memory_lock::InMemoryLock;
pub use in_memory_log_store::InMemoryLogStore;
pub use in_memory_memory_store::InMemoryMemoryStore;
pub use in_memory_metrics_store::InMemoryMetricsStore;
pub use mock_clock::MockClock;
pub use mock_env::MockEnvironment;

/// Fixed-time test clock for use in `HookPorts` struct literals and `test_default` calls.
///
/// Fixed at 2026-01-01T00:00:00Z (epoch 1735689600).
pub static TEST_CLOCK: std::sync::LazyLock<MockClock> =
    std::sync::LazyLock::new(|| MockClock::fixed("2026-01-01T00:00:00Z", 1_735_689_600));
pub use mock_executor::MockExecutor;
pub use mock_extractor::MockExtractor;
pub use mock_git::MockGitInfo;
pub use mock_release_client::MockReleaseClient;
pub use scripted_input::ScriptedInput;
pub mod mock_worktree;
pub use mock_worktree::MockWorktreeManager;
pub mod in_memory_backlog;
pub use in_memory_backlog::InMemoryBacklogRepository;
