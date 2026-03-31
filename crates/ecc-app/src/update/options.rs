/// Options for the update operation.
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Target version to install (None = latest).
    pub target_version: Option<String>,
    /// Preview actions without writing anything.
    pub dry_run: bool,
    /// Include prerelease versions when querying latest.
    pub include_prerelease: bool,
}


