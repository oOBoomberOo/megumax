use super::{Handle, Replacer};
use glob::{MatchOptions, Pattern};
use log::debug;
use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct Template {
	input: Pattern,
	output: Handle,
	options: MatchOptions,
	replacers: Vec<Replacer>,
}

impl Template {
	pub fn new(input: Pattern, output: Handle) -> Self {
		let options = MatchOptions::default();
		let replacers = Vec::default();
		Self {
			input,
			output,
			options,
			replacers,
		}
	}
	pub fn with_replacers(mut self, replacers: Vec<Replacer>) -> Self {
		self.replacers = replacers;
		self
	}
	pub fn match_path<P: AsRef<Path>>(&self, path: P) -> bool {
		self.input.matches_path_with(path.as_ref(), self.options)
	}

	pub fn outpath<P: AsRef<Path>>(&self, path: P, replacer: &Replacer) -> Option<PathBuf> {
		let parent = path.as_ref().parent()?;
		debug!(
			"Parent of {} is {}",
			path.as_ref().display(),
			parent.display()
		);
		let relative = self.output.clone().into_path(replacer);
		debug!("Create path {} from {}", relative.display(), self.output);
		let result = parent.join(relative);
		debug!("Combined into {}", result.display());
		Some(result)
	}

	pub fn replacers(&self) -> impl Iterator<Item = &Replacer> {
		self.replacers.iter()
	}
}

impl std::fmt::Debug for Template {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Template")
			.field("input", &self.input.as_str())
			.field("output", &self.output.0)
			.finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detect_template() {
		let input = Pattern::new("**/foo.template").unwrap();
		let output = Handle::from("{{key}}.mcfunction");
		let template = Template::new(input, output);

		assert!(template.match_path("foo.template"));
		assert!(template.match_path("data/somewhere/foo.template"));
		assert!(template.match_path("foo.template/bar/baz/foo.template"));
	}

	#[test]
	fn generate_template_path() {
		let input = Pattern::new("**/foo.template").unwrap();
		let output = Handle::from("{{key}}.mcfunction");

		let mut foo_replacer = Replacer::default();
		foo_replacer.insert("key", "foo");

		let mut bar_replacer = Replacer::default();
		bar_replacer.insert("key", "bar");

		let mut baz_replacer = Replacer::default();
		baz_replacer.insert("key", "baz");

		let template = Template::new(input, output);

		let result = template
			.outpath("data/foo.template", &foo_replacer)
			.unwrap();
		let expect = PathBuf::from("data/foo.mcfunction");
		assert_eq!(result, expect);

		let result = template
			.outpath("data/nested/foo.template", &bar_replacer)
			.unwrap();
		let expect = PathBuf::from("data/nested/bar.mcfunction");
		assert_eq!(result, expect);

		let result = template
			.outpath("src/toml/foo.template", &baz_replacer)
			.unwrap();
		let expect = PathBuf::from("src/toml/baz.mcfunction");
		assert_eq!(result, expect);
	}
}
