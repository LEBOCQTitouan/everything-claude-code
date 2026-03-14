use ecc_app::version::version;

pub fn run() -> anyhow::Result<()> {
    println!("ecc v{}", version());
    Ok(())
}
