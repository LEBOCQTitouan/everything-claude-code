use ecc_domain::config::manifest::{EccManifest, MANIFEST_FILENAME};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Read an existing manifest from a directory via the FileSystem port.
/// Returns None if not found or corrupted.
pub fn read_manifest(fs: &dyn FileSystem, dir: &Path) -> Option<EccManifest> {
    let manifest_path = dir.join(MANIFEST_FILENAME);
    let content = fs.read_to_string(&manifest_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    if parsed.get("version").is_none() || parsed.get("artifacts").is_none() {
        return None;
    }
    serde_json::from_value(parsed).ok()
}

/// Write a manifest to a directory via the FileSystem port.
pub fn write_manifest(
    fs: &dyn FileSystem,
    dir: &Path,
    manifest: &EccManifest,
) -> Result<(), ecc_ports::fs::FsError> {
    fs.create_dir_all(dir)?;
    let manifest_path = dir.join(MANIFEST_FILENAME);
    let json = serde_json::to_string_pretty(manifest)
        .expect("manifest serialization should not fail");
    fs.write(&manifest_path, &format!("{json}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::manifest::{create_manifest, Artifacts};
    use ecc_test_support::InMemoryFileSystem;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn sample_artifacts() -> Artifacts {
        Artifacts {
            agents: vec!["agent1.md".into(), "agent2.md".into()],
            commands: vec!["cmd1.md".into()],
            skills: vec!["skill1".into()],
            rules: {
                let mut m = BTreeMap::new();
                m.insert("common".into(), vec!["rule1.md".into()]);
                m
            },
            hook_descriptions: vec!["hook1".into()],
        }
    }

    #[test]
    fn read_manifest_not_found() {
        let fs = InMemoryFileSystem::new();
        assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
    }

    #[test]
    fn read_manifest_invalid_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.claude/.ecc-manifest.json", "not json");
        assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
    }

    #[test]
    fn read_manifest_missing_version() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.claude/.ecc-manifest.json",
            r#"{"artifacts": {}}"#,
        );
        assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
    }

    #[test]
    fn write_and_read_manifest_roundtrip() {
        let fs = InMemoryFileSystem::new();
        let dir = Path::new("/project/.claude");
        let original = create_manifest(
            "4.0.0",
            "2026-03-14T00:00:00Z",
            &["rust".into()],
            sample_artifacts(),
        );
        write_manifest(&fs, dir, &original).unwrap();
        let read_back = read_manifest(&fs, dir).unwrap();
        assert_eq!(read_back, original);
    }

    #[test]
    fn json_uses_camel_case() {
        let m = create_manifest("4.0.0", "now", &[], Artifacts::default());
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("installedAt"));
        assert!(json.contains("updatedAt"));
        assert!(json.contains("hookDescriptions"));
        assert!(!json.contains("installed_at"));
    }
}
