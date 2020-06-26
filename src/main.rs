use std::path::PathBuf;
use structopt::StructOpt;

mod app;
mod config;
mod replacer;
mod template;
mod toml;
mod utils;

use replacer::{Handle, Replacer};
use template::Template;

/// A rust CLI that apply global search-and-replace across the entire project when building
#[derive(Debug, StructOpt)]
pub struct Command {
	/// Path to the config file
	#[structopt(long, short, parse(from_os_str), default_value = "megu.toml")]
	config: PathBuf,

	/// Include hidden files and directories in the output
	#[structopt(long, short = "H")]
	hidden: bool,
}

fn main() {
	env_logger::init();
	let opts = Command::from_args();

	if let Err(err) = run(opts) {
		eprintln!("{}", err);
	}
}

fn run(opts: Command) -> config::Result<()> {
	let config = config::load_config(&opts.config)?;
	app::build(config, opts)?;
	Ok(())
}
