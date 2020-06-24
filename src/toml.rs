use super::config::resolve_symbol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub keys: HashMap<String, String>,
	pub build: BuildConfig,
}

impl Config {
	pub fn compile(self) -> (HashMap<String, String>, BuildConfig) {
		let keys = self
			.keys
			.into_iter()
			.map(surround_key_with_braces)
			.collect();
		let build = self.build;
		(keys, build)
	}
}

fn surround_key_with_braces(item: (String, String)) -> (String, String) {
	let (key, value) = item;
	(format!("{{{{{}}}}}", key), value) // Surround the key with two braces
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
	pub output: PathBuf,
	#[serde(default = "current_dir")]
	pub src: PathBuf,
}

impl BuildConfig {
	pub fn compile(self) -> (PathBuf, PathBuf) {
		let output = resolve_symbol(self.output);
		let src = resolve_symbol(self.src);
		(output, src)
	}
}

fn current_dir() -> PathBuf {
	PathBuf::from(".")
}
