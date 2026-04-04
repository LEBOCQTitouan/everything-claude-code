//! `campaign` subcommand group: init, append-decision, show.
//!
//! Manages campaign.md files for incremental grill-me decision persistence.

use std::path::Path;

use crate::commands::campaign_io;
use crate::output::WorkflowOutput;

const CAMPAIGN_TEMPLATE: &str = "\
# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
";

/// `campaign init <spec-dir>` — create campaign.md and update state.json.
pub fn run_init(spec_dir: &str, project_dir: &Path) -> WorkflowOutput {
    match init_campaign(spec_dir, project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("campaign init failed: {e}")),
    }
}

fn init_campaign(spec_dir: &str, project_dir: &Path) -> Result<WorkflowOutput, anyhow::Error> {
    let dir = Path::new(spec_dir);
    std::fs::create_dir_all(dir)
        .map_err(|e| anyhow::anyhow!("Failed to create spec dir {spec_dir}: {e}"))?;

    let campaign_path = dir.join("campaign.md");
    if campaign_path.exists() {
        return Ok(WorkflowOutput::pass(format!(
            "Campaign already exists at {}",
            campaign_path.display()
        )));
    }

    std::fs::write(&campaign_path, CAMPAIGN_TEMPLATE).map_err(|e| {
        anyhow::anyhow!("Failed to write campaign.md at {}: {e}", campaign_path.display())
    })?;

    // Update state.json: set campaign_path and bump version to 2
    let _ = crate::io::with_state_lock(project_dir, || {
        let state = crate::io::read_state(project_dir)?;
        if let Some(mut s) = state {
            s.artifacts.campaign_path = Some(campaign_path.to_string_lossy().to_string());
            s.version = 2;
            crate::io::write_state_atomic(project_dir, &s)?;
        }
        Ok::<(), anyhow::Error>(())
    });

    Ok(WorkflowOutput::pass(format!(
        "Campaign created at {}",
        campaign_path.display()
    )))
}

/// `campaign append-decision --question Q --answer A --source S`
pub fn run_append_decision(
    question: &str,
    answer: &str,
    source: &str,
    project_dir: &Path,
) -> WorkflowOutput {
    // Validate source
    if source != "recommended" && source != "user" {
        return WorkflowOutput::block(format!(
            "Invalid source '{source}'. Must be 'recommended' or 'user'."
        ));
    }

    match append_decision(question, answer, source, project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("campaign append-decision failed: {e}")),
    }
}

fn append_decision(
    question: &str,
    answer: &str,
    source: &str,
    project_dir: &Path,
) -> Result<WorkflowOutput, anyhow::Error> {
    let state = crate::io::read_state(project_dir)?
        .ok_or_else(|| anyhow::anyhow!("No workflow state found"))?;
    let campaign_path_str = state
        .artifacts
        .campaign_path
        .ok_or_else(|| anyhow::anyhow!("campaign_path not set in state.json. Run: ecc-workflow campaign init <spec-dir>"))?;
    let campaign_path = Path::new(&campaign_path_str);

    let content = std::fs::read_to_string(campaign_path).map_err(|e| {
        anyhow::anyhow!("Failed to read campaign at {}: {e}", campaign_path.display())
    })?;

    if !content.contains("## Grill-Me Decisions") {
        return Ok(WorkflowOutput::warn(
            "Malformed campaign.md: missing '## Grill-Me Decisions' section".to_string(),
        ));
    }

    let next_n = campaign_io::next_decision_number(&content);
    let escaped_q = campaign_io::escape_table_cell(question);
    let escaped_a = campaign_io::escape_table_cell(answer);
    let new_row = format!("| {next_n} | {escaped_q} | {escaped_a} | {source} |\n");

    // Insert the new row after the last row in the Grill-Me Decisions table
    let new_content = insert_row_in_decisions_table(&content, &new_row);

    campaign_io::atomic_write(campaign_path, &new_content)?;

    Ok(WorkflowOutput::pass(format!(
        "Decision #{next_n} appended to {}",
        campaign_path.display()
    )))
}

/// Insert a row at the end of the `## Grill-Me Decisions` table.
fn insert_row_in_decisions_table(content: &str, row: &str) -> String {
    let mut result = String::with_capacity(content.len() + row.len());
    let mut in_decisions = false;
    let mut last_table_line = false;

    for line in content.lines() {
        if line.contains("## Grill-Me Decisions") {
            in_decisions = true;
        } else if in_decisions && line.starts_with("## ") {
            // Reached next section — insert row before it
            if last_table_line {
                result.push_str(row);
                last_table_line = false;
            }
            in_decisions = false;
        }

        result.push_str(line);
        result.push('\n');

        if in_decisions && line.trim().starts_with('|') {
            last_table_line = true;
        } else if in_decisions && !line.trim().is_empty() {
            last_table_line = false;
        }
    }

    // If decisions table was the last section, append row at end
    if in_decisions && last_table_line {
        result.push_str(row);
    }

    result
}

/// `campaign show` — output campaign.md content as JSON.
pub fn run_show(project_dir: &Path) -> WorkflowOutput {
    match show_campaign(project_dir) {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("campaign show failed: {e}")),
    }
}

fn show_campaign(project_dir: &Path) -> Result<WorkflowOutput, anyhow::Error> {
    let state = crate::io::read_state(project_dir)?
        .ok_or_else(|| anyhow::anyhow!("No workflow state found"))?;
    let campaign_path_str = state
        .artifacts
        .campaign_path
        .ok_or_else(|| anyhow::anyhow!("campaign_path not set"))?;

    let content = std::fs::read_to_string(&campaign_path_str).map_err(|e| {
        anyhow::anyhow!("Failed to read campaign at {campaign_path_str}: {e}")
    })?;

    let decisions = campaign_io::parse_decisions(&content);

    #[derive(serde::Serialize)]
    struct CampaignJson {
        decisions: Vec<campaign_io::Decision>,
        artifacts: Vec<String>,
        agent_outputs: Vec<String>,
        commit_trail: Vec<String>,
    }

    let json = serde_json::to_string(&CampaignJson {
        decisions,
        artifacts: vec![],
        agent_outputs: vec![],
        commit_trail: vec![],
    })
    .map_err(|e| anyhow::anyhow!("Failed to serialize campaign: {e}"))?;

    Ok(WorkflowOutput::pass(json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_state(dir: &std::path::Path, campaign_path: Option<&str>, version: u32) {
        std::fs::create_dir_all(dir).unwrap();
        let cp = campaign_path
            .map(|p| format!(r#""{p}""#))
            .unwrap_or_else(|| "null".to_string());
        let json = format!(
            r#"{{"phase":"plan","concern":"dev","feature":"test","started_at":"2026-01-01T00:00:00Z","toolchain":{{"test":null,"lint":null,"build":null}},"artifacts":{{"plan":null,"solution":null,"implement":null,"campaign_path":{cp},"spec_path":null,"design_path":null,"tasks_path":null}},"completed":[],"version":{version}}}"#
        );
        std::fs::write(dir.join("state.json"), json).unwrap();
    }

    fn make_state_dir(tmp: &TempDir) -> std::path::PathBuf {
        let sd = tmp.path().join("state");
        std::fs::create_dir_all(&sd).unwrap();
        sd
    }

    #[test]
    fn init_creates_campaign_md() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        setup_state(&sd, None, 1);
        let spec_dir = tmp.path().join("specs/test");
        let out = run_init(spec_dir.to_str().unwrap(), &sd);
        assert!(matches!(out.status, crate::output::Status::Pass), "got: {:?} {}", out.status, out.message);
        assert!(spec_dir.join("campaign.md").exists());
        let c = std::fs::read_to_string(spec_dir.join("campaign.md")).unwrap();
        assert!(c.contains("## Grill-Me Decisions"));
    }

    #[test]
    fn init_sets_campaign_path_and_version() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        setup_state(&sd, None, 1);
        let spec_dir = tmp.path().join("specs/test");
        run_init(spec_dir.to_str().unwrap(), &sd);
        let state = crate::io::read_state(&sd).unwrap().unwrap();
        assert!(state.artifacts.campaign_path.is_some());
        assert_eq!(state.version, 2);
    }

    #[test]
    fn init_idempotent() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        setup_state(&sd, None, 1);
        let spec_dir = tmp.path().join("specs/test");
        let out1 = run_init(spec_dir.to_str().unwrap(), &sd);
        let out2 = run_init(spec_dir.to_str().unwrap(), &sd);
        assert!(matches!(out1.status, crate::output::Status::Pass));
        assert!(matches!(out2.status, crate::output::Status::Pass));
        assert!(out2.message.contains("already exists"));
    }

    #[test]
    fn init_creates_missing_dir() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        setup_state(&sd, None, 1);
        let spec_dir = tmp.path().join("deep/nested/specs");
        let out = run_init(spec_dir.to_str().unwrap(), &sd);
        assert!(matches!(out.status, crate::output::Status::Pass));
        assert!(spec_dir.join("campaign.md").exists());
    }

    #[test]
    fn append_adds_numbered_row() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, CAMPAIGN_TEMPLATE).unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        let out = run_append_decision("Q1", "A1", "recommended", &sd);
        assert!(matches!(out.status, crate::output::Status::Pass), "got: {:?} {}", out.status, out.message);
        let c = std::fs::read_to_string(&campaign).unwrap();
        assert!(c.contains("| 1 | Q1 | A1 | recommended |"), "content: {c}");
    }

    #[test]
    fn append_auto_numbers() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, CAMPAIGN_TEMPLATE).unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        run_append_decision("Q1", "A1", "recommended", &sd);
        run_append_decision("Q2", "A2", "user", &sd);
        run_append_decision("Q3", "A3", "recommended", &sd);
        let c = std::fs::read_to_string(&campaign).unwrap();
        assert!(c.contains("| 1 |") && c.contains("| 2 |") && c.contains("| 3 |"), "content: {c}");
    }

    #[test]
    fn append_escapes_pipes_and_newlines() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, CAMPAIGN_TEMPLATE).unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        run_append_decision("Q with | pipe", "A with\nnewline", "user", &sd);
        let c = std::fs::read_to_string(&campaign).unwrap();
        assert!(c.contains(r"Q with \| pipe"), "content: {c}");
        assert!(c.contains("A with<br>newline"), "content: {c}");
    }

    #[test]
    fn append_invalid_source_blocks() {
        let tmp = TempDir::new().unwrap();
        let out = run_append_decision("Q", "A", "invalid", tmp.path());
        assert!(matches!(out.status, crate::output::Status::Block));
        assert!(out.message.contains("Invalid source"));
    }

    #[test]
    fn append_malformed_warns() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, "# No decisions section").unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        let out = run_append_decision("Q", "A", "recommended", &sd);
        assert!(matches!(out.status, crate::output::Status::Warn));
        assert!(out.message.contains("Malformed"));
    }

    #[test]
    fn show_returns_json() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, CAMPAIGN_TEMPLATE).unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        let out = run_show(&sd);
        assert!(matches!(out.status, crate::output::Status::Pass));
        let v: serde_json::Value = serde_json::from_str(&out.message).unwrap();
        assert!(v["decisions"].is_array());
    }

    #[test]
    fn show_returns_all_decisions() {
        let tmp = TempDir::new().unwrap();
        let sd = make_state_dir(&tmp);
        let campaign = tmp.path().join("campaign.md");
        std::fs::write(&campaign, CAMPAIGN_TEMPLATE).unwrap();
        setup_state(&sd, Some(campaign.to_str().unwrap()), 2);
        run_append_decision("Q1", "A1", "recommended", &sd);
        run_append_decision("Q2", "A2", "user", &sd);
        let out = run_show(&sd);
        let v: serde_json::Value = serde_json::from_str(&out.message).unwrap();
        assert_eq!(v["decisions"].as_array().unwrap().len(), 2);
    }
}
