//! Helper functions for cartography hooks — corrupt-delta cleanup utility.
//!
//! The legacy pipeline (agent invocation, delta collection)
//! has moved to the doc-orchestrator pipeline
//! (see `skills/cartography-processing/SKILL.md`).
//! Only the `clean_corrupt_deltas` helper is retained here for use by `delta_writer`.

use std::path::Path;

use ecc_domain::cartography::SessionDelta;
use tracing::warn;

use crate::hook::HookPorts;

/// Delete any delta files in the cartography dir that contain invalid JSON.
pub(super) fn clean_corrupt_deltas(ports: &HookPorts<'_>, cartography_dir: &Path) {
    let entries = match ports.fs.read_dir(cartography_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries {
        let name = entry
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !name.starts_with("pending-delta-") || !name.ends_with(".json") {
            continue;
        }

        match ports.fs.read_to_string(&entry) {
            Ok(content) => {
                if serde_json::from_str::<SessionDelta>(&content).is_err() {
                    warn!(
                        "stop_cartography: deleting corrupt delta file: {}",
                        entry.display()
                    );
                    if let Err(e) = ports.fs.remove_file(&entry) {
                        tracing::warn!(
                            target: "cartography::io",
                            operation = "remove_file",
                            path = %entry.display(),
                            error = %e
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    "stop_cartography: cannot read delta file {}: {}",
                    entry.display(),
                    e
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_helpers.rs"]
mod tests;
