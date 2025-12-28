use crate::sha_validation;
use crate::sha_validation::SHAError;
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct DownloadProgress {}

pub async fn download_file(
    url: impl AsRef<str>,
    path: impl AsRef<Path>,
    sender: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    Ok(())
}

pub async fn download_and_validate_file(
    url: impl AsRef<str>,
    path: impl AsRef<Path>,
    hash: impl AsRef<str>,
    sender: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    let path = path.as_ref();
	download_file(url, path, sender).await?;

    match sha_validation::validate_file(path, hash) {
        true => Ok(()),
        false => Err(anyhow!(SHAError::FailedValidation(format!(
            "Failed to validate file: {}",
            path.display()
        )))),
    }
}
