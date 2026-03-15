mod common;

use common::EccTestEnv;
use predicates::prelude::*;

#[test]
fn version_prints_version_string() {
    let env = EccTestEnv::new();
    env.cmd()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ecc"));
}

#[test]
fn completion_bash_produces_output() {
    let env = EccTestEnv::new();
    env.cmd()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
