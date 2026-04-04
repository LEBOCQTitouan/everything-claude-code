use ecc_domain::config::validate::{REQUIRED_PATTERN_SECTIONS, extract_frontmatter};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

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

    let categories = match fs.read_dir(&patterns_dir) {
        Ok(entries) => entries
            .into_iter()
            .filter(|p| fs.is_dir(p))
            .collect::<Vec<_>>(),
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read patterns directory: {e}\n"));
            return false;
        }
    };

    let mut file_count: usize = 0;
    let mut category_count: usize = 0;
    let mut has_errors = false;

    for category_path in &categories {
        let category_name = category_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let files = match fs.read_dir(category_path) {
            Ok(entries) => entries
                .into_iter()
                .filter(|p| {
                    p.extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>(),
            Err(_) => continue,
        };

        if !files.is_empty() {
            category_count += 1;
            file_count += files.len();
        }

        for file_path in &files {
            let file_label = format!(
                "{}/{}",
                category_name,
                file_path.file_name().unwrap_or_default().to_string_lossy()
            );
            let content = match fs.read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    terminal.stderr_write(&format!("ERROR: {file_label} - {e}\n"));
                    has_errors = true;
                    continue;
                }
            };

            if !validate_pattern_file(&file_label, &category_name, &content, terminal) {
                has_errors = true;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!(
        "Validated {} pattern files across {} categories\n",
        file_count, category_count
    ));
    true
}

fn validate_pattern_file(
    label: &str,
    expected_category: &str,
    content: &str,
    terminal: &dyn TerminalIO,
) -> bool {
    let mut has_errors = false;

    // --- Frontmatter validation ---
    let fm = match extract_frontmatter(content) {
        Some(map) => map,
        None => {
            terminal.stderr_write(&format!("ERROR: {label} - No frontmatter found\n"));
            return false;
        }
    };

    for field in &["name", "category", "tags", "languages", "difficulty"] {
        match fm.get(*field) {
            Some(v) if !v.trim().is_empty() => {}
            _ => {
                terminal.stderr_write(&format!(
                    "ERROR: {label} - Missing required frontmatter field '{field}'\n"
                ));
                has_errors = true;
            }
        }
    }

    // Check category matches parent directory
    if let Some(cat) = fm.get("category")
        && !cat.trim().is_empty()
        && cat.trim() != expected_category
    {
        terminal.stderr_write(&format!(
            "ERROR: {label} - category frontmatter '{cat}' does not match directory '{expected_category}'\n"
        ));
        has_errors = true;
    }

    // --- Section validation ---
    for &section in REQUIRED_PATTERN_SECTIONS {
        let heading = format!("## {section}");
        if !content.contains(&heading) {
            terminal.stderr_write(&format!(
                "ERROR: {label} - Missing required section '{section}'\n"
            ));
            has_errors = true;
        } else if section_body_is_empty(content, section) {
            terminal.stderr_write(&format!(
                "ERROR: {label} - Section '{section}' has empty body\n"
            ));
            has_errors = true;
        }
    }

    !has_errors
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
}
