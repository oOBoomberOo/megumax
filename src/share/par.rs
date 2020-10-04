use anyhow::{Context, Result};
use smol::fs;
use std::path::Path;

pub use fs::File;

pub async fn ensure_parent<P: AsRef<Path>>(path: P) -> Result<()> {
	let path = path.as_ref();

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.await
			.with_context(|| "Create all directories")
			.with_context(|| format!("From: {}", parent.display()))?;
	}

	Ok(())
}

pub async fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
	let path = path.as_ref();
	ensure_parent(&path).await?;
	File::create(path)
		.await
		.with_context(|| "Creating File writer")
		.with_context(|| format!("At: {}", path.display()))
}

pub async fn open_file<P: AsRef<Path>>(path: P) -> Result<File> {
	let path = path.as_ref();
	File::open(&path)
		.await
		.with_context(|| "Opening File reader")
		.with_context(|| format!("At: {}", path.display()))
}
