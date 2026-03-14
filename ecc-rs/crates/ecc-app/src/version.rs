/// Returns the current ECC version from Cargo metadata.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
