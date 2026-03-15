mod common;

use common::EccTestEnv;

/// Bug #1 regression: `ecc hook` with 1 arg (hook_id only) must parse successfully.
#[test]
fn hook_one_arg_no_profiles() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "check:hook:enabled"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// `ecc hook` with 2 args (hook_id + profiles CSV) must parse successfully.
#[test]
fn hook_two_arg_with_profiles() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["hook", "check:hook:enabled", "standard,strict"])
        .write_stdin("{}")
        .assert()
        .success();
}

/// Bug #1 exact regression: 3-arg legacy format (hook_id, script_path, profiles)
/// must be accepted by clap, not rejected as "unexpected argument".
#[test]
fn hook_three_arg_legacy_format() {
    let env = EccTestEnv::new();
    env.cmd()
        .args([
            "hook",
            "check:hook:enabled",
            "path/to/script.js",
            "standard,strict",
        ])
        .write_stdin("{}")
        .assert()
        .success();
}

/// All install flags must be accepted together without clap errors.
#[test]
fn install_all_flags_accepted() {
    let env = EccTestEnv::new();
    env.cmd()
        .args([
            "install",
            "--dry-run",
            "--force",
            "--no-interactive",
            "--clean",
        ])
        .assert()
        .success();
}

/// Unknown subcommands must fail with a non-zero exit code.
#[test]
fn unknown_subcommand_fails() {
    let env = EccTestEnv::new();
    env.cmd().arg("foobar").assert().failure();
}
