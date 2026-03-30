//! Session lifecycle hooks — start and end.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_date, format_time};
use std::path::Path;

use super::epoch_secs;
use super::helpers::{
    append_project_detection, build_summary_section, count_files_with_ext, extract_session_summary,
    find_files_by_suffix, find_last_updated_line,
};

/// session-start: load previous context, detect project type.
pub fn session_start(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let sessions_dir = home.join(".claude").join("sessions");
    let learned_dir = home.join(".claude").join("learned-skills");

    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }
    if let Err(e) = ports.fs.create_dir_all(&learned_dir) {
        warn!("Cannot create learned-skills dir: {}", e);
    }

    let mut stderr_parts: Vec<String> = Vec::new();

    // Find recent session files
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if !session_files.is_empty() {
        stderr_parts.push(format!(
            "[SessionStart] Found {} recent session(s)",
            session_files.len()
        ));

        // Try to load latest session content
        if let Some(latest) = session_files.first() {
            stderr_parts.push(format!("[SessionStart] Latest: {}", latest.display()));
            if let Ok(content) = ports.fs.read_to_string(latest)
                && !content.contains("[Session context goes here]")
                && !content.trim().is_empty()
            {
                // Output previous session summary to stdout
                let mut out = format!("Previous session summary:\n{}", content);
                if !stdin.is_empty() {
                    out = format!("{}\n{}", out, stdin);
                }
                // Load learned skills count
                let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
                if learned_count > 0 {
                    stderr_parts.push(format!(
                        "[SessionStart] {} learned skill(s) available in {}",
                        learned_count,
                        learned_dir.display()
                    ));
                }

                // Detect project type
                append_project_detection(&mut stderr_parts, ports);

                return HookResult {
                    stdout: out,
                    stderr: format!("{}\n", stderr_parts.join("\n")),
                    exit_code: 0,
                };
            }
        }
    }

    // Load learned skills count
    let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
    if learned_count > 0 {
        stderr_parts.push(format!(
            "[SessionStart] {} learned skill(s) available in {}",
            learned_count,
            learned_dir.display()
        ));
    }

    // Detect project type
    append_project_detection(&mut stderr_parts, ports);

    if stderr_parts.is_empty() {
        return HookResult::passthrough(stdin);
    }

    HookResult {
        stdout: stdin.to_string(),
        stderr: format!("{}\n", stderr_parts.join("\n")),
        exit_code: 0,
    }
}

/// session-end: persist session summary from transcript.
pub fn session_end(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    // Parse transcript_path from stdin JSON
    let transcript_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("transcript_path")?.as_str().map(|s| s.to_string()))
        .or_else(|| ports.env.var("CLAUDE_TRANSCRIPT_PATH"));

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let today = format_date(&datetime_from_epoch(epoch_secs()));
    let short_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .map(|s| if s.len() > 8 { s[..8].to_string() } else { s })
        .unwrap_or_else(|| "unknown".to_string());
    let session_file = sessions_dir.join(format!("{}-{}-session.tmp", today, short_id));
    let current_time = format_time(&datetime_from_epoch(epoch_secs()));

    // Try to extract summary from transcript
    let summary = transcript_path.as_deref().and_then(|tp| {
        let path = Path::new(tp);
        if ports.fs.exists(path) {
            ports
                .fs
                .read_to_string(path)
                .ok()
                .and_then(|content| extract_session_summary(&content))
        } else {
            None
        }
    });

    if ports.fs.exists(&session_file) {
        // Update existing session file
        if let Ok(existing) = ports.fs.read_to_string(&session_file) {
            let mut updated = existing.replace(
                &find_last_updated_line(&existing),
                &format!("**Last Updated:** {}", current_time),
            );

            if let Some(ref summary) = summary {
                let summary_section = build_summary_section(summary);
                // Replace existing summary section
                if let Some(pos) = updated.find("## Session Summary") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else if let Some(pos) = updated.find("## Current State") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else {
                    updated.push_str(&summary_section);
                }
            }

            if let Err(e) = ports.fs.write(&session_file, &updated) {
                let msg = format!("[Warning] Failed to write session: {}", e);
                warn!("{}", msg);
                return HookResult::warn(stdin, &format!("{msg}\n"));
            }
        }
    } else {
        // Create new session file
        let summary_section = if let Some(ref summary) = summary {
            build_summary_section(summary)
        } else {
            "## Current State\n\n[Session context goes here]\n\n\
             ### Completed\n- [ ]\n\n\
             ### In Progress\n- [ ]\n\n\
             ### Notes for Next Session\n-\n\n\
             ### Context to Load\n```\n[relevant files]\n```"
                .to_string()
        };

        let template = format!(
            "# Session: {today}\n\
             **Date:** {today}\n\
             **Started:** {time}\n\
             **Last Updated:** {time}\n\n\
             ---\n\n\
             {summary}\n",
            today = today,
            time = current_time,
            summary = summary_section,
        );

        if let Err(e) = ports.fs.write(&session_file, &template) {
            let msg = format!("[Warning] Failed to write session: {}", e);
            warn!("{}", msg);
            return HookResult::warn(stdin, &format!("{msg}\n"));
        }
    }

    HookResult::passthrough(stdin)
}
