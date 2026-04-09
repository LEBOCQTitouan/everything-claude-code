use ecc_domain::config::validate::UNSAFE_CODE_PATTERNS;
use std::collections::HashMap;

/// Scan code blocks for unsafe code patterns. Returns warning messages only.
pub(super) fn scan_unsafe_code(fm: &HashMap<String, String>, content: &str, label: &str) -> String {
    let suppress_unsafe = fm
        .get("unsafe-examples")
        .map(|v| v.trim() == "true")
        .unwrap_or(false);
    if suppress_unsafe {
        return String::new();
    }

    let mut warnings = String::new();
    let in_code_block = scan_code_blocks(content);
    for pattern in UNSAFE_CODE_PATTERNS {
        if in_code_block.contains(*pattern) {
            warnings.push_str(&format!(
                "WARN: {label} - unsafe code pattern '{pattern}' found in code block\n"
            ));
        }
    }
    warnings
}

/// Collect all text inside code blocks (between ``` markers) in the content.
pub(super) fn scan_code_blocks(content: &str) -> String {
    let mut result = String::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
