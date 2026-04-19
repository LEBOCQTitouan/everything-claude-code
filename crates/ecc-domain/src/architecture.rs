//! Architecture tests for ecc-domain crate-level invariants.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn walk_rs_files(dir: &Path, acc: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).expect("read_dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_rs_files(&path, acc);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                acc.push(path);
            }
        }
    }

    #[test]
    fn no_io_imports_in_domain() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        walk_rs_files(&src_root, &mut files);

        // Build forbidden patterns at runtime to avoid self-match in this file.
        let forbidden = [
            concat!("use ", "std::fs"),
            concat!("use ", "std::net"),
            concat!("use ", "std::process"),
            concat!("use ", "tokio"),
            concat!("use ", "async_std"),
            concat!("use ", "reqwest"),
            concat!("use ", "std::io::Read"),
            concat!("use ", "std::io::Write"),
        ];

        for file in &files {
            let contents = fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));

            // Skip #[cfg(test)] sections — tests may use I/O freely.
            // Simplest: split on the first occurrence of `#[cfg(test)]`
            // and only check the production half.
            let production = match contents.find("#[cfg(test)]") {
                Some(idx) => &contents[..idx],
                None => contents.as_str(),
            };

            for pat in &forbidden {
                assert!(
                    !production.contains(pat),
                    "domain purity violation: {} contains `{pat}`",
                    file.display()
                );
            }
        }
    }
}
