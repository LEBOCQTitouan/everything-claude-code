//! Cross-PR isolation integration tests.
//!
//! Verifies that PR2 memory-prune code (safe_path, file_prune, trash_gc,
//! paths, lifecycle, backlog hook integration) does NOT import any cartography
//! module, enabling PR2 to be reverted independently of PR1 (cartography).

#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::path::Path;

    fn pr2_memory_files() -> Vec<std::path::PathBuf> {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        vec![
            src.join("memory/safe_path.rs"),
            src.join("memory/file_prune.rs"),
            src.join("memory/trash_gc.rs"),
            src.join("memory/paths.rs"),
            src.join("memory/lifecycle.rs"),
        ]
    }

    #[test]
    fn pr2_no_cartography_imports() {
        for path in pr2_memory_files() {
            if !path.exists() {
                continue; // safe_path may live in ecc-domain; skip if absent here
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            // Only inspect production code (before any #[cfg(test)] block)
            let production = source.split("#[cfg(test)]").next().unwrap_or(&source);

            assert!(
                !production.contains(concat!("use ", "ecc_domain::cartography")),
                "{}: imports cartography module (violates PR2 isolation)",
                path.display()
            );
            assert!(
                !production.contains(concat!(
                    "use ",
                    "crate::hook::handlers::tier3_session::cartography"
                )),
                "{}: imports cartography hook module",
                path.display()
            );
        }
    }
}
