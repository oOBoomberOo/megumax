use crate::config::Config;
use crate::core::{Link, Walker};
use anyhow::{Context, Result};
use colorful::*;
use path_solver::{Resource, Template};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub fn run(config: Config) -> Result<()> {
	log::info!("{}", "Megumax is running...".light_red());
	log::info!(
		"  ├─ {} {}",
		"⬅".green(),
		config.source.to_string_lossy().blue()
	);
	log::info!(
		"  ╰─ {} {}",
		"➡".green(),
		config.dest.to_string_lossy().blue()
	);
	log::info!("");

	let files = Walker::from_config(&config);
	config.clear_build_dir()?;

	for link in files {
		let link = link?;

		let resources = link.to_resources(&config.template)?.peekable();

		let path = format_path(&link.from);
		log::info!("  Generate {}:", path.light_yellow());

		for resource in resources {
			let path = build(resource, &link, &config.keys)?;
			log::info!("    {} {}", "✔".light_green(), path.blue());
		}
		log::info!("");
	}

	Ok(())
}

fn build(resource: Resource, link: &Link, keys: &Template) -> Result<String> {
	let link = link.with_resource(&resource);

	let mut reader = BufReader::new(link.read()?);
	let mut writer = BufWriter::new(link.create()?);

	let mut buffer = String::new();
	reader
		.read_to_string(&mut buffer)
		.with_context(|| format!("Read from `{}`", link.from.display()))?;

	let content = resource.replace(&buffer);
	let content = keys.replace(&content);
	writer
		.write_all(content.as_bytes())
		.with_context(|| format!("Write to `{}`", link.to.display()))?;

	Ok(format_path(&link.to))
}

fn format_path(path: &Path) -> String {
	let components: Vec<String> = path
		.components()
		.map(|s| s.as_os_str())
		.map(|s| s.to_string_lossy())
		.map(|s| s.to_string())
		.collect();
	components.join("/")
}
