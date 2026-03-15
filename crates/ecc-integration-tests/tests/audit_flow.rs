mod common;

use common::EccTestEnv;

#[test]
fn audit_after_install_passes() {
    let env = EccTestEnv::new();

    // Install first so there's something to audit
    env.install(&[]).success();

    // Audit should pass on a valid installation
    env.cmd()
        .arg("audit")
        .env("HOME", env.home.path())
        .assert()
        .success();
}

#[test]
fn audit_empty_home_reports_findings() {
    let env = EccTestEnv::new();

    // Audit on an empty HOME — should fail (nothing installed)
    env.cmd()
        .arg("audit")
        .env("HOME", env.home.path())
        .assert()
        .failure();
}
