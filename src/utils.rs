use std::io::{self, Bytes, Read};
use std::iter::FusedIterator;
use std::ops::Range;
use std::str::{from_utf8, from_utf8_unchecked, Utf8Error};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] io::Error),
	#[error(transparent)]
	InvalidString(#[from] Utf8Error),
}

pub struct StringStream<R> {
	buffer: Vec<u8>,
	inner: Bytes<R>,
	ended: bool,
}

impl<R: Read> StringStream<R> {
	pub const DEFAULT_SIZE: usize = 128;

	pub fn new(inner: R) -> Self {
		Self::with_size(inner, Self::DEFAULT_SIZE)
	}

	pub fn with_size(inner: R, buffer_size: usize) -> Self {
		assert!(buffer_size >= 4); // Assert that buffer size cannot be less than the maximum size of UTF-8 string

		let buffer = Vec::with_capacity(buffer_size);
		let inner = inner.bytes();
		Self {
			buffer,
			inner,
			ended: false,
		}
	}

	pub fn size(&self) -> usize {
		self.buffer.capacity()
	}

	fn size_needed(&self) -> usize {
		self.size() - self.buffer.len()
	}

	fn possible_range(&self) -> Range<usize> {
		let min = self.size() - 3;
		let max = self.size();
		min..max
	}

	fn parse(&mut self) -> Result<String> {
		let range = self.possible_range();
		let buffer = &mut self.buffer;
		let result = from_utf8(&buffer).map(|s| s.to_owned());

		let result = match result {
			Ok(v) => Ok(v),
			Err(e) => {
				let at = e.valid_up_to();

				if range.contains(&at) {
					let result = Self::unfinish_buffer(buffer, at);
					return Ok(result);
				} else {
					Err(e.into())
				}
			}
		};

		buffer.clear();
		result
	}

	/// Parse a valid part of the buffer to UTF-8 string *and* left the invalid part inside the buffer.
	fn unfinish_buffer(buffer: &mut Vec<u8>, at: usize) -> String {
		let (a, b) = buffer.split_at(at);

		let result = if a.is_empty() {
			String::default()
		} else {
			// This function is only called when we know the invalid part of the string
			unsafe { from_utf8_unchecked(a).to_owned() }
		};

		let mut tmp = b.to_vec();
		buffer.clear();
		buffer.append(&mut tmp);
		result
	}
}

impl<R: Read> Iterator for StringStream<R> {
	type Item = Result<String>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.ended {
			return None;
		}

		let n = self.size_needed();
		let tmp = self
			.inner
			.by_ref()
			.take(n)
			.collect::<Result<Vec<_>, _>>()
			.map_err(Error::from);

		match tmp {
			Ok(v) => self.buffer.extend(v),
			Err(e) => return Some(Err(e)),
		}

		if self.buffer.len() < self.size() {
			self.ended = true;
		}

		let result = self.parse();
		Some(result)
	}
}

impl<R: Read> FusedIterator for StringStream<R> {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_string() {
		let content = "Never gonna give you up, Never gonna let you down";

		let reader = StringStream::with_size(content.as_bytes(), 10);
		let result: Result<String> = reader.collect();

		assert_eq!(result.unwrap(), content);
	}

	#[test]
	fn invalid_string() {
		let content = vec![0, 159, 146, 150];
		let reader = StringStream::with_size(content.as_slice(), 10);
		let result: Result<String> = reader.collect();

		match result {
			Err(Error::InvalidString(_)) => (),
			_ => panic!("Expecting `Error::InvalidString` but got {:?}", result),
		}
	}

	use proptest::prelude::*;

	proptest! {
		#[test]
		fn incomplete_string(x in 4usize..1024) {
			let content = vec![
				0xf0, 0x9f, 0x8e, 0x88, 0xf0, 0x9f, 0xa7, 0xa8, 0xf0, 0x9f, 0x8e, 0x89,
			];
			let reader = StringStream::with_size(content.as_slice(), x);
			let result: Result<String> = reader.collect();

			prop_assert_eq!(result.unwrap(), "🎈🧨🎉");
		}

		#[test]
		fn unicode_input(size in 4usize..1024, content in "\\PC*") {
			let reader = StringStream::with_size(content.as_bytes(), size);
			let result: Result<String> = reader.collect();
			prop_assert_eq!(result.unwrap(), content);
		}
	}
}
