use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub use fs::File;

pub fn ensure_parent<P: AsRef<Path>>(path: P) -> Result<()> {
	let path = path.as_ref();

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.with_context(|| "Create all directories")
			.with_context(|| format!("From: {}", parent.display()))?;
	}

	Ok(())
}

pub fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
	let path = path.as_ref();
	ensure_parent(&path)?;
	File::create(path)
		.with_context(|| "Creating File writer")
		.with_context(|| format!("At: {}", path.display()))
}

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<File> {
	let path = path.as_ref();
	File::open(&path)
		.with_context(|| "Opening File reader")
		.with_context(|| format!("At: {}", path.display()))
}
