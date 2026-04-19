//! Cartography hook handlers — stop:cartography writes session deltas,
//! start:cartography processes them via the cartographer agent.

pub mod dedupe_io;
mod delta_helpers;
mod delta_reminder;
mod delta_writer;

pub use delta_reminder::start_cartography;
pub use delta_writer::stop_cartography;

#[cfg(test)]
#[path = "tests_helpers_err002.rs"]
pub mod tests_helpers;

#[cfg(test)]
pub mod tests {
    /// PC-035: The cartographer agent name matches an existing agents/ file.
    #[test]
    fn agent_name_matches_file() {
        // The agent name "cartographer" must correspond to agents/cartographer.md
        // in the workspace root. We use CARGO_MANIFEST_DIR to find the workspace root.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // manifest_dir = crates/ecc-app, workspace root is two levels up
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root");
        let agent_file = workspace_root.join("agents").join("cartographer.md");
        assert!(
            agent_file.exists(),
            "agents/cartographer.md must exist at {}",
            agent_file.display()
        );
    }
}
