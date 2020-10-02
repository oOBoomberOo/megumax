use super::toml::ConfigFormat;
use crate::core::Link;
use anyhow::{Context, Result};
use path_solver::{Pool, Template};
use std::path::{Path, PathBuf};

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
	let path = path.as_ref();
	log::debug!("Load config from {}", path.display());
	let content = read_from_path(path)?;
	let format: ConfigFormat = toml::from_str(&content)?;
	let config = format.compile(path.to_path_buf());
	Ok(config)
}

pub fn load_from_string(content: &str) -> Result<Config> {
	let format: ConfigFormat = toml::from_str(content)?;
	log::debug!("Config Content: {:#?}", format);
	let config = format.compile("megu.toml".into());
	Ok(config)
}

pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
	let path = path.as_ref();
	std::fs::read_to_string(path)
		.with_context(|| format!("Cannot find config file at `{}`", path.display()))
}

#[derive(Debug, Clone)]
pub struct Config {
	pub source: PathBuf,
	pub dest: PathBuf,
	pub config_path: PathBuf,
	pub template: Pool,
	pub keys: Template,
}

impl Config {
	pub fn new(
		source: PathBuf,
		dest: PathBuf,
		config_path: PathBuf,
		template: Pool,
		keys: Template,
	) -> Self {
		Self {
			source,
			dest,
			config_path,
			template,
			keys,
		}
	}

	pub fn replace_prefix(&self, path: &Path) -> Result<PathBuf> {
		Link::replace_prefix(path, &self.source, &self.dest)
	}
}

impl Config {
	pub fn clear_build_dir(&self) -> Result<()> {
		log::debug!("Clearing destination directory...");

		let path = &self.dest;

		if !path.exists() {
			log::debug!("Build directory doesn't exists, skipped.");
			return Ok(());
		}

		std::fs::remove_dir_all(path)?;
		Ok(())
	}
}

pub fn resolve_symbol(path: PathBuf) -> PathBuf {
	let path_str = path.to_string_lossy();
	let result = shellexpand::tilde(&path_str);
	PathBuf::from(result.as_ref())
}
