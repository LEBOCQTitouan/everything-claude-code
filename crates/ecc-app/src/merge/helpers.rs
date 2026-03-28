use ecc_ports::fs::FileSystem;
use std::path::Path;

pub(super) fn read_json(fs: &dyn FileSystem, path: &Path) -> Result<serde_json::Value, String> {
    let content = fs
        .read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))
}

pub(super) fn read_json_or_default(fs: &dyn FileSystem, path: &Path) -> serde_json::Value {
    read_json(fs, path).unwrap_or_else(|_| serde_json::json!({}))
}

pub(super) fn copy_dir_recursive(
    fs: &dyn FileSystem,
    src: &Path,
    dest: &Path,
) -> Result<(), String> {
    fs.create_dir_all(dest)
        .map_err(|e| format!("Cannot create directory {}: {e}", dest.display()))?;

    let entries = fs
        .read_dir_recursive(src)
        .map_err(|e| format!("Cannot read directory {}: {e}", src.display()))?;

    for entry in entries {
        if let Ok(relative) = entry.strip_prefix(src) {
            let dest_path = dest.join(relative);
            if fs.is_dir(&entry) {
                fs.create_dir_all(&dest_path)
                    .map_err(|e| format!("Cannot create dir: {e}"))?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs.create_dir_all(parent)
                        .map_err(|e| format!("Cannot create dir: {e}"))?;
                }
                fs.copy(&entry, &dest_path)
                    .map_err(|e| format!("Cannot copy {}: {e}", entry.display()))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod tests;
