use crate::utils::check_expression_block;

mod message;

#[cfg(not(feature = "async"))]
#[path = "app/sync.rs"]
mod imports;

#[cfg(feature = "async")]
#[path = "app/parallel.rs"]
mod imports;

pub use imports::*;
