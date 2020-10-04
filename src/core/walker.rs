use super::{Filter, Link};
use crate::config::Config;
use crate::share::replace_prefix;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Walker {
	source: PathBuf,
	destination: PathBuf,
	iter: Iter,
	filter: Filter,
}

impl Walker {
	pub fn new(source: PathBuf, destination: PathBuf) -> Self {
		let iter = create_walker(&source, &destination);
		let filter = Filter::from_path(&source, &destination);
		Self {
			source,
			destination,
			iter,
			filter,
		}
	}

	pub fn from_config(config: &Config) -> Self {
		Self::new(config.source.clone(), config.dest.clone())
	}

	fn create_link(&self, path: &Path) -> Result<Link> {
		let from = path.to_path_buf();
		let to = replace_prefix(path, self.source.as_ref(), self.destination.as_ref())?;
		Ok(Link::new(from, to))
	}
}

use extras::*;

#[cfg(not(feature = "async"))]
mod extras {
	use super::*;
	use anyhow::Error;
	pub use ignore::{DirEntry, Walk as Iter, WalkBuilder};

	pub fn create_walker(source: &Path, _dest: &Path) -> Iter {
		WalkBuilder::new(source).hidden(false).build()
	}

	impl Iterator for Walker {
		type Item = Result<Link>;

		fn next(&mut self) -> Option<Self::Item> {
			loop {
				let entry = self.iter.next()?;
				let path = entry.map(|x| x.into_path()).map_err(Error::from);

				let path = match path {
					Ok(path) => path,
					Err(e) => return Some(Err(e)),
				};

				if self.filter.check(&path) {
					let link = self.create_link(&path);
					return Some(link);
				}
			}
		}
	}
}

#[cfg(feature = "async")]
mod extras {
	use super::*;
	use anyhow::Error;
	pub use async_walkdir::WalkDir as Iter;
	use futures::prelude::*;

	pub fn create_walker(source: &Path, _dest: &Path) -> Iter {
		Iter::new(&source)
	}

	impl Walker {
		pub async fn next(&mut self) -> Option<Result<Link>> {
			loop {
				let entry = self.iter.next().await?;
				let path = entry.map(|x| x.path()).map_err(Error::from);

				let path = match path {
					Ok(path) => path,
					Err(e) => return Some(Err(e)),
				};

				if self.filter.check(&path) {
					let link = self.create_link(&path);
					return Some(link);
				}
			}
		}
	}
}
