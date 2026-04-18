/// Artifact naming and identification.
pub mod artifact;
/// Update operation errors.
pub mod error;
/// Update plan types.
pub mod plan;
/// Platform and architecture detection.
pub mod platform;
/// Version parsing and comparison.
pub mod version;

pub use artifact::ArtifactName;
pub use error::UpdateError;
pub use plan::UpdatePlan;
pub use platform::{Architecture, Platform};
pub use version::Version;
