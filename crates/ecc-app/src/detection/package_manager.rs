use ecc_domain::detection::package_manager::{
    DETECTION_PRIORITY, DetectionSource, PackageManagerResult, find_config,
};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Detect package manager from lock file in project directory.
pub fn detect_from_lock_file(fs: &dyn FileSystem, dir: &Path) -> Option<&'static str> {
    for pm in DETECTION_PRIORITY {
        if fs.exists(&dir.join(pm.lock_file)) {
            return Some(pm.name);
        }
    }
    None
}

/// Detect package manager from package.json packageManager field.
pub fn detect_from_package_json(fs: &dyn FileSystem, dir: &Path) -> Option<&'static str> {
    let pkg_path = dir.join("package.json");
    let content = fs.read_to_string(&pkg_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let pm_field = parsed.get("packageManager")?.as_str()?;
    let pm_name = pm_field.split('@').next()?;
    find_config(pm_name).map(|c| c.name)
}

/// Read a package manager name from a JSON config file.
fn read_pm_from_json(fs: &dyn FileSystem, path: &Path) -> Option<String> {
    let content = fs.read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    parsed
        .get("packageManager")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Get the package manager to use for a project.
///
/// Detection priority: env var > project config > package.json > lock file > global config > npm.
pub fn get_package_manager(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    dir: &Path,
) -> PackageManagerResult {
    use ecc_domain::detection::package_manager::NPM;

    // 1. Environment variable
    if let Some(env_pm) = env.var("CLAUDE_PACKAGE_MANAGER")
        && let Some(config) = find_config(&env_pm)
    {
        return PackageManagerResult {
            name: env_pm,
            config,
            source: DetectionSource::Environment,
        };
    }

    // 2. Project-specific config
    let project_config_path = dir.join(".claude").join("package-manager.json");
    if let Some(pm_name) = read_pm_from_json(fs, &project_config_path)
        && let Some(config) = find_config(&pm_name)
    {
        return PackageManagerResult {
            name: pm_name,
            config,
            source: DetectionSource::ProjectConfig,
        };
    }

    // 3. package.json packageManager field
    if let Some(pm_name) = detect_from_package_json(fs, dir)
        && let Some(config) = find_config(pm_name)
    {
        return PackageManagerResult {
            name: pm_name.to_string(),
            config,
            source: DetectionSource::PackageJson,
        };
    }

    // 4. Lock file detection
    if let Some(pm_name) = detect_from_lock_file(fs, dir)
        && let Some(config) = find_config(pm_name)
    {
        return PackageManagerResult {
            name: pm_name.to_string(),
            config,
            source: DetectionSource::LockFile,
        };
    }

    // 5. Global user preference
    if let Some(home) = env.home_dir()
        && let Some(pm_name) =
            read_pm_from_json(fs, &home.join(".claude").join("package-manager.json"))
        && let Some(config) = find_config(&pm_name)
    {
        return PackageManagerResult {
            name: pm_name,
            config,
            source: DetectionSource::GlobalConfig,
        };
    }

    // 6. Default to npm
    PackageManagerResult {
        name: "npm".to_string(),
        config: &NPM,
        source: DetectionSource::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};

    // --- detect_from_lock_file ---

    #[test]
    fn detect_lock_file_pnpm() {
        let fs = InMemoryFileSystem::new().with_file("/project/pnpm-lock.yaml", "");
        assert_eq!(
            detect_from_lock_file(&fs, Path::new("/project")),
            Some("pnpm")
        );
    }

    #[test]
    fn detect_lock_file_npm() {
        let fs = InMemoryFileSystem::new().with_file("/project/package-lock.json", "{}");
        assert_eq!(
            detect_from_lock_file(&fs, Path::new("/project")),
            Some("npm")
        );
    }

    #[test]
    fn detect_lock_file_yarn() {
        let fs = InMemoryFileSystem::new().with_file("/project/yarn.lock", "");
        assert_eq!(
            detect_from_lock_file(&fs, Path::new("/project")),
            Some("yarn")
        );
    }

    #[test]
    fn detect_lock_file_bun() {
        let fs = InMemoryFileSystem::new().with_file("/project/bun.lockb", "");
        assert_eq!(
            detect_from_lock_file(&fs, Path::new("/project")),
            Some("bun")
        );
    }

    #[test]
    fn detect_lock_file_priority() {
        // Both pnpm and npm lock files present — pnpm wins
        let fs = InMemoryFileSystem::new()
            .with_file("/project/pnpm-lock.yaml", "")
            .with_file("/project/package-lock.json", "{}");
        assert_eq!(
            detect_from_lock_file(&fs, Path::new("/project")),
            Some("pnpm")
        );
    }

    #[test]
    fn detect_lock_file_none() {
        let fs = InMemoryFileSystem::new();
        assert_eq!(detect_from_lock_file(&fs, Path::new("/project")), None);
    }

    // --- detect_from_package_json ---

    #[test]
    fn detect_package_json_pnpm() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/package.json",
            r#"{"packageManager": "pnpm@8.0.0"}"#,
        );
        assert_eq!(
            detect_from_package_json(&fs, Path::new("/project")),
            Some("pnpm")
        );
    }

    #[test]
    fn detect_package_json_yarn() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/package.json", r#"{"packageManager": "yarn@3"}"#);
        assert_eq!(
            detect_from_package_json(&fs, Path::new("/project")),
            Some("yarn")
        );
    }

    #[test]
    fn detect_package_json_no_field() {
        let fs =
            InMemoryFileSystem::new().with_file("/project/package.json", r#"{"name": "test"}"#);
        assert_eq!(detect_from_package_json(&fs, Path::new("/project")), None);
    }

    #[test]
    fn detect_package_json_unknown_pm() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/package.json",
            r#"{"packageManager": "unknown@1"}"#,
        );
        assert_eq!(detect_from_package_json(&fs, Path::new("/project")), None);
    }

    #[test]
    fn detect_package_json_missing_file() {
        let fs = InMemoryFileSystem::new();
        assert_eq!(detect_from_package_json(&fs, Path::new("/project")), None);
    }

    #[test]
    fn detect_package_json_invalid_json() {
        let fs = InMemoryFileSystem::new().with_file("/project/package.json", "not json");
        assert_eq!(detect_from_package_json(&fs, Path::new("/project")), None);
    }

    // --- get_package_manager ---

    #[test]
    fn get_pm_env_var() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PACKAGE_MANAGER", "pnpm");
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "pnpm");
        assert_eq!(result.source, DetectionSource::Environment);
    }

    #[test]
    fn get_pm_env_var_invalid() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PACKAGE_MANAGER", "unknown");
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.source, DetectionSource::Default);
    }

    #[test]
    fn get_pm_project_config() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.claude/package-manager.json",
            r#"{"packageManager": "yarn"}"#,
        );
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "yarn");
        assert_eq!(result.source, DetectionSource::ProjectConfig);
    }

    #[test]
    fn get_pm_package_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/package.json", r#"{"packageManager": "bun@1.0"}"#);
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "bun");
        assert_eq!(result.source, DetectionSource::PackageJson);
    }

    #[test]
    fn get_pm_lock_file() {
        let fs = InMemoryFileSystem::new().with_file("/project/yarn.lock", "");
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "yarn");
        assert_eq!(result.source, DetectionSource::LockFile);
    }

    #[test]
    fn get_pm_global_config() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/package-manager.json",
            r#"{"packageManager": "pnpm"}"#,
        );
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "pnpm");
        assert_eq!(result.source, DetectionSource::GlobalConfig);
    }

    #[test]
    fn get_pm_default_npm() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "npm");
        assert_eq!(result.source, DetectionSource::Default);
    }

    #[test]
    fn get_pm_priority_env_over_lock() {
        let fs = InMemoryFileSystem::new().with_file("/project/yarn.lock", "");
        let env = MockEnvironment::new().with_var("CLAUDE_PACKAGE_MANAGER", "pnpm");
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "pnpm");
        assert_eq!(result.source, DetectionSource::Environment);
    }

    #[test]
    fn get_pm_priority_project_config_over_package_json() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/project/.claude/package-manager.json",
                r#"{"packageManager": "yarn"}"#,
            )
            .with_file("/project/package.json", r#"{"packageManager": "pnpm@8"}"#);
        let env = MockEnvironment::new();
        let result = get_package_manager(&fs, &env, Path::new("/project"));
        assert_eq!(result.name, "yarn");
        assert_eq!(result.source, DetectionSource::ProjectConfig);
    }
}
