use super::Replacer;
use log::debug;
use path_dedot::ParseDot;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn ensure_parent<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
	let path = path.as_ref();
	debug!("Ensuring parent of {} exists.", path.display());
	if let Some(parent) = path.parent() {
		debug!("{} doesn't exists, create one.", parent.display());
		std::fs::create_dir_all(parent)?;
	} else {
		debug!("Parent of {} already exists.", path.display());
	}

	Ok(())
}

/// Remove file or directory from given path
pub fn remove_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
	let path = path.as_ref();

	if path.is_file() {
		std::fs::remove_file(path)
	} else {
		std::fs::remove_dir_all(path)
	}
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
	let path = path.as_ref();
	path.parse_dot().map(|cow| cow.to_path_buf())
}

#[derive(Debug, Clone)]
pub struct Asset {
	pub kind: AssetKind,
	pub source: PathBuf,
}

impl Asset {
	pub fn new<P: Into<PathBuf>>(kind: AssetKind, source: P) -> Self {
		Self {
			kind,
			source: source.into(),
		}
	}

	pub fn from_path<P: AsRef<Path> + Into<PathBuf>>(path: P) -> std::io::Result<Self> {
		let reader = std::fs::read_to_string(&path);
		match reader {
			Ok(content) => {
				let result = Self::new(AssetKind::Text(content), path);
				Ok(result)
			}
			Err(err) if err.kind() == ErrorKind::InvalidData => {
				let result = Self::new(AssetKind::Binary, path);
				Ok(result)
			}
			Err(err) => Err(err),
		}
	}

	pub fn copy<P: AsRef<Path>>(&self, path: P) -> std::io::Result<u64> {
		ensure_parent(&path)?;
		std::fs::copy(&self.source, path)
	}

	pub fn metadata(&self) -> Replacer {
		let mut result = Replacer::default();

		if let Some(filename) = self.source.file_name() {
			result.insert("filename", filename.to_string_lossy());
		}

		if let Some(filestem) = self.source.file_stem() {
			result.insert("filestem", filestem.to_string_lossy());
		}

		if let Some(extension) = self.source.extension() {
			result.insert("extension", extension.to_string_lossy());
		}

		result
	}
}

#[derive(Debug, Clone)]
pub enum AssetKind {
	Text(String),
	Binary,
}
