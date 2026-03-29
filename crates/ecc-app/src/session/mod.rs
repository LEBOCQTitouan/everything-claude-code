//! Session use cases — I/O operations for session management.
//!
//! Pure types and functions live in `ecc-domain::session::manager`.
//! This module provides the I/O layer that depends on `FileSystem`.

pub mod aliases;

use ecc_domain::session::manager::{
    GetAllSessionsOptions, SessionDetail, SessionFilename, SessionListItem, SessionListResult,
    get_session_stats, parse_session_filename, parse_session_metadata,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

// ---------------------------------------------------------------------------
// I/O functions (depend on FileSystem port)
// ---------------------------------------------------------------------------

/// Check whether a session entry matches the given filter options.
fn entry_matches_filters(
    fname: &str,
    options: &GetAllSessionsOptions,
) -> Option<SessionFilename> {
    if !fname.ends_with(".tmp") {
        return None;
    }
    let parsed = parse_session_filename(fname)?;
    if let Some(ref date_filter) = options.date
        && parsed.date != *date_filter
    {
        return None;
    }
    if let Some(ref search_filter) = options.search
        && !parsed.short_id.contains(search_filter.as_str())
    {
        return None;
    }
    Some(parsed)
}

/// Paginate a sorted list of sessions, returning a `SessionListResult`.
fn paginate_sessions(sessions: Vec<SessionListItem>, options: &GetAllSessionsOptions) -> SessionListResult {
    let total = sessions.len();
    let offset = options.offset;
    let limit = options.limit.max(1);
    let paginated: Vec<SessionListItem> = sessions.into_iter().skip(offset).take(limit).collect();
    let has_more = offset + limit < total;
    SessionListResult { sessions: paginated, total, offset, limit, has_more }
}

/// List sessions in a directory with optional filtering and pagination.
pub fn get_all_sessions(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    options: &GetAllSessionsOptions,
) -> SessionListResult {
    let empty = SessionListResult {
        sessions: vec![],
        total: 0,
        offset: options.offset,
        limit: options.limit,
        has_more: false,
    };

    if !fs.exists(sessions_dir) {
        return empty;
    }

    let entries = match fs.read_dir(sessions_dir) {
        Ok(e) => e,
        Err(_) => return empty,
    };

    let mut sessions: Vec<SessionListItem> = Vec::new();
    for entry in &entries {
        let Some(fname) = entry.file_name().and_then(|n| n.to_str().map(String::from)) else {
            continue;
        };
        let Some(parsed) = entry_matches_filters(&fname, options) else {
            continue;
        };
        let size = fs.read_to_string(entry).map(|c| c.len()).unwrap_or(0);
        sessions.push(SessionListItem {
            filename: fname,
            short_id: parsed.short_id,
            date: parsed.date,
            session_path: entry.clone(),
            has_content: size > 0,
            size,
        });
    }

    sessions.sort_by(|a, b| b.filename.cmp(&a.filename));
    paginate_sessions(sessions, options)
}

/// Find a session by ID (short ID prefix, filename, or filename without `.tmp`).
pub fn get_session_by_id(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    id: &str,
    include_content: bool,
) -> Option<SessionDetail> {
    let entries = fs.read_dir(sessions_dir).ok()?;

    for entry in &entries {
        let fname = entry.file_name()?.to_str()?.to_string();

        if !fname.ends_with(".tmp") {
            continue;
        }

        let parsed = parse_session_filename(&fname)?;

        let short_id_match =
            !id.is_empty() && parsed.short_id != "no-id" && parsed.short_id.starts_with(id);
        let filename_match = fname == id || fname == format!("{id}.tmp");
        let no_id_match = parsed.short_id == "no-id" && fname == format!("{id}-session.tmp");

        if !short_id_match && !filename_match && !no_id_match {
            continue;
        }

        let session_path = sessions_dir.join(&fname);
        let content_str = fs.read_to_string(&session_path).ok();
        let size = content_str.as_ref().map_or(0, String::len);

        let (content, metadata, stats) = if include_content {
            let meta = parse_session_metadata(content_str.as_deref());
            let st = get_session_stats(content_str.as_deref().unwrap_or(""));
            (content_str, Some(meta), Some(st))
        } else {
            (None, None, None)
        };

        return Some(SessionDetail {
            filename: fname,
            short_id: parsed.short_id,
            date: parsed.date,
            session_path,
            size,
            content,
            metadata,
            stats,
        });
    }

    None
}

/// Write content to a session file.
pub fn write_session_content(
    fs: &dyn FileSystem,
    path: &Path,
    content: &str,
) -> Result<(), ecc_ports::fs::FsError> {
    fs.write(path, content)
}

/// Delete a session file. Returns `Ok(true)` if deleted, `Ok(false)` if not found.
pub fn delete_session(fs: &dyn FileSystem, path: &Path) -> Result<bool, ecc_ports::fs::FsError> {
    if !fs.exists(path) {
        return Ok(false);
    }
    fs.remove_file(path)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
