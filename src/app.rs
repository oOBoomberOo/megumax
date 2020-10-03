mod message;

#[cfg(not(feature = "async"))]
#[path = "app/sync.rs"]
mod imports;

#[cfg(feature = "async")]
#[path = "app/parallel.rs"]
mod imports;

pub use imports::*;

#[cfg(test)]
#[cfg(not(feature = "async"))]
mod tests {
	use super::*;
	use proptest::prelude::*;

	proptest! {
		#[test]
		fn mock_file_creation(content in "\\PC*") {
			let reader = content.as_bytes();
			let mut writer = Vec::new();
			generate_text(reader, &mut writer, |s| s).unwrap();
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
			generate_text(reader, &mut writer, |s| s).unwrap_err();

			let result = String::from_utf8(writer).unwrap();
			prop_assert_eq!(result, content);
		}
	}
}
