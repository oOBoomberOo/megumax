use super::Link;
use crate::config::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(not(feature = "async"))]
pub struct Walker {
	source: PathBuf,
	destination: PathBuf,
	iter: Walk,
}
#[cfg(feature = "async")]
pub struct Walker {
	source: PathBuf,
	destination: PathBuf,
	ignore: Gitignore,
	iter: WalkDir,
}

impl Walker {
	pub fn from_config(config: &Config) -> Self {
		Self::new(config.source.clone(), config.dest.clone())
	}

	fn create_link(&self, path: &Path) -> Result<Link> {
		let from = path.to_path_buf();
		let to = Link::replace_prefix(path, &self.source, &self.destination)?;
		Ok(Link::new(from, to))
	}
}

use extras::*;

#[cfg(not(feature = "async"))]
mod extras {
	use super::*;
	pub use ignore::{Walk, WalkBuilder};

	impl Walker {
		pub fn new(source: PathBuf, destination: PathBuf) -> Self {
			let iter = WalkBuilder::new(&source).hidden(false).build();
			Self {
				source,
				destination,
				iter,
			}
		}

		fn precondition(&self, path: &Path) -> bool {
			let is_file = path.is_file();
			let is_from_destination = path.starts_with(&self.destination);

			is_file && !is_from_destination
		}
	}

	impl Iterator for Walker {
		type Item = Result<Link>;

		fn next(&mut self) -> Option<Self::Item> {
			while let Some(entry) = self.iter.next() {
				match entry {
					Ok(entry) => {
						let path = entry.path();

						if self.precondition(path) {
							let link = self.create_link(path);
							return Some(link);
						}
					}
					Err(e) => {
						return Some(Err(e.into()));
					}
				}
			}

			None
		}
	}
}

#[cfg(feature = "async")]
mod extras {
	use super::*;
	pub use async_walkdir::WalkDir;
	use futures::prelude::*;
	pub use ignore::gitignore::Gitignore;

	impl Walker {
		pub fn new(source: PathBuf, destination: PathBuf) -> Self {
			let (ignore, _) = Gitignore::new(&source);
			let iter = WalkDir::new(&source);
			Self {
				source,
				destination,
				ignore,
				iter,
			}
		}

		pub async fn next(&mut self) -> Option<Result<Link>> {
			while let Some(entry) = self.iter.next().await {
				match entry {
					Ok(entry) => {
						let path = entry.path();
						if self.precondition(&path) {
							let link = self.create_link(&path);
							return Some(link);
						}
					}
					Err(e) => return Some(Err(e.into())),
				}
			}

			None
		}

		fn precondition(&self, path: &Path) -> bool {
			let is_file = path.is_file();
			let is_from_destination = path.starts_with(&self.destination);
			let gitignore = self.ignore.matched(path, !is_file).is_ignore();

			is_file && !is_from_destination && !gitignore
		}
	}
}
