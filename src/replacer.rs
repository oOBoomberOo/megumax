use super::utils::ensure_parent;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Replacer {
	keys: HashMap<String, String>,
}

impl Replacer {
	/// We assume that `keys` from config doesn't have brace surround it.
	/// This method will surround it with brace first and then create the struct.
	pub fn from_config(keys: HashMap<String, String>) -> Self {
		keys.into_iter().map(surround_key_with_braces).collect()
	}

	pub fn keys(&self) -> impl Iterator<Item = (&str, &str)> {
		self.keys.iter().map(transmute_ref)
	}

	pub fn apply(&self, handle: Handle) -> Handle {
		handle.apply(&self)
	}

	pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
		let key = key.into();
		let value = value.into();
		let (key, value) = surround_key_with_braces((key, value));
		self.keys.insert(key, value)
	}
}

// Transmuting references because compiler complains about it
fn transmute_ref<'a>((a, b): (&'a String, &'a String)) -> (&'a str, &'a str) {
	(a, b)
}

fn surround_key_with_braces(item: (String, String)) -> (String, String) {
	let (key, value) = item;
	let open_brace = "{{".to_owned();
	let close_brace = "}}";

	let key = open_brace + &key + close_brace; // Surround the key with two braces. format!() macro look horrible here so I do this instead.
	(key, value)
}

impl FromIterator<(String, String)> for Replacer {
	fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
		let keys = iter.into_iter().collect();
		Self { keys }
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Handle(pub String);

impl Handle {
	pub fn inner(&self) -> &str {
		&self.0
	}

	pub fn into_path(self, replacer: &Replacer) -> PathBuf {
		let path = self.apply(replacer);
		PathBuf::from(path)
	}

	pub fn into_string(self) -> String {
		self.0
	}

	pub fn apply(self, replacer: &Replacer) -> Self {
		let content = replacer.keys().fold(self.into_string(), replace_key);
		Self(content)
	}

	pub fn write_to<P: AsRef<Path>>(self, path: P) -> std::io::Result<()> {
		ensure_parent(&path)?;
		std::fs::write(path, self.inner())
	}
}

impl std::fmt::Display for Handle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

fn replace_key(content: String, (from, to): (&str, &str)) -> String {
	content.replace(from, to)
}

impl From<String> for Handle {
	fn from(inner: String) -> Self {
		Self(inner)
	}
}

impl From<&String> for Handle {
	fn from(inner: &String) -> Self {
		Self::from(inner.to_owned())
	}
}

impl From<&str> for Handle {
	fn from(inner: &str) -> Self {
		Self::from(inner.to_owned())
	}
}

impl From<Handle> for PathBuf {
	fn from(handle: Handle) -> Self {
		Self::from(handle.inner())
	}
}
