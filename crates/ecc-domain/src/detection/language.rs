/// A rule for detecting a programming language by marker files and extensions.
pub struct LanguageRule {
    pub lang_type: &'static str,
    pub markers: &'static [&'static str],
    pub extensions: &'static [&'static str],
}

/// Language detection rules — 12 entries matching the TypeScript implementation.
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
        markers: &["pom.xml", "build.gradle", "build.gradle.kts"],
        extensions: &[".java"],
    },
    LanguageRule {
        lang_type: "csharp",
        markers: &[],
        extensions: &[".cs", ".csproj", ".sln"],
    },
    LanguageRule {
        lang_type: "swift",
        markers: &["Package.swift"],
        extensions: &[".swift"],
    },
    LanguageRule {
        lang_type: "kotlin",
        markers: &[],
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
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- LANGUAGE_RULES ---

    #[test]
    fn language_rules_has_12_entries() {
        assert_eq!(LANGUAGE_RULES.len(), 12);
    }
}
