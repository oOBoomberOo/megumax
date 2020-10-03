use anyhow::{Context, Result};
use path_solver::{Pool, Resource};
use std::path::{Path, PathBuf};

mod walker;

#[cfg(not(feature = "async"))]
mod extra {
	pub use std::fs::File;
}

#[cfg(feature = "async")]
mod extra {
	pub use smol::fs::File;
}

use extra::*;
pub use walker::*;

pub mod special {
	pub const ITERATION_COUNT: &str = "[nth]";
}

#[derive(Debug, Clone)]
pub struct Link {
	pub from: PathBuf,
	pub to: PathBuf,
}

#[cfg(feature = "async")]
impl Link {
	pub async fn read(&self) -> Result<File> {
		let path = &self.from;
		log::debug!("Reading `{}`", path.display());
		File::open(path)
			.await
			.with_context(|| format!("`{}` cannot be read", path.display()))
	}

	pub async fn create(&self) -> Result<File> {
		let path = &self.to;
		Self::ensure_parent(path).await?;
		log::debug!("Creating `{}`", path.display());
		File::create(path)
			.await
			.with_context(|| format!("`{}` cannot be created", path.display()))
	}

	async fn ensure_parent<P>(path: P) -> Result<()>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();

		if let Some(parent) = path.parent() {
			smol::fs::create_dir_all(parent)
				.await
				.with_context(|| format!("Creating directories from `{}`", parent.display()))?;
		}

		Ok(())
	}
}

#[cfg(not(feature = "async"))]
impl Link {
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
}

impl Link {
	pub fn new(from: PathBuf, to: PathBuf) -> Self {
		Self { from, to }
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

	pub fn replace_prefix(path: &Path, from: &Path, to: &Path) -> Result<PathBuf> {
		let rel_path = path.strip_prefix(from)?;
		let result = to.join(rel_path);
		Ok(result)
	}
}
