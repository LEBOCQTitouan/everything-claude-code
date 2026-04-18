// BL-132 fixture: clap-derive file with diagram INSIDE impl block — classifier MUST PASS.
//
// Expected classifier verdict: PASS
// Reason: the diagram lives inside an `impl` block, not directly above a derive target.
// Clap only promotes `///` above the derive-target into `--help`, so impl-block docs are safe.

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub name: String,
}

impl Args {
    /// Validate name against schema.
    ///
    /// ```text
    /// [name?] --Y--> [valid] --> return
    ///         --N--> [invalid] --> error
    /// ```
    ///
    /// <!-- keep in sync with: args_validate_happy_path -->
    pub fn validate(&self) -> bool {
        !self.name.is_empty()
    }
}
