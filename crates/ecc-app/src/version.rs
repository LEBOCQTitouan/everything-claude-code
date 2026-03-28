/// Returns the current ECC version from Cargo metadata.
///
/// If the `ECC_DEV_MODE` environment variable is set at runtime,
/// appends `-dev` to indicate a source-installed (non-release) build.
pub fn version() -> String {
    let base = env!("CARGO_PKG_VERSION");
    if std::env::var("ECC_DEV_MODE").is_ok() {
        format!("{base}-dev")
    } else {
        base.to_owned()
    }
}
