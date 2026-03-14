use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// ECC manifest tracking installed artifacts.
/// Stored at `~/.claude/.ecc-manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    pub installed_at: String,
    #[serde(default)]
    pub files: BTreeMap<String, FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub hash: String,
    pub source: FileSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileSource {
    Ecc,
    User,
    Merged,
}

impl Manifest {
    pub fn new(version: &str, installed_at: &str) -> Self {
        Self {
            version: version.to_string(),
            installed_at: installed_at.to_string(),
            files: BTreeMap::new(),
        }
    }

    pub fn add_file(&self, path: &str, hash: &str, source: FileSource) -> Self {
        let mut files = self.files.clone();
        files.insert(
            path.to_string(),
            FileEntry {
                hash: hash.to_string(),
                source,
            },
        );
        Self {
            version: self.version.clone(),
            installed_at: self.installed_at.clone(),
            files,
        }
    }

    pub fn remove_file(&self, path: &str) -> Self {
        let mut files = self.files.clone();
        files.remove(path);
        Self {
            version: self.version.clone(),
            installed_at: self.installed_at.clone(),
            files,
        }
    }

    pub fn get_file(&self, path: &str) -> Option<&FileEntry> {
        self.files.get(path)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manifest_is_empty() {
        let m = Manifest::new("4.0.0", "2026-03-14");
        assert_eq!(m.file_count(), 0);
        assert_eq!(m.version, "4.0.0");
    }

    #[test]
    fn add_file_returns_new_manifest() {
        let m = Manifest::new("4.0.0", "2026-03-14");
        let m2 = m.add_file("agents/foo.md", "abc123", FileSource::Ecc);
        assert_eq!(m.file_count(), 0); // original unchanged
        assert_eq!(m2.file_count(), 1);
    }

    #[test]
    fn remove_file_returns_new_manifest() {
        let m = Manifest::new("4.0.0", "2026-03-14")
            .add_file("a.md", "hash1", FileSource::Ecc)
            .add_file("b.md", "hash2", FileSource::User);
        let m2 = m.remove_file("a.md");
        assert_eq!(m.file_count(), 2);
        assert_eq!(m2.file_count(), 1);
        assert!(m2.get_file("a.md").is_none());
    }

    #[test]
    fn roundtrip_json() {
        let m = Manifest::new("4.0.0", "2026-03-14")
            .add_file("agents/foo.md", "abc", FileSource::Ecc);
        let json = serde_json::to_string_pretty(&m).unwrap();
        let m2: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }
}
