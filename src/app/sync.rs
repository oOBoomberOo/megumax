use super::message;
use crate::config::Config;
use crate::core::{Link, Walker};
use crate::utils::StringStream;
use anyhow::Result;
use path_solver::{Resource, Template};
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

	let result = generate_text(reader, writer, apply_template);

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
pub fn generate_text<R, W, F>(reader: R, mut writer: W, f: F) -> Result<()>
where
	R: Read,
	W: Write,
	F: Fn(String) -> String,
{
	let stream = StringStream::new(reader);

	for content in stream {
		let content = f(content?);
		writer.write_all(content.as_bytes())?;
	}

	Ok(())
}
