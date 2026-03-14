use ecc_ports::fs::FileSystem;
use std::path::Path;

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

/// Check whether any file in `dir` (non-recursive) has one of the given extensions.
pub fn has_file_with_extension(
    fs: &dyn FileSystem,
    dir: &Path,
    extensions: &[&str],
) -> bool {
    let Ok(entries) = fs.read_dir(dir) else {
        return false;
    };
    entries.iter().any(|entry| {
        let name = entry.to_string_lossy();
        extensions.iter().any(|ext| name.ends_with(ext))
    })
}

/// Detect languages present in the given project directory.
///
/// Checks marker files and file extensions for each language rule.
/// If both TypeScript and JavaScript are detected, JavaScript is removed.
pub fn detect_languages(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let mut languages: Vec<String> = Vec::new();

    for rule in LANGUAGE_RULES {
        let has_marker = rule.markers.iter().any(|m| fs.exists(&dir.join(m)));
        let has_ext = !rule.extensions.is_empty()
            && has_file_with_extension(fs, dir, rule.extensions);

        if has_marker || has_ext {
            languages.push(rule.lang_type.to_string());
        }
    }

    // Deduplicate: if both typescript and javascript detected, keep typescript
    if languages.contains(&"typescript".to_string())
        && languages.contains(&"javascript".to_string())
    {
        languages.retain(|l| l != "javascript");
    }

    languages
}

/// Extract dependency names from package.json (dependencies + devDependencies).
pub fn get_package_json_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let pkg_path = dir.join("package.json");
    let Ok(content) = fs.read_to_string(&pkg_path) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    for section in &["dependencies", "devDependencies"] {
        if let Some(obj) = parsed.get(section).and_then(|v| v.as_object()) {
            for key in obj.keys() {
                deps.push(key.clone());
            }
        }
    }
    deps
}

/// Extract Python dependency names from requirements.txt and pyproject.toml.
pub fn get_python_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let mut deps = Vec::new();

    // requirements.txt
    if let Ok(content) = fs.read_to_string(&dir.join("requirements.txt")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('-')
            {
                continue;
            }
            let name = trimmed
                .split(['>', '=', '<', '!', '[', ';'])
                .next()
                .unwrap_or("")
                .trim()
                .to_lowercase();
            if !name.is_empty() {
                deps.push(name);
            }
        }
    }

    // pyproject.toml
    if let Ok(content) = fs.read_to_string(&dir.join("pyproject.toml")) {
        let re_block =
            regex::Regex::new(r"dependencies\s*=\s*\[([\s\S]*?)\]").unwrap();
        if let Some(captures) = re_block.captures(&content) {
            let block = &captures[1];
            let re_quoted = regex::Regex::new(r#""([^"]+)""#).unwrap();
            for m in re_quoted.captures_iter(block) {
                let raw = &m[1];
                let name = raw
                    .split(['>', '=', '<', '!', '[', ';'])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
                if !name.is_empty() {
                    deps.push(name);
                }
            }
        }
    }

    deps
}

/// Extract Go module dependency paths from go.mod require block.
pub fn get_go_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let Ok(content) = fs.read_to_string(&dir.join("go.mod")) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    let re = regex::Regex::new(r"require\s*\(([\s\S]*?)\)").unwrap();
    if let Some(captures) = re.captures(&content) {
        let block = &captures[1];
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if let Some(module_path) = trimmed.split_whitespace().next() {
                deps.push(module_path.to_string());
            }
        }
    }

    deps
}

/// Extract Rust crate names from Cargo.toml [dependencies] and [dev-dependencies].
pub fn get_rust_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let Ok(content) = fs.read_to_string(&dir.join("Cargo.toml")) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    let re_header =
        regex::Regex::new(r"^\[(dev-)?dependencies\]\s*$").unwrap();
    let re_name = regex::Regex::new(r"^([a-zA-Z0-9_-]+)\s*=").unwrap();

    let mut in_deps_section = false;
    for line in content.lines() {
        if re_header.is_match(line) {
            in_deps_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_deps_section = false;
            continue;
        }
        if in_deps_section
            && let Some(name_match) = re_name.captures(line)
        {
            deps.push(name_match[1].to_string());
        }
    }

    deps
}

/// Extract PHP dependency names from composer.json (require + require-dev).
pub fn get_composer_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let Ok(content) = fs.read_to_string(&dir.join("composer.json")) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    for section in &["require", "require-dev"] {
        if let Some(obj) = parsed.get(section).and_then(|v| v.as_object()) {
            for key in obj.keys() {
                deps.push(key.clone());
            }
        }
    }
    deps
}

/// Extract Elixir dependency names from mix.exs `{:name` patterns.
pub fn get_elixir_deps(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let Ok(content) = fs.read_to_string(&dir.join("mix.exs")) else {
        return Vec::new();
    };

    let mut deps = Vec::new();
    let re = regex::Regex::new(r"\{:(\w+)").unwrap();
    for m in re.captures_iter(&content) {
        deps.push(m[1].to_string());
    }
    deps
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

    // --- LANGUAGE_RULES ---

    #[test]
    fn language_rules_has_12_entries() {
        assert_eq!(LANGUAGE_RULES.len(), 12);
    }

    // --- detect_languages ---

    #[test]
    fn detect_languages_with_marker_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/Cargo.toml", "[package]");
        let langs = detect_languages(&fs, dir());
        assert!(langs.contains(&"rust".to_string()));
    }

    #[test]
    fn detect_languages_with_extensions() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/main.py", "print('hello')");
        let langs = detect_languages(&fs, dir());
        assert!(langs.contains(&"python".to_string()));
    }

    #[test]
    fn detect_languages_dedup_ts_js() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/tsconfig.json", "{}")
            .with_file("/project/package.json", "{}");
        let langs = detect_languages(&fs, dir());
        assert!(langs.contains(&"typescript".to_string()));
        assert!(!langs.contains(&"javascript".to_string()));
    }

    #[test]
    fn detect_languages_no_files() {
        let fs = InMemoryFileSystem::new();
        let langs = detect_languages(&fs, dir());
        assert!(langs.is_empty());
    }

    #[test]
    fn detect_languages_multiple() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/go.mod", "module test")
            .with_file("/project/main.rs", "fn main() {}");
        let langs = detect_languages(&fs, dir());
        assert!(langs.contains(&"golang".to_string()));
        assert!(langs.contains(&"rust".to_string()));
    }

    // --- has_file_with_extension ---

    #[test]
    fn has_file_with_extension_found() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/app.ts", "");
        assert!(has_file_with_extension(&fs, dir(), &[".ts", ".tsx"]));
    }

    #[test]
    fn has_file_with_extension_not_found() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/readme.md", "");
        assert!(!has_file_with_extension(&fs, dir(), &[".ts", ".tsx"]));
    }

    // --- get_package_json_deps ---

    #[test]
    fn package_json_deps_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/package.json",
            r#"{"dependencies":{"react":"^18"},"devDependencies":{"vitest":"^1"}}"#,
        );
        let deps = get_package_json_deps(&fs, dir());
        assert!(deps.contains(&"react".to_string()));
        assert!(deps.contains(&"vitest".to_string()));
    }

    #[test]
    fn package_json_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_package_json_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn package_json_deps_invalid_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/package.json", "not json");
        let deps = get_package_json_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    // --- get_python_deps ---

    #[test]
    fn python_deps_requirements_txt() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/requirements.txt",
            "django>=4.0\nflask\n# comment\n-r other.txt\n",
        );
        let deps = get_python_deps(&fs, dir());
        assert!(deps.contains(&"django".to_string()));
        assert!(deps.contains(&"flask".to_string()));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn python_deps_pyproject_toml() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/pyproject.toml",
            r#"
[project]
dependencies = [
    "fastapi>=0.100",
    "uvicorn",
]
"#,
        );
        let deps = get_python_deps(&fs, dir());
        assert!(deps.contains(&"fastapi".to_string()));
        assert!(deps.contains(&"uvicorn".to_string()));
    }

    #[test]
    fn python_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_python_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn python_deps_empty_content() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/requirements.txt", "");
        let deps = get_python_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    // --- get_go_deps ---

    #[test]
    fn go_deps_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/go.mod",
            "module example.com/test\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9\n\tgithub.com/stretchr/testify v1.8\n)\n",
        );
        let deps = get_go_deps(&fs, dir());
        assert!(deps.contains(&"github.com/gin-gonic/gin".to_string()));
        assert!(deps.contains(&"github.com/stretchr/testify".to_string()));
    }

    #[test]
    fn go_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_go_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn go_deps_no_require_block() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/go.mod", "module example.com/test\n");
        let deps = get_go_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    // --- get_rust_deps ---

    #[test]
    fn rust_deps_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/Cargo.toml",
            "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1\"\ntokio = { version = \"1\" }\n\n[dev-dependencies]\nassert_cmd = \"2\"\n",
        );
        let deps = get_rust_deps(&fs, dir());
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"assert_cmd".to_string()));
    }

    #[test]
    fn rust_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_rust_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn rust_deps_empty_sections() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/Cargo.toml", "[package]\nname = \"test\"\n");
        let deps = get_rust_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    // --- get_composer_deps ---

    #[test]
    fn composer_deps_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/composer.json",
            r#"{"require":{"laravel/framework":"^10"},"require-dev":{"phpunit/phpunit":"^10"}}"#,
        );
        let deps = get_composer_deps(&fs, dir());
        assert!(deps.contains(&"laravel/framework".to_string()));
        assert!(deps.contains(&"phpunit/phpunit".to_string()));
    }

    #[test]
    fn composer_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_composer_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn composer_deps_invalid_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/composer.json", "not json");
        let deps = get_composer_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    // --- get_elixir_deps ---

    #[test]
    fn elixir_deps_valid() {
        let fs = InMemoryFileSystem::new().with_file(
            "/project/mix.exs",
            "defp deps do\n  [{:phoenix, \"~> 1.7\"}, {:ecto, \"~> 3.10\"}]\nend\n",
        );
        let deps = get_elixir_deps(&fs, dir());
        assert!(deps.contains(&"phoenix".to_string()));
        assert!(deps.contains(&"ecto".to_string()));
    }

    #[test]
    fn elixir_deps_missing_file() {
        let fs = InMemoryFileSystem::new();
        let deps = get_elixir_deps(&fs, dir());
        assert!(deps.is_empty());
    }

    #[test]
    fn elixir_deps_no_deps_block() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/mix.exs", "defmodule MyApp do\nend\n");
        let deps = get_elixir_deps(&fs, dir());
        assert!(deps.is_empty());
    }
}
