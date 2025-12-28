use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum SHAError{
	#[error("SHA validation failed for file: {0}")]
	FailedValidation(String),
}

pub fn validate_file(path: impl AsRef<Path>, hash: impl AsRef<str>) -> bool {
	let Ok(file) = File::open(path.as_ref()) else {
		return false;
	};

	let mut reader = BufReader::new(file);
	let mut hasher = Sha1::new();
	let mut buffer = [0u8; 8192];

	loop {
		match reader.read(&mut buffer) {
			Ok(0) => break,
			Ok(n) => hasher.update(&buffer[..n]),
			Err(_) => return false,
		}
	}

	let result = hasher.finalize();
	let computed: String = result.iter().map(|b| format!("{:02x}", b)).collect();
	computed.eq_ignore_ascii_case(hash.as_ref())
}
