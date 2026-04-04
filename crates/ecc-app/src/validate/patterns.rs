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

    for category in &categories {
        let files = match fs.read_dir(category) {
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
    }

    terminal.stdout_write(&format!(
        "Validated {} pattern files across {} categories\n",
        file_count, category_count
    ));
    true
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
}
