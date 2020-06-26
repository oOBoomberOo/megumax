use std::path::PathBuf;

pub fn current_dir() -> PathBuf {
	PathBuf::from(".")
}

pub fn output_dir() -> PathBuf {
	PathBuf::from("build")
}
