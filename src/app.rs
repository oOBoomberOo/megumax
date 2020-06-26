use super::config::{Config, Result};
use super::Command;
use ignore::{DirEntry, Walk, WalkBuilder};
use log::{debug, error, info};
use std::path::Path;

pub fn build(config: Config, opts: Command) -> Result<()> {
	info!("Begin building project...");
	let root = config.source();
	let files = Files::new(root, opts.hidden);

	config.clear_build_dir()?;

	for entry in files {
		let path = entry.path();

		if path.is_file() {
			config.on(path)?;
		}
	}

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
