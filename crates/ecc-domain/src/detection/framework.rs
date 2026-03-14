use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::language::{
    get_composer_deps, get_elixir_deps, get_go_deps, get_package_json_deps,
    get_python_deps, get_rust_deps,
};

/// A rule for detecting a framework by marker files and package dependencies.
pub struct FrameworkRule {
    pub framework: &'static str,
    pub language: &'static str,
    pub markers: &'static [&'static str],
    pub package_keys: &'static [&'static str],
}

/// Framework detection rules — 23 entries matching the TypeScript implementation.
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
];

/// Result of full project type detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectType {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub primary: String,
    pub project_dir: String,
}

/// Detect frameworks present in the given project directory.
///
/// For each framework rule, checks marker files and dependency lists.
/// The appropriate dependency extractor is selected based on `rule.language`.
pub fn detect_frameworks(
    fs: &dyn FileSystem,
    dir: &Path,
    languages: &[String],
) -> Vec<String> {
    // Pre-compute dependency lists only for languages actually detected
    let npm_deps = if languages.iter().any(|l| l == "typescript" || l == "javascript") {
        get_package_json_deps(fs, dir)
    } else {
        Vec::new()
    };
    let py_deps = if languages.contains(&"python".to_string()) {
        get_python_deps(fs, dir)
    } else {
        Vec::new()
    };
    let go_deps = if languages.contains(&"golang".to_string()) {
        get_go_deps(fs, dir)
    } else {
        Vec::new()
    };
    let rust_deps = if languages.contains(&"rust".to_string()) {
        get_rust_deps(fs, dir)
    } else {
        Vec::new()
    };
    let composer_deps = if languages.contains(&"php".to_string()) {
        get_composer_deps(fs, dir)
    } else {
        Vec::new()
    };
    let elixir_deps = if languages.contains(&"elixir".to_string()) {
        get_elixir_deps(fs, dir)
    } else {
        Vec::new()
    };

    let mut frameworks = Vec::new();

    for rule in FRAMEWORK_RULES {
        let has_marker = rule.markers.iter().any(|m| fs.exists(&dir.join(m)));

        let has_dep = if !rule.package_keys.is_empty() {
            let dep_list: &[String] = match rule.language {
                "python" => &py_deps,
                "typescript" | "javascript" => &npm_deps,
                "golang" => &go_deps,
                "rust" => &rust_deps,
                "php" => &composer_deps,
                "elixir" => &elixir_deps,
                _ => &[],
            };
            rule.package_keys.iter().any(|key| {
                let key_lower = key.to_lowercase();
                dep_list
                    .iter()
                    .any(|dep| dep.to_lowercase().contains(&key_lower))
            })
        } else {
            false
        };

        if has_marker || has_dep {
            frameworks.push(rule.framework.to_string());
        }
    }

    frameworks
}

const FRONTEND_SIGNALS: &[&str] = &[
    "react", "vue", "angular", "svelte", "nextjs", "nuxt", "astro", "remix",
];

const BACKEND_SIGNALS: &[&str] = &[
    "django", "fastapi", "flask", "express", "nestjs", "rails", "spring",
    "laravel", "phoenix", "gin", "echo", "actix", "axum",
];

/// Detect the full project type: languages, frameworks, and primary type.
///
/// Calls `detect_languages` then `detect_frameworks`, determines the primary
/// type (first framework, or first language, or "unknown"), and checks for
/// fullstack (both frontend and backend signals present).
pub fn detect_project_type(fs: &dyn FileSystem, dir: &Path) -> ProjectType {
    let languages = super::language::detect_languages(fs, dir);
    let frameworks = detect_frameworks(fs, dir, &languages);

    let mut primary = if !frameworks.is_empty() {
        frameworks[0].clone()
    } else if !languages.is_empty() {
        languages[0].clone()
    } else {
        "unknown".to_string()
    };

    let has_frontend = frameworks.iter().any(|f| FRONTEND_SIGNALS.contains(&f.as_str()));
    let has_backend = frameworks.iter().any(|f| BACKEND_SIGNALS.contains(&f.as_str()));

    if has_frontend && has_backend {
        primary = "fullstack".to_string();
    }

    ProjectType {
        languages,
        frameworks,
        primary,
        project_dir: dir.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    const DIR: &str = "/project";

    fn dir() -> &'static Path {
        Path::new(DIR)
    }

    // --- FRAMEWORK_RULES ---

    #[test]
    fn framework_rules_has_23_entries() {
        assert_eq!(FRAMEWORK_RULES.len(), 23);
    }

    // --- detect_frameworks ---

    #[test]
    fn detect_frameworks_with_markers() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/manage.py", "");
        let langs = vec!["python".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"django".to_string()));
    }

    #[test]
    fn detect_frameworks_with_deps() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/package.json",
            r#"{"dependencies":{"react":"^18","express":"^4"}}"#,
        );
        let langs = vec!["typescript".to_string(), "javascript".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"react".to_string()));
        assert!(fws.contains(&"express".to_string()));
    }

    #[test]
    fn detect_frameworks_no_match() {
        let fs = InMemoryFileSystem::new();
        let langs = vec!["rust".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.is_empty());
    }

    #[test]
    fn detect_frameworks_rails_by_marker() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/config/routes.rb", "");
        let langs = vec!["ruby".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"rails".to_string()));
    }

    #[test]
    fn detect_frameworks_go_gin() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/go.mod",
            "module test\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9\n)\n",
        );
        let langs = vec!["golang".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"gin".to_string()));
    }

    #[test]
    fn detect_frameworks_rust_axum() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/Cargo.toml",
            "[package]\nname = \"test\"\n\n[dependencies]\naxum = \"0.7\"\n",
        );
        let langs = vec!["rust".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"axum".to_string()));
    }

    #[test]
    fn detect_frameworks_phoenix() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/mix.exs",
            "defp deps do\n  [{:phoenix, \"~> 1.7\"}]\nend\n",
        );
        let langs = vec!["elixir".to_string()];
        let fws = detect_frameworks(&fs, dir(), &langs);
        assert!(fws.contains(&"phoenix".to_string()));
    }

    // --- detect_project_type ---

    #[test]
    fn detect_project_type_no_files() {
        let result = detect_project_type(&InMemoryFileSystem::new(), dir());
        assert!(result.languages.is_empty());
        assert!(result.frameworks.is_empty());
        assert_eq!(result.primary, "unknown");
        assert_eq!(result.project_dir, DIR);
    }

    #[test]
    fn detect_project_type_typescript_project() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/tsconfig.json", "{}")
            .with_file(
                "/project/package.json",
                r#"{"dependencies":{"next":"^14"}}"#,
            );
        let result = detect_project_type(&fs, dir());
        assert!(result.languages.contains(&"typescript".to_string()));
        assert!(result.frameworks.contains(&"nextjs".to_string()));
        assert_eq!(result.primary, "nextjs");
    }

    #[test]
    fn detect_project_type_fullstack() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/tsconfig.json", "{}")
            .with_file(
                "/project/package.json",
                r#"{"dependencies":{"react":"^18","express":"^4"}}"#,
            );
        let result = detect_project_type(&fs, dir());
        assert!(result.frameworks.contains(&"react".to_string()));
        assert!(result.frameworks.contains(&"express".to_string()));
        assert_eq!(result.primary, "fullstack");
    }

    #[test]
    fn detect_project_type_language_only() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/Cargo.toml", "[package]\nname = \"test\"\n");
        let result = detect_project_type(&fs, dir());
        assert!(result.languages.contains(&"rust".to_string()));
        assert!(result.frameworks.is_empty());
        assert_eq!(result.primary, "rust");
    }

    #[test]
    fn detect_project_type_python_django() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/requirements.txt", "django>=4.0\n")
            .with_file("/project/manage.py", "");
        let result = detect_project_type(&fs, dir());
        assert!(result.languages.contains(&"python".to_string()));
        assert!(result.frameworks.contains(&"django".to_string()));
        assert_eq!(result.primary, "django");
    }

    #[test]
    fn detect_project_type_project_dir_preserved() {
        let fs = InMemoryFileSystem::new();
        let result = detect_project_type(&fs, Path::new("/custom/path"));
        assert_eq!(result.project_dir, "/custom/path");
    }
}
