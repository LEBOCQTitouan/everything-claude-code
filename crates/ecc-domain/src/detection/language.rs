/// A rule for detecting a programming language by marker files and extensions.
pub struct LanguageRule {
    pub lang_type: &'static str,
    pub markers: &'static [&'static str],
    pub extensions: &'static [&'static str],
}

/// Language detection rules for all supported programming languages.
pub const LANGUAGE_RULES: &[LanguageRule] = &[
    LanguageRule {
        lang_type: "python",
        markers: &[
            "requirements.txt",
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "Pipfile",
            "poetry.lock",
        ],
        extensions: &[".py"],
    },
    LanguageRule {
        lang_type: "typescript",
        markers: &["tsconfig.json", "tsconfig.build.json"],
        extensions: &[".ts", ".tsx"],
    },
    LanguageRule {
        lang_type: "javascript",
        markers: &["package.json", "jsconfig.json"],
        extensions: &[".js", ".jsx", ".mjs"],
    },
    LanguageRule {
        lang_type: "golang",
        markers: &["go.mod", "go.sum"],
        extensions: &[".go"],
    },
    LanguageRule {
        lang_type: "rust",
        markers: &["Cargo.toml", "Cargo.lock"],
        extensions: &[".rs"],
    },
    LanguageRule {
        lang_type: "ruby",
        markers: &["Gemfile", "Gemfile.lock", "Rakefile"],
        extensions: &[".rb"],
    },
    LanguageRule {
        lang_type: "java",
        // Note: build.gradle.kts is Kotlin DSL — listed under kotlin, not here.
        markers: &["pom.xml", "build.gradle"],
        extensions: &[".java"],
    },
    LanguageRule {
        lang_type: "csharp",
        markers: &["Directory.Build.props", "NuGet.Config"],
        extensions: &[".cs", ".csproj", ".sln"],
    },
    LanguageRule {
        lang_type: "swift",
        markers: &["Package.swift"],
        extensions: &[".swift"],
    },
    LanguageRule {
        lang_type: "kotlin",
        markers: &["build.gradle.kts", "settings.gradle.kts"],
        extensions: &[".kt", ".kts"],
    },
    LanguageRule {
        lang_type: "elixir",
        markers: &["mix.exs"],
        extensions: &[".ex", ".exs"],
    },
    LanguageRule {
        lang_type: "php",
        markers: &["composer.json", "composer.lock"],
        extensions: &[".php"],
    },
    LanguageRule {
        lang_type: "cpp",
        markers: &["CMakeLists.txt", "Makefile", ".clang-format"],
        extensions: &[".c", ".cpp", ".h", ".hpp", ".cc", ".cxx"],
    },
    LanguageRule {
        lang_type: "shell",
        markers: &[],
        extensions: &[".sh", ".bash", ".zsh"],
    },
    LanguageRule {
        lang_type: "yaml",
        markers: &[],
        extensions: &[".yml", ".yaml"],
    },
    LanguageRule {
        lang_type: "json",
        markers: &[],
        extensions: &[".json", ".jsonc"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_rules_has_16_entries() {
        assert_eq!(LANGUAGE_RULES.len(), 16);
    }

    #[test]
    fn all_language_rules_have_nonempty_type_and_extensions() {
        for rule in LANGUAGE_RULES {
            assert!(!rule.lang_type.is_empty(), "lang_type must not be empty");
            assert!(
                !rule.extensions.is_empty(),
                "{} must have at least one extension",
                rule.lang_type
            );
        }
    }

    #[test]
    fn no_duplicate_lang_types() {
        let mut seen = std::collections::HashSet::new();
        for rule in LANGUAGE_RULES {
            assert!(
                seen.insert(rule.lang_type),
                "duplicate lang_type: {}",
                rule.lang_type
            );
        }
    }

    #[test]
    fn no_duplicate_extensions_across_rules() {
        let mut seen = std::collections::HashMap::new();
        for rule in LANGUAGE_RULES {
            for ext in rule.extensions {
                if let Some(prev) = seen.insert(*ext, rule.lang_type) {
                    panic!(
                        "extension '{}' claimed by both '{}' and '{}'",
                        ext, prev, rule.lang_type
                    );
                }
            }
        }
    }

    #[test]
    fn no_marker_overlap_between_language_rules() {
        let mut seen = std::collections::HashMap::new();
        for rule in LANGUAGE_RULES {
            for marker in rule.markers {
                if let Some(prev) = seen.insert(*marker, rule.lang_type) {
                    panic!(
                        "marker '{}' claimed by both '{}' and '{}'",
                        marker, prev, rule.lang_type
                    );
                }
            }
        }
    }

    #[test]
    fn all_extensions_start_with_dot() {
        for rule in LANGUAGE_RULES {
            for ext in rule.extensions {
                assert!(
                    ext.starts_with('.'),
                    "{}: extension '{}' must start with '.'",
                    rule.lang_type,
                    ext
                );
            }
        }
    }
}
