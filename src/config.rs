use super::toml::{consts::current_dir, ConfigFormat};
use super::utils::{normalize_path, remove_all, Asset, AssetKind};
use super::{Handle, Replacer, Template};
use glob::PatternError;
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Cannot find config file at {0}")]
	MissingConfig(PathBuf),

	#[error(transparent)]
	Toml(#[from] toml::de::Error),

	#[error("[{1}] {0}")]
	Io(std::io::Error, PathBuf),

	#[error("Unable to strip prefix {1} from {0}")]
	StripPrefix(PathBuf, PathBuf),

	#[error("Cannot find parent path for {0}")]
	Parent(PathBuf),

	#[error(transparent)]
	Pattern(#[from] PatternError),
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
	let path = path.as_ref();
	info!("Load config from {}", path.display());
	let content = read_from_path(&path)?;
	load_from_string(&content)
}

pub fn load_from_string(content: &str) -> Result<Config> {
	let format: ConfigFormat = toml::from_str(content)?;
	debug!("Config Content: {:#?}", format);
	format.compile()
}

pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
	let path = path.as_ref();
	std::fs::read_to_string(&path).map_err(|_| Error::MissingConfig(path.to_path_buf()))
}

#[derive(Debug)]
pub struct Config {
	pub master_keys: Replacer,
	pub src: Option<PathBuf>,
	pub build: PathBuf,
	pub templates: Vec<Template>,
}

impl Config {
	pub fn new(
		master_keys: Replacer,
		src: Option<PathBuf>,
		build: PathBuf,
		templates: Vec<Template>,
	) -> Self {
		Self {
			master_keys,
			src,
			build,
			templates,
		}
	}

	pub fn source(&self) -> PathBuf {
		self.src.to_owned().unwrap_or_else(current_dir)
	}

	pub fn templates(&self) -> impl Iterator<Item = &Template> {
		self.templates.iter()
	}

	pub fn match_templates<P: AsRef<Path>>(&self, path: P) -> Vec<&Template> {
		self.templates().filter(|t| t.match_path(&path)).collect()
	}

	pub fn on<P: AsRef<Path>>(&self, path: P) -> Result<()> {
		let path = path.as_ref();
		info!("Handling {}...", path.display());
		let matches = self.match_templates(&path);

		if matches.is_empty() {
			debug!("Doesn't have any match.");
			self.normal_file(&path)
		} else {
			debug!("Found {} matches.", matches.len());
			self.template_file(&path, matches)
		}
	}

	pub fn normal_file(&self, path: &Path) -> Result<()> {
		let output = self.out_path(&path)?;
		info!(
			"Generate file from {} to {}",
			path.display(),
			output.display()
		);
		let asset = new_asset(path)?;
		self.file_helper(&asset, &output, &Replacer::default())?;
		Ok(())
	}

	pub fn file_helper(&self, asset: &Asset, path: &Path, replacer: &Replacer) -> Result<()> {
		match &asset.kind {
			AssetKind::Text(content) => {
				let content = Handle::from(content);
				let replaced = replacer.apply(content);
				let master_key = self.replace(replaced);
				master_key
					.write_to(&path)
					.map_err(|err| Error::Io(err, path.to_owned()))?;
			}
			AssetKind::Binary => {
				asset
					.copy(path)
					.map_err(|err| Error::Io(err, path.to_owned()))?;
			}
		}

		Ok(())
	}

	pub fn template_file(&self, path: &Path, matches: Vec<&Template>) -> Result<()> {
		let asset = new_asset(path)?;

		for template in matches {
			for replacer in template.replacers() {
				let output = template
					.outpath(path, &replacer)
					.ok_or_else(|| Error::Parent(path.to_owned()))?;
				let output = self.out_path(output)?;
				let output =
					normalize_path(&output).map_err(|err| Error::Io(err, output.to_path_buf()))?;
				self.file_helper(&asset, &output, replacer)?;
			}
		}

		Ok(())
	}

	pub fn replace(&self, content: Handle) -> Handle {
		self.master_keys.apply(content)
	}

	pub fn relative_path<'p>(&self, path: &'p Path) -> Result<&'p Path> {
		match &self.src {
			Some(source) => strip_prefix(path, source),
			None => Ok(path),
		}
	}

	pub fn out_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
		let stripped = self.relative_path(path.as_ref())?;
		let result = self.build.join(stripped);
		Ok(result)
	}
}

impl Config {
	pub fn clear_build_dir(&self) -> Result<()> {
		info!("Clearing build-dir...");
		if !self.build.exists() {
			info!("Build directory doesn't exists, skipped.");
			return Ok(());
		}

		self.clean_build_dir_helper()
			.map_err(|err| Error::Io(err, self.build.to_owned()))
	}

	fn clean_build_dir_helper(&self) -> std::io::Result<()> {
		// Clear directory's content but not the directory itself
		for entry in self.build.read_dir()? {
			let path = entry?.path();
			info!("Removing {}...", path.display());
			remove_all(path)?;
		}

		Ok(())
	}
}

pub fn strip_prefix<'p, P: AsRef<Path>>(path: &'p Path, prefix: P) -> Result<&'p Path> {
	path.strip_prefix(&prefix)
		.map_err(|_| Error::StripPrefix(path.to_path_buf(), prefix.as_ref().to_path_buf()))
}

pub fn new_asset(path: &Path) -> Result<Asset> {
	Asset::from_path(path).map_err(|err| Error::Io(err, path.to_owned()))
}

pub fn resolve_symbol(path: PathBuf) -> PathBuf {
	let path_str = path.to_string_lossy();
	let result = shellexpand::tilde(&path_str);
	PathBuf::from(result.as_ref())
}

#[cfg(test)]
mod tests {
	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn replace_content() {
		let raw_config = r#"
		[build]
		output = "build/"

		[keys]
		name = "Colorful Cauldron"
		version = "1.0.0"
		'model.cauldron.empty' = "8080001"
		'model.cauldron.slightly_filled' = "8080002"
		'model.cauldron.half_filled' = "8080003"
		'model.cauldron.fully_filled' = "8080004"
		'model.dye_bottle' = "8080005"
		"#;
		let config = load_from_string(raw_config).unwrap();

		let content = r#"
		say This is {{name}} version {{version}}

		give @s leather_chestplate{CustomModelData: {{model.cauldron.empty}}}
		give @s leather_chestplate{CustomModelData: {{model.cauldron.slightly_filled}}}
		give @s leather_chestplate{CustomModelData: {{model.cauldron.half_filled}}}
		give @s leather_chestplate{CustomModelData: {{model.cauldron.fully_filled}}}
		give @s potion{Potion: "minecraft:water", CustomModelData: {{model.dye_bottle}}}
		"#;

		let result = config.replace(content.into());
		let expect = r#"
		say This is Colorful Cauldron version 1.0.0

		give @s leather_chestplate{CustomModelData: 8080001}
		give @s leather_chestplate{CustomModelData: 8080002}
		give @s leather_chestplate{CustomModelData: 8080003}
		give @s leather_chestplate{CustomModelData: 8080004}
		give @s potion{Potion: "minecraft:water", CustomModelData: 8080005}
		"#;

		assert_eq!(result, expect.into());
	}

	#[test]
	fn outpath() {
		let raw_config = r#"
		[build]
		output = "build/"
		"#;
		let config = load_from_string(raw_config).unwrap();

		let paths = vec![
			PathBuf::from("./data/minecraft/tags/functions/tick.json"),
			PathBuf::from("./data/global/advancements/root.json"),
		];

		let result: Vec<_> = paths
			.into_iter()
			.map(|path| config.out_path(path).unwrap())
			.collect();
		let expect = vec![
			PathBuf::from("build/data/minecraft/tags/functions/tick.json"),
			PathBuf::from("build/data/global/advancements/root.json"),
		];

		assert_eq!(result, expect);
	}

	#[test]
	fn outpath_with_custom_src() {
		let raw_config = r#"
		[build]
		output = "build/"
		src = "src/"
		"#;
		let config = load_from_string(raw_config).unwrap();

		let paths = vec![
			PathBuf::from("src/data/minecraft/tags/functions/tick.json"),
			PathBuf::from("src/data/global/advancements/root.json"),
		];

		let result: Vec<_> = paths
			.into_iter()
			.map(|path| config.out_path(path).unwrap())
			.collect();
		let expect = vec![
			PathBuf::from("build/data/minecraft/tags/functions/tick.json"),
			PathBuf::from("build/data/global/advancements/root.json"),
		];

		assert_eq!(result, expect);
	}

	#[test]
	#[should_panic]
	fn outpath_with_path_outside_of_src() {
		let raw_config = r#"
		[build]
		output = "build/"
		src = "src/"
		"#;
		let config = load_from_string(raw_config).unwrap();
		config.out_path("foo/bar/test.json").unwrap();
	}
}
