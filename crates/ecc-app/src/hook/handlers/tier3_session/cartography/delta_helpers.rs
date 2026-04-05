//! Helper functions for cartography hooks — agent invocation, delta collection, and
//! file scanning utilities.
//!
//! These functions were previously used by the `start:cartography` handler for
//! inline delta processing. That processing has moved to the doc-orchestrator
//! pipeline (see `skills/cartography-processing/SKILL.md`). The functions are
//! retained for test coverage of the domain logic.

#![allow(dead_code, unused_imports)]

use std::path::{Path, PathBuf};

use ecc_domain::cartography::{
    build_cross_reference_matrix, derive_slug, infer_element_type_from_path, validate_flow,
    validate_journey, ChangedFile, ElementEntry, ElementType, ProjectType, SessionDelta,
};
use serde::Serialize;
use tracing::warn;

use crate::hook::{HookPorts, HookResult};

/// Enriched context passed to the cartographer agent.
///
/// Includes the delta plus all context the agent needs to perform delta-merge,
/// link to existing flows, and detect external I/O.
#[derive(Debug, Serialize)]
pub(super) struct AgentContext<'a> {
    pub(super) delta: &'a SessionDelta,
    pub(super) existing_journey: Option<String>,
    pub(super) existing_flow: Option<String>,
    pub(super) flow_files: Vec<String>,
    pub(super) external_io_patterns: Vec<String>,
}

/// start:cartography (legacy full-pipeline implementation) — process pending deltas
/// by invoking the cartographer agent.
///
/// - Reads `CLAUDE_PROJECT_DIR` to find the project root.
/// - If no pending deltas exist in `.claude/cartography/`, exits immediately.
/// - Discards uncommitted changes in `docs/cartography/` via `git checkout`.
/// - Creates scaffold directories if missing.
/// - Acquires file lock `.claude/cartography/cartography-merge.lock` (skips if held).
/// - Reads pending deltas, filters already-processed ones, sorts by timestamp ascending.
/// - Invokes cartographer agent for each delta.
/// - On success: archives deltas to `processed/`.
/// - On failure: runs `git reset HEAD docs/cartography/` and logs error.
pub(super) fn process_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "start_cartography", "executing handler");

    let project_dir = match ports.env.var("CLAUDE_PROJECT_DIR") {
        Some(d) => PathBuf::from(d),
        None => {
            warn!("start_cartography: CLAUDE_PROJECT_DIR not set, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    let cartography_dir = project_dir.join(".claude").join("cartography");

    // Step 1: Check for pending deltas — if none, exit immediately (AC-006.5)
    let pending_deltas = collect_pending_deltas(ports, &cartography_dir);
    if pending_deltas.is_empty() {
        tracing::debug!("start_cartography: no pending deltas, passthrough");
        return HookResult::passthrough(stdin);
    }

    // Step 2: Discard uncommitted changes in docs/cartography/ (AC-002.3)
    let docs_cartography = project_dir.join("docs").join("cartography");
    if let Ok(status_out) = ports.shell.run_command_in_dir(
        "git",
        &["status", "--porcelain", "docs/cartography/"],
        &project_dir,
    ) && !status_out.stdout.trim().is_empty()
    {
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["checkout", "--", "docs/cartography/"],
            &project_dir,
        );
    }

    // Step 3: Create scaffold if missing (AC-001.1, AC-001.4)
    let journeys_dir = docs_cartography.join("journeys");
    let flows_dir = docs_cartography.join("flows");
    let elements_dir = docs_cartography.join("elements");
    if !ports.fs.exists(&journeys_dir) {
        let _ = ports.fs.create_dir_all(&journeys_dir);
    }
    if !ports.fs.exists(&flows_dir) {
        let _ = ports.fs.create_dir_all(&flows_dir);
    }
    if !ports.fs.exists(&elements_dir) {
        let _ = ports.fs.create_dir_all(&elements_dir);
    }
    let readme_path = docs_cartography.join("README.md");
    if !ports.fs.exists(&readme_path) {
        let _ = ports.fs.write(
            &readme_path,
            "# Cartography\n\nAuto-generated documentation of user journeys and data flows.\n",
        );
    }
    let elements_readme_path = elements_dir.join("README.md");
    if !ports.fs.exists(&elements_readme_path) {
        let _ = ports.fs.write(
            &elements_readme_path,
            "# Cartography Elements\n\nAuto-generated documentation of system elements.\n",
        );
    }

    // Step 4: Try to acquire file lock (AC-002.4)
    let lock_path = cartography_dir.join("cartography-merge.lock");
    if ports.fs.exists(&lock_path) {
        tracing::debug!("start_cartography: lock held, skipping");
        return HookResult::passthrough(stdin);
    }
    // Write lock file
    if ports.fs.write(&lock_path, "locked").is_err() {
        warn!("start_cartography: cannot acquire lock, skipping");
        return HookResult::passthrough(stdin);
    }

    // Step 5: Filter out already-processed deltas and sort by timestamp (AC-006.6, AC-006.7)
    let processed_dir = cartography_dir.join("processed");
    let mut unprocessed: Vec<(PathBuf, SessionDelta)> = pending_deltas
        .into_iter()
        .filter_map(|path| {
            let content = ports.fs.read_to_string(&path).ok()?;
            let delta: SessionDelta = serde_json::from_str(&content).ok()?;
            // Check if already processed
            let file_name = path.file_name()?.to_str()?.to_string();
            let processed_path = processed_dir.join(&file_name);
            if ports.fs.exists(&processed_path) {
                return None;
            }
            Some((path, delta))
        })
        .collect();

    // Sort by timestamp ascending
    unprocessed.sort_by_key(|(_, delta)| delta.timestamp);

    if unprocessed.is_empty() {
        let _ = ports.fs.remove_file(&lock_path);
        return HookResult::passthrough(stdin);
    }

    // Step 6: Invoke cartographer agent for each delta (AC-006.3, AC-006.4)
    let mut success = true;
    for (_path, delta) in &unprocessed {
        if !invoke_agent_for_delta(ports, delta, &docs_cartography, &flows_dir, &project_dir) {
            success = false;
            break;
        }
    }

    // Step 7: Dispatch element generator if any element targets exist (AC-002.1, AC-002.4)
    if success {
        let has_element_targets = unprocessed.iter().any(|(_, delta)| {
            delta.changed_files.iter().any(|f| {
                let et = infer_element_type_from_path(&f.path);
                !matches!(et, ElementType::Unknown)
            })
        });

        if has_element_targets {
            success =
                invoke_element_generator(ports, &docs_cartography, &elements_dir, &project_dir);
        }
    }

    if success {
        // git add docs/cartography/ && git commit
        let _ = ports
            .shell
            .run_command_in_dir("git", &["add", "docs/cartography/"], &project_dir);
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["commit", "-m", "docs(cartography): update"],
            &project_dir,
        );

        // Archive processed deltas (AC-006.8)
        let _ = ports.fs.create_dir_all(&processed_dir);
        for (path, _) in &unprocessed {
            if let Some(file_name) = path.file_name() {
                let dest = processed_dir.join(file_name);
                let _ = ports.fs.rename(path, &dest);
            }
        }
    } else {
        // On failure: git reset, do not archive (AC-006.3)
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["reset", "HEAD", "docs/cartography/"],
            &project_dir,
        );
        let msg = "[start_cartography] agent failed; changes reset, deltas not archived\n";
        warn!("{}", msg.trim());
        let _ = ports.fs.remove_file(&lock_path);
        return HookResult {
            stdout: stdin.to_string(),
            stderr: msg.to_string(),
            exit_code: 0,
        };
    }

    // Release lock
    let _ = ports.fs.remove_file(&lock_path);

    HookResult::passthrough(stdin)
}

/// Invoke the cartographer agent for a single delta, writing output to the appropriate file.
///
/// Returns `true` on success (agent exited 0), `false` otherwise.
pub(super) fn invoke_agent_for_delta(
    ports: &HookPorts<'_>,
    delta: &SessionDelta,
    docs_cartography: &Path,
    flows_dir: &Path,
    project_dir: &Path,
) -> bool {
    let raw_classification = delta
        .changed_files
        .first()
        .map(|f| f.classification.as_str())
        .unwrap_or("unknown");
    let slug = derive_slug(raw_classification);

    let journey_path = docs_cartography
        .join("journeys")
        .join(format!("{}.md", slug));
    let flow_path = docs_cartography.join("flows").join(format!("{}.md", slug));

    let context = AgentContext {
        delta,
        existing_journey: ports.fs.read_to_string(&journey_path).ok(),
        existing_flow: ports.fs.read_to_string(&flow_path).ok(),
        flow_files: collect_flow_slugs(ports, flows_dir),
        external_io_patterns: detect_external_io_patterns(delta),
    };

    let context_json = match serde_json::to_string(&context) {
        Ok(j) => j,
        Err(_) => return false,
    };

    match ports.shell.run_command_in_dir(
        "claude",
        &["--agent", "cartographer", "--input", &context_json],
        project_dir,
    ) {
        Ok(out) if out.exit_code == 0 => {
            let output = out.stdout.trim();
            if !output.is_empty() {
                let is_journey = validate_journey(output).is_ok();
                let is_flow = validate_flow(output).is_ok();
                if is_journey {
                    let _ = ports.fs.write(&journey_path, output);
                } else if is_flow {
                    let _ = ports.fs.write(&flow_path, output);
                }
            }
            true
        }
        _ => false,
    }
}

/// Invoke the element generator agent and regenerate INDEX.md.
///
/// Returns `true` on success (agent exited 0 and INDEX written), `false` otherwise.
pub(super) fn invoke_element_generator(
    ports: &HookPorts<'_>,
    docs_cartography: &Path,
    elements_dir: &Path,
    project_dir: &Path,
) -> bool {
    let prompt = format!(
        "Generate element documentation for all elements in {}",
        elements_dir.display()
    );

    match ports.shell.run_command_in_dir(
        "claude",
        &[
            "--print",
            "--agent",
            "cartography-element-generator",
            &prompt,
        ],
        project_dir,
    ) {
        Ok(out) if out.exit_code == 0 => {
            // Regenerate INDEX.md using cross-reference matrix (AC-003.1, AC-003.3)
            let element_entries = collect_element_entries(ports, elements_dir);
            let journey_slugs = collect_slugs(ports, &docs_cartography.join("journeys"));
            let flow_slugs = collect_slugs(ports, &docs_cartography.join("flows"));
            let index_content =
                build_cross_reference_matrix(&element_entries, &journey_slugs, &flow_slugs);
            let index_path = elements_dir.join("INDEX.md");
            let _ = ports.fs.write(&index_path, &index_content);
            true
        }
        _ => false,
    }
}

/// Collect element entries from `*.md` files in the elements directory (excluding INDEX.md).
pub(super) fn collect_element_entries(
    ports: &HookPorts<'_>,
    elements_dir: &Path,
) -> Vec<ElementEntry> {
    if !ports.fs.exists(elements_dir) {
        return Vec::new();
    }
    let entries = match ports.fs.read_dir(elements_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            if !name.ends_with(".md") || name == "INDEX.md" || name == "README.md" {
                return None;
            }
            let slug = name[..name.len() - 3].to_string();
            let content = ports.fs.read_to_string(&path).ok()?;
            // Parse participating flows and journeys from markdown content
            let participating_flows =
                extract_links_from_section(&content, "## Participating Flows");
            let participating_journeys =
                extract_links_from_section(&content, "## Participating Journeys");
            Some(ElementEntry {
                slug,
                element_type: ElementType::Unknown,
                purpose: String::new(),
                uses: Vec::new(),
                used_by: Vec::new(),
                participating_flows,
                participating_journeys,
                sources: Vec::new(),
                last_updated: String::new(),
            })
        })
        .collect()
}

/// Extract slug references from a markdown section (lines containing `[slug](path)` links).
pub(super) fn extract_links_from_section(content: &str, section_header: &str) -> Vec<String> {
    let mut in_section = false;
    let mut slugs = Vec::new();
    for line in content.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == section_header;
            continue;
        }
        if in_section {
            // Extract slug from markdown links like [slug-name](../flows/slug-name.md)
            if let Some(start) = line.find('[')
                && let Some(end) = line[start..].find(']')
            {
                let name = &line[start + 1..start + end];
                if !name.is_empty() {
                    slugs.push(name.to_string());
                }
            }
        }
    }
    slugs
}

/// Collect slugs from `*.md` files in a directory (file stem, excluding INDEX.md and README.md).
pub(super) fn collect_slugs(ports: &HookPorts<'_>, dir: &Path) -> Vec<String> {
    if !ports.fs.exists(dir) {
        return Vec::new();
    }
    let entries = match ports.fs.read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .into_iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_string();
            if name.ends_with(".md") && name != "INDEX.md" && name != "README.md" {
                Some(name[..name.len() - 3].to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Collect all `pending-delta-*.json` files from the cartography directory.
pub(super) fn collect_pending_deltas(
    ports: &HookPorts<'_>,
    cartography_dir: &Path,
) -> Vec<PathBuf> {
    if !ports.fs.exists(cartography_dir) {
        return Vec::new();
    }
    let entries = match ports.fs.read_dir(cartography_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .into_iter()
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.starts_with("pending-delta-") && name.ends_with(".json")
        })
        .collect()
}

/// Collect slugs of all existing flow files in the flows directory.
///
/// Returns the stem of each `.md` file (e.g., `ecc-app` for `ecc-app.md`).
pub(super) fn collect_flow_slugs(ports: &HookPorts<'_>, flows_dir: &Path) -> Vec<String> {
    if !ports.fs.exists(flows_dir) {
        return Vec::new();
    }
    let entries = match ports.fs.read_dir(flows_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .into_iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_string();
            if name.ends_with(".md") {
                Some(name[..name.len() - 3].to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Detect external I/O patterns from the changed file paths in a delta.
///
/// Looks for indicators like `http`, `database`, `fs`, `api` in file path components.
pub(super) fn detect_external_io_patterns(delta: &SessionDelta) -> Vec<String> {
    let io_keywords = ["http", "database", "fs", "api"];
    let mut patterns: Vec<String> = io_keywords
        .iter()
        .filter(|&&kw| {
            delta
                .changed_files
                .iter()
                .any(|f| f.path.to_lowercase().contains(kw))
        })
        .map(|&kw| kw.to_string())
        .collect();
    patterns.sort();
    patterns.dedup();
    patterns
}

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
                    let _ = ports.fs.remove_file(&entry);
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
