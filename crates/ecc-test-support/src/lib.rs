//! Test doubles for Everything Claude Code.
//!
//! Provides in-memory and mock implementations of [`ecc_ports`] traits
//! ([`InMemoryFileSystem`], [`MockExecutor`], [`MockEnvironment`],
//! [`BufferedTerminal`], [`ScriptedInput`]) enabling fully deterministic,
//! I/O-free testing of application use cases.

pub mod buffered_terminal;
pub mod in_memory_fs;
pub mod in_memory_lock;
pub mod mock_env;
pub mod mock_executor;
pub mod scripted_input;

pub use buffered_terminal::BufferedTerminal;
pub use in_memory_fs::InMemoryFileSystem;
pub use in_memory_lock::InMemoryLock;
pub use mock_env::MockEnvironment;
pub use mock_executor::MockExecutor;
pub use scripted_input::ScriptedInput;
