use ecc_ports::extract::{ExtractError, TarballExtractor};
use flate2::read::GzDecoder;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Production tarball extractor using flate2 + tar crates.
///
/// Extracts `.tar.gz` archives while guarding against zip-slip path traversal.
/// Any entry whose resolved path does not start with the canonical destination
/// directory is rejected with [`ExtractError::ZipSlip`].
pub struct FlateExtractor;

impl FlateExtractor {
    /// Create a new `FlateExtractor`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlateExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TarballExtractor for FlateExtractor {
    fn extract(&self, tarball: &Path, dest: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        let file = std::fs::File::open(tarball).map_err(|e| {
            ExtractError::Io(format!("cannot open tarball {}: {e}", tarball.display()))
        })?;

        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        // Canonicalize dest so we can do lexical prefix checks.
        // dest may not yet exist, so we canonicalize the parent and append the stem.
        let dest_canonical = canonical_or_create(dest).map_err(|e| {
            ExtractError::Io(format!("cannot resolve dest {}: {e}", dest.display()))
        })?;

        let entries = archive.entries().map_err(|e| {
            ExtractError::CorruptArchive(format!("cannot read archive entries: {e}"))
        })?;

        let mut extracted: Vec<PathBuf> = Vec::new();

        for entry in entries {
            let mut entry = entry.map_err(|e| {
                ExtractError::CorruptArchive(format!("corrupt entry in archive: {e}"))
            })?;

            let entry_path = entry
                .path()
                .map_err(|e| ExtractError::CorruptArchive(format!("invalid entry path: {e}")))?;

            // Strip any leading `/` or `..` components for lexical check.
            let stripped = strip_leading_root(&entry_path);

            // Build candidate output path.
            let output_path = dest_canonical.join(&stripped);

            // Zip-slip guard: normalise the output path and verify it stays
            // within dest_canonical.
            let normalised = normalise_path(&output_path);
            if !normalised.starts_with(&dest_canonical) {
                return Err(ExtractError::ZipSlip(format!(
                    "entry '{}' would escape destination directory",
                    entry_path.display()
                )));
            }

            // Create parent directories as needed.
            if let Some(parent) = normalised.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ExtractError::Io(format!("failed to create {}: {e}", parent.display()))
                })?;
            }

            // Unpack the entry.
            entry.unpack(&normalised).map_err(|e| {
                ExtractError::Io(format!("failed to unpack {}: {e}", normalised.display()))
            })?;

            extracted.push(normalised);
        }

        Ok(extracted)
    }
}

/// Canonicalise `path` if it exists, otherwise create it and canonicalise.
fn canonical_or_create(path: &Path) -> std::io::Result<PathBuf> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    path.canonicalize()
}

/// Strip a leading root component (`/`) from a path so it can be safely
/// joined onto the destination.
fn strip_leading_root(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    // Skip the root component (e.g. `/`) if present.
    if let Some(std::path::Component::RootDir) = components.peek() {
        components.next();
    }
    components.collect()
}

/// Lexically normalise a path by resolving `.` and `..` without touching
/// the filesystem. Returns the canonicalised path as best we can without
/// `std::fs::canonicalize` (which requires the path to exist).
fn normalise_path(path: &Path) -> PathBuf {
    let mut out: Vec<std::path::Component> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {
                // `.` — skip
            }
            std::path::Component::ParentDir => {
                // `..` — pop last non-root component
                match out.last() {
                    Some(std::path::Component::RootDir) | Some(std::path::Component::Prefix(_)) => {
                        // Cannot go above root — leave as-is (zip-slip guard will catch it)
                    }
                    Some(_) => {
                        out.pop();
                    }
                    None => {
                        out.push(component);
                    }
                }
            }
            c => out.push(c),
        }
    }
    out.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_valid_tar_gz(dir: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        use flate2::{Compression, write::GzEncoder};
        use tar::Builder;

        let tarball_path = dir.join("test.tar.gz");
        let file = std::fs::File::create(&tarball_path).unwrap();
        let gz = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(gz);

        for (name, content) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *content).unwrap();
        }

        builder.finish().unwrap();
        tarball_path
    }

    fn create_zip_slip_tar_gz(dir: &Path) -> PathBuf {
        use flate2::{Compression, write::GzEncoder};

        let tarball_path = dir.join("zip-slip.tar.gz");
        let file = std::fs::File::create(&tarball_path).unwrap();
        let gz = GzEncoder::new(file, Compression::default());

        // Craft a raw tar entry with a path traversal in the name field.
        // The tar format: 100-byte name field followed by other fields.
        // We write it directly as raw bytes to bypass the library's validation.
        let content = b"root:x:0:0:root:/root:/bin/bash";
        let mut tar_bytes: Vec<u8> = vec![0u8; 512]; // one tar header block

        // name field (bytes 0..100): fill with path traversal
        let malicious_path = b"../../../evil-file";
        let path_len = malicious_path.len().min(99);
        tar_bytes[..path_len].copy_from_slice(&malicious_path[..path_len]);

        // mode field (bytes 100..108): "0000644\0"
        tar_bytes[100..107].copy_from_slice(b"0000644");
        tar_bytes[107] = 0;

        // uid (108..116)
        tar_bytes[108..115].copy_from_slice(b"0000000");

        // gid (116..124)
        tar_bytes[116..123].copy_from_slice(b"0000000");

        // size (124..136): content length in octal
        let size_str = format!("{:011o}\0", content.len());
        tar_bytes[124..136].copy_from_slice(size_str.as_bytes());

        // mtime (136..148)
        tar_bytes[136..147].copy_from_slice(b"00000000000");
        tar_bytes[147] = 0;

        // typeflag (156): '0' = regular file
        tar_bytes[156] = b'0';

        // magic (257..265): "ustar  \0" (GNU)
        tar_bytes[257..265].copy_from_slice(b"ustar  \0");

        // Compute checksum (bytes 148..156 = 8 spaces during computation)
        for b in tar_bytes[148..156].iter_mut() {
            *b = b' ';
        }
        let checksum: u32 = tar_bytes.iter().map(|&b| b as u32).sum();
        let chk_str = format!("{:06o}\0 ", checksum);
        tar_bytes[148..156].copy_from_slice(chk_str.as_bytes());

        // Write the tar header + content + padding, then two zero blocks (end-of-archive)
        let mut all_bytes: Vec<u8> = Vec::new();
        all_bytes.extend_from_slice(&tar_bytes); // header block
        all_bytes.extend_from_slice(content); // content
        // Pad to 512-byte boundary
        let padding = 512 - (content.len() % 512);
        if padding < 512 {
            all_bytes.extend(std::iter::repeat_n(0u8, padding));
        }
        // Two end-of-archive blocks
        all_bytes.extend(std::iter::repeat_n(0u8, 1024));

        let mut gz_writer = gz;
        std::io::Write::write_all(&mut gz_writer, &all_bytes).unwrap();
        gz_writer.finish().unwrap();

        tarball_path
    }

    /// PC-021: Zip-slip path traversal rejected during extraction
    #[test]
    fn zip_slip_prevention() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("extracted");
        std::fs::create_dir_all(&dest).unwrap();

        let tarball = create_zip_slip_tar_gz(tmp.path());
        let extractor = FlateExtractor::new();
        let result = extractor.extract(&tarball, &dest);

        assert!(result.is_err(), "zip-slip entry must be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExtractError::ZipSlip(_)),
            "expected ZipSlip error, got: {err:?}"
        );
    }

    #[test]
    fn valid_archive_extracts_successfully() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("extracted");
        std::fs::create_dir_all(&dest).unwrap();

        let entries = &[
            ("bin/ecc", b"ecc-binary".as_slice()),
            ("bin/ecc-workflow", b"ecc-workflow-binary".as_slice()),
        ];
        let tarball = create_valid_tar_gz(tmp.path(), entries);

        let extractor = FlateExtractor::new();
        let result = extractor.extract(&tarball, &dest);
        assert!(result.is_ok(), "valid archive should extract: {result:?}");
        let paths = result.unwrap();
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("ecc")));
    }

    // ── PC-032: extract_valid_tarball ─────────────────────────────────────

    #[test]
    fn extract_valid_tarball() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("extracted");
        std::fs::create_dir_all(&dest).unwrap();

        let entries: &[(&str, &[u8])] = &[
            ("bin/ecc", b"ecc-binary-content"),
            ("bin/ecc-workflow", b"ecc-workflow-binary-content"),
        ];
        let tarball = create_valid_tar_gz(tmp.path(), entries);

        let extractor = FlateExtractor::new();
        let result = extractor.extract(&tarball, &dest);
        assert!(result.is_ok(), "valid tarball must extract: {result:?}");

        let paths = result.unwrap();
        assert_eq!(paths.len(), 2, "should extract exactly 2 files");

        // Both ecc and ecc-workflow must be present
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(
            names.contains(&"ecc".to_string()),
            "ecc binary must be extracted"
        );
        assert!(
            names.contains(&"ecc-workflow".to_string()),
            "ecc-workflow binary must be extracted"
        );

        // Content must be correct
        let ecc_path = paths
            .iter()
            .find(|p| p.file_name().unwrap() == "ecc")
            .unwrap();
        let content = std::fs::read(ecc_path).unwrap();
        assert_eq!(content, b"ecc-binary-content");
    }

    // ── PC-033: windows_exe_preserved ────────────────────────────────────

    #[test]
    fn windows_exe_preserved() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("extracted");
        std::fs::create_dir_all(&dest).unwrap();

        // Tarball with Windows .exe entries
        let entries: &[(&str, &[u8])] = &[
            ("bin/ecc.exe", b"ecc-windows-binary"),
            ("bin/ecc-workflow.exe", b"ecc-workflow-windows-binary"),
        ];
        let tarball = create_valid_tar_gz(tmp.path(), entries);

        let extractor = FlateExtractor::new();
        let result = extractor.extract(&tarball, &dest);
        assert!(result.is_ok(), "windows tarball must extract: {result:?}");

        let paths = result.unwrap();
        let names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        // .exe extensions must be preserved
        assert!(
            names.contains(&"ecc.exe".to_string()),
            "ecc.exe must be extracted with .exe extension, got: {names:?}"
        );
        assert!(
            names.contains(&"ecc-workflow.exe".to_string()),
            "ecc-workflow.exe must be extracted with .exe extension, got: {names:?}"
        );
    }

    #[test]
    fn corrupt_archive_returns_error() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("extracted");
        std::fs::create_dir_all(&dest).unwrap();

        // Write garbage as the tarball
        let tarball_path = tmp.path().join("corrupt.tar.gz");
        let mut f = std::fs::File::create(&tarball_path).unwrap();
        f.write_all(b"this is not a valid gzip file").unwrap();

        let extractor = FlateExtractor::new();
        let result = extractor.extract(&tarball_path, &dest);
        assert!(result.is_err());
        // Should be CorruptArchive or Io error
        assert!(
            matches!(
                result.unwrap_err(),
                ExtractError::CorruptArchive(_) | ExtractError::Io(_)
            ),
            "corrupt archive should produce CorruptArchive or Io error"
        );
    }
}
