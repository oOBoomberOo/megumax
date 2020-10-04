use crate::config::Config;
use crate::core::Link;
use colorful::*;
use megumax_template::Resource;
use std::path::Path;

pub fn create(link: &Link) {
	let path = format_path(&link.from);
	log::info!("  Generate {}:", path.light_yellow());
}

pub fn create_resource(resource: &Resource) {
	let path = format_path(&resource.path);
	log::info!("    {} {}", "✔".light_green(), path.blue());
}

pub fn config_info(config: &Config) {
	let source = format_path(&config.source);
	let dest = format_path(&config.dest);

	log::info!("{}", "Megumax is running...".light_red());
	log::info!("  ├─ {} {}", "⬅".green(), source.blue());
	log::info!("  ╰─ {} {}", "➡".green(), dest.blue());
	log::info!("");
}

pub fn newline() {
	log::info!("");
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
