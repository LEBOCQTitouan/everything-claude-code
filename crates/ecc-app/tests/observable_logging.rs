/// PC-004 — Error-discard sites emit log::warn! (programmatic test).
///
/// Scans specific ecc-app source files listed in US-001 AC-001.4 for bare
/// `Err(_) =>` patterns that discard errors silently without a corresponding
/// `log::warn!`.
///
/// Only the files listed in the US-001 design's File Changes table are checked.
/// Other files are out of scope for this pass condition.

#[test]
fn no_bare_error_discards() {
    // Files specified in the US-001 design's File Changes table (items 6-14).
    // Only these files are checked — they are the ones that need to be fixed.
    let files_to_check = [
        "src/config/audit/mod.rs",
        "src/config/clean.rs",
        "src/config/merge.rs",
        "src/detect.rs",
        "src/install/helpers/settings.rs",
        "src/merge/mod.rs",
        "src/smart_merge.rs",
        "src/hook/handlers/tier1_simple/helpers.rs",
        "src/hook/handlers/tier3_session/helpers.rs",
    ];

    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations: Vec<String> = Vec::new();

    for relative_path in &files_to_check {
        let full_path = crate_root.join(relative_path);
        check_file(&full_path, relative_path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "Found bare error-discard sites (Err arm without log::warn!) in US-001 target files:\n{}",
        violations.join("\n")
    );
}

/// Check a single Rust source file for bare error discards.
///
/// Strategy: find lines containing an `Err(` match arm pattern (`=>`).
/// Then look ahead in the same arm for `log::warn!`.
/// If `log::warn!` is absent, report a violation.
fn check_file(path: &std::path::Path, display: &str, violations: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        // If the file doesn't exist, skip it (it may have been removed)
        return;
    };
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if !is_err_arm(trimmed) {
            continue;
        }

        // Look at this line and the next lines until arm end for `log::warn!`
        let arm_end = find_arm_end(&lines, i);
        let arm_slice = &lines[i..=arm_end];

        let has_warn = arm_slice
            .iter()
            .any(|l| l.contains("log::warn!") || l.contains("warn!("));

        // Not a violation if the arm re-raises as Err(...), records to .errors/.push(), or
        // captures in a structured error field using format!.
        let is_recording_or_reraise = arm_slice.iter().any(|l| {
            l.contains("report.errors.push(")
                || l.contains(".errors.push(")
                || l.contains("error: Some(format!")
                || l.contains("error: Some(e.to_string")
                || (l.contains("Err(") && (l.contains("format!") || l.contains("return Err")))
        });

        if !has_warn && !is_recording_or_reraise {
            violations.push(format!("  {}:{}: {}", display, i + 1, trimmed));
        }
    }
}

/// Detect a match arm that catches an Err and discards it without propagation.
///
/// Returns true for patterns like:
/// - `Err(_) => ...`
/// - `Err(e) => ...`
/// - `Err(_e) => ...`
///
/// Returns false for:
/// - Lines that propagate with `?`
/// - `if let Err` / `while let Err` guards
/// - Comments
fn is_err_arm(line: &str) -> bool {
    if !line.contains("Err(") || !line.contains("=>") {
        return false;
    }
    if line.starts_with("//") || line.starts_with("*") || line.starts_with("///") {
        return false;
    }
    if line.contains("if let Err") || line.contains("while let Err") {
        return false;
    }
    // Lines that propagate the error are not discards
    if line.contains("return Err(") {
        return false;
    }
    // Find the binding inside Err(...)
    let err_idx = match line.find("Err(") {
        Some(i) => i,
        None => return false,
    };
    let rest = &line[err_idx + 4..]; // after "Err("
    let close = match rest.find(')') {
        Some(i) => i,
        None => return false,
    };
    let inner = &rest[..close]; // the binding pattern

    // Must be a simple binding (_, e, _e, some_var)
    let is_binding = inner == "_"
        || inner.starts_with('_')
        || (inner.chars().all(|c| c.is_alphanumeric() || c == '_') && !inner.is_empty());

    if !is_binding {
        return false;
    }

    // Confirm `=>` follows the Err(...) pattern
    let after_err = &line[err_idx..];
    after_err.contains("=>")
}

/// Find the (approximate) end line of a match arm body.
///
/// Scans forward from `start_line`, counting unmatched `{` / `}`.
fn find_arm_end(lines: &[&str], start_line: usize) -> usize {
    let mut depth = 0i32;
    let mut found_open = false;

    for (i, line) in lines[start_line..].iter().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    found_open = true;
                }
                '}' => {
                    depth -= 1;
                }
                _ => {}
            }
        }
        if found_open && depth <= 0 {
            return start_line + i;
        }
        // For one-liner arms without braces, stop at 10 lines ahead
        if !found_open && i >= 10 {
            return start_line + i;
        }
    }

    lines.len().saturating_sub(1)
}
