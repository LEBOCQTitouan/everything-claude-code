//! Domain purity test — ensures ecc-domain has zero I/O imports.
//!
//! Scans all .rs files in `crates/ecc-domain/src/` for forbidden import patterns.
//! This closes the gap where only `worktree.rs` had a self-check (BL-138).

use std::fs;
use std::path::Path;

/// Forbidden import patterns that indicate I/O usage in the domain layer.
const FORBIDDEN_PATTERNS: &[&str] = &[
    "use std::fs",
    "use std::io",
    "use std::process",
    "use std::net",
    "use tokio",
    "std::fs::",
    "std::io::",
    "std::process::",
    "std::net::",
    "tokio::",
];

fn collect_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn domain_crate_has_zero_io_imports() {
    let domain_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(
        domain_src.exists(),
        "domain src dir not found: {domain_src:?}"
    );

    let mut files = Vec::new();
    collect_rs_files(&domain_src, &mut files);
    assert!(!files.is_empty(), "no .rs files found in domain src");

    let mut violations = Vec::new();

    for file in &files {
        let content = fs::read_to_string(file).unwrap();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("* ") {
                continue;
            }
            for pattern in FORBIDDEN_PATTERNS {
                if trimmed.contains(pattern) {
                    let relative = file.strip_prefix(&domain_src).unwrap_or(file);
                    violations.push(format!(
                        "  {}:{}: {}",
                        relative.display(),
                        line_num + 1,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Domain purity violation! ecc-domain must have zero I/O imports.\n\
         Found {} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
