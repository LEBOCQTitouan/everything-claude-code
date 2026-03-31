use ecc_ports::env::Environment;
use ecc_ports::extract::TarballExtractor;
use ecc_ports::fs::FileSystem;
use ecc_ports::lock::FileLock;
use ecc_ports::release::ReleaseClient;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

/// Bundles all port references needed by the update orchestrator.
pub struct UpdateContext<'a> {
    pub fs: &'a dyn FileSystem,
    pub env: &'a dyn Environment,
    pub shell: &'a dyn ShellExecutor,
    pub terminal: &'a dyn TerminalIO,
    pub release_client: &'a dyn ReleaseClient,
    pub lock: &'a dyn FileLock,
    pub extractor: &'a dyn TarballExtractor,
}
