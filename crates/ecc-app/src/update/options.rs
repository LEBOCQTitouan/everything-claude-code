/// Options for the update operation.
#[derive(Debug, Clone)]
pub struct UpdateOptions {
    /// Target version to install (None = latest).
    pub target_version: Option<String>,
    /// Preview actions without writing anything.
    pub dry_run: bool,
    /// Include prerelease versions when querying latest.
    pub include_prerelease: bool,
}

impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            target_version: None,
            dry_run: false,
            include_prerelease: false,
        }
    }
}
