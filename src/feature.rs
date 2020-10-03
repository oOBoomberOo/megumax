use std::path::PathBuf;
use structopt::StructOpt;

/// Easy to use templating CLI
#[derive(Debug, StructOpt)]
pub struct Command {
	/// Path to the config file
	#[structopt(long, short, parse(from_os_str), default_value = "megu.toml")]
	pub config: PathBuf,

	/// No output printed to stdout
	#[structopt(long, short)]
	pub quiet: bool,
}
