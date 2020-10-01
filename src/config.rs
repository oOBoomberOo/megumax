use super::toml::ConfigFormat;
use anyhow::{Context, Result};
use path_solver::Pool;
use std::path::{Path, PathBuf};

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
	let path = path.as_ref();
	log::debug!("Load config from {}", path.display());
	let content = read_from_path(&path)?;
	load_from_string(&content)
}

pub fn load_from_string(content: &str) -> Result<Config> {
	let format: ConfigFormat = toml::from_str(content)?;
	log::debug!("Config Content: {:#?}", format);
	format.compile()
}

pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
	let path = path.as_ref();
	std::fs::read_to_string(path)
		.with_context(|| format!("Cannot find config file at `{}`", path.display()))
}

#[derive(Debug)]
pub struct Config {
	pub source: PathBuf,
	pub dest: PathBuf,
	pub template: Pool,
}

impl Config {
	pub fn new(src: PathBuf, build: PathBuf, pool: Pool) -> Self {
		Self {
			source: src,
			dest: build,
			template: pool,
		}
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
