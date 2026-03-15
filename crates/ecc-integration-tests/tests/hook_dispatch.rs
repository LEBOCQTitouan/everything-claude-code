mod common;

use common::EccTestEnv;
use predicates::prelude::*;

/// Known hook dispatches and exits zero.
#[test]
fn known_hook_passthrough_exits_zero() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "check:hook:enabled"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// Unknown hook ID produces a warning on stderr but still exits zero (passthrough).
#[test]
fn unknown_hook_warns_on_stderr() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "totally:unknown:hook"])
        .write_stdin("{}")
        .assert()
        .success()
        .stderr(predicate::str::contains("unknown").or(predicate::str::contains("Unknown")));
}

/// Stdin payload flows through to stdout on an unknown hook (passthrough behavior).
#[test]
fn hook_stdin_flows_through() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    // Unknown hooks pass stdin through to stdout
    env.cmd()
        .args(["hook", "totally:unknown:hook"])
        .write_stdin(payload)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bash"));
}

/// Hook disabled via ECC_DISABLED_HOOKS env var results in passthrough.
#[test]
fn hook_disabled_via_env_var() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    env.cmd()
        .env("ECC_DISABLED_HOOKS", "check:hook:enabled")
        .args(["hook", "check:hook:enabled"])
        .write_stdin(payload)
        .assert()
        .success()
        // When disabled, stdin is passed through (not the "yes" output)
        .stdout(predicate::str::contains("Bash"));
}

/// 3-arg legacy format actually dispatches (not just parses).
/// check:hook:enabled outputs "yes" — verifying it actually ran the handler.
#[test]
fn hook_three_arg_legacy_dispatches_correctly() {
    let env = EccTestEnv::new();
    let payload = r#"{"tool":"Bash","input":{"command":"echo hello"}}"#;
    env.cmd()
        .args([
            "hook",
            "check:hook:enabled",
            "path/to/old-script.js",
            "standard",
        ])
        .write_stdin(payload)
        .assert()
        .success()
        // check:hook:enabled outputs "yes" when it successfully dispatches
        .stdout(predicate::str::contains("yes"));
}
