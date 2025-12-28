use crate::assets::Assets;
use crate::download_util::{download_and_validate_file, download_file, DownloadProgress};
use crate::manifest_v2::ReleaseType;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionManifest {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimal_launcher_version: u8,
    #[serde(rename = "releaseTime")]
    pub release_time: chrono::DateTime<chrono::Utc>,
    pub time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "type")]
    pub release_type: ReleaseType,
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "complianceLevel", skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<u8>,
    pub downloads: Downloads,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Arguments {
    Post113(Post113),
    Pre113(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post113 {
    pub game: Vec<GameArgument>,
    pub jvm: Vec<GameArgument>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GameArgument {
    /// Simple string argument like "--username"
    Plain(String),
    /// Conditional argument with rules
    Conditional(ConditionalArgument),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConditionalArgument {
    pub rules: Vec<Rule>,
    pub value: ArgumentValue,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ArgumentValue {
    /// Single value like "--demo"
    Single(String),
    /// Multiple values like ["--width", "${resolution_width}"]
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
    #[serde(default)]
    pub os: Option<OsRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Downloads {
    pub client: Download,
    pub server: Option<Download>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Download {
    #[serde(skip_serializing_if = "Option::is_none", alias = "path")]
    pub id: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Logging {
    pub client: ClientLogging,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientLogging {
    pub argument: String,
    #[serde(rename = "type")]
    pub log_type: String,
    pub file: Download,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LibraryItem {
    pub name: String,
    pub downloads: LibraryDownload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LibraryDownload {
    pub name: String,
    pub artifact: Download,
}

impl VersionManifest {
    pub async fn from_url(url: impl AsRef<str>) -> Result<Self> {
        let url = url.as_ref();
        let response = reqwest::get(url).await?;
        let text = response.text().await?;
        let json_result = serde_json::from_str::<Self>(&text);

        if let Err(ref e) = json_result {
            let line = e.line();
            let column = e.column();
            log::error!("Failed to deserialize VersionManifest from {}: {}", url, e);
            log::error!("Error at line {}, column {}", line, column);

            // Show context around the error (60 chars before and after)
            let error_offset = text.lines().take(line - 1).map(|l| l.len() + 1).sum::<usize>() + column - 1;
            let start = error_offset.saturating_sub(60);
            let end = (error_offset + 60).min(text.len());
            let context = &text[start..end];

            log::error!("Context around error:");
            log::error!("{}", context);
        }

        Ok(json_result?)
    }

    pub async fn download_client(
        &self,
        path: impl AsRef<Path>,
        validate: bool,
        sender: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
    ) -> Result<()> {
        let path = path.as_ref();
        let url = &self.downloads.client.url;
        let hash = &self.downloads.client.sha1;

        if validate {
            download_and_validate_file(url, path, hash, sender).await?;
        } else {
            download_file(url, path, sender).await?;
        }

        Ok(())
    }
    pub async fn download_server(
        &self,
        path: impl AsRef<Path>,
        validate: bool,
        sender: Option<tokio::sync::mpsc::Sender<DownloadProgress>>,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(server) = &self.downloads.server {
            let url = &server.url;
            let hash = &server.sha1;

            if validate {
                download_and_validate_file(url, path, hash, sender).await?;
            } else {
                download_file(url, path, sender).await?;
            }
        } else {
            return Err(anyhow!("No server download available"));
        }

        Ok(())
    }

    pub async fn assets(&self) -> Result<Assets> {
        Assets::from_url(&self.asset_index.url).await
    }
}

#[cfg(test)]
mod test {
    use crate::setup_logging;
    #[tokio::test]
    async fn download_server() {
        use crate::manifest_v2::ManifestV2;
        use crate::version_manifest::VersionManifest;
        setup_logging();

        let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");
        let release_id = &manifest.latest.release;
        let version: anyhow::Result<Option<VersionManifest>> = manifest.version(release_id).await;
        if let Ok(Some(version)) = version {
            let output = format!("target/test/server-{}.jar", release_id);
            version.download_server(output, true, None).await.unwrap();
        }else{
            panic!("Failed to fetch version.");
        }
    }
    #[tokio::test]
    async fn download_client() {
        use crate::manifest_v2::ManifestV2;
        use crate::version_manifest::VersionManifest;
        setup_logging();

        let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");
        let release_id = &manifest.latest.release;
        let version: anyhow::Result<Option<VersionManifest>> = manifest.version(release_id).await;
        if let Ok(Some(version)) = version {
            let output = format!("target/test/client-{}.jar", release_id);
            version.download_client(output, true, None).await.unwrap();
        }else{
            panic!("Failed to fetch version.");
        }
    }
}
