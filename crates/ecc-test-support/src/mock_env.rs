use ecc_ports::env::{Architecture, Environment, Platform};
use std::collections::HashMap;
use std::path::PathBuf;

/// Mock environment for testing.
pub struct MockEnvironment {
    vars: HashMap<String, String>,
    home: Option<PathBuf>,
    cwd: Option<PathBuf>,
    platform: Platform,
    architecture: Architecture,
}

impl MockEnvironment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            home: Some(PathBuf::from("/home/test")),
            cwd: Some(PathBuf::from("/project")),
            platform: Platform::Linux,
            architecture: Architecture::Amd64,
        }
    }

    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_home(mut self, path: &str) -> Self {
        self.home = Some(PathBuf::from(path));
        self
    }

    pub fn with_home_none(mut self) -> Self {
        self.home = None;
        self
    }

    pub fn with_current_dir(mut self, path: &str) -> Self {
        self.cwd = Some(PathBuf::from(path));
        self
    }

    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Set the architecture for this mock environment.
    pub fn with_architecture(mut self, architecture: Architecture) -> Self {
        self.architecture = architecture;
        self
    }
}

impl Default for MockEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment for MockEnvironment {
    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).cloned()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn current_dir(&self) -> Option<PathBuf> {
        self.cwd.clone()
    }

    fn temp_dir(&self) -> PathBuf {
        PathBuf::from("/tmp")
    }

    fn platform(&self) -> Platform {
        self.platform
    }

    fn architecture(&self) -> Architecture {
        self.architecture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_architecture() {
        let env = MockEnvironment::new().with_architecture(Architecture::Arm64);
        assert_eq!(env.architecture(), Architecture::Arm64);
    }

    /// PC-009: MockEnvironment supports with_current_exe builder and returns it.
    #[test]
    fn mock_env_with_current_exe() {
        let expected = PathBuf::from("/usr/local/bin/ecc");
        let env = MockEnvironment::new().with_current_exe("/usr/local/bin/ecc");
        assert_eq!(env.current_exe(), Some(expected));
    }

    #[test]
    fn mock_env_current_exe_default() {
        let env = MockEnvironment::new();
        // Default should be Some(.../ecc) per design
        assert!(env.current_exe().is_some());
    }
}
