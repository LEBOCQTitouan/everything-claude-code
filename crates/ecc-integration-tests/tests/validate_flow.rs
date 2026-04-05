mod common;

use common::EccTestEnv;

/// Helper: run `ecc validate --ecc-root <workspace> <target>` against real workspace assets.
fn validate_target(target: &str) {
    let env = EccTestEnv::new();
    env.cmd()
        .arg("validate")
        .arg("--ecc-root")
        .arg(EccTestEnv::ecc_root())
        .arg(target)
        .assert()
        .success();
}

#[test]
fn validate_agents_passes() {
    validate_target("agents");
}

#[test]
fn validate_commands_passes() {
    validate_target("commands");
}

#[test]
fn validate_hooks_passes() {
    validate_target("hooks");
}

#[test]
fn validate_skills_passes() {
    validate_target("skills");
}

#[test]
fn validate_rules_passes() {
    validate_target("rules");
}

#[test]
fn validate_paths_passes() {
    validate_target("paths");
}

#[test]
fn validate_conventions_passes() {
    validate_target("conventions");
}

#[test]
fn validate_patterns_passes() {
    validate_target("patterns");
}

#[test]
fn validate_teams_passes() {
    validate_target("teams");
}
