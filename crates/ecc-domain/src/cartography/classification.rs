//! File classification for cartography deltas.
//!
//! Maps file paths to component classifications based on project type.
//! Pure function — no I/O, no side effects.

use super::types::ProjectType;

/// Classify a changed file path based on the project type.
///
/// Returns a component classification string:
/// - Rust: crate name (e.g., `crates/ecc-domain/src/lib.rs` → `ecc-domain`)
/// - JS/TS: package name (e.g., `packages/ui/src/index.ts` → `ui`)
/// - Other: top-level directory name
pub fn classify_file(path: &str, project_type: &ProjectType) -> String {
    let parts: Vec<&str> = path.splitn(4, '/').collect();
    match project_type {
        ProjectType::Rust => {
            // crates/<crate-name>/... → <crate-name>
            if parts.len() >= 2 && parts[0] == "crates" {
                return parts[1].to_string();
            }
            // Fallback: first path component
            parts[0].to_string()
        }
        ProjectType::Javascript | ProjectType::Typescript => {
            // packages/<package>/... or apps/<app>/... → <package>/<app>
            if parts.len() >= 2 && (parts[0] == "packages" || parts[0] == "apps") {
                return parts[1].to_string();
            }
            parts[0].to_string()
        }
        _ => {
            // Unknown: top-level directory
            parts[0].to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_rust_crate() {
        assert_eq!(
            classify_file("crates/ecc-domain/src/lib.rs", &ProjectType::Rust),
            "ecc-domain"
        );
        assert_eq!(
            classify_file("crates/ecc-app/src/hook/mod.rs", &ProjectType::Rust),
            "ecc-app"
        );
        // Fallback: non-crate path
        assert_eq!(
            classify_file("src/main.rs", &ProjectType::Rust),
            "src"
        );
    }

    #[test]
    fn classify_jsts_and_unknown() {
        assert_eq!(
            classify_file("packages/ui/src/index.ts", &ProjectType::Typescript),
            "ui"
        );
        assert_eq!(
            classify_file("apps/web/src/App.tsx", &ProjectType::Javascript),
            "web"
        );
        // Fallback: non-packages path
        assert_eq!(
            classify_file("src/index.ts", &ProjectType::Typescript),
            "src"
        );
        // Unknown project type
        assert_eq!(
            classify_file("lib/utils.py", &ProjectType::Unknown),
            "lib"
        );
        assert_eq!(
            classify_file("cmd/server/main.go", &ProjectType::Go),
            "cmd"
        );
    }
}
