use crate::solver::Solver;
use crate::template::Template;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Resource {
	pub path: PathBuf,
	pub template: Template,
}

impl Resource {
	pub fn new(path: PathBuf, template: Template) -> Self {
		Self { path, template }
	}

	pub fn replace(&self, content: &str) -> String {
		self.template.replace(content)
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
	type Item = Resource;

	fn next(&mut self) -> Option<Self::Item> {
		let template = self.inner.next()?;
		let path = template.replace(&self.path);
		let result = Resource::new(path.into(), template);
		Some(result)
	}
}
