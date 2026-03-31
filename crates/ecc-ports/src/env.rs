use std::path::PathBuf;

// Re-export the canonical Architecture and Platform types from the domain layer.
pub use ecc_domain::update::platform::{Architecture, Platform};

/// PC-008 compile-check: verifies current_exe() exists on the Environment trait.
/// This function is dead code but must compile.
#[doc(hidden)]
#[allow(dead_code)]
fn _pc008_current_exe_check(env: &dyn Environment) -> Option<PathBuf> {
    env.current_exe()
}

/// Port for environment access (env vars, home dir, platform info).
pub trait Environment: Send + Sync {
    /// Return the value of an environment variable, or `None` if unset.
    fn var(&self, name: &str) -> Option<String>;
    /// Return the current user's home directory, or `None` if unavailable.
    fn home_dir(&self) -> Option<PathBuf>;
    /// Return the current working directory, or `None` if unavailable.
    fn current_dir(&self) -> Option<PathBuf>;
    /// Return the system's temporary directory.
    fn temp_dir(&self) -> PathBuf;
    /// Return the host operating system platform.
    fn platform(&self) -> Platform;
    /// Return the host CPU architecture.
    fn architecture(&self) -> Architecture;
    /// Return the path to the current executable, or `None` if unavailable.
    fn current_exe(&self) -> Option<PathBuf>;
}

#[cfg(test)]
mod tests {
    use super::{Architecture, Environment, Platform};

    /// PC-007: Architecture and Platform must be the domain types (re-exported).
    /// This test asserts type identity: a domain Architecture is accepted where
    /// a ports Architecture is expected, which only compiles if they are the same type.
    #[test]
    fn architecture_is_domain_type() {
        fn accept_domain_arch(a: ecc_domain::update::platform::Architecture) -> Architecture {
            a
        }
        let _ = accept_domain_arch(ecc_domain::update::platform::Architecture::Arm64);
    }

    #[test]
    fn platform_is_domain_type() {
        fn accept_domain_platform(p: ecc_domain::update::platform::Platform) -> Platform {
            p
        }
        let _ = accept_domain_platform(ecc_domain::update::platform::Platform::Linux);
    }

    /// PC-008: Environment trait must include current_exe() -> Option<PathBuf>.
    #[test]
    fn current_exe_in_trait() {
        fn _assert_current_exe_on_trait(env: &dyn Environment) -> Option<std::path::PathBuf> {
            env.current_exe()
        }
    }
}
