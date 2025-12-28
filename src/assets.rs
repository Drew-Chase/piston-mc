#![doc = include_str!("../.wiki/Assets.md")]

use crate::assets::AssetError::{AssetFailedToValidate, AssetNotFound};
use crate::download_util::{FileDownloadArguments, MultiDownloadProgress, download_multiple_files};
use crate::sha_validation::validate_file;
use anyhow::{Result, anyhow};
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

const MINECRAFT_RESOURCE_CDN: &str = "https://resources.download.minecraft.net";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Assets {
    pub url: String,
    pub asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub objects: HashMap<String, AssetItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetItem {
    pub hash: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetValidationResult {
    pub asset_id: String,
    pub succeeded: Vec<String>,
    pub failed: Vec<AssetValidationFailureResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetValidationFailureResult {
    pub hash: String,
    pub reason: AssetValidationFailureReason,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AssetValidationFailureReason {
    FileNotFound,
    HashNotMatching,
}

#[derive(thiserror::Error, Debug)]
pub enum AssetError {
    #[error("Asset '{name}' not found in path '{path}'")]
    AssetNotFound { name: String, path: PathBuf },
    #[error("Asset '{name}' is not valid in path '{path}'")]
    AssetFailedToValidate { name: String, path: PathBuf },
}

impl Assets {
    pub async fn from_url(url: impl AsRef<str>) -> Result<Self> {
        let url = url.as_ref();
        debug!("Fetching versions manifest");

        let manifest = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        let objects = manifest.get("objects").ok_or_else(|| anyhow!("missing `objects`"))?;
        let objects: HashMap<String, AssetItem> = serde_json::from_value(objects.clone())?;
        debug!("Found {} versions in manifest", objects.len());

        let id = url.split("/").last().ok_or_else(|| anyhow!("invalid url"))?.trim_end_matches(".json");

        Ok(Assets { asset_id: id.to_string(), path: None, url: url.to_string(), objects })
    }

    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path).await?;
        let mut assets = serde_json::from_str::<Self>(&content)?;
        assets.path = Some(path.to_path_buf());
        Ok(assets)
    }

    pub async fn download(
        &mut self,
        directory: impl AsRef<Path>,
        parallel: u16,
        sender: Option<tokio::sync::mpsc::Sender<MultiDownloadProgress>>,
    ) -> Result<()> {
        let directory = directory.as_ref();
        if !directory.exists() {
            tokio::fs::create_dir_all(&directory).await?;
        }
        self.path = Some(directory.to_path_buf());
        let mut file = tokio::fs::File::create(directory.join(format!("{}.json", self.asset_id))).await?;
        file.write_all(serde_json::to_string(&self)?.as_bytes()).await?;
        let download_items: Vec<FileDownloadArguments> = self
            .objects
            .values()
            .map(|item| FileDownloadArguments {
                url: item.get_download_url(),
                sha1: Some(item.hash.clone()),
                sender: None,
                path: item.get_download_path(directory).to_string_lossy().into_owned(),
            })
            .collect();

        download_multiple_files(download_items, parallel, sender).await?;

        Ok(())
    }

    pub async fn validate(&self, parallel: u16) -> Result<AssetValidationResult> {
        let path = self.path.as_ref().ok_or_else(|| anyhow!("Asset path was not set"))?;

        let items: Vec<_> = self.objects.iter().map(|(name, item)| (name.clone(), item.clone())).collect();

        let results: Vec<_> = stream::iter(items)
            .map(|(name, item)| {
                let path = path.clone();
                tokio::task::spawn_blocking(move || (name, item.validate(&path)))
            })
            .buffer_unordered(parallel as usize)
            .collect()
            .await;

        let mut result = AssetValidationResult { asset_id: self.asset_id.clone(), succeeded: vec![], failed: vec![] };

        for join_result in results {
            let (name, validation) = join_result?;
            match validation {
                Ok(_) => result.succeeded.push(name),
                Err(err) => result.failed.push(AssetValidationFailureResult {
                    hash: name,
                    reason: match err {
                        AssetNotFound { .. } => AssetValidationFailureReason::FileNotFound,
                        AssetFailedToValidate { .. } => AssetValidationFailureReason::HashNotMatching,
                    },
                }),
            }
        }

        Ok(result)
    }
}

impl AssetItem {
    pub fn get_download_url(&self) -> String {
        let hash = self.hash.clone();
        let dir = hash.chars().take(2).collect::<String>();
        format!("{}/{}/{}", MINECRAFT_RESOURCE_CDN, dir, hash)
    }
    pub fn get_download_path(&self, asset_dir: impl AsRef<Path>) -> PathBuf {
        let asset_dir = asset_dir.as_ref();
        let hash = self.hash.clone();
        let dir = hash.chars().take(2).collect::<String>();
        asset_dir.join(dir).join(&hash)
    }

    pub fn validate(&self, asset_dir: impl AsRef<Path>) -> Result<(), AssetError> {
        let file_path = self.get_download_path(&asset_dir);
        if !file_path.exists() {
            return Err(AssetNotFound { name: self.hash.clone(), path: file_path });
        }

        if !validate_file(&file_path, &self.hash) {
            return Err(AssetFailedToValidate { name: self.hash.clone(), path: file_path });
        }

        Ok(())
    }
}
