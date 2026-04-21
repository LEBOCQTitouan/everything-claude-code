use super::error::ConfigAppError;
use ecc_domain::config::merge::{FileToReview, contents_differ};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Pre-scan a directory to identify files that need review (new or changed).
/// Returns `(files_to_review, unchanged_filenames, errors)`.
/// Errors are accumulated rather than causing early return — callers should propagate them.
pub fn pre_scan_directory(
    fs: &dyn FileSystem,
    src_dir: &Path,
    dest_dir: &Path,
    ext: &str,
) -> (Vec<FileToReview>, Vec<String>, Vec<String>) {
    let mut files_to_review = Vec::new();
    let mut unchanged = Vec::new();
    let mut errors = Vec::new();

    let entries = match fs.read_dir(src_dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("pre_scan_directory: cannot read {}: {e}", src_dir.display());
            errors.push(format!(
                "pre_scan_directory: cannot read {}: {e}",
                src_dir.display()
            ));
            return (files_to_review, unchanged, errors);
        }
    };

    let src_files: Vec<String> = entries
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            if name.ends_with(ext) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    for filename in src_files {
        let src_path = src_dir.join(&filename);
        let dest_path = dest_dir.join(&filename);

        if !fs.exists(&dest_path) {
            files_to_review.push(FileToReview {
                filename,
                src_path,
                dest_path,
                is_new: true,
            });
        } else {
            let src_content = match fs.read_to_string(&src_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        "pre_scan_directory: cannot read {}: {e}",
                        src_path.display()
                    );
                    errors.push(format!(
                        "pre_scan_directory: cannot read {}: {e}",
                        src_path.display()
                    ));
                    continue;
                }
            };
            let dest_content = fs.read_to_string(&dest_path).unwrap_or_default();

            if contents_differ(&src_content, &dest_content) {
                files_to_review.push(FileToReview {
                    filename,
                    src_path,
                    dest_path,
                    is_new: false,
                });
            } else {
                unchanged.push(filename);
            }
        }
    }

    (files_to_review, unchanged, errors)
}

/// Copy a file from source to destination.
/// In dry-run mode, the copy is skipped.
pub fn apply_accept(
    fs: &dyn FileSystem,
    src_path: &Path,
    dest_path: &Path,
    dry_run: bool,
) -> Result<(), ConfigAppError> {
    if dry_run {
        return Ok(());
    }

    if let Some(parent) = dest_path.parent() {
        fs.create_dir_all(parent)
            .map_err(|e| ConfigAppError::CreateDir {
                path: parent.display().to_string(),
                reason: e.to_string(),
            })?;
    }

    fs.copy(src_path, dest_path)
        .map_err(|e| ConfigAppError::CopyFile {
            src: src_path.display().to_string(),
            dest: dest_path.display().to_string(),
            reason: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::fs::FsError;
    use ecc_test_support::InMemoryFileSystem;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    /// A filesystem wrapper that injects failures for specific paths.
    struct FailingFs {
        inner: InMemoryFileSystem,
        fail_read_dir: HashSet<PathBuf>,
        fail_read_to_string: HashSet<PathBuf>,
    }

    impl FailingFs {
        fn new(inner: InMemoryFileSystem) -> Self {
            Self {
                inner,
                fail_read_dir: HashSet::new(),
                fail_read_to_string: HashSet::new(),
            }
        }

        fn with_fail_read_dir(mut self, path: impl Into<PathBuf>) -> Self {
            self.fail_read_dir.insert(path.into());
            self
        }

        fn with_fail_read_to_string(mut self, path: impl Into<PathBuf>) -> Self {
            self.fail_read_to_string.insert(path.into());
            self
        }
    }

    impl ecc_ports::fs::FileSystem for FailingFs {
        fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
            if self.fail_read_to_string.contains(path) {
                return Err(FsError::PermissionDenied(path.to_path_buf()));
            }
            self.inner.read_to_string(path)
        }

        fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
            self.inner.read_bytes(path)
        }

        fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
            self.inner.write(path, content)
        }

        fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<(), FsError> {
            self.inner.write_bytes(path, content)
        }

        fn exists(&self, path: &Path) -> bool {
            self.inner.exists(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.inner.is_dir(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.inner.is_file(path)
        }

        fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
            self.inner.create_dir_all(path)
        }

        fn remove_file(&self, path: &Path) -> Result<(), FsError> {
            self.inner.remove_file(path)
        }

        fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
            self.inner.remove_dir_all(path)
        }

        fn copy(&self, from: &Path, to: &Path) -> Result<(), FsError> {
            self.inner.copy(from, to)
        }

        fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
            if self.fail_read_dir.contains(path) {
                return Err(FsError::PermissionDenied(path.to_path_buf()));
            }
            self.inner.read_dir(path)
        }

        fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
            self.inner.read_dir_recursive(path)
        }

        fn create_symlink(&self, target: &Path, link: &Path) -> Result<(), FsError> {
            self.inner.create_symlink(target, link)
        }

        fn read_symlink(&self, link: &Path) -> Result<PathBuf, FsError> {
            self.inner.read_symlink(link)
        }

        fn is_symlink(&self, path: &Path) -> bool {
            self.inner.is_symlink(path)
        }

        fn set_permissions(&self, path: &Path, mode: u32) -> Result<(), FsError> {
            self.inner.set_permissions(path, mode)
        }

        fn is_executable(&self, path: &Path) -> bool {
            self.inner.is_executable(path)
        }

        fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError> {
            self.inner.rename(from, to)
        }

        fn canonicalize(&self, path: &Path) -> Result<std::path::PathBuf, std::io::Error> {
            self.inner.canonicalize(path)
        }
    }

    // --- pre_scan_directory ---

    #[test]
    fn pre_scan_directory_new_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content a")
            .with_file("/src/b.md", "content b");

        let (to_review, unchanged, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(to_review.len(), 2);
        assert!(unchanged.is_empty());
        assert!(to_review[0].is_new);
        assert!(to_review[1].is_new);
    }

    #[test]
    fn pre_scan_directory_changed_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "new content")
            .with_file("/dest/a.md", "old content");

        let (to_review, unchanged, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(to_review.len(), 1);
        assert!(unchanged.is_empty());
        assert!(!to_review[0].is_new);
    }

    #[test]
    fn pre_scan_directory_unchanged_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "same content")
            .with_file("/dest/a.md", "same content");

        let (to_review, unchanged, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(to_review.is_empty());
        assert_eq!(unchanged, vec!["a.md"]);
    }

    #[test]
    fn pre_scan_directory_filters_by_ext() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content")
            .with_file("/src/b.txt", "content");

        let (to_review, _, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(to_review.len(), 1);
        assert_eq!(to_review[0].filename, "a.md");
    }

    #[test]
    fn pre_scan_directory_mixed() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/new.md", "brand new")
            .with_file("/src/changed.md", "updated")
            .with_file("/dest/changed.md", "original")
            .with_file("/src/same.md", "same")
            .with_file("/dest/same.md", "same");

        let (to_review, unchanged, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(to_review.len(), 2);
        assert_eq!(unchanged, vec!["same.md"]);
    }

    #[test]
    fn pre_scan_directory_empty_src() {
        let fs = InMemoryFileSystem::new();
        let (to_review, unchanged, _errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(to_review.is_empty());
        assert!(unchanged.is_empty());
    }

    // --- apply_accept ---

    #[test]
    fn apply_accept_copies_file() {
        let fs = InMemoryFileSystem::new().with_file("/src/a.md", "content");

        let result = apply_accept(&fs, Path::new("/src/a.md"), Path::new("/dest/a.md"), false);
        assert!(result.is_ok());
        assert_eq!(
            fs.read_to_string(Path::new("/dest/a.md")).unwrap(),
            "content"
        );
    }

    #[test]
    fn apply_accept_dry_run_skips() {
        let fs = InMemoryFileSystem::new().with_file("/src/a.md", "content");

        let result = apply_accept(&fs, Path::new("/src/a.md"), Path::new("/dest/a.md"), true);
        assert!(result.is_ok());
        assert!(!fs.exists(Path::new("/dest/a.md")));
    }

    #[test]
    fn apply_accept_creates_parent_dirs() {
        let fs = InMemoryFileSystem::new().with_file("/src/a.md", "content");

        let result = apply_accept(
            &fs,
            Path::new("/src/a.md"),
            Path::new("/dest/sub/dir/a.md"),
            false,
        );
        assert!(result.is_ok());
        assert!(fs.exists(Path::new("/dest/sub/dir/a.md")));
    }

    #[test]
    fn apply_accept_error_on_missing_src() {
        let fs = InMemoryFileSystem::new();

        let result = apply_accept(
            &fs,
            Path::new("/nonexistent.md"),
            Path::new("/dest/a.md"),
            false,
        );
        assert!(result.is_err());
    }

    // --- error path tests (PC-010, PC-011) ---

    #[test]
    fn pre_scan_directory_unreadable_src_reports_error() {
        let inner = InMemoryFileSystem::new();
        let fs = FailingFs::new(inner).with_fail_read_dir("/src");

        let (to_review, unchanged, errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");

        assert!(to_review.is_empty(), "no files should be reviewed on error");
        assert!(unchanged.is_empty());
        assert_eq!(errors.len(), 1, "one error should be reported");
        assert!(
            errors[0].contains("/src"),
            "error should mention the unreadable path"
        );
    }

    #[test]
    fn pre_scan_file_read_failure_reports_error() {
        // File exists but reading it fails for src — should report error not classify as changed
        let inner = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content")
            .with_file("/dest/a.md", "content");
        let fs = FailingFs::new(inner).with_fail_read_to_string("/src/a.md");

        let (to_review, unchanged, errors) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");

        assert!(
            to_review.is_empty(),
            "file should not be queued for review when read fails"
        );
        assert!(unchanged.is_empty(), "file should not be marked unchanged");
        assert_eq!(errors.len(), 1, "one read error should be reported");
        assert!(
            errors[0].contains("/src/a.md"),
            "error should mention the unreadable file"
        );
    }
}
