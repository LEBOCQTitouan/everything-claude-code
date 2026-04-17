//! Validate use case — validates content files (agents, commands, hooks, skills, rules, paths).

mod agents;
mod code_block_scanning;
mod commands;
mod conventions;
mod cross_ref_validation;
mod frontmatter_validation;
mod hooks;
mod paths;
mod patterns;
mod rules;
mod section_validation;
mod skills;
mod statusline;
mod teams;
pub(crate) mod tool_manifest_loader;
pub(crate) mod tool_manifest_path_resolver;

/// Which content type to validate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidateTarget {
    Agents,
    Commands,
    Conventions,
    Hooks,
    Skills,
    Rules,
    Paths,
    Patterns,
    Statusline,
    Teams,
}

/// Run validation for the given target. Returns `true` on success, `false` on errors.
pub fn run_validate(
    fs: &dyn ecc_ports::fs::FileSystem,
    terminal: &dyn ecc_ports::terminal::TerminalIO,
    env: &dyn ecc_ports::env::Environment,
    target: &ValidateTarget,
    root: &std::path::Path,
) -> bool {
    match target {
        ValidateTarget::Agents => agents::validate_agents(root, fs, terminal),
        ValidateTarget::Commands => commands::validate_commands(root, fs, terminal),
        ValidateTarget::Conventions => conventions::validate_conventions(root, fs, terminal),
        ValidateTarget::Hooks => hooks::validate_hooks(root, fs, terminal),
        ValidateTarget::Skills => skills::validate_skills(root, fs, terminal),
        ValidateTarget::Rules => rules::validate_rules(root, fs, terminal),
        ValidateTarget::Paths => paths::validate_paths(root, fs, terminal),
        ValidateTarget::Patterns => patterns::validate_patterns(root, fs, terminal),
        ValidateTarget::Statusline => statusline::validate_statusline(root, fs, terminal, env),
        ValidateTarget::Teams => teams::validate_teams(root, fs, terminal),
    }
}

/// Run pattern validation with optional --fix (auto-regenerate index.md).
pub fn run_validate_patterns(
    fs: &dyn ecc_ports::fs::FileSystem,
    terminal: &dyn ecc_ports::terminal::TerminalIO,
    _env: &dyn ecc_ports::env::Environment,
    root: &std::path::Path,
    fix: bool,
) -> bool {
    let valid = patterns::validate_patterns(root, fs, terminal);
    if fix && valid {
        patterns::generate_index(root, fs, terminal);
    }
    valid
}
