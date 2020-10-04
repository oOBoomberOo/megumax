#[cfg(feature = "async")]
#[path = "par.rs"]
mod imports;

#[cfg(not(feature = "async"))]
#[path = "sync.rs"]
mod imports;

pub use imports::*;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn stringify_path<P: AsRef<Path>>(path: &P) -> Result<&str> {
	let path = path.as_ref();
	path.to_str()
		.with_context(|| format!("`{}` is not a valid UTF-8 string", path.display()))
}

pub fn replace_prefix<P: AsRef<Path>>(path: P, from: P, to: P) -> Result<PathBuf> {
	let from = from.as_ref();
	let to = to.as_ref();
	let path = path.as_ref();
	let path = path.strip_prefix(from).with_context(|| {
		format!(
			"Strip prefix `{}` from `{}`",
			from.display(),
			path.display()
		)
	})?;
	let result = to.join(path);
	Ok(result)
}
