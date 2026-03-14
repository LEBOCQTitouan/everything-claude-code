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

pub const FRONTEND_SIGNALS: &[&str] = &[
    "react", "vue", "angular", "svelte", "nextjs", "nuxt", "astro", "remix",
];

pub const BACKEND_SIGNALS: &[&str] = &[
    "django", "fastapi", "flask", "express", "nestjs", "rails", "spring",
    "laravel", "phoenix", "gin", "echo", "actix", "axum",
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- FRAMEWORK_RULES ---

    #[test]
    fn framework_rules_has_23_entries() {
        assert_eq!(FRAMEWORK_RULES.len(), 23);
    }
}
