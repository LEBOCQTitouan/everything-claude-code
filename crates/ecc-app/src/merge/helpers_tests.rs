use super::super::*;
use ecc_domain::config::merge::{self, FileToReview};
use ecc_ports::shell::CommandOutput;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
use std::path::Path;

fn no_color_env() -> MockEnvironment {
    MockEnvironment::new().with_var("NO_COLOR", "1")
}

// --- prompt_file_review ---

#[test]
fn prompt_accept() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new content")
        .with_file("/dest/agent.md", "old content");
    let terminal = BufferedTerminal::new().with_input("a");
    let env = no_color_env();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };

    let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
    assert_eq!(choice, ReviewChoice::Accept);
    assert!(!apply_all);
}

#[test]
fn prompt_keep() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new")
        .with_file("/dest/agent.md", "old");
    let terminal = BufferedTerminal::new().with_input("k");
    let env = no_color_env();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };

    let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
    assert_eq!(choice, ReviewChoice::Keep);
}

#[test]
fn prompt_smart_merge() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new")
        .with_file("/dest/agent.md", "old");
    let terminal = BufferedTerminal::new().with_input("s");
    let env = no_color_env();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };

    let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
    assert_eq!(choice, ReviewChoice::SmartMerge);
}

#[test]
fn prompt_accept_all() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new")
        .with_file("/dest/agent.md", "old");
    let terminal = BufferedTerminal::new().with_input("A");
    let env = no_color_env();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };

    let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
    assert_eq!(choice, ReviewChoice::Accept);
    assert!(apply_all);
}

#[test]
fn prompt_keep_all() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new")
        .with_file("/dest/agent.md", "old");
    let terminal = BufferedTerminal::new().with_input("K");
    let env = no_color_env();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };

    let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
    assert_eq!(choice, ReviewChoice::Keep);
    assert!(apply_all);
}

#[test]
fn prompt_new_file_shows_preview() {
    let fs = InMemoryFileSystem::new().with_file("/src/new.md", "# New agent\nLine 2\nLine 3");
    let terminal = BufferedTerminal::new().with_input("a");
    let env = no_color_env();
    let file = FileToReview {
        filename: "new.md".into(),
        src_path: "/src/new.md".into(),
        dest_path: "/dest/new.md".into(),
        is_new: true,
    };

    prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
    let output = terminal.stdout_output().join("");
    assert!(output.contains("NEW: new.md"));
    assert!(output.contains("3 lines"));
}

// --- apply_review_choice ---

#[test]
fn apply_accept_copies_file() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new content")
        .with_dir("/dest");
    let shell = MockExecutor::new();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: true,
    };
    let mut report = merge::empty_report();

    apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, false, &mut report);

    assert_eq!(report.added, vec!["agent.md"]);
    assert_eq!(
        fs.read_to_string(Path::new("/dest/agent.md")).unwrap(),
        "new content"
    );
}

#[test]
fn apply_accept_dry_run_does_not_copy() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agent.md", "new content")
        .with_dir("/dest");
    let shell = MockExecutor::new();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: true,
    };
    let mut report = merge::empty_report();

    apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, true, &mut report);

    assert_eq!(report.added, vec!["agent.md"]);
    assert!(!fs.exists(Path::new("/dest/agent.md")));
}

#[test]
fn apply_keep_skips() {
    let fs = InMemoryFileSystem::new();
    let shell = MockExecutor::new();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };
    let mut report = merge::empty_report();

    apply_review_choice(&fs, &shell, ReviewChoice::Keep, &file, false, &mut report);

    assert_eq!(report.skipped, vec!["agent.md"]);
}

#[test]
fn apply_smart_merge_success() {
    let fs = InMemoryFileSystem::new()
        .with_file("/dest/agent.md", "old content")
        .with_file("/src/agent.md", "new content");
    let shell = MockExecutor::new().with_command("claude").on(
        "claude",
        CommandOutput {
            stdout: "merged content".to_string(),
            stderr: String::new(),
            exit_code: 0,
        },
    );
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };
    let mut report = merge::empty_report();

    apply_review_choice(
        &fs,
        &shell,
        ReviewChoice::SmartMerge,
        &file,
        false,
        &mut report,
    );

    assert_eq!(report.smart_merged, vec!["agent.md"]);
    assert_eq!(
        fs.read_to_string(Path::new("/dest/agent.md")).unwrap(),
        "merged content"
    );
}

#[test]
fn apply_smart_merge_failure_records_error() {
    let fs = InMemoryFileSystem::new()
        .with_file("/dest/agent.md", "old")
        .with_file("/src/agent.md", "new");
    let shell = MockExecutor::new();
    let file = FileToReview {
        filename: "agent.md".into(),
        src_path: "/src/agent.md".into(),
        dest_path: "/dest/agent.md".into(),
        is_new: false,
    };
    let mut report = merge::empty_report();

    apply_review_choice(
        &fs,
        &shell,
        ReviewChoice::SmartMerge,
        &file,
        false,
        &mut report,
    );

    assert!(!report.errors.is_empty());
    assert!(report.errors[0].contains("smart merge failed"));
}

// --- merge_directory ---

#[test]
fn merge_directory_force_mode() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/planner.md", "new planner")
        .with_file("/src/agents/reviewer.md", "new reviewer")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: true,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.added.len(), 2);
    assert!(fs.exists(Path::new("/dest/agents/planner.md")));
}

#[test]
fn merge_directory_skips_unchanged() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/planner.md", "same content")
        .with_file("/dest/agents/planner.md", "same content");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = default_merge_options();
    options.force = true;

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert!(report.added.is_empty());
    assert!(report.updated.is_empty());
    assert_eq!(report.unchanged, vec!["planner.md"]);
}

#[test]
fn merge_directory_non_interactive_accepts_all() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/a.md", "new a")
        .with_file("/src/agents/b.md", "new b")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: false,
        interactive: false,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.added.len(), 2);
}

#[test]
fn merge_directory_dry_run() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/new.md", "content")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: true,
        force: true,
        interactive: true,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.added, vec!["new.md"]);
    assert!(!fs.exists(Path::new("/dest/agents/new.md")));
}

// --- merge_skills ---

#[test]
fn merge_skills_force_copies_directory() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/src/skills/tdd")
        .with_file("/src/skills/tdd/SKILL.md", "# TDD Skill")
        .with_file("/src/skills/tdd/examples.md", "# Examples")
        .with_dir("/dest/skills");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: true,
        apply_all: None,
    };

    let report = merge_skills(
        &ctx,
        Path::new("/src/skills"),
        Path::new("/dest/skills"),
        &mut options,
    );

    assert_eq!(report.added, vec!["tdd"]);
    assert!(fs.exists(Path::new("/dest/skills/tdd/SKILL.md")));
    assert!(fs.exists(Path::new("/dest/skills/tdd/examples.md")));
}

#[test]
fn merge_skills_with_subdirectories() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/src/skills/security-review")
        .with_file("/src/skills/security-review/SKILL.md", "# Security Review")
        .with_dir("/src/skills/security-review/references")
        .with_file(
            "/src/skills/security-review/references/owasp.md",
            "# OWASP Top 10",
        )
        .with_file(
            "/src/skills/security-review/references/checklist.md",
            "# Checklist",
        )
        .with_dir("/dest/skills");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: true,
        apply_all: None,
    };

    let report = merge_skills(
        &ctx,
        Path::new("/src/skills"),
        Path::new("/dest/skills"),
        &mut options,
    );

    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
    assert_eq!(report.added, vec!["security-review"]);
    assert!(fs.exists(Path::new("/dest/skills/security-review/SKILL.md")));
    assert!(fs.exists(Path::new(
        "/dest/skills/security-review/references/owasp.md"
    )));
    assert!(fs.exists(Path::new(
        "/dest/skills/security-review/references/checklist.md"
    )));
}

#[test]
fn merge_skills_unchanged_skipped() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/src/skills/tdd")
        .with_file("/src/skills/tdd/SKILL.md", "same content")
        .with_dir("/dest/skills/tdd")
        .with_file("/dest/skills/tdd/SKILL.md", "same content");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = default_merge_options();
    options.force = true;

    let report = merge_skills(
        &ctx,
        Path::new("/src/skills"),
        Path::new("/dest/skills"),
        &mut options,
    );

    assert!(report.added.is_empty());
    assert_eq!(report.unchanged, vec!["tdd"]);
}

// --- merge_rules ---

#[test]
fn merge_rules_force_mode() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/src/rules/common")
        .with_file("/src/rules/common/style.md", "# Style")
        .with_dir("/src/rules/typescript")
        .with_file("/src/rules/typescript/types.md", "# Types")
        .with_dir("/dest/rules");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: true,
        apply_all: None,
    };

    let groups = vec!["common".to_string(), "typescript".to_string()];
    let report = merge_rules(
        &ctx,
        Path::new("/src/rules"),
        Path::new("/dest/rules"),
        &groups,
        &mut options,
    );

    assert_eq!(report.added.len(), 2);
    assert!(fs.exists(Path::new("/dest/rules/common/style.md")));
    assert!(fs.exists(Path::new("/dest/rules/typescript/types.md")));
}

// --- merge_hooks ---

#[test]
fn merge_hooks_adds_new() {
    let source = serde_json::json!({"hooks": {"PreToolUse": [{"description": "ECC format", "hooks": [{"command": "ecc-hook format"}]}]}});
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
        .with_file("/settings.json", "{}");

    let (added, existing, legacy) = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        false,
    )
    .unwrap();

    assert_eq!(added, 1);
    assert_eq!(existing, 0);
    assert_eq!(legacy, 0);

    let updated: serde_json::Value =
        serde_json::from_str(&fs.read_to_string(Path::new("/settings.json")).unwrap()).unwrap();
    assert!(updated["hooks"]["PreToolUse"].is_array());
}

#[test]
fn merge_hooks_dedup() {
    let hook = serde_json::json!({"description": "ECC format", "hooks": [{"command": "ecc-hook format"}]});
    let source = serde_json::json!({"hooks": { "PreToolUse": [hook.clone()] }});
    let settings = serde_json::json!({"hooks": { "PreToolUse": [hook] }});
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
        .with_file("/settings.json", &serde_json::to_string(&settings).unwrap());

    let (added, existing, _) = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        false,
    )
    .unwrap();

    assert_eq!(added, 0);
    assert_eq!(existing, 1);
}

#[test]
fn merge_hooks_removes_legacy() {
    let legacy_hook = serde_json::json!({"description": "old hook", "hooks": [{"command": "node /path/to/everything-claude-code/dist/hooks/run.js"}]});
    let settings = serde_json::json!({"hooks": { "PreToolUse": [legacy_hook] }});
    let source = serde_json::json!({"hooks": {}});
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
        .with_file("/settings.json", &serde_json::to_string(&settings).unwrap());

    let (_, _, legacy_removed) = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        false,
    )
    .unwrap();

    assert_eq!(legacy_removed, 1);
}

#[test]
fn merge_hooks_writes_when_only_legacy_removed() {
    let legacy_hook = serde_json::json!({"description": "old hook", "hooks": [{"command": "node /path/to/everything-claude-code/dist/hooks/run.js"}]});
    let user_hook = serde_json::json!({"description": "user hook", "hooks": [{"command": "my-custom-hook"}]});
    let settings = serde_json::json!({"hooks": { "PreToolUse": [legacy_hook, user_hook] }});
    let source = serde_json::json!({"hooks": {}});
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
        .with_file("/settings.json", &serde_json::to_string(&settings).unwrap());

    let (added, _, legacy_removed) = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        false,
    )
    .unwrap();

    assert_eq!(added, 0);
    assert_eq!(legacy_removed, 1);

    let updated: serde_json::Value =
        serde_json::from_str(&fs.read_to_string(Path::new("/settings.json")).unwrap()).unwrap();
    let pre_hooks = updated["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre_hooks.len(), 1);
    assert_eq!(pre_hooks[0]["description"], "user hook");
}

#[test]
fn merge_hooks_dry_run_does_not_write() {
    let source = serde_json::json!({"hooks": {"PreToolUse": [{"hooks": [{"command": "ecc-hook test"}]}]}});
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", &serde_json::to_string(&source).unwrap())
        .with_file("/settings.json", "{}");

    let (added, _, _) = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        true,
    )
    .unwrap();

    assert_eq!(added, 1);
    let settings = fs.read_to_string(Path::new("/settings.json")).unwrap();
    assert_eq!(settings, "{}");
}

#[test]
fn merge_hooks_invalid_hooks_json() {
    let fs = InMemoryFileSystem::new()
        .with_file("/hooks.json", "not json")
        .with_file("/settings.json", "{}");

    let result = merge_hooks(
        &fs,
        Path::new("/hooks.json"),
        Path::new("/settings.json"),
        false,
    );

    assert!(result.is_err());
}

// --- merge_directory with scripted prompts ---

#[test]
fn merge_directory_interactive_accept() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/new.md", "# New agent")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new().with_input("a");
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = default_merge_options();

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.added, vec!["new.md"]);
}

#[test]
fn merge_directory_interactive_keep() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/existing.md", "updated content")
        .with_file("/dest/agents/existing.md", "old content");
    let terminal = BufferedTerminal::new().with_input("k");
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = default_merge_options();

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.skipped, vec!["existing.md"]);
}

// --- merge_directory edge cases ---

#[test]
fn merge_directory_empty_source_returns_empty_report() {
    let fs = InMemoryFileSystem::new()
        .with_dir("/src/agents")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: false,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert!(report.added.is_empty());
    assert!(report.updated.is_empty());
    assert!(report.unchanged.is_empty());
    assert!(report.errors.is_empty());
}

#[test]
fn merge_directory_nonexistent_source_returns_empty_report() {
    let fs = InMemoryFileSystem::new().with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: false,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/nonexistent/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert!(report.added.is_empty());
    assert!(report.errors.is_empty());
}

#[test]
fn merge_directory_apply_all_accept_skips_remaining_prompts() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/a.md", "content a")
        .with_file("/src/agents/b.md", "content b")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new().with_input("A");
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = default_merge_options();

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert_eq!(report.added.len(), 2, "both files should be accepted");
    assert!(fs.exists(Path::new("/dest/agents/a.md")));
    assert!(fs.exists(Path::new("/dest/agents/b.md")));
    assert_eq!(options.apply_all, Some(ReviewChoice::Accept));
}

#[test]
fn merge_directory_wrong_extension_ignored() {
    let fs = InMemoryFileSystem::new()
        .with_file("/src/agents/readme.txt", "ignore me")
        .with_dir("/dest/agents");
    let terminal = BufferedTerminal::new();
    let env = no_color_env();
    let shell = MockExecutor::new();
    let ctx = MergeContext {
        fs: &fs,
        terminal: &terminal,
        env: &env,
        shell: &shell,
    };
    let mut options = MergeOptions {
        dry_run: false,
        force: true,
        interactive: false,
        apply_all: None,
    };

    let report = merge_directory(
        &ctx,
        Path::new("/src/agents"),
        Path::new("/dest/agents"),
        "Agents",
        ".md",
        &mut options,
    );

    assert!(report.added.is_empty());
    assert!(report.updated.is_empty());
}
