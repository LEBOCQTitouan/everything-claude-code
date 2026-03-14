use ecc_ports::env::{Environment, Platform};
use std::path::PathBuf;

/// Production environment adapter using `std::env`.
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
