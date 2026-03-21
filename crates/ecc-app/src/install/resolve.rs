//! ECC root directory resolution.

use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use std::path::PathBuf;

/// Resolve the ECC root directory containing agents/, commands/, etc.
///
/// Resolution order:
/// 1. `ECC_ROOT` env var (explicit override)
/// 2. Repo root detection (dev scenario: walk up from cwd looking for Cargo.toml + agents/)
/// 3. Relative to binary: `parent_of_binary/..` (handles `~/.ecc/bin/ecc` → `~/.ecc/`)
/// 4. `$HOME/.ecc/`
/// 5. Legacy npm global install paths (backward compat)
/// 6. Error with instructions
pub fn resolve_ecc_root(fs: &dyn FileSystem, env: &dyn Environment) -> Result<PathBuf, String> {
    // 1. ECC_ROOT env var (explicit override)
    if let Some(ecc_root) = env.var("ECC_ROOT") {
        let root = PathBuf::from(&ecc_root);
        if fs.exists(&root.join("agents")) {
            return Ok(root);
        }
    }

    // 2. Repo root detection (dev scenario: walk up from cwd looking for Cargo.toml + agents/)
    if let Some(cwd) = env.current_dir() {
        let mut dir = cwd.as_path();
        loop {
            if fs.exists(&dir.join("Cargo.toml")) && fs.exists(&dir.join("agents")) {
                return Ok(dir.to_path_buf());
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    // 3. Relative to binary: parent/.. (e.g. ~/.ecc/bin/ecc → ~/.ecc/)
    if let Ok(exe) = std::env::current_exe()
        && let Some(bin_dir) = exe.parent()
    {
        let relative = bin_dir.join("..");
        if fs.exists(&relative.join("agents")) {
            return Ok(relative);
        }
    }

    // 4. $HOME/.ecc/
    if let Some(home) = env.home_dir() {
        let home_ecc = home.join(".ecc");
        if fs.exists(&home_ecc.join("agents")) {
            return Ok(home_ecc);
        }
    }

    // 5. Legacy npm paths (backward compat)
    let npm_paths = [
        "/usr/local/lib/node_modules/@lebocqtitouan/ecc",
        "/usr/lib/node_modules/@lebocqtitouan/ecc",
    ];

    for path in &npm_paths {
        let p = PathBuf::from(path);
        if fs.exists(&p.join("agents")) {
            return Ok(p);
        }
    }

    Err(
        "Cannot find ECC assets directory. Install with: \
         curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash\n\
         Or set ECC_ROOT environment variable / use --ecc-root flag."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    #[test]
    fn resolve_ecc_root_finds_home_ecc() {
        let fs = InMemoryFileSystem::new().with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/.ecc"));
    }

    #[test]
    fn resolve_ecc_root_finds_legacy_npm_path() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/usr/local/lib/node_modules/@lebocqtitouan/ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/usr/local/lib/node_modules/@lebocqtitouan/ecc")
        );
    }

    #[test]
    fn resolve_ecc_root_prefers_home_over_npm() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/user/.ecc/agents")
            .with_dir("/usr/local/lib/node_modules/@lebocqtitouan/ecc/agents");
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        // $HOME/.ecc/ should be preferred over npm paths
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/.ecc"));
    }

    #[test]
    fn resolve_ecc_root_errors_when_no_paths_found() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_home("/home/user");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot find ECC assets directory")
        );
    }

    #[test]
    fn resolve_ecc_root_uses_ecc_root_env_var() {
        let fs = InMemoryFileSystem::new().with_dir("/opt/ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/opt/ecc");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/opt/ecc"));
    }

    #[test]
    fn resolve_ecc_root_ecc_root_env_var_takes_priority() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/opt/ecc/agents")
            .with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/opt/ecc");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/opt/ecc"));
    }

    #[test]
    fn resolve_ecc_root_skips_invalid_ecc_root_env_var() {
        let fs = InMemoryFileSystem::new().with_dir("/home/user/.ecc/agents");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_var("ECC_ROOT", "/nonexistent/path");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        // Falls through to $HOME/.ecc/
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/.ecc"));
    }

    #[test]
    fn resolve_ecc_root_finds_repo_root() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/code/ecc/agents")
            .with_file("/code/ecc/Cargo.toml", "[workspace]");
        let env = MockEnvironment::new()
            .with_home("/home/user")
            .with_current_dir("/code/ecc/crates/ecc-cli");

        let result = resolve_ecc_root(&fs, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/code/ecc"));
    }
}
