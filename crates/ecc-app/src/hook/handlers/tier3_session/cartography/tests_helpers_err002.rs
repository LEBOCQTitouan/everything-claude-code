//! ERR-002 audit assertion: no silent `let _ =` error suppressions in delta_helpers.rs.
//! PC-031: no `#[allow(dead_code)]` at module scope in delta_helpers.rs.

/// PC-031: module must not suppress dead-code warnings — remove the attribute AND the dead code.
#[test]
fn no_dead_code_allow() {
    const SOURCE: &str = include_str!("delta_helpers.rs");

    // Production code must not have #[allow(dead_code)] at module scope
    // Check only the first 20 lines (module-level attributes are at top)
    let first_lines: String = SOURCE.lines().take(20).collect::<Vec<_>>().join("\n");
    assert!(
        !first_lines.contains("#![allow(dead_code") &&
        !first_lines.contains("#[allow(dead_code"),
        "module must not use #[allow(dead_code)] — remove the attribute AND the dead code it gates"
    );
}

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
