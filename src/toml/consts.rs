use std::path::PathBuf;

pub fn current_dir() -> PathBuf {
	PathBuf::from("src")
}

pub fn output_dir() -> PathBuf {
	PathBuf::from("build")
}
