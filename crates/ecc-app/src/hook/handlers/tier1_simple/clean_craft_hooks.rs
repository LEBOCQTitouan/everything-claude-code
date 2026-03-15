use crate::hook::{HookPorts, HookResult};
use std::path::Path;

use super::helpers::extract_file_path;

/// Domain-layer path indicators.
const DOMAIN_INDICATORS: &[&str] = &[
    "/domain/",
    "/entities/",
    "/ecc-domain/",
    "/core/",
    "/domain\\",
    "/entities\\",
    "/ecc-domain\\",
    "/core\\",
];

/// Infra import patterns that violate the dependency rule.
const INFRA_IMPORT_PATTERNS: &[&str] = &[
    "use crate::infra",
    "use crate::adapter",
    "use crate::infrastructure",
    "use super::infra",
    "use super::adapter",
    "mod infra",
    "mod adapter",
    "from \"../infrastructure",
    "from \"../adapters",
    "from '../infrastructure",
    "from '../adapters",
    "import.*from.*infrastructure",
    "import.*from.*adapters",
];

/// Generic names that indicate poor naming expressiveness.
const GENERIC_NAMES: &[&str] = &[
    "data", "temp", "result", "value", "item", "obj", "flag", "info", "manager", "processor",
    "handler", "helper",
];

/// pre:edit:boundary-crossing — block edits that introduce outward imports in domain files.
pub fn pre_edit_boundary_crossing(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Check if file is in a domain layer
    let is_domain = DOMAIN_INDICATORS
        .iter()
        .any(|ind| file_path.contains(ind));
    if !is_domain {
        return HookResult::passthrough(stdin);
    }

    // Extract edit content from stdin JSON
    let edit_content = extract_edit_content(stdin);
    if edit_content.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Check for infrastructure imports in the edit content
    let has_infra_import = edit_content.lines().any(|line| {
        let trimmed = line.trim();
        INFRA_IMPORT_PATTERNS
            .iter()
            .any(|pat| trimmed.contains(pat))
    });

    if has_infra_import {
        return HookResult::block(
            stdin,
            "[Hook] BLOCKED: This import points outward. Source code dependencies must point inward.\n\
             [Hook] Domain/entity files must not import from infrastructure or adapter layers.\n\
             [Hook] Use a port interface (trait/interface) instead.\n",
        );
    }

    HookResult::passthrough(stdin)
}

/// post:edit:boy-scout-delta — suggest one small improvement near the edit location.
pub fn post_edit_boy_scout_delta(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let path = Path::new(&file_path);
    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut suggestions: Vec<String> = Vec::new();

    // Scan for TODO/FIXME/HACK comments
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim().to_uppercase();
        if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("HACK") {
            suggestions.push(format!(
                "Line {}: {} — consider resolving this TODO/FIXME",
                i + 1,
                line.trim()
            ));
            break;
        }
    }

    // Scan for magic numbers (bare numeric literals > 1 outside of common patterns)
    if suggestions.is_empty() {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments, test files, const declarations
            if trimmed.starts_with("//")
                || trimmed.starts_with('#')
                || trimmed.starts_with("const ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("static ")
            {
                continue;
            }
            if has_magic_number(trimmed) {
                suggestions.push(format!(
                    "Line {}: possible magic number — consider extracting to a named constant",
                    i + 1,
                ));
                break;
            }
        }
    }

    if suggestions.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let msg = format!(
        "[Hook] Boy Scout Delta: {}\n\
         [Hook] \"Always leave the code better than you found it.\" — commit with chore(scout): ...\n",
        suggestions[0]
    );
    HookResult::warn(stdin, &msg)
}

/// post:edit:naming-review — warn about generic identifier names in edit content.
pub fn post_edit_naming_review(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    let edit_content = extract_edit_content(stdin);
    if edit_content.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let mut findings: Vec<String> = Vec::new();

    for line in edit_content.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        // Extract identifiers from the line (simple heuristic)
        for word in extract_identifiers(trimmed) {
            let lower = word.to_lowercase();
            if GENERIC_NAMES.iter().any(|g| lower == *g || lower.ends_with(g)) {
                findings.push(format!("'{}' — consider a more descriptive name", word));
                if findings.len() >= 3 {
                    break;
                }
            }
        }
        if findings.len() >= 3 {
            break;
        }
    }

    if findings.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let msg = format!(
        "[Hook] Naming review: generic name(s) detected:\n{}\n\
         [Hook] Names should reveal intent. Consider domain-specific alternatives.\n",
        findings
            .iter()
            .map(|f| format!("[Hook]   {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    );
    HookResult::warn(stdin, &msg)
}

/// post:edit:newspaper-check — warn if private functions appear before public ones.
pub fn post_edit_newspaper_check(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Only check source files
    if !matches!(
        ext.as_str(),
        "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "py" | "java"
    ) {
        return HookResult::passthrough(stdin);
    }

    let path = Path::new(&file_path);
    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let violation = check_newspaper_ordering(&content, &ext);

    if let Some(msg) = violation {
        HookResult::warn(
            stdin,
            &format!(
                "[Hook] Newspaper Rule: {}\n\
                 [Hook] Public functions should appear before private ones (high-level first, details later).\n",
                msg
            ),
        )
    } else {
        HookResult::passthrough(stdin)
    }
}

/// pre:edit:stepdown-warning — warn if edited content mixes abstraction levels.
pub fn pre_edit_stepdown_warning(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    let edit_content = extract_edit_content(stdin);
    if edit_content.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let mut high_level_count = 0u32;
    let mut low_level_count = 0u32;

    for line in edit_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        // High-level indicators: named function calls
        if contains_function_call(trimmed) {
            high_level_count += 1;
        }

        // Low-level indicators: array indexing, bitwise ops, raw string manipulation
        if contains_low_level_ops(trimmed) {
            low_level_count += 1;
        }
    }

    // Only warn if both are present above threshold
    if high_level_count >= 2 && low_level_count >= 2 {
        return HookResult::warn(
            stdin,
            "[Hook] Stepdown Rule: this edit mixes abstraction levels.\n\
             [Hook] High-level function calls and low-level operations (indexing, bitwise, string manipulation) appear together.\n\
             [Hook] Consider extracting low-level details into well-named helper functions.\n",
        );
    }

    HookResult::passthrough(stdin)
}

// --- Internal helpers ---

/// Extract edit content (new_string or content) from stdin JSON.
fn extract_edit_content(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            let ti = v.get("tool_input")?;
            ti.get("new_string")
                .or_else(|| ti.get("content"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Check if a line contains a magic number (numeric literal > 1, not 0/1 constants).
fn has_magic_number(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        // Skip string literals
        if ch == '"' || ch == '\'' {
            for next in chars.by_ref() {
                if next == ch {
                    break;
                }
            }
            continue;
        }
        // Look for standalone numeric literals
        if ch.is_ascii_digit() && ch != '0' && ch != '1' {
            // Check that previous char is not alphanumeric (part of identifier)
            // This is a simple heuristic
            let mut num_str = String::new();
            num_str.push(ch);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' || next == '_' {
                    num_str.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            // Exclude common non-magic numbers
            if (num_str != "2" || line.contains("/ 2") || line.contains("* 2"))
                && num_str.replace('_', "").parse::<f64>().is_ok_and(|n| {
                    n > 1.0 && !line.contains("0x") && !line.contains("0b")
                })
            {
                return true;
            }
        }
    }
    false
}

/// Extract identifiers from a line of code (simple word extraction).
fn extract_identifiers(line: &str) -> Vec<&str> {
    line.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty() && w.len() > 1 && w.chars().next().is_some_and(|c| c.is_alphabetic()))
        .collect()
}

/// Check newspaper ordering — public before private in the same scope.
fn check_newspaper_ordering(content: &str, ext: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut last_public_line: Option<usize> = None;
    let mut first_private_before_public: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let (is_public, is_private) = match ext {
            "rs" => (
                trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn "),
                trimmed.starts_with("fn ") && !trimmed.starts_with("fn main"),
            ),
            "ts" | "tsx" | "js" | "jsx" => (
                trimmed.starts_with("export "),
                (trimmed.starts_with("function ")
                    || trimmed.starts_with("async function ")
                    || trimmed.starts_with("const "))
                    && !trimmed.starts_with("export"),
            ),
            _ => (false, false),
        };

        if is_private && last_public_line.is_none() && first_private_before_public.is_none() {
            first_private_before_public = Some(i);
        }
        if is_public {
            if let Some(priv_line) = first_private_before_public {
                return Some(format!(
                    "private function at line {} appears before public function at line {}",
                    priv_line + 1,
                    i + 1
                ));
            }
            last_public_line = Some(i);
        }
    }

    None
}

/// Check if a line contains a named function call (high-level indicator).
fn contains_function_call(line: &str) -> bool {
    // Simple heuristic: word followed by ( that isn't a keyword
    let keywords = [
        "if", "for", "while", "match", "return", "let", "const", "var", "fn", "pub",
    ];
    for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if word.is_empty() || keywords.contains(&word) {
            continue;
        }
        // Check if this word is followed by ( in the original line
        if let Some(pos) = line.find(word) {
            let after = &line[pos + word.len()..];
            if after.starts_with('(') && word.len() > 1 {
                return true;
            }
        }
    }
    false
}

/// Check if a line contains low-level operations (array indexing, bitwise, raw string ops).
fn contains_low_level_ops(line: &str) -> bool {
    // Array indexing
    if line.contains('[') && line.contains(']') && !line.contains("//") {
        return true;
    }
    // Bitwise operations
    if line.contains(" & ") || line.contains(" | ") || line.contains(" ^ ") || line.contains(" << ") || line.contains(" >> ") {
        return true;
    }
    // Raw string manipulation
    if line.contains(".chars()") || line.contains(".bytes()") || line.contains(".char_at") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
        }
    }

    // --- pre_edit_boundary_crossing ---

    #[test]
    fn boundary_crossing_blocks_domain_file_with_infra_import() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"crates/ecc-domain/src/model.rs","new_string":"use crate::infra::database;\nfn query() {}"}}"#;
        let result = pre_edit_boundary_crossing(stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("points outward"));
    }

    #[test]
    fn boundary_crossing_allows_domain_file_with_domain_import() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/domain/order.rs","new_string":"use super::value_objects::Money;\nfn price() {}"}}"#;
        let result = pre_edit_boundary_crossing(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn boundary_crossing_allows_non_domain_file() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/adapters/http.rs","new_string":"use crate::infra::server;\n"}}"#;
        let result = pre_edit_boundary_crossing(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn boundary_crossing_passthrough_on_empty_path() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_edit_boundary_crossing("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_boy_scout_delta ---

    #[test]
    fn boy_scout_suggests_on_todo_near_edit() {
        let fs = InMemoryFileSystem::new().with_file(
            "src/lib.rs",
            "fn foo() {\n    // TODO: refactor this\n    let x = 1;\n}\n",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_boy_scout_delta(stdin, &ports);
        assert!(result.stderr.contains("Boy Scout Delta"));
        assert!(result.stderr.contains("TODO"));
    }

    #[test]
    fn boy_scout_passthrough_on_clean_file() {
        let fs = InMemoryFileSystem::new().with_file(
            "src/lib.rs",
            "/// Well documented\npub fn calculate_total(items: &[Item]) -> Money {\n    items.iter().map(|i| i.price()).sum()\n}\n",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_boy_scout_delta(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn boy_scout_passthrough_on_missing_file() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"nonexistent.rs"}}"#;
        let result = post_edit_boy_scout_delta(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_naming_review ---

    #[test]
    fn naming_review_warns_on_generic_names() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs","new_string":"let tempData = fetch_items();\nlet result = process(tempData);\n"}}"#;
        let result = post_edit_naming_review(stdin, &ports);
        assert!(result.stderr.contains("Naming review"));
        assert!(result.stderr.contains("more descriptive name"));
    }

    #[test]
    fn naming_review_passthrough_on_good_names() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs","new_string":"let invoice_line_items = fetch_invoices();\nlet total_amount = calculate_sum(invoice_line_items);\n"}}"#;
        let result = post_edit_naming_review(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn naming_review_passthrough_on_empty_content() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = post_edit_naming_review("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_newspaper_check ---

    #[test]
    fn newspaper_check_warns_when_private_before_public() {
        let fs = InMemoryFileSystem::new().with_file(
            "src/lib.rs",
            "fn private_helper() {}\n\npub fn public_api() {}\n",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_newspaper_check(stdin, &ports);
        assert!(result.stderr.contains("Newspaper Rule"));
        assert!(result.stderr.contains("private function"));
    }

    #[test]
    fn newspaper_check_passes_correct_ordering() {
        let fs = InMemoryFileSystem::new().with_file(
            "src/lib.rs",
            "pub fn public_api() {}\n\nfn private_helper() {}\n",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_newspaper_check(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn newspaper_check_passthrough_for_non_source() {
        let fs = InMemoryFileSystem::new().with_file("README.md", "# Title\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = post_edit_newspaper_check(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- pre_edit_stepdown_warning ---

    #[test]
    fn stepdown_warns_on_mixed_abstraction() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs","new_string":"let result = calculate_total(items);\nlet first = items[0];\nlet mask = flags & 0xFF;\nlet output = format_report(result);\nlet byte = data[offset];\n"}}"#;
        let result = pre_edit_stepdown_warning(stdin, &ports);
        assert!(result.stderr.contains("Stepdown Rule"));
        assert!(result.stderr.contains("mixes abstraction levels"));
    }

    #[test]
    fn stepdown_passes_uniform_level() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs","new_string":"let orders = fetch_orders();\nlet total = calculate_total(orders);\nlet report = generate_report(total);\n"}}"#;
        let result = pre_edit_stepdown_warning(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn stepdown_passthrough_on_empty_content() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_edit_stepdown_warning("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- helper tests ---

    #[test]
    fn extract_edit_content_from_new_string() {
        let json = r#"{"tool_input":{"new_string":"let x = 1;"}}"#;
        assert_eq!(extract_edit_content(json), "let x = 1;");
    }

    #[test]
    fn extract_edit_content_from_content() {
        let json = r#"{"tool_input":{"content":"fn main() {}"}}"#;
        assert_eq!(extract_edit_content(json), "fn main() {}");
    }

    #[test]
    fn extract_edit_content_empty() {
        assert_eq!(extract_edit_content("{}"), "");
    }

    #[test]
    fn check_newspaper_ordering_rs_violation() {
        let content = "fn private() {}\n\npub fn public_api() {}\n";
        let result = check_newspaper_ordering(content, "rs");
        assert!(result.is_some());
        assert!(result.unwrap().contains("private function"));
    }

    #[test]
    fn check_newspaper_ordering_rs_correct() {
        let content = "pub fn public_api() {}\n\nfn private() {}\n";
        let result = check_newspaper_ordering(content, "rs");
        assert!(result.is_none());
    }

    #[test]
    fn check_newspaper_ordering_ts_violation() {
        let content = "function helper() {}\n\nexport function api() {}\n";
        let result = check_newspaper_ordering(content, "ts");
        assert!(result.is_some());
    }

    #[test]
    fn contains_function_call_detects_calls() {
        assert!(contains_function_call("calculate_total(items)"));
        assert!(contains_function_call("let x = foo(bar)"));
    }

    #[test]
    fn contains_function_call_ignores_keywords() {
        assert!(!contains_function_call("if (condition)"));
        assert!(!contains_function_call("for (item)"));
    }

    #[test]
    fn contains_low_level_ops_detects_indexing() {
        assert!(contains_low_level_ops("let x = items[0]"));
    }

    #[test]
    fn contains_low_level_ops_detects_bitwise() {
        assert!(contains_low_level_ops("let mask = flags & 0xFF"));
    }

    #[test]
    fn contains_low_level_ops_clean_line() {
        assert!(!contains_low_level_ops("let total = calculate(items)"));
    }
}
