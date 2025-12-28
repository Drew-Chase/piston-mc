use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
const MINECRAFT_RESOURCE_CDN: &str = "https://resources.download.minecraft.net";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Assets {
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
        let manifest = reqwest::get(url).await?.json::<Self>().await?;
        debug!("Found {} versions in manifest", manifest.objects.len());
        Ok(manifest)
    }
}

impl AssetItem {
    pub fn get_download_url(&self) -> String {
        let hash = self.hash.clone();
        let dir = hash.chars().take(2).collect::<String>();
        format!("{}/{}/{}", MINECRAFT_RESOURCE_CDN, dir, hash)
    }
}
