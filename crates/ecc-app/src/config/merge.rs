use ecc_domain::config::merge::{contents_differ, FileToReview};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Pre-scan a directory to identify files that need review (new or changed).
/// Returns `(files_to_review, unchanged_filenames)`.
pub fn pre_scan_directory(
    fs: &dyn FileSystem,
    src_dir: &Path,
    dest_dir: &Path,
    ext: &str,
) -> (Vec<FileToReview>, Vec<String>) {
    let mut files_to_review = Vec::new();
    let mut unchanged = Vec::new();

    let entries = match fs.read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return (files_to_review, unchanged),
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
            let src_content = fs.read_to_string(&src_path).unwrap_or_default();
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

    (files_to_review, unchanged)
}

/// Copy a file from source to destination.
/// In dry-run mode, the copy is skipped.
pub fn apply_accept(
    fs: &dyn FileSystem,
    src_path: &Path,
    dest_path: &Path,
    dry_run: bool,
) -> Result<(), String> {
    if dry_run {
        return Ok(());
    }

    if let Some(parent) = dest_path.parent() {
        fs.create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    fs.copy(src_path, dest_path)
        .map_err(|e| format!("Failed to copy file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // --- pre_scan_directory ---

    #[test]
    fn pre_scan_directory_new_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content a")
            .with_file("/src/b.md", "content b");

        let (to_review, unchanged) =
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

        let (to_review, unchanged) =
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

        let (to_review, unchanged) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(to_review.is_empty());
        assert_eq!(unchanged, vec!["a.md"]);
    }

    #[test]
    fn pre_scan_directory_filters_by_ext() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content")
            .with_file("/src/b.txt", "content");

        let (to_review, _) =
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

        let (to_review, unchanged) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(to_review.len(), 2);
        assert_eq!(unchanged, vec!["same.md"]);
    }

    #[test]
    fn pre_scan_directory_empty_src() {
        let fs = InMemoryFileSystem::new();
        let (to_review, unchanged) =
            pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(to_review.is_empty());
        assert!(unchanged.is_empty());
    }

    // --- apply_accept ---

    #[test]
    fn apply_accept_copies_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content");

        let result = apply_accept(
            &fs,
            Path::new("/src/a.md"),
            Path::new("/dest/a.md"),
            false,
        );
        assert!(result.is_ok());
        assert_eq!(
            fs.read_to_string(Path::new("/dest/a.md")).unwrap(),
            "content"
        );
    }

    #[test]
    fn apply_accept_dry_run_skips() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content");

        let result = apply_accept(
            &fs,
            Path::new("/src/a.md"),
            Path::new("/dest/a.md"),
            true,
        );
        assert!(result.is_ok());
        assert!(!fs.exists(Path::new("/dest/a.md")));
    }

    #[test]
    fn apply_accept_creates_parent_dirs() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content");

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
}
