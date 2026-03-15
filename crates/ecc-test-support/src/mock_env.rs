use ecc_ports::env::{Environment, Platform};
use std::collections::HashMap;
use std::path::PathBuf;

/// Mock environment for testing.
pub struct MockEnvironment {
    vars: HashMap<String, String>,
    home: Option<PathBuf>,
    cwd: Option<PathBuf>,
    platform: Platform,
}

impl MockEnvironment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            home: Some(PathBuf::from("/home/test")),
            cwd: Some(PathBuf::from("/project")),
            platform: Platform::Linux,
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
}
