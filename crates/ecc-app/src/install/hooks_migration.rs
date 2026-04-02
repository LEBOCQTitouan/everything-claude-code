//! hooks.json migration: replace `ecc-hook` references with `ecc hook`.
//!
//! Idempotent — running the migration twice produces the same output.
//! Preserves custom user hooks that do not contain `ecc-hook`.

/// Result of a hooks.json migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// Number of entries that were migrated.
    pub migrated_count: usize,
    /// Number of entries that were already migrated (idempotent no-op).
    pub already_migrated: usize,
    /// Number of custom entries preserved unchanged.
    pub custom_preserved: usize,
}

/// Migrate hooks.json content: replace `ecc-hook ` with `ecc hook `.
///
/// Only modifies entries containing the literal `ecc-hook ` pattern (note trailing space).
/// Custom hooks (not containing `ecc-hook`) are preserved.
/// Idempotent: `ecc hook ` does not match the `ecc-hook ` pattern.
pub fn migrate_hooks_json(content: &str) -> (String, MigrationReport) {
    let ecc_hook_dash_count = content.matches("ecc-hook ").count();
    let ecc_hook_space_count = content.matches("ecc hook ").count();

    let result = content.replace("ecc-hook ", "ecc hook ");

    let mut custom_preserved = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"command\"")
            && !trimmed.contains("ecc-hook")
            && !trimmed.contains("ecc hook")
        {
            custom_preserved += 1;
        }
    }

    let report = MigrationReport {
        migrated_count: ecc_hook_dash_count,
        already_migrated: ecc_hook_space_count,
        custom_preserved,
    };

    (result, report)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn replaces_ecc_hook() {
        let input = r#""command": "ecc-hook \"pre:bash\" \"standard\""#;
        let (output, report) = migrate_hooks_json(input);
        assert!(
            output.contains("ecc hook "),
            "should replace ecc-hook with ecc hook, got: {output}"
        );
        assert!(
            !output.contains("ecc-hook "),
            "should not contain ecc-hook after migration"
        );
        assert_eq!(report.migrated_count, 1);
    }

    #[test]
    fn idempotent() {
        let input = r#""command": "ecc hook \"pre:bash\" \"standard\""#;
        let (output1, report1) = migrate_hooks_json(input);
        let (output2, report2) = migrate_hooks_json(&output1);
        assert_eq!(
            output1, output2,
            "second migration should produce same output"
        );
        assert_eq!(report2.migrated_count, 0, "no new migrations on second run");
        assert_eq!(report1.already_migrated, 1);
    }

    #[test]
    fn preserves_custom_hooks() {
        let input = concat!(
            r#"  "command": "ecc-hook \"pre:bash\" \"standard\""#,
            "\n",
            r#"  "command": "my-custom-tool --check""#,
            "\n",
            r#"  "command": "ecc-hook \"pre:edit\" \"strict\""#,
        );
        let (output, report) = migrate_hooks_json(input);
        assert!(
            output.contains("my-custom-tool --check"),
            "custom hook preserved"
        );
        assert_eq!(report.migrated_count, 2, "two ecc-hook entries migrated");
        assert_eq!(report.custom_preserved, 1, "one custom hook preserved");
    }

    #[test]
    fn safety_gate() {
        let (output, report) = migrate_hooks_json("");
        assert_eq!(output, "");
        assert_eq!(report.migrated_count, 0);
    }
}
