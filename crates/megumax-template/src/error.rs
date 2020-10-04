use thiserror::Error;

#[derive(Debug, Error)]
#[error("Cannot find `{key}` key in the template pool")]
pub struct KeyLookUpError {
	key: String,
}

impl KeyLookUpError {
	pub fn new(key: impl Into<String>) -> Self {
		Self { key: key.into() }
	}
}
