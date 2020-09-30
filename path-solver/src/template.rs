use crate::resource::Resources;
use crate::solver::Solver;
use regex::Regex;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Template<'a> {
	keys: HashMap<String, &'a str>,
}

impl<'a> Template<'a> {
	pub fn new(keys: HashMap<String, &'a str>) -> Self {
		Self { keys }
	}

	pub fn replace(&self, content: &str) -> String {
		self.keys
			.iter()
			.fold(content.to_string(), |f, (k, v)| f.replace(k, v))
	}

	pub fn insert(mut self, key: String, value: &'a str) -> Self {
		self.set(key, value);
		self
	}

	pub fn set(&mut self, key: String, value: &'a str) {
		self.keys.insert(key, value);
	}
}

impl<'a> FromIterator<(String, &'a str)> for Template<'a> {
	fn from_iter<T: IntoIterator<Item = (String, &'a str)>>(iter: T) -> Self {
		let keys = iter.into_iter().collect();
		Self::new(keys)
	}
}

#[derive(Debug)]
pub struct Pool {
	pool: HashMap<String, Vec<String>>,
	capture_rule: Regex,
}

impl Pool {
	pub fn new(capture_rule: Regex) -> Self {
		let pool = HashMap::new();
		Self { capture_rule, pool }
	}

	pub fn default_rule() -> Self {
		let rule = Regex::new(r"(\[[\w\d_\-]+?\])").unwrap();
		Self::new(rule)
	}

	pub fn get(&self, key: &str) -> Option<&[String]> {
		self.pool.get(key).map(|v| v.as_slice())
	}

	pub fn insert(&mut self, key: String, value: Vec<String>) {
		self.pool.insert(key, value);
	}

	pub fn append(&mut self, key: impl Into<String>, value: impl Into<String>) {
		let list = self.pool.entry(key.into()).or_default();
		list.push(value.into());
	}

	pub fn intersect(&self, keys: &[String]) -> Option<Vec<&[String]>> {
		let mut result = Vec::new();

		for key in keys {
			let variants = self.get(key)?;
			result.push(variants);
		}

		Some(result)
	}
}

impl Pool {
	pub fn capture(&self, content: &str) -> Vec<String> {
		self.capture_rule
			.captures_iter(content)
			.filter_map(|capture| capture.get(1))
			.map(|matches| matches.as_str())
			.map(|matches| matches.to_string())
			.collect()
	}

	pub fn template_resources<P: Into<String>>(&self, path: P) -> Option<Resources> {
		let path = path.into();
		let keys = self.capture(&path);
		let list = self.intersect(&keys)?;
		let inner = Solver::new(list, keys)?;
		let result = Resources::new(path, inner);
		Some(result)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn capture_flag() {
		let pool = Pool::default_rule();
		let content = "/[color]/foo/[shape]";

		let result = pool.capture(content);
		let expect = vec!["[color]".to_string(), "[shape]".into()];

		assert_eq!(result, expect);
	}

	#[test]
	fn template_resource() {
		let mut pool = Pool::default_rule();
		pool.append("[color]", "red");
		pool.append("[color]", "green");
		pool.append("[color]", "blue");
		pool.append("[color]", "yellow");

		let path = "/foo/[color]_wool.mcfunction";

		let resources = pool.template_resources(path).unwrap();

		let result: Vec<_> = resources.map(|r| r.path).collect();
		let expect: Vec<_> = vec![
			"/foo/red_wool.mcfunction",
			"/foo/green_wool.mcfunction",
			"/foo/blue_wool.mcfunction",
			"/foo/yellow_wool.mcfunction",
		]
		.into_iter()
		.map(PathBuf::from)
		.collect();

		assert_eq!(result, expect);
	}

	#[test]
	fn empty_template() {
		let pool = Pool::default_rule();
		let path = "/dummy";
		let resources = pool.template_resources(path).unwrap();
		let result: Vec<_> = resources.map(|r| r.path).collect();
		let expect: Vec<_> = vec!["/dummy"].into_iter().map(PathBuf::from).collect();

		assert_eq!(result, expect)
	}
}
