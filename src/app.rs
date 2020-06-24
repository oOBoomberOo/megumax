use super::config::CompiledConfig as Config;
use super::Command;
use ignore::{DirEntry, Walk, WalkBuilder};
use log::{debug, error};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Config(#[from] super::config::Error),
}

pub fn build(config: Config, opts: Command) -> Result<()> {
	let root = &config.src;
	let files = Files::new(root, opts.hidden);

	config.clear_build_dir()?;

	for entry in files {
		let path = entry.path();

		if path.is_file() {
			read_or_copy(path, &config)?;
		}
	}

	Ok(())
}

pub fn read_or_copy<P: AsRef<Path>>(path: P, config: &Config) -> Result<()> {
	let path = path.as_ref();
	let content = std::fs::read_to_string(&path);

	match content {
		// A valid UTF-8 file, apply search-and-replace behavior
		Ok(content) => {
			let result = config.replace(content);
			config.write(path, result)?;
		}
		// The file is not in UTF-8 encoding, so we treat it as non-text file and just copy the content.
		Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
			let from = path;
			let to = config.out_path(path)?;
			std::fs::copy(from, to)?;
		}
		Err(err) => return Err(err.into()),
	};

	Ok(())
}

pub struct Files {
	iter: Walk,
}

impl Files {
	pub fn new<P: AsRef<Path>>(root: P, hidden: bool) -> Self {
		debug!("Iterate over all files in {}", root.as_ref().display());
		let iter = WalkBuilder::new(root).hidden(hidden).build();
		Self { iter }
	}
}

impl Iterator for Files {
	type Item = DirEntry;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(entry) = self.iter.next() {
			match entry {
				Ok(entry) => return Some(entry),
				Err(err) => error!("{}", err),
			}
		}
		None
	}
}
