use super::{check_expression_block, message};
use crate::config::Config;
use crate::core::{Link, Walker};
use crate::utils::StringStream;
use anyhow::Result;
use megumax_template::{Resource, Template};
use std::io::{BufReader, BufWriter, Read, Write};

pub fn build_project(config: &Config) -> Result<()> {
	message::config_info(config);

	let files = Walker::from_config(&config);
	config.clear_build_dir()?;

	for link in files {
		let link = link?;

		message::create(&link);
		let resources = link.to_resources(&config.template)?;
		resources
			.map(|x| create(x, &link, &config.keys))
			.try_for_each(|result| result.map(|x| message::create_resource(&x)))?;
		message::newline();
	}

	Ok(())
}

fn create(resource: Resource, link: &Link, keys: &Template) -> Result<Resource> {
	let link = link.with_resource(&resource);

	let reader = BufReader::new(link.read()?);
	let writer = BufWriter::new(link.create()?);

	let apply_template = |content: String| {
		let content = resource.replace(&content);
		keys.replace(&content)
	};

	let result = generate_text(reader, writer, apply_template, check_expression_block);

	// Returning error mean the file is a binary file and we just have to copy its content and not replacing anything.
	if result.is_err() {
		std::fs::remove_file(&link.to)?;
		std::fs::copy(&link.from, &link.to)?;
	}

	Ok(resource)
}

/// Read string from the Reader, transform it using `f` function and write it into the Writer.
///
/// Note: This function never load the entire string into memory.
fn generate_text<R, W, F, G>(reader: R, mut writer: W, f: F, g: G) -> Result<()>
where
	R: Read,
	W: Write,
	F: Fn(String) -> String,
	G: Fn(&str) -> bool,
{
	let stream = StringStream::new(reader, g);

	for content in stream {
		let content = f(content?);
		writer.write_all(content.as_bytes())?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use proptest::prelude::*;

	proptest! {
		#[test]
		fn mock_file_creation(content in "\\PC*") {
			let reader = content.as_bytes();
			let mut writer = Vec::new();
			generate_text(reader, &mut writer, |s| s, check_expression_block).unwrap();
			let result = String::from_utf8(writer).unwrap();
			prop_assert_eq!(result, content);
		}

		// TODO: Make this test work with unicode
		#[test]
		fn mock_binary_file(content in r#"[a-zA-Z0-9]{128}"#) {
			let mut invalid_content = content.clone().into_bytes();
			invalid_content.extend(&[0, 159, 146, 150]);

			let reader = invalid_content.as_slice();
			let mut writer = Vec::new();
			generate_text(reader, &mut writer, |s| s, check_expression_block).unwrap_err();

			let result = String::from_utf8(writer).unwrap();
			prop_assert_eq!(result, content);
		}
	}
}
