use super::config::{resolve_symbol, Config, ConfigBuilder};
use megumax_template::{Pool, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod consts;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFormat {
	#[serde(default)]
	pub template: TemplateFormat,
	#[serde(default)]
	pub keys: KeyFormat,
	pub build: BuildFormat,
}

impl ConfigFormat {
	pub fn compile(self, path: PathBuf) -> Config {
		log::debug!("Compile config format...");
		let (src, dest) = self.build.compile();
		let template = self.template.compile();
		let keys = self.keys.compile();

		ConfigBuilder::new(src, dest, path)
			.with_template(template)
			.with_keys(keys)
			.build()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildFormat {
	#[serde(default = "consts::output_dir")]
	pub output: PathBuf,
	#[serde(default = "consts::current_dir")]
	pub src: PathBuf,
}

impl BuildFormat {
	fn compile(self) -> (PathBuf, PathBuf) {
		let src = resolve_symbol(self.src);
		log::debug!("Resolve source path into {:?}", src);
		let build = resolve_symbol(self.output);
		log::debug!("Resolve build path into {:?}", build);
		(src, build)
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TemplateFormat(HashMap<String, Vec<String>>);

impl TemplateFormat {
	fn compile(self) -> Pool {
		log::debug!("Compile template format...");
		let mut pool = Pool::default_rule();

		for (key, value) in self.0 {
			let key = format!("[{}]", key);
			pool.insert(key, value);
		}

		pool
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyFormat(HashMap<String, String>);

impl KeyFormat {
	fn compile(self) -> Template {
		let mut pool = HashMap::with_capacity(self.0.capacity());

		for (key, value) in self.0 {
			let key = format!("[{}]", key);
			pool.insert(key, value);
		}

		Template::new(pool)
	}
}
