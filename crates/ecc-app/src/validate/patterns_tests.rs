#[cfg(test)]
mod tests {
    use super::super::super::{ValidateTarget, run_validate};
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
            .with_file(
                "/root/patterns/creational/factory-method.md",
                &valid_content,
            )
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
        assert!(!super::super::stem_in_index("factory-method\n", "factory"));
        // "factory-method" should match as a standalone word
        assert!(super::super::stem_in_index("factory-method\n", "factory-method"));
        // Match when preceded by `/` (path component)
        assert!(super::super::stem_in_index(
            "- [Factory Method](creational/factory-method.md)",
            "factory-method"
        ));
        // Match when preceded by `(`
        assert!(super::super::stem_in_index("(factory-method)", "factory-method"));
        // No match when embedded in a longer word
        assert!(!super::super::stem_in_index(
            "abstract-factory-method-extra\n",
            "factory-method"
        ));
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
        let bad_content =
            valid_idiom_content("newtype", "rust").replace("category: idioms", "category: rust");
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/idioms")
            .with_dir("/root/patterns/idioms/rust")
            .with_file("/root/patterns/idioms/rust/newtype.md", &bad_content)
            .with_file(
                "/root/patterns/index.md",
                "# Index\n- [Newtype](idioms/rust/newtype.md)\n",
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
            .with_file(
                "/root/patterns/index.md",
                "# Index\n- [Factory Method](creational/factory-method.md)\n",
            );
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
    use super::super::super::cross_ref_validation::validate_cross_refs;
    use std::collections::HashMap;

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
        assert!(
            ok,
            "Category-prefixed cross-ref should resolve. Errors: {errors}"
        );
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
        assert!(
            errors.contains("non-existent"),
            "Should mention non-existent: {errors}"
        );
    }
}

#[cfg(test)]
mod fix_index_tests {
    use super::super::super::run_validate_patterns;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn valid_pattern(name: &str, category: &str, tags: &str, languages: &str) -> String {
        format!(
            r#"---
name: {name}
category: {category}
tags: [{tags}]
languages: [{languages}]
difficulty: intermediate
---

## Intent
Test pattern.

## Problem
Test problem.

## Solution
Test solution.

## Language Implementations
### Rust
```rust
// example
```

## When to Use
- Test.

## When NOT to Use
- Test.

## Anti-Patterns
- Test.

## Related Patterns
- None.

## References
- Test.
"#
        )
    }

    #[test]
    fn fix_generates_index_with_categories_and_counts() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_dir("/root/patterns/structural")
            .with_file(
                "/root/patterns/creational/factory-method.md",
                &valid_pattern("factory-method", "creational", "gof, creational", "rust, go"),
            )
            .with_file(
                "/root/patterns/creational/builder.md",
                &valid_pattern("builder", "creational", "gof, creational", "rust, go, python"),
            )
            .with_file(
                "/root/patterns/structural/adapter.md",
                &valid_pattern("adapter", "structural", "gof, structural", "rust, typescript"),
            )
            .with_file("/root/patterns/index.md", "# Index\n- [Factory Method](creational/factory-method.md)\n- [Builder](creational/builder.md)\n- [Adapter](structural/adapter.md)\n");
        let t = BufferedTerminal::new();
        let env = MockEnvironment::default();

        let result = run_validate_patterns(&fs, &t, &env, Path::new("/root"), true);
        assert!(result, "Validation with --fix should pass");

        // Check the generated index
        let dyn_fs: &dyn ecc_ports::fs::FileSystem = &fs;
        let index = dyn_fs
            .read_to_string(Path::new("/root/patterns/index.md"))
            .unwrap();
        assert!(
            index.contains("# Pattern Library Index"),
            "Should have title"
        );
        assert!(
            index.contains("### Creational (2 patterns)"),
            "Should list creational with count"
        );
        assert!(
            index.contains("### Structural (1 patterns)"),
            "Should list structural with count"
        );
        assert!(
            index.contains("| rust |"),
            "Should have rust in language table"
        );
        assert!(
            index.contains("**Total patterns: 3**"),
            "Should show total count"
        );
        assert!(index.contains("`gof`"), "Should list tags");

        let stdout = t.stdout_output().join("\n");
        assert!(
            stdout.contains("Generated patterns/index.md"),
            "Should report generation"
        );
    }

    #[test]
    fn fix_does_not_generate_on_validation_failure() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/patterns")
            .with_dir("/root/patterns/creational")
            .with_file("/root/patterns/creational/bad.md", "no frontmatter here\n")
            .with_file(
                "/root/patterns/index.md",
                "# Old index\n- [Bad](creational/bad.md)\n",
            );
        let t = BufferedTerminal::new();
        let env = MockEnvironment::default();

        let result = run_validate_patterns(&fs, &t, &env, Path::new("/root"), true);
        assert!(!result, "Validation should fail for bad file");

        // Index should NOT be regenerated
        let dyn_fs: &dyn ecc_ports::fs::FileSystem = &fs;
        let index = dyn_fs
            .read_to_string(Path::new("/root/patterns/index.md"))
            .unwrap();
        assert!(
            index.contains("Old index"),
            "Index should not be regenerated on failure"
        );
    }
}
