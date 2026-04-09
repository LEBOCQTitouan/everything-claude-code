/// Configuration for a package manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerConfig {
    pub name: &'static str,
    pub lock_file: &'static str,
    pub install_cmd: &'static str,
    pub run_cmd: &'static str,
    pub exec_cmd: &'static str,
    pub test_cmd: &'static str,
    pub build_cmd: &'static str,
    pub dev_cmd: &'static str,
}

pub const NPM: PackageManagerConfig = PackageManagerConfig {
    name: "npm",
    lock_file: "package-lock.json",
    install_cmd: "npm install",
    run_cmd: "npm run",
    exec_cmd: "npx",
    test_cmd: "npm test",
    build_cmd: "npm run build",
    dev_cmd: "npm run dev",
};

pub const PNPM: PackageManagerConfig = PackageManagerConfig {
    name: "pnpm",
    lock_file: "pnpm-lock.yaml",
    install_cmd: "pnpm install",
    run_cmd: "pnpm",
    exec_cmd: "pnpm dlx",
    test_cmd: "pnpm test",
    build_cmd: "pnpm build",
    dev_cmd: "pnpm dev",
};

pub const YARN: PackageManagerConfig = PackageManagerConfig {
    name: "yarn",
    lock_file: "yarn.lock",
    install_cmd: "yarn",
    run_cmd: "yarn",
    exec_cmd: "yarn dlx",
    test_cmd: "yarn test",
    build_cmd: "yarn build",
    dev_cmd: "yarn dev",
};

pub const BUN: PackageManagerConfig = PackageManagerConfig {
    name: "bun",
    lock_file: "bun.lockb",
    install_cmd: "bun install",
    run_cmd: "bun run",
    exec_cmd: "bunx",
    test_cmd: "bun test",
    build_cmd: "bun run build",
    dev_cmd: "bun run dev",
};

/// All supported package managers.
pub const ALL_CONFIGS: &[&PackageManagerConfig] = &[&NPM, &PNPM, &YARN, &BUN];

/// Priority order for lock file detection.
pub const DETECTION_PRIORITY: &[&PackageManagerConfig] = &[&PNPM, &BUN, &YARN, &NPM];

/// Error type for package manager operations.
#[derive(Debug, thiserror::Error)]
pub enum PackageManagerError {
    #[error("Script name must be a non-empty string")]
    EmptyScriptName,
    #[error("Script name contains unsafe characters: {0}")]
    UnsafeScriptName(String),
    #[error("Arguments contain unsafe characters: {0}")]
    UnsafeArgs(String),
}

/// Regex for safe script/binary names.
const SAFE_NAME_PATTERN: &str = r"^[@a-zA-Z0-9_./-]+$";

/// Regex for safe command arguments.
const SAFE_ARGS_PATTERN: &str = r#"^[@a-zA-Z0-9\s_./:=,'"*+\-]+$"#;

/// Look up a config by name.
pub fn find_config(name: &str) -> Option<&'static PackageManagerConfig> {
    ALL_CONFIGS.iter().find(|c| c.name == name).copied()
}

/// Detection source for a package manager result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSource {
    Environment,
    ProjectConfig,
    PackageJson,
    LockFile,
    GlobalConfig,
    Default,
}

impl DetectionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::ProjectConfig => "project-config",
            Self::PackageJson => "package.json",
            Self::LockFile => "lock-file",
            Self::GlobalConfig => "global-config",
            Self::Default => "default",
        }
    }
}

/// Result of detecting the package manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerResult {
    pub name: String,
    pub config: &'static PackageManagerConfig,
    pub source: DetectionSource,
}

use std::sync::LazyLock;

static SAFE_NAME_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(SAFE_NAME_PATTERN).expect("BUG: invalid SAFE_NAME_PATTERN regex"));

static SAFE_ARGS_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(SAFE_ARGS_PATTERN).expect("BUG: invalid SAFE_ARGS_PATTERN regex"));

/// Validate a script name contains only safe characters.
pub fn validate_script_name(name: &str) -> Result<(), PackageManagerError> {
    if name.is_empty() {
        return Err(PackageManagerError::EmptyScriptName);
    }
    if !SAFE_NAME_RE.is_match(name) {
        return Err(PackageManagerError::UnsafeScriptName(name.to_string()));
    }
    Ok(())
}

/// Validate command arguments contain only safe characters.
pub fn validate_args(args: &str) -> Result<(), PackageManagerError> {
    if args.is_empty() {
        return Ok(());
    }
    if !SAFE_ARGS_RE.is_match(args) {
        return Err(PackageManagerError::UnsafeArgs(args.to_string()));
    }
    Ok(())
}

/// Get the command to run a script with the given package manager config.
pub fn get_run_command(
    config: &PackageManagerConfig,
    script: &str,
) -> Result<String, PackageManagerError> {
    validate_script_name(script)?;

    Ok(match script {
        "install" => config.install_cmd.to_string(),
        "test" => config.test_cmd.to_string(),
        "build" => config.build_cmd.to_string(),
        "dev" => config.dev_cmd.to_string(),
        _ => format!("{} {script}", config.run_cmd),
    })
}

/// Get the command to execute a package binary.
pub fn get_exec_command(
    config: &PackageManagerConfig,
    binary: &str,
    args: Option<&str>,
) -> Result<String, PackageManagerError> {
    validate_script_name(binary)?;
    if let Some(a) = args {
        validate_args(a)?;
    }

    let base = format!("{} {binary}", config.exec_cmd);
    Ok(match args {
        Some(a) if !a.is_empty() => format!("{base} {a}"),
        _ => base,
    })
}

/// Escape regex special characters in a string.
fn escape_regex(s: &str) -> String {
    let special = r".*+?^${}()|[]\";
    let mut escaped = String::with_capacity(s.len() * 2);
    for ch in s.chars() {
        if special.contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Generate a regex pattern that matches commands for all package managers.
pub fn get_command_pattern(action: &str) -> String {
    let trimmed = action.trim();
    let patterns = match trimmed {
        "dev" => vec![
            "npm run dev".to_string(),
            "pnpm( run)? dev".to_string(),
            "yarn dev".to_string(),
            "bun run dev".to_string(),
        ],
        "install" => vec![
            "npm install".to_string(),
            "pnpm install".to_string(),
            "yarn( install)?".to_string(),
            "bun install".to_string(),
        ],
        "test" => vec![
            "npm test".to_string(),
            "pnpm test".to_string(),
            "yarn test".to_string(),
            "bun test".to_string(),
        ],
        "build" => vec![
            "npm run build".to_string(),
            "pnpm( run)? build".to_string(),
            "yarn build".to_string(),
            "bun run build".to_string(),
        ],
        _ => {
            let escaped = escape_regex(trimmed);
            vec![
                format!("npm run {escaped}"),
                format!("pnpm( run)? {escaped}"),
                format!("yarn {escaped}"),
                format!("bun run {escaped}"),
            ]
        }
    };

    format!("({})", patterns.join("|"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Config constants ---

    #[test]
    fn npm_config() {
        assert_eq!(NPM.name, "npm");
        assert_eq!(NPM.lock_file, "package-lock.json");
        assert_eq!(NPM.install_cmd, "npm install");
        assert_eq!(NPM.exec_cmd, "npx");
    }

    #[test]
    fn pnpm_config() {
        assert_eq!(PNPM.name, "pnpm");
        assert_eq!(PNPM.lock_file, "pnpm-lock.yaml");
        assert_eq!(PNPM.run_cmd, "pnpm");
        assert_eq!(PNPM.exec_cmd, "pnpm dlx");
    }

    #[test]
    fn yarn_config() {
        assert_eq!(YARN.name, "yarn");
        assert_eq!(YARN.lock_file, "yarn.lock");
        assert_eq!(YARN.install_cmd, "yarn");
        assert_eq!(YARN.exec_cmd, "yarn dlx");
    }

    #[test]
    fn bun_config() {
        assert_eq!(BUN.name, "bun");
        assert_eq!(BUN.lock_file, "bun.lockb");
        assert_eq!(BUN.exec_cmd, "bunx");
    }

    #[test]
    fn all_configs_has_four() {
        assert_eq!(ALL_CONFIGS.len(), 4);
    }

    #[test]
    fn detection_priority_pnpm_first() {
        assert_eq!(DETECTION_PRIORITY[0].name, "pnpm");
    }

    // --- find_config ---

    #[test]
    fn find_config_npm() {
        assert_eq!(find_config("npm").unwrap().name, "npm");
    }

    #[test]
    fn find_config_unknown() {
        assert!(find_config("unknown").is_none());
    }

    // --- validate_script_name ---

    #[test]
    fn validate_name_valid() {
        assert!(validate_script_name("test").is_ok());
        assert!(validate_script_name("@scope/pkg").is_ok());
        assert!(validate_script_name("my-script").is_ok());
    }

    #[test]
    fn validate_name_empty() {
        assert!(validate_script_name("").is_err());
    }

    #[test]
    fn validate_name_unsafe() {
        assert!(validate_script_name("rm -rf /").is_err());
        assert!(validate_script_name("$(whoami)").is_err());
    }

    // --- validate_args ---

    #[test]
    fn validate_args_valid() {
        assert!(validate_args("--port=3000").is_ok());
        assert!(validate_args("--config 'test.json'").is_ok());
    }

    #[test]
    fn validate_args_empty() {
        assert!(validate_args("").is_ok());
    }

    #[test]
    fn validate_args_unsafe() {
        assert!(validate_args("$(whoami)").is_err());
        assert!(validate_args("; rm -rf /").is_err());
    }

    // --- get_run_command ---

    #[test]
    fn run_command_install() {
        assert_eq!(get_run_command(&NPM, "install").unwrap(), "npm install");
        assert_eq!(get_run_command(&YARN, "install").unwrap(), "yarn");
    }

    #[test]
    fn run_command_test() {
        assert_eq!(get_run_command(&NPM, "test").unwrap(), "npm test");
        assert_eq!(get_run_command(&PNPM, "test").unwrap(), "pnpm test");
    }

    #[test]
    fn run_command_build() {
        assert_eq!(get_run_command(&NPM, "build").unwrap(), "npm run build");
        assert_eq!(get_run_command(&PNPM, "build").unwrap(), "pnpm build");
    }

    #[test]
    fn run_command_dev() {
        assert_eq!(get_run_command(&NPM, "dev").unwrap(), "npm run dev");
        assert_eq!(get_run_command(&YARN, "dev").unwrap(), "yarn dev");
    }

    #[test]
    fn run_command_custom() {
        assert_eq!(get_run_command(&NPM, "lint").unwrap(), "npm run lint");
        assert_eq!(get_run_command(&PNPM, "lint").unwrap(), "pnpm lint");
        assert_eq!(get_run_command(&YARN, "lint").unwrap(), "yarn lint");
        assert_eq!(get_run_command(&BUN, "lint").unwrap(), "bun run lint");
    }

    #[test]
    fn run_command_empty_name() {
        assert!(get_run_command(&NPM, "").is_err());
    }

    #[test]
    fn run_command_unsafe_name() {
        assert!(get_run_command(&NPM, "$(whoami)").is_err());
    }

    // --- get_exec_command ---

    #[test]
    fn exec_command_npm() {
        assert_eq!(get_exec_command(&NPM, "jest", None).unwrap(), "npx jest");
    }

    #[test]
    fn exec_command_with_args() {
        assert_eq!(
            get_exec_command(&NPM, "jest", Some("--watch")).unwrap(),
            "npx jest --watch"
        );
    }

    #[test]
    fn exec_command_pnpm() {
        assert_eq!(
            get_exec_command(&PNPM, "create-react-app", Some("my-app")).unwrap(),
            "pnpm dlx create-react-app my-app"
        );
    }

    #[test]
    fn exec_command_bun() {
        assert_eq!(
            get_exec_command(&BUN, "vitest", None).unwrap(),
            "bunx vitest"
        );
    }

    #[test]
    fn exec_command_empty_binary() {
        assert!(get_exec_command(&NPM, "", None).is_err());
    }

    #[test]
    fn exec_command_unsafe_args() {
        assert!(get_exec_command(&NPM, "jest", Some("; rm -rf /")).is_err());
    }

    // --- get_command_pattern ---

    #[test]
    fn command_pattern_dev() {
        let pattern = get_command_pattern("dev");
        assert!(pattern.contains("npm run dev"));
        assert!(pattern.contains("pnpm( run)? dev"));
        assert!(pattern.contains("yarn dev"));
        assert!(pattern.contains("bun run dev"));
    }

    #[test]
    fn command_pattern_install() {
        let pattern = get_command_pattern("install");
        assert!(pattern.contains("npm install"));
        assert!(pattern.contains("yarn( install)?"));
    }

    #[test]
    fn command_pattern_test() {
        let pattern = get_command_pattern("test");
        assert!(pattern.contains("npm test"));
        assert!(pattern.contains("bun test"));
    }

    #[test]
    fn command_pattern_build() {
        let pattern = get_command_pattern("build");
        assert!(pattern.contains("npm run build"));
        assert!(pattern.contains("pnpm( run)? build"));
    }

    #[test]
    fn command_pattern_custom() {
        let pattern = get_command_pattern("lint");
        assert!(pattern.contains("npm run lint"));
        assert!(pattern.contains("pnpm( run)? lint"));
        assert!(pattern.contains("yarn lint"));
        assert!(pattern.contains("bun run lint"));
    }

    #[test]
    fn command_pattern_trimmed() {
        let pattern = get_command_pattern("  dev  ");
        assert!(pattern.contains("npm run dev"));
    }

    #[test]
    fn command_pattern_regex_escaping() {
        let pattern = get_command_pattern("test.spec");
        // The dot should be escaped
        assert!(pattern.contains(r"test\.spec"));
    }

    // --- DetectionSource ---

    #[test]
    fn detection_source_as_str() {
        assert_eq!(DetectionSource::Environment.as_str(), "environment");
        assert_eq!(DetectionSource::LockFile.as_str(), "lock-file");
        assert_eq!(DetectionSource::Default.as_str(), "default");
    }

    // --- PackageManagerError ---

    /// PC-006: PackageManagerError enum variants compile and are accessible.
    #[test]
    fn package_manager_error_empty_name_is_debug() {
        let err = PackageManagerError::EmptyScriptName;
        let s = format!("{err:?}");
        assert!(!s.is_empty());
    }

    #[test]
    fn package_manager_error_unsafe_name_carries_message() {
        let err = PackageManagerError::UnsafeScriptName("$(whoami)".to_string());
        let s = err.to_string();
        assert!(s.contains("$(whoami)"), "expected name in error, got: {s}");
    }

    #[test]
    fn package_manager_error_unsafe_args_carries_message() {
        let err = PackageManagerError::UnsafeArgs("; rm -rf /".to_string());
        let s = err.to_string();
        assert!(s.contains("; rm"), "expected args in error, got: {s}");
    }
}
