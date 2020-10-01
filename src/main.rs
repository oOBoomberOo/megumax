use anyhow::{Context, Result};
use colorful::*;
use flexi_logger::{style, Level, Logger};
use std::path::PathBuf;
use structopt::StructOpt;

use megumax::{app, config};

/// A rust CLI that apply global search-and-replace across the entire project when building
#[derive(Debug, StructOpt)]
pub struct Command {
	/// Path to the config file
	#[structopt(long, short, parse(from_os_str), default_value = "megu.toml")]
	config: PathBuf,

	/// Include hidden files and directories in the output
	#[structopt(long, short = "H")]
	hidden: bool,

	/// No output printed to stdout
	#[structopt(long, short)]
	quiet: bool,
}

fn main() {
	let opts = Command::from_args();

	if let Err(err) = run(opts) {
		eprintln!("{} {:#}", "⚠".red(), err);
	}
}

fn run(opts: Command) -> Result<()> {
	if !opts.quiet {
		init_logger().unwrap();
	}

	let config = config::load_config(&opts.config)?;
	app::run(config)?;
	Ok(())
}

fn init_logger() -> Result<()> {
	Logger::with_str("megumax=info")
		.format(|w, _, record| match record.level() {
			Level::Info => write!(w, "{}", record.args()),
			Level::Error => write!(w, "{} {}", "⚠".red(), record.args()),
			level => write!(w, "[{}] {}", style(level, level), record.args()),
		})
		.start()
		.with_context(|| "Initializing logger")?;
	Ok(())
}
