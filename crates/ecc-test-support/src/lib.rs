//! Test doubles for Everything Claude Code.
//!
//! Provides in-memory and mock implementations of [`ecc_ports`] traits
//! ([`InMemoryFileSystem`], [`MockExecutor`], [`MockEnvironment`],
//! [`BufferedTerminal`], [`ScriptedInput`]) enabling fully deterministic,
//! I/O-free testing of application use cases.

pub mod buffered_terminal;
pub mod in_memory_config_store;
pub mod in_memory_cost_store;
pub mod in_memory_fs;
pub mod in_memory_lock;
pub mod in_memory_log_store;
pub mod in_memory_metrics_store;
pub mod in_memory_memory_store;
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
pub use in_memory_fs::InMemoryFileSystem;
pub use in_memory_lock::InMemoryLock;
pub use in_memory_log_store::InMemoryLogStore;
pub use in_memory_metrics_store::InMemoryMetricsStore;
pub use in_memory_memory_store::InMemoryMemoryStore;
pub use mock_clock::MockClock;
pub use mock_env::MockEnvironment;
pub use mock_executor::MockExecutor;
pub use mock_extractor::MockExtractor;
pub use mock_git::MockGitInfo;
pub use mock_release_client::MockReleaseClient;
pub use scripted_input::ScriptedInput;
pub mod mock_worktree;
pub use mock_worktree::MockWorktreeManager;
