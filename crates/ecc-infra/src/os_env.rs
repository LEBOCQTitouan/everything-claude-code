use ecc_ports::env::{Architecture, Environment, Platform};
use std::path::PathBuf;

/// Production environment adapter using `std::env`.
///
/// # Pattern
///
/// Adapter \[Hexagonal Architecture\] — implements `ecc_ports::env::Environment`
pub struct OsEnvironment;

impl Environment for OsEnvironment {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        home_dir_impl()
    }

    fn current_dir(&self) -> Option<PathBuf> {
        std::env::current_dir().ok()
    }

    fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }

    fn platform(&self) -> Platform {
        Platform::current()
    }

    fn architecture(&self) -> Architecture {
        Architecture::current()
    }

    fn current_exe(&self) -> Option<std::path::PathBuf> {
        std::env::current_exe().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::env::Environment;

    // ── PC-036: os_env_current_exe ─────────────────────────────────────────

    #[test]
    fn os_env_current_exe() {
        let env = OsEnvironment;
        // The test binary itself is an executable, so current_exe() must return Some.
        let exe = env.current_exe();
        assert!(
            exe.is_some(),
            "current_exe() must return Some when running inside a test binary"
        );
        let path = exe.unwrap();
        assert!(
            path.is_absolute(),
            "current_exe() must return an absolute path"
        );
    }
}

fn home_dir_impl() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
}
