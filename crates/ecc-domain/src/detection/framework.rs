/// A rule for detecting a framework by marker files and package dependencies.
pub struct FrameworkRule {
    pub framework: &'static str,
    pub language: &'static str,
    pub markers: &'static [&'static str],
    pub package_keys: &'static [&'static str],
}

/// Framework detection rules for all supported web/app frameworks.
pub const FRAMEWORK_RULES: &[FrameworkRule] = &[
    // Python frameworks
    FrameworkRule {
        framework: "django",
        language: "python",
        markers: &["manage.py"],
        package_keys: &["django"],
    },
    FrameworkRule {
        framework: "fastapi",
        language: "python",
        markers: &[],
        package_keys: &["fastapi"],
    },
    FrameworkRule {
        framework: "flask",
        language: "python",
        markers: &[],
        package_keys: &["flask"],
    },
    // JavaScript/TypeScript frameworks
    FrameworkRule {
        framework: "nextjs",
        language: "typescript",
        markers: &["next.config.js", "next.config.mjs", "next.config.ts"],
        package_keys: &["next"],
    },
    FrameworkRule {
        framework: "react",
        language: "typescript",
        markers: &[],
        package_keys: &["react"],
    },
    FrameworkRule {
        framework: "vue",
        language: "typescript",
        markers: &["vue.config.js"],
        package_keys: &["vue"],
    },
    FrameworkRule {
        framework: "angular",
        language: "typescript",
        markers: &["angular.json"],
        package_keys: &["@angular/core"],
    },
    FrameworkRule {
        framework: "svelte",
        language: "typescript",
        markers: &["svelte.config.js"],
        package_keys: &["svelte"],
    },
    FrameworkRule {
        framework: "express",
        language: "javascript",
        markers: &[],
        package_keys: &["express"],
    },
    FrameworkRule {
        framework: "nestjs",
        language: "typescript",
        markers: &["nest-cli.json"],
        package_keys: &["@nestjs/core"],
    },
    FrameworkRule {
        framework: "remix",
        language: "typescript",
        markers: &[],
        package_keys: &["@remix-run/node", "@remix-run/react"],
    },
    FrameworkRule {
        framework: "astro",
        language: "typescript",
        markers: &["astro.config.mjs", "astro.config.ts"],
        package_keys: &["astro"],
    },
    FrameworkRule {
        framework: "nuxt",
        language: "typescript",
        markers: &["nuxt.config.js", "nuxt.config.ts"],
        package_keys: &["nuxt"],
    },
    FrameworkRule {
        framework: "electron",
        language: "typescript",
        markers: &[],
        package_keys: &["electron"],
    },
    // Ruby frameworks
    FrameworkRule {
        framework: "rails",
        language: "ruby",
        markers: &["config/routes.rb", "bin/rails"],
        package_keys: &[],
    },
    // Go frameworks
    FrameworkRule {
        framework: "gin",
        language: "golang",
        markers: &[],
        package_keys: &["github.com/gin-gonic/gin"],
    },
    FrameworkRule {
        framework: "echo",
        language: "golang",
        markers: &[],
        package_keys: &["github.com/labstack/echo"],
    },
    // Rust frameworks
    FrameworkRule {
        framework: "actix",
        language: "rust",
        markers: &[],
        package_keys: &["actix-web"],
    },
    FrameworkRule {
        framework: "axum",
        language: "rust",
        markers: &[],
        package_keys: &["axum"],
    },
    // Java frameworks
    FrameworkRule {
        framework: "spring",
        language: "java",
        markers: &[],
        package_keys: &["spring-boot", "org.springframework"],
    },
    // PHP frameworks
    FrameworkRule {
        framework: "laravel",
        language: "php",
        markers: &["artisan"],
        package_keys: &["laravel/framework"],
    },
    FrameworkRule {
        framework: "symfony",
        language: "php",
        markers: &["symfony.lock"],
        package_keys: &["symfony/framework-bundle"],
    },
    // Elixir frameworks
    FrameworkRule {
        framework: "phoenix",
        language: "elixir",
        markers: &[],
        package_keys: &["phoenix"],
    },
    // Kotlin frameworks
    FrameworkRule {
        framework: "ktor",
        language: "kotlin",
        markers: &[],
        package_keys: &["io.ktor"],
    },
    // C#/.NET frameworks
    // Note: markers are AND-combined with the csharp language rule (.cs/.csproj/.sln).
    // appsettings.json alone won't trigger — the project must also have C# extensions.
    FrameworkRule {
        framework: "aspnetcore",
        language: "csharp",
        markers: &["appsettings.json", "Program.cs"],
        package_keys: &["Microsoft.AspNetCore"],
    },
];

/// Result of full project type detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectType {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub primary: String,
    pub project_dir: String,
}

pub const FRONTEND_SIGNALS: &[&str] = &[
    "react", "vue", "angular", "svelte", "nextjs", "nuxt", "astro", "remix", "electron",
];

pub const BACKEND_SIGNALS: &[&str] = &[
    "django",
    "fastapi",
    "flask",
    "express",
    "nestjs",
    "rails",
    "spring",
    "laravel",
    "symfony",
    "phoenix",
    "gin",
    "echo",
    "actix",
    "axum",
    "ktor",
    "aspnetcore",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::language::LANGUAGE_RULES;

    #[test]
    fn framework_rules_has_25_entries() {
        assert_eq!(FRAMEWORK_RULES.len(), 25);
    }

    #[test]
    fn all_framework_rules_have_nonempty_name_and_language() {
        for rule in FRAMEWORK_RULES {
            assert!(
                !rule.framework.is_empty(),
                "framework name must not be empty"
            );
            assert!(
                !rule.language.is_empty(),
                "language must not be empty for {}",
                rule.framework
            );
        }
    }

    #[test]
    fn all_framework_rules_have_markers_or_package_keys() {
        for rule in FRAMEWORK_RULES {
            assert!(
                !rule.markers.is_empty() || !rule.package_keys.is_empty(),
                "framework '{}' must have markers or package_keys",
                rule.framework
            );
        }
    }

    #[test]
    fn all_framework_rules_reference_known_languages() {
        let known_langs: Vec<&str> = LANGUAGE_RULES.iter().map(|r| r.lang_type).collect();
        for rule in FRAMEWORK_RULES {
            assert!(
                known_langs.contains(&rule.language),
                "framework '{}' references unknown language '{}'",
                rule.framework,
                rule.language
            );
        }
    }

    #[test]
    fn no_duplicate_framework_names() {
        let mut seen = std::collections::HashSet::new();
        for rule in FRAMEWORK_RULES {
            assert!(
                seen.insert(rule.framework),
                "duplicate framework: {}",
                rule.framework
            );
        }
    }

    #[test]
    fn every_framework_is_classified_as_frontend_or_backend() {
        let all_signals: Vec<&str> = FRONTEND_SIGNALS
            .iter()
            .chain(BACKEND_SIGNALS.iter())
            .copied()
            .collect();
        for rule in FRAMEWORK_RULES {
            assert!(
                all_signals.contains(&rule.framework),
                "framework '{}' not in FRONTEND_SIGNALS or BACKEND_SIGNALS",
                rule.framework
            );
        }
    }
}
