use std::ops::Range;
use std::str::{from_utf8, from_utf8_unchecked, Utf8Error};
use thiserror::Error;

use imports::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] io::Error),
	#[error(transparent)]
	InvalidString(#[from] Utf8Error),
}

pub fn check_expression_block(input: &str) -> bool {
	input
		.rfind('[')
		.map(|n| input[n..].find(']'))
		.map(|x| x.is_some())
		.unwrap_or(true)
}

pub struct StringStream<R, F> {
	buffer: Vec<u8>,
	size: usize,
	inner: Bytes<R>,
	ended: bool,
	f: F,
}

impl<R, F> StringStream<R, F>
where
	F: Fn(&str) -> bool,
	R: Read,
{
	pub const DEFAULT_SIZE: usize = 128;

	pub fn new(inner: R, f: F) -> Self {
		Self::with_size(inner, f, Self::DEFAULT_SIZE)
	}

	pub fn size(&self) -> usize {
		self.buffer.len()
	}

	fn size_needed(&self) -> usize {
		self.size
	}

	fn possible_range(&self) -> Range<usize> {
		let min = self.size().saturating_sub(3);
		let max = self.size();
		min..max
	}

	fn parse(&mut self) -> Result<String> {
		let range = self.possible_range();
		let buffer = &mut self.buffer;
		let result = from_utf8(&buffer).map(|s| s.to_owned());

		let result = match result {
			Ok(v) => Ok(v),
			Err(e) => match e.error_len() {
				Some(_) => Err(e.into()),
				None => {
					let at = e.valid_up_to();

					if range.contains(&at) {
						let result = Self::unfinish_buffer(buffer, at);
						return Ok(result);
					} else {
						Err(e.into())
					}
				}
			},
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

	fn should_stop(&self, input: &Result<String>) -> bool {
		self.ended || input.as_ref().map(|s| (self.f)(&s)).unwrap_or(true)
	}

	fn reinsert_content(&mut self, content: &str) {
		let tmp = self.buffer.clone();
		self.buffer = content.as_bytes().to_vec();
		self.buffer.extend(tmp);
	}
}

#[cfg(not(feature = "async"))]
mod imports {
	use super::*;
	pub use std::io::{self, Bytes, Read};
	use std::iter::FusedIterator;

	impl<R: Read, F> StringStream<R, F>
	where
		F: Fn(&str) -> bool,
	{
		pub fn with_size(inner: R, f: F, buffer_size: usize) -> Self {
			assert!(buffer_size >= 4); // Assert that buffer size cannot be less than the maximum size of UTF-8 string

			let buffer = Vec::with_capacity(buffer_size);
			let inner = inner.bytes();
			Self {
				buffer,
				inner,
				size: buffer_size,
				ended: false,
				f,
			}
		}
	}

	impl<R, F> Iterator for StringStream<R, F>
	where
		R: Read,
		F: Fn(&str) -> bool,
	{
		type Item = Result<String>;

		fn next(&mut self) -> Option<Self::Item> {
			if self.ended {
				return None;
			}

			loop {
				let prev_size = self.size();
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

				let new_size = self.size();
				if new_size - prev_size < n {
					self.ended = true;
				}

				let result = self.parse();

				if self.should_stop(&result) {
					return Some(result);
				}

				if let Ok(content) = result {
					self.reinsert_content(&content);
				}
			}
		}
	}

	impl<R: Read, F: Fn(&str) -> bool> FusedIterator for StringStream<R, F> {}
}

#[cfg(feature = "async")]
mod imports {
	use super::*;
	use futures::StreamExt;
	pub use smol::io::{self, AsyncRead as Read, Bytes};

	impl<R, F> StringStream<R, F>
	where
		R: Read + io::AsyncReadExt,
		F: Fn(&str) -> bool,
	{
		pub fn with_size(inner: R, f: F, buffer_size: usize) -> Self {
			assert!(buffer_size >= 4); // Assert that buffer size cannot be less than the maximum size of UTF-8 string
			let buffer = Vec::with_capacity(buffer_size);
			let inner = inner.bytes();
			Self {
				buffer,
				size: buffer_size,
				inner,
				ended: false,
				f,
			}
		}
	}

	impl<R, F> StringStream<R, F>
	where
		R: Read + Unpin,
		F: Fn(&str) -> bool,
	{
		pub async fn next(&mut self) -> Option<Result<String>> {
			if self.ended {
				return None;
			}

			loop {
				let prev_size = self.size();
				let n = self.size_needed();

				let mut stream = self.inner.by_ref().take(n);
				while let Some(byte) = stream.next().await {
					match byte {
						Ok(v) => self.buffer.push(v),
						Err(e) => return Some(Err(e.into())),
					}
				}

				let new_size = self.size();
				if new_size - prev_size < n {
					self.ended = true;
				}

				let result = self.parse();

				if self.should_stop(&result) {
					return Some(result);
				}

				if let Ok(content) = result {
					self.reinsert_content(&content);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(feature = "async")]
	fn collect<R, F>(mut stream: StringStream<R, F>) -> Result<String>
	where
		R: Read + Unpin,
		F: Fn(&str) -> bool,
	{
		smol::block_on(async {
			let mut result = String::new();
			while let Some(n) = stream.next().await {
				result += n?.as_ref();
			}

			Ok(result)
		})
	}

	#[cfg(not(feature = "async"))]
	fn collect<R: Read, F: Fn(&str) -> bool>(stream: StringStream<R, F>) -> Result<String> {
		stream.collect()
	}

	#[test]
	#[cfg(not(feature = "async"))]
	fn partial_template() {
		let content = "hello [world]";
		let mut reader = StringStream::with_size(content.as_bytes(), check_expression_block, 8);

		let result = reader.next().unwrap();
		assert_eq!(result.unwrap(), content);
	}

	#[test]
	#[cfg(feature = "async")]
	fn partial_template() {
		let content = "hello [world]";
		let mut reader = StringStream::with_size(content.as_bytes(), check_expression_block, 8);

		let result = smol::block_on(reader.next()).unwrap();
		assert_eq!(result.unwrap(), content);
	}

	#[test]
	fn valid_string() {
		let content = "Never gonna give you up, Never gonna let you down";

		let reader = StringStream::with_size(content.as_bytes(), |_| true, 10);
		let result = collect(reader);

		assert_eq!(result.unwrap(), content);
	}

	#[test]
	fn invalid_string() {
		let content = vec![0, 159, 146, 150];
		let reader = StringStream::with_size(content.as_slice(), |_| true, 10);
		let result = collect(reader);

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
			let reader = StringStream::with_size(content.as_slice(), |_| true, x);
			let result = collect(reader);

			prop_assert_eq!(result.unwrap(), "🎈🧨🎉");
		}

		#[test]
		fn unicode_input(size in 4usize..1024, content in "\\PC*") {
			let reader = StringStream::with_size(content.as_bytes(), |_| true, size);
			let result = collect(reader);
			prop_assert_eq!(result.unwrap(), content);
		}
	}
}
