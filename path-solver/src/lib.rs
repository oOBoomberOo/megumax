pub mod resource;
pub mod solver;
pub mod template;
pub mod variant;
pub mod error;

pub use resource::{Resource, Resources};
pub use solver::Solver;
pub use template::{Pool, Template};
pub use variant::{variant, Variant};
