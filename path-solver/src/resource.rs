use crate::solver::Solver;
use crate::template::Template;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Resource<'a> {
	pub path: PathBuf,
	pub template: Template<'a>,
}

impl<'a> Resource<'a> {
	pub fn new(path: PathBuf, template: Template<'a>) -> Self {
		Self { path, template }
	}
}

pub struct Resources<'a> {
	path: String,
	inner: Solver<'a>,
}

impl<'a> Resources<'a> {
	pub fn new(path: String, inner: Solver<'a>) -> Self {
		Self { path, inner }
	}
}

impl<'a> Iterator for Resources<'a> {
	type Item = Resource<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let template = self.inner.next()?;
		let path = template.replace(&self.path);
		let result = Resource::new(path.into(), template);
		Some(result)
	}
}
