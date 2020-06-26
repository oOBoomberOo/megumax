use super::config::{resolve_symbol, Config, Error};
use super::{Handle, Replacer, Template};
use glob::Pattern;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod consts;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFormat {
	#[serde(default)]
	pub keys: HashMap<String, String>,
	#[serde(default)]
	pub template: Vec<TemplateFormat>,
	pub build: BuildFormat,
}

impl ConfigFormat {
	pub fn compile(self) -> Result<Config> {
		info!("Compile config format...");
		let master_keys = Replacer::from_config(self.keys);
		let (src, build) = compile_build(self.build);
		let templates = compile_templates(self.template)?;
		let result = Config::new(master_keys, src, build, templates);
		Ok(result)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildFormat {
	#[serde(default = "consts::output_dir")]
	pub output: PathBuf,
	#[serde(default)]
	pub src: Option<PathBuf>,
}

pub fn compile_build(build: BuildFormat) -> (Option<PathBuf>, PathBuf) {
	let src = build.src.map(resolve_symbol);
	debug!("Resolve source path into {:?}", src);
	let build = resolve_symbol(build.output);
	debug!("Resolve build path into {:?}", build);
	(src, build)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateFormat {
	input: String,
	output: String,
	#[serde(default)]
	values: Vec<HashMap<String, String>>,
}

impl TemplateFormat {
	pub fn compile(self) -> Result<Template> {
		info!("Compile template format...");
		let input = self.input.parse::<Pattern>()?;
		let output = Handle::from(self.output);
		let replacers = compile_replacers(self.values);

		let result = Template::new(input, output).with_replacers(replacers);
		Ok(result)
	}
}

pub fn compile_templates(templates: Vec<TemplateFormat>) -> Result<Vec<Template>> {
	templates.into_iter().map(|v| v.compile()).collect()
}

pub fn compile_replacers(replacers: Vec<HashMap<String, String>>) -> Vec<Replacer> {
	replacers.into_iter().map(Replacer::from_config).collect()
}
