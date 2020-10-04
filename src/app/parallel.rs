use super::{check_expression_block, message};
use crate::config::Config;
use crate::core::{Link, Walker};
use crate::utils::StringStream;
use anyhow::Result;
use futures::prelude::*;
use megumax_template::{Resource, Template};
use smol::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};

pub fn build_project(config: &Config) -> Result<()> {
	smol::block_on(async { build_project_inner(config).await })
}

async fn build_project_inner(config: &Config) -> Result<()> {
	message::config_info(config);

	let mut files = Walker::from_config(&config);
	config.clear_build_dir()?;

	while let Some(link) = files.next().await {
		let link = link?;

		message::create(&link);
		let resources = link.to_resources(&config.template)?;
		let mut streams = stream::iter(resources).map(|x| create(x, &link, &config.keys));

		while let Some(resource) = streams.next().await {
			let resource = resource.await?;
			message::create_resource(&resource);
		}

		message::newline();
	}

	Ok(())
}

async fn create(resource: Resource, link: &Link, keys: &Template) -> Result<Resource> {
	let link = link.with_resource(&resource);

	let reader = BufReader::new(link.read().await?);
	let writer = BufWriter::new(link.create().await?);

	let apply_template = |content: String| {
		let content = resource.replace(&content);
		keys.replace(&content)
	};

	let result = generate_text(reader, writer, apply_template, check_expression_block).await;

	// Returning error mean the file is a binary file and we just have to copy its content and not replacing anything.
	if result.is_err() {
		smol::fs::remove_file(&link.to).await?;
		smol::fs::copy(&link.from, &link.to).await?;
	}

	Ok(resource)
}

/// Read string from the Reader, transform it using `f` function and write it into the Writer.
async fn generate_text<R, W, F, G>(reader: R, mut writer: W, f: F, g: G) -> Result<()>
where
	R: AsyncRead + std::marker::Unpin,
	W: AsyncWrite + std::marker::Unpin,
	F: Fn(String) -> String,
	G: Fn(&str) -> bool,
{
	let mut stream = StringStream::new(reader, g);

	while let Some(content) = stream.next().await {
		let content = f(content?);
		writer.write_all(content.as_bytes()).await?;
	}

	writer.flush().await?;

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
			smol::block_on(async {
				generate_text(reader, &mut writer, |s| s, check_expression_block).await
			}).unwrap();
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
			smol::block_on(async {
				generate_text(reader, &mut writer, |s| s, check_expression_block).await
			}).unwrap_err();

			let result = String::from_utf8(writer).unwrap();
			prop_assert_eq!(result, content);
		}
	}
}
