use ecc_domain::config::validate::{
    REQUIRED_PATTERN_SECTIONS, UNSAFE_CODE_PATTERNS, VALID_PATTERN_DIFFICULTIES,
    VALID_PATTERN_LANGUAGES, extract_frontmatter, parse_tool_list,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Context passed to per-file validation helpers to reduce parameter count.
struct ValidationCtx<'a> {
    label: &'a str,
    stem: &'a str,
    expected_category: &'a str,
    content: &'a str,
    all_stems: &'a [String],
}

pub(super) fn validate_patterns(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let patterns_dir = root.join("patterns");
    if !fs.exists(&patterns_dir) {
        terminal.stdout_write("No patterns directory found, skipping validation\n");
        return true;
    }

    let root_entries = match fs.read_dir(&patterns_dir) {
        Ok(e) => e,
        Err(err) => {
            terminal.stderr_write(&format!("ERROR: Cannot read patterns directory: {err}\n"));
            return false;
        }
    };

    warn_root_level_files(&root_entries, fs, terminal);

    let categories: Vec<_> = root_entries.into_iter().filter(|p| fs.is_dir(p)).collect();
    let index_content = read_index_content(&patterns_dir, fs);
    let all_stems = collect_pattern_stems(&categories, fs);

    // Collect per-file work items for parallel validation
    let work_items = collect_work_items(&categories, fs);

    // Parallel pattern file validation via rayon
    let results: Vec<(String, String, String, bool)> = work_items
        .par_iter()
        .map(|(category_name, file_path)| {
            let stem = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let file_label = format!(
                "{}/{}",
                category_name,
                file_path.file_name().unwrap_or_default().to_string_lossy()
            );
            let content = match fs.read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    return (file_label, stem, format!("ERROR: read failed - {e}"), false);
                }
            };

            let mut errors = String::new();
            let mut has_errors = false;

            if !index_content.is_empty() && !stem_in_index(&index_content, &stem) {
                errors.push_str(&format!(
                    "ERROR: {file_label} - pattern '{stem}' is not listed in patterns/index.md\n"
                ));
                has_errors = true;
            }

            let ctx = ValidationCtx {
                label: &file_label,
                stem: &stem,
                expected_category: category_name,
                content: &content,
                all_stems: &all_stems,
            };
            let (file_errors, file_ok) = validate_pattern_file(&ctx);
            errors.push_str(&file_errors);
            if !file_ok {
                has_errors = true;
            }

            (file_label, stem, errors, !has_errors)
        })
        .collect();

    // Aggregate results sequentially (terminal output must be serial)
    let mut has_errors = false;
    let mut file_count: usize = 0;
    for (_label, _stem, errors, ok) in &results {
        file_count += 1;
        if !errors.is_empty() {
            terminal.stderr_write(errors);
        }
        if !ok {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    let category_count = categories
        .iter()
        .filter(|cat| {
            fs.read_dir(cat)
                .map(|entries| {
                    entries.iter().any(|p| {
                        p.extension()
                            .map(|ext| ext.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
        .count();

    terminal.stdout_write(&format!(
        "Validated {} pattern files across {} categories\n",
        file_count, category_count
    ));
    true
}

/// Warn about root-level .md files (not in a category subdir), skip them.
fn warn_root_level_files(
    entries: &[std::path::PathBuf],
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) {
    for entry in entries {
        if !fs.is_dir(entry)
            && entry
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
        {
            let fname = entry
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if fname != "index.md" {
                terminal.stderr_write(&format!(
                    "WARN: {fname} - root-level .md file in patterns/ is not in a category subdirectory, skipping\n"
                ));
            }
        }
    }
}

/// Read the index.md content for coverage checking.
fn read_index_content(patterns_dir: &Path, fs: &dyn FileSystem) -> String {
    let index_path = patterns_dir.join("index.md");
    if fs.exists(&index_path) {
        fs.read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    }
}

/// Collect all pattern stems across all categories for cross-ref resolution.
///
/// For the `idioms` category, recurses one level into language subdirectories
/// (e.g., `idioms/rust/`, `idioms/go/`) and collects stems from those subdirs.
/// Stems include both bare names (`newtype`) and category-prefixed variants
/// (`idioms/newtype`) for disambiguation.
fn collect_pattern_stems(categories: &[std::path::PathBuf], fs: &dyn FileSystem) -> Vec<String> {
    let mut all_stems: Vec<String> = Vec::new();
    for category_path in categories {
        let category_name = category_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dirs_to_scan = if category_name == "idioms" {
            collect_idiom_subdirs(category_path, fs)
        } else {
            vec![category_path.clone()]
        };
        for dir in &dirs_to_scan {
            if let Ok(files) = fs.read_dir(dir) {
                for file_path in files {
                    if file_path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                        && let Some(stem) = file_path.file_stem()
                    {
                        let stem_str = stem.to_string_lossy().to_string();
                        // Add category-prefixed variant for disambiguation
                        all_stems.push(format!("{category_name}/{stem_str}"));
                        all_stems.push(stem_str);
                    }
                }
            }
        }
    }
    all_stems
}

/// Collect language subdirectories within the idioms category.
fn collect_idiom_subdirs(
    idioms_path: &std::path::Path,
    fs: &dyn FileSystem,
) -> Vec<std::path::PathBuf> {
    let mut subdirs = Vec::new();
    if let Ok(entries) = fs.read_dir(idioms_path) {
        for entry in entries {
            if fs.is_dir(&entry) {
                subdirs.push(entry);
            }
        }
    }
    subdirs
}

/// Collect (category_name, file_path) pairs for all .md files in category dirs.
///
/// For the `idioms` category, recurses into language subdirectories and uses
/// `idioms` as the category_name for all nested files (matching frontmatter convention).
fn collect_work_items(
    categories: &[std::path::PathBuf],
    fs: &dyn FileSystem,
) -> Vec<(String, std::path::PathBuf)> {
    let mut items = Vec::new();
    for category_path in categories {
        let category_name = category_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dirs_to_scan = if category_name == "idioms" {
            collect_idiom_subdirs(category_path, fs)
        } else {
            vec![category_path.clone()]
        };
        for dir in &dirs_to_scan {
            if let Ok(entries) = fs.read_dir(dir) {
                for file_path in entries {
                    if file_path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                    {
                        items.push((category_name.clone(), file_path));
                    }
                }
            }
        }
    }
    items
}

/// Check whether stem appears in the index as a path component (word-boundary-aware).
///
/// Matches patterns like `/factory-method.md`, `(factory-method)`, or the stem
/// preceded by a word boundary character and followed by `.md`, `)`, or whitespace.
fn stem_in_index(index_content: &str, stem: &str) -> bool {
    let boundary_before = |c: char| -> bool {
        matches!(c, '/' | '(' | '[' | ' ' | '\t' | '\n' | '-') || c == '`'
    };
    let boundary_after = |s: &str| -> bool {
        s.is_empty()
            || s.starts_with(".md")
            || s.starts_with(')')
            || s.starts_with(']')
            || s.starts_with(char::is_whitespace)
            || s.starts_with('`')
    };

    let mut search_from = 0;
    while let Some(pos) = index_content[search_from..].find(stem) {
        let abs_pos = search_from + pos;
        let before_ok =
            abs_pos == 0 || boundary_before(index_content.as_bytes()[abs_pos - 1] as char);
        let after_start = abs_pos + stem.len();
        let after_ok = boundary_after(&index_content[after_start..]);
        if before_ok && after_ok {
            return true;
        }
        search_from = abs_pos + 1;
    }
    false
}

/// Validate a single pattern file. Returns (error_messages, is_valid).
fn validate_pattern_file(ctx: &ValidationCtx<'_>) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let fm = match extract_frontmatter(ctx.content) {
        Some(map) => map,
        None => {
            return (
                format!("ERROR: {} - No frontmatter found\n", ctx.label),
                false,
            );
        }
    };

    let (fm_errs, fm_ok) = validate_frontmatter_fields(&fm, ctx.label, ctx.expected_category);
    errors.push_str(&fm_errs);
    if !fm_ok {
        has_errors = true;
    }

    let (lang_errs, lang_ok) = validate_languages(&fm, ctx.content, ctx.label);
    errors.push_str(&lang_errs);
    if !lang_ok {
        has_errors = true;
    }

    let (diff_errs, diff_ok) = validate_difficulty(&fm, ctx.label);
    errors.push_str(&diff_errs);
    if !diff_ok {
        has_errors = true;
    }

    let (ref_errs, ref_ok) = validate_cross_refs(&fm, ctx.stem, ctx.all_stems, ctx.label);
    errors.push_str(&ref_errs);
    if !ref_ok {
        has_errors = true;
    }

    errors.push_str(&scan_unsafe_code(&fm, ctx.content, ctx.label));

    // Warn (but don't error) if file exceeds recommended size
    let line_count = ctx.content.lines().count();
    if line_count > ecc_domain::config::validate::PATTERN_SIZE_WARNING_LINES {
        errors.push_str(&format!(
            "WARN: {} - File has {} lines (exceeds {} recommended max)\n",
            ctx.label,
            line_count,
            ecc_domain::config::validate::PATTERN_SIZE_WARNING_LINES,
        ));
    }

    let (sec_errs, sec_ok) = validate_sections(ctx.content, ctx.label);
    errors.push_str(&sec_errs);
    if !sec_ok {
        has_errors = true;
    }

    (errors, !has_errors)
}

/// Check required frontmatter fields and category match.
fn validate_frontmatter_fields(
    fm: &HashMap<String, String>,
    label: &str,
    expected_category: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    for field in &["name", "category", "tags", "languages", "difficulty"] {
        match fm.get(*field) {
            Some(v) if !v.trim().is_empty() => {}
            _ => {
                errors.push_str(&format!(
                    "ERROR: {label} - Missing required frontmatter field '{field}'\n"
                ));
                has_errors = true;
            }
        }
    }

    if let Some(cat) = fm.get("category")
        && !cat.trim().is_empty()
        && cat.trim() != expected_category
    {
        errors.push_str(&format!(
            "ERROR: {label} - category frontmatter '{cat}' does not match directory '{expected_category}'\n"
        ));
        has_errors = true;
    }

    (errors, !has_errors)
}

/// Validate the languages list and implementation heading matching.
fn validate_languages(
    fm: &HashMap<String, String>,
    content: &str,
    label: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let Some(raw_langs) = fm.get("languages") else {
        return (errors, true);
    };

    let langs = parse_tool_list(raw_langs.trim());
    if langs.is_empty() {
        errors.push_str(&format!("ERROR: {label} - languages list is empty\n"));
        return (errors, false);
    }

    let is_all = langs.len() == 1 && langs[0] == "all";
    let mut lang_err = false;
    for lang in &langs {
        if !VALID_PATTERN_LANGUAGES.contains(&lang.as_str()) {
            errors.push_str(&format!(
                "ERROR: {label} - unrecognized language '{lang}'\n"
            ));
            lang_err = true;
            has_errors = true;
        }
    }

    if !is_all && !lang_err {
        let impl_headings = extract_impl_headings(content);
        for heading_lang in &impl_headings {
            let heading_lower = heading_lang.to_lowercase();
            if !langs.iter().any(|l| l.to_lowercase() == heading_lower) {
                errors.push_str(&format!(
                    "ERROR: {label} - Language Implementations heading '### {heading_lang}' not listed in frontmatter languages\n"
                ));
                has_errors = true;
            }
        }
    }

    (errors, !has_errors)
}

/// Validate the difficulty field value.
fn validate_difficulty(fm: &HashMap<String, String>, label: &str) -> (String, bool) {
    if let Some(diff) = fm.get("difficulty")
        && !diff.trim().is_empty()
        && !VALID_PATTERN_DIFFICULTIES.contains(&diff.trim())
    {
        return (
            format!("ERROR: {label} - unrecognized difficulty '{diff}'\n"),
            false,
        );
    }
    (String::new(), true)
}

/// Validate cross-references in related-patterns.
fn validate_cross_refs(
    fm: &HashMap<String, String>,
    stem: &str,
    all_stems: &[String],
    label: &str,
) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    let Some(raw_refs) = fm.get("related-patterns") else {
        return (errors, true);
    };

    let refs = parse_tool_list(raw_refs.trim());
    for ref_name in &refs {
        if ref_name == stem {
            errors.push_str(&format!(
                "WARN: {label} - self-reference in related-patterns: '{ref_name}'\n"
            ));
        } else if !all_stems.iter().any(|s| s == ref_name) {
            errors.push_str(&format!(
                "ERROR: {label} - cross-reference to non-existent pattern '{ref_name}'\n"
            ));
            has_errors = true;
        }
    }

    (errors, !has_errors)
}

/// Scan code blocks for unsafe code patterns. Returns warning messages only.
fn scan_unsafe_code(fm: &HashMap<String, String>, content: &str, label: &str) -> String {
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

/// Check that all required sections are present and have non-empty bodies.
fn validate_sections(content: &str, label: &str) -> (String, bool) {
    let mut errors = String::new();
    let mut has_errors = false;

    for &section in REQUIRED_PATTERN_SECTIONS {
        let heading = format!("## {section}");
        if !content.contains(&heading) {
            errors.push_str(&format!(
                "ERROR: {label} - Missing required section '{section}'\n"
            ));
            has_errors = true;
        } else if section_body_is_empty(content, section) {
            errors.push_str(&format!(
                "ERROR: {label} - Section '{section}' has empty body\n"
            ));
            has_errors = true;
        }
    }

    (errors, !has_errors)
}

/// Extract all `### <Name>` headings found under `## Language Implementations`.
fn extract_impl_headings(content: &str) -> Vec<String> {
    let section_heading = "## Language Implementations";
    let Some(start) = content.find(section_heading) else {
        return Vec::new();
    };
    let after = &content[start + section_heading.len()..];
    // Scope to this section only (up to next ## heading)
    let section_body = match after.find("\n## ") {
        Some(end) => &after[..end],
        None => after,
    };
    section_body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("### ").map(|rest| rest.trim().to_string())
        })
        .collect()
}

/// Collect all text inside code blocks (between ``` markers) in the content.
fn scan_code_blocks(content: &str) -> String {
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

/// Returns true if the `## <section>` heading is present but its body is empty.
///
/// "Empty" means no non-whitespace content before the next `## ` heading or end of file.
fn section_body_is_empty(content: &str, section: &str) -> bool {
    let heading = format!("## {section}");
    let Some(start) = content.find(&heading) else {
        return false; // not present — separate check handles missing sections
    };
    // Advance past the heading line
    let after_heading = &content[start + heading.len()..];
    // Find the end of the section: next `## ` or end of string
    let body = match after_heading.find("\n## ") {
        Some(next) => &after_heading[..next],
        None => after_heading,
    };
    body.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    /// Build a valid pattern file content string with all required fields and sections.
    fn valid_pattern_content() -> String {
        r#"---
name: factory-method
category: creational
tags: [design-pattern, gof, creational]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent
Provides an interface for creating objects in a superclass.

## Problem
Direct object creation couples clients to concrete classes.

## Solution
Define a factory method in the base class.

## Language Implementations
### Rust
```rust
fn create() -> Box<dyn Product> { Box::new(ConcreteProduct) }
```

## When to Use
- When you don't know ahead of time what class you want to instantiate.

## When NOT to Use
- When there's only one concrete implementation.

## Anti-Patterns
- God factory that knows about all products.

## Related Patterns
- Abstract Factory is a generalization of this pattern.

## References
- GoF Design Patterns, page 107.
"#
        .to_string()
    }

    #[test]
    fn no_patterns_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(result);
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("skipping validation"))
        );
    }

    #[test]
    fn empty_dir_succeeds() {
        let fs = InMemoryFileSystem::new().with_dir("/root/patterns");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(result);
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("0 pattern files across 0 categories"))
        );
    }

    #[test]
    fn valid_pattern_passes() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file(
                "/root/patterns/creational/factory-method.md",
                &valid_pattern_content(),
            );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected valid pattern to pass. stderr: {:?}",
            t.stderr_output()
        );
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("1 pattern files across 1 categories"))
        );
    }

    #[test]
    fn missing_category_field_errors() {
        let content = r#"---
name: factory-method
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Some intent text here that is non-empty.

## Problem
Problem text.

## Solution
Solution text.

## Language Implementations
### Rust
Implementation.

## When to Use
- Some use case.

## When NOT to Use
- Some counter case.

## Anti-Patterns
- Bad pattern.

## Related Patterns
- Other pattern.

## References
- Some book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected missing category to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("category")),
            "Expected ERROR about 'category', got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn category_dir_mismatch_errors() {
        let content = r#"---
name: factory-method
category: structural
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Some intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter case.

## Anti-Patterns
- Bad pattern.

## Related Patterns
- Other.

## References
- Book.
"#;
        // File is in creational/ but category frontmatter says structural
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected category mismatch to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") || s.contains("mismatch") || s.contains("category")),
            "Expected error about category mismatch, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn missing_section_errors() {
        // Missing the "Problem" section
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Some intent.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter case.

## Anti-Patterns
- Bad pattern.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected missing section to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("Problem")),
            "Expected ERROR about missing 'Problem' section, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn empty_section_body_errors() {
        // The "Intent" section has no body (followed immediately by another section)
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent

## Problem
Problem text.

## Solution
Solution text.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter case.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected empty section body to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("Intent")),
            "Expected ERROR about empty 'Intent' section, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn language_implementations_section_required() {
        // Missing the "Language Implementations" section entirely
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected missing Language Implementations to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("Language Implementations")),
            "Expected ERROR about missing 'Language Implementations' section, got: {:?}",
            t.stderr_output()
        );
    }

    // Helper: pattern content with innerHTML (unsafe code) in a code block
    fn unsafe_code_content(suppress: bool) -> String {
        // Note: the string below deliberately tests the unsafe-code scanner.
        // The unsafe marker used is "innerHTML" which is an XSS risk in real code
        // but is intentional here as test data for the scanner.
        let suppress_line = if suppress {
            "unsafe-examples: true\n"
        } else {
            ""
        };
        // Build the unsafe marker via concatenation to document intent
        let unsafe_marker = ["inner", "HTML"].concat();
        format!(
            r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
{suppress_line}---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
```javascript
el.{unsafe_marker} = userInput;
```

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#
        )
    }

    #[test]
    fn invalid_cross_ref_errors() {
        // related-patterns frontmatter references a pattern that does not exist
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
related-patterns: [non-existent-pattern]
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Non-existent pattern.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected invalid cross-ref to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("non-existent-pattern")),
            "Expected ERROR about 'non-existent-pattern', got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn lang_impl_mismatch_errors() {
        // Language Implementations has ### Go but frontmatter only lists rust
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Rust implementation.

### Go
Go implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected lang impl mismatch to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("Go")),
            "Expected ERROR about 'Go' not in frontmatter, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn languages_all_skips_impl_check() {
        // languages: [all] — any ### headings allowed, no mismatch error
        let content = r#"---
name: hexagonal
category: architecture
tags: [architecture]
languages: [all]
difficulty: advanced
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Any Language
Generic implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/architecture")
            .with_file("/root/patterns/architecture/hexagonal.md", content)
            .with_file("/root/patterns/index.md", "hexagonal\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected languages:[all] to skip impl check. stderr: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn empty_languages_errors() {
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: []
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected empty languages list to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("languages")),
            "Expected ERROR about empty languages, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn invalid_language_errors() {
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust, brainfuck]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Rust impl.

### Brainfuck
BF impl.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected invalid language to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("brainfuck")),
            "Expected ERROR about 'brainfuck', got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn invalid_difficulty_errors() {
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: expert
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected invalid difficulty to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("expert")),
            "Expected ERROR about 'expert', got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn unsafe_code_warns() {
        // Code block contains an unsafe pattern — should emit a WARNING (not error)
        let content = unsafe_code_content(false);
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", &content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected unsafe code to only warn, not fail. stderr: {:?}",
            t.stderr_output()
        );
        let unsafe_marker = ["inner", "HTML"].concat();
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("WARN") && s.contains(&unsafe_marker)),
            "Expected WARN about unsafe pattern, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn unsafe_examples_suppresses() {
        // unsafe-examples: true suppresses the warning
        let content = unsafe_code_content(true);
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", &content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected unsafe-examples:true to suppress warning. stderr: {:?}",
            t.stderr_output()
        );
        let unsafe_marker = ["inner", "HTML"].concat();
        assert!(
            !t.stderr_output()
                .iter()
                .any(|s| s.contains("WARN") && s.contains(&unsafe_marker)),
            "Expected no WARN about unsafe pattern when unsafe-examples:true, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn self_reference_warns() {
        // related-patterns lists the pattern's own name
        let content = r#"---
name: factory-method
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
related-patterns: [factory-method]
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- This pattern itself.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content)
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected self-reference to only warn. stderr: {:?}",
            t.stderr_output()
        );
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("WARN") && s.contains("self")),
            "Expected WARN about self-reference, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn root_level_file_warns() {
        // An .md file directly in patterns/ (not in a category subdir) is warned and skipped
        let valid_content = valid_pattern_content();
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", &valid_content)
            .with_file("/root/patterns/readme.md", "# Root level file\n")
            .with_file("/root/patterns/index.md", "factory-method\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected root-level file to only warn, not fail. stderr: {:?}",
            t.stderr_output()
        );
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("WARN") && s.contains("readme.md")),
            "Expected WARN about root-level 'readme.md', got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn yaml_flow_list_syntax() {
        // Both [rust, go] and ["rust", "go"] flow styles should parse correctly
        let content_bare = r#"---
name: factory-method
category: creational
tags: [design-pattern, gof]
languages: [rust, go]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Rust impl.

### Go
Go impl.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let content_quoted = r#"---
name: abstract-factory
category: creational
tags: ["design-pattern", "gof"]
languages: ["rust", "go"]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Rust impl.

### Go
Go impl.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", content_bare)
            .with_file(
                "/root/patterns/creational/abstract-factory.md",
                content_quoted,
            )
            .with_file(
                "/root/patterns/index.md",
                "factory-method\nabstract-factory\n",
            );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected both flow-list syntaxes to pass. stderr: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn missing_from_index_errors() {
        // Pattern exists on disk but is NOT listed in patterns/index.md
        let content = valid_pattern_content();
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", &content)
            .with_file(
                "/root/patterns/index.md",
                "# Pattern Index\n\n(no patterns listed)\n",
            );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(!result, "Expected missing from index to fail");
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("ERROR") && s.contains("factory-method")),
            "Expected ERROR about 'factory-method' not in index, got: {:?}",
            t.stderr_output()
        );
    }

    #[test]
    fn success_message_counts() {
        // Multiple files across categories — success message shows correct N and M
        let content_a = valid_pattern_content(); // category: creational
        let content_b = r#"---
name: abstract-factory
category: creational
tags: [design-pattern]
languages: [rust]
difficulty: intermediate
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Rust
Implementation.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let content_c = r#"---
name: hexagonal
category: architecture
tags: [architecture]
languages: [all]
difficulty: advanced
---

## Intent
Intent.

## Problem
Problem.

## Solution
Solution.

## Language Implementations
### Any Language
Generic.

## When to Use
- Use case.

## When NOT to Use
- Counter.

## Anti-Patterns
- Bad.

## Related Patterns
- Other.

## References
- Book.
"#;
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_dir("/root/patterns/architecture")
            .with_file("/root/patterns/creational/factory-method.md", &content_a)
            .with_file("/root/patterns/creational/abstract-factory.md", content_b)
            .with_file("/root/patterns/architecture/hexagonal.md", content_c)
            .with_file(
                "/root/patterns/index.md",
                "factory-method\nabstract-factory\nhexagonal\n",
            );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            result,
            "Expected all patterns to pass. stderr: {:?}",
            t.stderr_output()
        );
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("3 pattern files across 2 categories")),
            "Expected '3 pattern files across 2 categories', got: {:?}",
            t.stdout_output()
        );
    }

    #[test]
    fn stem_in_index_word_boundary_match() {
        // "factory" should NOT match when index only contains "factory-method"
        assert!(!super::stem_in_index("factory-method\n", "factory"));
        // "factory-method" should match as a standalone word
        assert!(super::stem_in_index("factory-method\n", "factory-method"));
        // Match when preceded by `/` (path component)
        assert!(super::stem_in_index(
            "- [Factory Method](creational/factory-method.md)",
            "factory-method"
        ));
        // Match when preceded by `(`
        assert!(super::stem_in_index("(factory-method)", "factory-method"));
        // No match when embedded in a longer word
        assert!(!super::stem_in_index("abstract-factory-method-extra\n", "factory-method"));
    }

    /// Build a valid idiom pattern file for a single language.
    fn valid_idiom_content(name: &str, language: &str) -> String {
        format!(
            r#"---
name: {name}
category: idioms
tags: [idiom, {language}]
languages: [{language}]
difficulty: intermediate
---

## Intent
Idiomatic {language} pattern for {name}.

## Problem
Non-idiomatic code reduces readability.

## Solution
Use the {name} pattern.

## Language Implementations
### {lang_title}
```{language}
// example
```

## When to Use
- When writing idiomatic {language} code.

## When NOT to Use
- When the pattern adds unnecessary complexity.

## Anti-Patterns
- Over-applying the pattern.

## Related Patterns
- None.

## References
- {language} documentation.
"#,
            name = name,
            language = language,
            lang_title = match language {
                "rust" => "Rust",
                "go" => "Go",
                "python" => "Python",
                "typescript" => "TypeScript",
                "kotlin" => "Kotlin",
                _ => language,
            }
        )
    }

    #[test]
    fn idiom_subdirectory_recursion_validates_nested_files() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/idioms")
            .with_dir("/root/patterns/idioms/rust")
            .with_dir("/root/patterns/idioms/go")
            .with_file(
                "/root/patterns/idioms/rust/newtype.md",
                &valid_idiom_content("newtype", "rust"),
            )
            .with_file(
                "/root/patterns/idioms/go/functional-options.md",
                &valid_idiom_content("functional-options", "go"),
            )
            .with_file("/root/patterns/index.md", "# Index\n- [Newtype](idioms/rust/newtype.md)\n- [Functional Options](idioms/go/functional-options.md)\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        let output = t.stdout_output().join("\n");
        // Should find 2 files across the idioms category
        assert!(
            result,
            "Validation should pass for valid idiom files in subdirectories. Output:\n{output}"
        );
        assert!(
            output.contains("2 pattern files"),
            "Should report 2 pattern files. Output:\n{output}"
        );
    }

    #[test]
    fn idiom_files_have_category_idioms_in_frontmatter() {
        // An idiom file with category != "idioms" should fail
        let bad_content = valid_idiom_content("newtype", "rust").replace("category: idioms", "category: rust");
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/idioms")
            .with_dir("/root/patterns/idioms/rust")
            .with_file("/root/patterns/idioms/rust/newtype.md", &bad_content)
            .with_file("/root/patterns/index.md", "# Index\n- [Newtype](idioms/rust/newtype.md)\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(
            !result,
            "Validation should fail when idiom file has wrong category"
        );
    }

    #[test]
    fn large_file_emits_size_warning() {
        // Create a valid pattern that exceeds 500 lines
        let mut content = valid_pattern_content();
        // Pad the References section with extra lines to exceed threshold
        for i in 0..500 {
            content.push_str(&format!("- Reference line {i}\n"));
        }
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/factory-method.md", &content)
            .with_file("/root/patterns/index.md", "# Index\n- [Factory Method](creational/factory-method.md)\n");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        // Should still pass (warning, not error)
        assert!(result, "Large file should pass validation with warning");
        let stderr = t.stderr_output().join("\n");
        assert!(
            stderr.contains("WARN") && stderr.contains("lines"),
            "Should emit size warning on stderr. Stderr:\n{stderr}"
        );
    }
}

#[cfg(test)]
mod cross_ref_tests {
    use super::*;

    #[test]
    fn category_prefixed_cross_ref_resolves() {
        let all_stems = vec![
            "ddd/repository".to_string(),
            "repository".to_string(),
            "data-access/repository".to_string(),
        ];
        let mut fm = HashMap::new();
        fm.insert(
            "related-patterns".to_string(),
            "[ddd/repository]".to_string(),
        );
        let (errors, ok) = validate_cross_refs(&fm, "adapter", &all_stems, "test");
        assert!(ok, "Category-prefixed cross-ref should resolve. Errors: {errors}");
        assert!(errors.is_empty(), "No errors expected. Got: {errors}");
    }

    #[test]
    fn bare_cross_ref_still_resolves() {
        let all_stems = vec![
            "creational/factory-method".to_string(),
            "factory-method".to_string(),
        ];
        let mut fm = HashMap::new();
        fm.insert(
            "related-patterns".to_string(),
            "[factory-method]".to_string(),
        );
        let (errors, ok) = validate_cross_refs(&fm, "builder", &all_stems, "test");
        assert!(ok, "Bare cross-ref should still resolve. Errors: {errors}");
    }

    #[test]
    fn non_existent_prefixed_ref_fails() {
        let all_stems = vec!["ddd/repository".to_string(), "repository".to_string()];
        let mut fm = HashMap::new();
        fm.insert(
            "related-patterns".to_string(),
            "[nonexistent/pattern]".to_string(),
        );
        let (errors, ok) = validate_cross_refs(&fm, "adapter", &all_stems, "test");
        assert!(!ok, "Non-existent prefixed ref should fail");
        assert!(errors.contains("non-existent"), "Should mention non-existent: {errors}");
    }
}
