use crate::config::Config;
use anyhow::{Context, Result};
use ignore::{Walk, WalkBuilder};
use path_solver::{Pool, Resource};
use std::fs::File;
use std::path::{Path, PathBuf};

pub mod special {
	pub const ITERATION_COUNT: &str = "[nth]";
}

pub struct Walker {
	source: PathBuf,
	destination: PathBuf,
	iter: Walk,
}

impl Walker {
	pub fn new(source: PathBuf, destination: PathBuf) -> Self {
		let iter = WalkBuilder::new(&source).hidden(false).build();
		Self {
			source,
			destination,
			iter,
		}
	}

	pub fn from_config(config: &Config) -> Self {
		Self::new(config.source.clone(), config.dest.clone())
	}

	fn create_link(&self, path: &Path) -> Result<Link> {
		let from = path.to_path_buf();
		let to = Link::replace_prefix(path, &self.source, &self.destination)?;
		Ok(Link::new(from, to))
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

#[derive(Debug, Clone)]
pub struct Link {
	pub from: PathBuf,
	pub to: PathBuf,
}

impl Link {
	pub fn new(from: PathBuf, to: PathBuf) -> Self {
		Self { from, to }
	}

	pub fn read(&self) -> Result<File> {
		let path = &self.from;
		log::debug!("Reading `{}`", path.display());
		File::open(path).with_context(|| format!("`{}` cannot be read", path.display()))
	}

	pub fn create(&self) -> Result<File> {
		let path = &self.to;
		Self::ensure_parent(path)?;
		log::debug!("Creating `{}`", path.display());
		File::create(path).with_context(|| format!("`{}` cannot be created", path.display()))
	}

	pub fn to_resources<'a>(&self, pool: &'a Pool) -> Result<impl Iterator<Item = Resource> + 'a> {
		let path = Self::stringify_path(&self.to)?;
		let resources = pool
			.template_resources(path)
			.with_context(|| format!("Looking up keyword in `{}`", path))?
			.enumerate()
			.map(|(n, mut resource)| {
				resource
					.template
					.set(special::ITERATION_COUNT.into(), n.to_string());
				resource
			});

		Ok(resources)
	}

	pub fn with_resource(&self, resource: &Resource) -> Self {
		let from = self.from.clone();
		let to = resource.path.clone();
		Self::new(from, to)
	}

	fn stringify_path(path: &Path) -> Result<&str> {
		path.to_str()
			.with_context(|| format!("`{}` is not a valid UTF-8 path", path.display()))
	}

	fn ensure_parent<P>(path: P) -> Result<()>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();

		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)
				.with_context(|| format!("Creating directories from `{}`", parent.display()))?;
		}

		Ok(())
	}

	pub fn replace_prefix(path: &Path, from: &Path, to: &Path) -> Result<PathBuf> {
		let rel_path = path.strip_prefix(from)?;
		let result = to.join(rel_path);
		Ok(result)
	}
}
