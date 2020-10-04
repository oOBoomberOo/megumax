use ignore::gitignore::Gitignore;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Filter {
	ignore: Gitignore,
	dest: PathBuf,
}

impl Filter {
	pub fn new(ignore: Gitignore, dest: PathBuf) -> Self {
		Self { ignore, dest }
	}

	pub fn check<P: AsRef<Path>>(&self, path: P) -> bool {
		let is_file = self.is_file(&path);
		let is_ignore = self.is_ignore(&path);
		let is_build_dir = self.is_build_dir(&path);

		is_file && !is_ignore && !is_build_dir
	}

	pub fn from_path<P: AsRef<Path>>(source: P, dest: P) -> Self {
		let (ignore, _) = Gitignore::new(source);
		let dest = dest.as_ref().to_path_buf();
		Self::new(ignore, dest)
	}

	fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
		path.as_ref().is_file()
	}

	fn is_ignore<P: AsRef<Path>>(&self, path: P) -> bool {
		let path = path.as_ref();
		let is_dir = path.is_dir();
		self.ignore.matched(path, is_dir).is_ignore()
	}

	fn is_build_dir<P: AsRef<Path>>(&self, path: P) -> bool {
		path.as_ref().starts_with(&self.dest)
	}
}
