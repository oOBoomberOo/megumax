use super::toml::Config;
use log::error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
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
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<CompiledConfig> {
	let path = path.as_ref();
	let content = read_from_path(&path)?;
	load_from_string(&content)
}

pub fn load_from_string(content: &str) -> Result<CompiledConfig> {
	let config = toml::from_str(content)?;
	let result = CompiledConfig::from_config(config);
	Ok(result)
}

pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
	let path = path.as_ref();
	std::fs::read_to_string(&path).map_err(|_| Error::MissingConfig(path.to_path_buf()))
}

pub struct CompiledConfig {
	pub keys: HashMap<String, String>,
	pub src: PathBuf,
	pub output: PathBuf,
}

impl CompiledConfig {
	pub fn replace(&self, content: String) -> String {
		self.keys
			.iter()
			.fold(content, |content, (from, to)| content.replace(from, to))
	}

	pub fn write<P: AsRef<Path>>(&self, path: P, content: String) -> Result<()> {
		let output = self.out_path(path)?;
		create_file(&output, content).map_err(|err| Error::Io(err, output))
	}

	pub fn clear_build_dir(&self) -> Result<()> {
		let no_output_dir = !self.output.exists();
		if no_output_dir {
			return Ok(());
		}

		std::fs::remove_dir_all(&self.output).map_err(|err| Error::Io(err, self.output.clone()))
	}

	pub fn relative_path<'p>(&self, path: &'p Path) -> Result<&'p Path> {
		strip_prefix(path, &self.src)
	}

	pub fn out_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
		let stripped = self.relative_path(path.as_ref())?;
		let result = self.output.join(stripped);
		Ok(result)
	}

	pub fn from_config(config: Config) -> Self {
		let (keys, build) = config.compile();
		let (output, src) = build.compile();
		Self { keys, src, output }
	}
}

fn strip_prefix<'p>(path: &'p Path, prefix: &Path) -> Result<&'p Path> {
	path.strip_prefix(prefix)
		.map_err(|_| Error::StripPrefix(path.to_path_buf(), prefix.to_owned()))
}

pub fn create_file<P: AsRef<Path>>(path: P, content: String) -> std::io::Result<()> {
	if let Some(parent) = path.as_ref().parent() {
		std::fs::create_dir_all(parent)?;
	}
	let mut file = File::create(path)?;
	let buffer = content.as_bytes();
	file.write_all(buffer)?;
	Ok(())
}

pub fn resolve_symbol(path: PathBuf) -> PathBuf {
	let path_str = path.to_string_lossy();
	let result = shellexpand::tilde(&path_str);
	PathBuf::from(result.as_ref())
}

#[cfg(test)]
mod tests {
	use super::*;

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

		let result = config.replace(content.to_owned());
		let expect = r#"
		say This is Colorful Cauldron version 1.0.0

		give @s leather_chestplate{CustomModelData: 8080001}
		give @s leather_chestplate{CustomModelData: 8080002}
		give @s leather_chestplate{CustomModelData: 8080003}
		give @s leather_chestplate{CustomModelData: 8080004}
		give @s potion{Potion: "minecraft:water", CustomModelData: 8080005}
		"#;

		assert_eq!(result, expect);
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
