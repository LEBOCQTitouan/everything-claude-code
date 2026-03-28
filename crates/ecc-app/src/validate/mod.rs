//! Validate use case — validates content files (agents, commands, hooks, skills, rules, paths).

mod agents;
mod commands;
mod hooks;
mod paths;
mod rules;
mod skills;
mod statusline;

/// Which content type to validate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidateTarget {
    Agents,
    Commands,
    Hooks,
    Skills,
    Rules,
    Paths,
    Statusline,
}

/// Run validation for the given target. Returns `true` on success, `false` on errors.
pub fn run_validate(
    fs: &dyn ecc_ports::fs::FileSystem,
    terminal: &dyn ecc_ports::terminal::TerminalIO,
    target: &ValidateTarget,
    root: &std::path::Path,
) -> bool {
    match target {
        ValidateTarget::Agents => agents::validate_agents(root, fs, terminal),
        ValidateTarget::Commands => commands::validate_commands(root, fs, terminal),
        ValidateTarget::Hooks => hooks::validate_hooks(root, fs, terminal),
        ValidateTarget::Skills => skills::validate_skills(root, fs, terminal),
        ValidateTarget::Rules => rules::validate_rules(root, fs, terminal),
        ValidateTarget::Paths => paths::validate_paths(root, fs, terminal),
        ValidateTarget::Statusline => statusline::validate_statusline(root, fs, terminal),
    }
}
