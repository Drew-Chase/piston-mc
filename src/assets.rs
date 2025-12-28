use crate::download_util::{MultiDownloadProgress, download_file};
use anyhow::{Result, anyhow};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

const MINECRAFT_RESOURCE_CDN: &str = "https://resources.download.minecraft.net";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Assets {
    pub url: String,
    pub asset_id: String,
    pub objects: HashMap<String, AssetItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetItem {
    pub hash: String,
    pub size: u64,
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

        Ok(Assets { asset_id: id.to_string(), url: url.to_string(), objects })
    }

    pub async fn download(
        &self,
        directory: impl AsRef<Path>,
        parallel: u16,
        sender: Option<tokio::sync::mpsc::Sender<MultiDownloadProgress>>,
    ) -> Result<()> {
        let directory = directory.as_ref();
        if !directory.exists() {
            tokio::fs::create_dir_all(&directory).await?;
        }
        let mut file = tokio::fs::File::create(directory.join(format!("{}.json", self.asset_id))).await?;
        file.write_all(serde_json::to_string(&self)?.as_bytes()).await?;

        Ok(())
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
}
