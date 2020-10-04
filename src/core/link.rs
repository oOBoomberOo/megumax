use super::special;
use crate::share::{create_file, open_file, stringify_path, File};
use anyhow::{Context, Result};
use path_solver::{Pool, Resource};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Link {
	pub from: PathBuf,
	pub to: PathBuf,
}

#[cfg(feature = "async")]
impl Link {
	pub async fn read(&self) -> Result<File> {
		open_file(&self.from).await
	}

	pub async fn create(&self) -> Result<File> {
		create_file(&self.to).await
	}
}

#[cfg(not(feature = "async"))]
impl Link {
	pub fn read(&self) -> Result<File> {
		open_file(&self.from)
	}

	pub fn create(&self) -> Result<File> {
		create_file(&self.to)
	}
}

impl Link {
	pub fn new(from: PathBuf, to: PathBuf) -> Self {
		Self { from, to }
	}

	pub fn to_resources<'a>(&self, pool: &'a Pool) -> Result<impl Iterator<Item = Resource> + 'a> {
		let path = stringify_path(&self.to)?;
		let resources = pool
			.template_resources(path)
			.with_context(|| format!("Looking up keyword in `{}`", path))?;

		let resources = resources.enumerate().map(special::nth_template);

		Ok(resources)
	}

	pub fn with_resource(&self, resource: &Resource) -> Self {
		let from = self.from.clone();
		let to = resource.path.clone();
		Self::new(from, to)
	}
}
