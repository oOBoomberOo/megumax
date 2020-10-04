use anyhow::{Context, Result};
use colorful::*;
use flexi_logger::{style, Level, Logger};
use megumax::{app, config};
use structopt::StructOpt;

mod feature;
use feature::Command;

fn main() {
	let opts = Command::from_args();

	if let Err(err) = run(opts) {
		eprintln!("{} {:#}", "⚠".red(), err);
	}
}

pub fn run(opts: Command) -> Result<()> {
	if !opts.quiet {
		init_logger().unwrap();
	}

	let config = config::load_config(&opts.config)?;
	app::build_project(&config)
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
