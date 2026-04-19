//! ERR-002 audit assertion: no silent `let _ =` error suppressions in delta_helpers.rs.

#[test]
fn no_silent_error_suppressions() {
    const SOURCE: &str = include_str!("delta_helpers.rs");

    // Production code (skip #[cfg(test)])
    let production = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);

    // Build forbidden patterns via concat! to avoid self-match.
    // The codebase accesses I/O through `ports.fs`, `ports.shell`, etc.
    let forbidden = [
        concat!("let ", "_ = ", "fs."),
        concat!("let ", "_ = ", "shell."),
        concat!("let ", "_ = ", "port."),
        concat!("let ", "_ = ", "ports.fs."),
        concat!("let ", "_ = ", "ports.shell."),
        concat!("let ", "_ = ", "ports."),
    ];

    for pat in forbidden {
        assert!(
            !production.contains(pat),
            "delta_helpers.rs contains silent error suppression: `{pat}`"
        );
    }
}
