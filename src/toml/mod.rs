use super::config::{resolve_symbol, Config};
use anyhow::Result;
use path_solver::Pool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod consts;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFormat {
	#[serde(default)]
	pub template: TemplateFormat,
	pub build: BuildFormat,
}

impl ConfigFormat {
	pub fn compile(self) -> Result<Config> {
		log::debug!("Compile config format...");
		let (src, build) = self.build.compile();
		let template = self.template.compile()?;
		let result = Config::new(src, build, template);
		Ok(result)
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
	fn compile(self) -> Result<Pool> {
		log::debug!("Compile template format...");
		let mut pool = Pool::default_rule();

		for (key, value) in self.0 {
			let key = format!("[{}]", key);
			pool.insert(key, value);
		}

		Ok(pool)
	}
}
