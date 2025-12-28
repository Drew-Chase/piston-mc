use crate::sha_validation;
use crate::sha_validation::SHAError;
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize)]
pub struct DownloadProgress {
    pub bytes_to_download: usize,
    pub bytes_downloaded: usize,
    pub bytes_per_second: usize,
}

pub async fn download_file(
    url: impl AsRef<str>,
    path: impl AsRef<Path>,
    sender: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
) -> Result<()> {
    let url = url.as_ref();
    let path = path.as_ref();
    let mut file = tokio::fs::File::create(path).await?;

    if let Some(sender) = sender {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0) as usize;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0;
        let start_time = Instant::now();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len();
            let elapsed = start_time.elapsed().as_secs_f64();
            let bytes_per_second = if elapsed > 0.0 {
                (downloaded as f64 / elapsed) as usize
            } else {
                0
            };

            let progress = DownloadProgress {
                bytes_to_download: total_size,
                bytes_downloaded: downloaded,
                bytes_per_second,
            };

            let _ = sender.send(progress).await;
        }
    } else {
        let result = reqwest::get(url).await?;
        let bytes = result.bytes().await?;
        file.write_all(bytes.iter().as_slice()).await?;
    }

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
