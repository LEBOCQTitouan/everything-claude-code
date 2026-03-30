use ecc_app::ecc_status;
use ecc_infra::os_fs::OsFileSystem;
use ecc_infra::os_env::OsEnvironment;
use ecc_infra::process_executor::ProcessExecutor;

pub fn run() -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let shell = ProcessExecutor;

    let status = ecc_status::ecc_status(&fs, &env, &shell);

    println!("ECC v{}", status.ecc_version);
    match status.workflow_version {
        Some(v) => println!("ecc-workflow v{v}"),
        None => println!("ecc-workflow: not found"),
    }
    println!();

    match status.workflow {
        Some(wf) => {
            println!("Phase: {} | Feature: {}", wf.phase, wf.feature);
            println!("Started: {}", wf.started_at);
        }
        None => println!("No active workflow"),
    }

    let a = &status.artifacts;
    let spec = if a.spec { "✓" } else { "✗" };
    let design = if a.design { "✓" } else { "✗" };
    let tasks = if a.tasks { "✓" } else { "✗" };
    println!("Artifacts: spec {spec} design {design} tasks {tasks}");

    println!();
    let c = &status.components;
    println!(
        "Components: {} agents, {} skills, {} commands, {} rules",
        c.agents, c.skills, c.commands, c.rules
    );
    println!("Hooks: {}", c.hooks);

    Ok(())
}
