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

#[cfg(test)]
mod tests {
    use super::version;
    use ecc_test_support::MockEnvironment;

    #[test]
    fn version_dev_mode_via_trait() {
        let env = MockEnvironment::new().with_var("ECC_DEV_MODE", "1");
        let result = version(&env);
        assert!(
            result.ends_with("-dev"),
            "expected version to end with -dev, got: {result}"
        );
    }
}
