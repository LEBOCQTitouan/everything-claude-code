//! Schema initialisation, migration, and connection helpers.

use std::path::Path;

use ecc_ports::memory_store::MemoryStoreError;
use rusqlite::{Connection, OpenFlags};

/// Open a connection to the SQLite database at `path`.
pub(super) fn open_connection(path: &Path) -> Result<Connection, MemoryStoreError> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .map_err(|e| MemoryStoreError::Database(e.to_string()))
}

/// Apply WAL mode, create the `memories` table, FTS5 virtual table, and triggers.
pub(super) fn apply_schema(conn: &Connection) -> Result<(), MemoryStoreError> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tier TEXT NOT NULL CHECK(tier IN ('working', 'episodic', 'semantic')),
            tags TEXT NOT NULL DEFAULT '',
            project_id TEXT,
            session_id TEXT,
            relevance_score REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            stale INTEGER NOT NULL DEFAULT 0,
            related_work_items TEXT NOT NULL DEFAULT '',
            source_path TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            title, content, tags,
            content='memories',
            content_rowid='id',
            tokenize='unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
            INSERT INTO memories_fts(rowid, title, content, tags)
            VALUES (new.id, new.title, new.content, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
            INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
            VALUES ('delete', old.id, old.title, old.content, old.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
            INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
            VALUES ('delete', old.id, old.title, old.content, old.tags);
            INSERT INTO memories_fts(rowid, title, content, tags)
            VALUES (new.id, new.title, new.content, new.tags);
        END;",
    )
    .map_err(|e| MemoryStoreError::Database(e.to_string()))
}

/// Set file permissions to 0600 on Unix.
pub(super) fn set_file_permissions(path: &Path) -> Result<(), MemoryStoreError> {
    #[cfg(unix)]
    if path.exists() {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
    }
    Ok(())
}

/// Rename the corrupt DB file and recreate an empty database.
pub(super) fn handle_corruption(path: &Path) -> Result<(), MemoryStoreError> {
    let corrupt_path = path.with_extension("db.corrupt");
    std::fs::rename(path, &corrupt_path)
        .map_err(|e| MemoryStoreError::Corruption(e.to_string()))?;
    eprintln!(
        "WARNING: memory.db was corrupted. Backed up to {:?}. Recreating empty database.",
        corrupt_path
    );
    let conn = open_connection(path)?;
    apply_schema(&conn)?;
    set_file_permissions(path)?;
    Ok(())
}

/// Initialise the store: create dirs, check integrity, apply schema, set permissions.
pub(super) fn init(path: &std::path::PathBuf) -> Result<(), MemoryStoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| MemoryStoreError::Database(e.to_string()))?;

        // Set directory permissions to 0700 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(parent, perms)
                .map_err(|e| MemoryStoreError::Database(e.to_string()))?;
        }
    }

    let conn = open_connection(path)?;

    // Check integrity first if DB already exists and has content
    if path.exists() && std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) > 0 {
        let integrity_ok: bool = conn
            .query_row("PRAGMA integrity_check", [], |row| {
                let result: String = row.get(0)?;
                Ok(result == "ok")
            })
            .unwrap_or(false);

        if !integrity_ok {
            drop(conn);
            return handle_corruption(path);
        }
    }

    apply_schema(&conn)?;
    set_file_permissions(path)?;

    Ok(())
}
