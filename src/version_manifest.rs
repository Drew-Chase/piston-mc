#![doc = include_str!("../.wiki/VersionManifest.md")]

#[cfg(feature = "assets")]
use crate::assets::Assets;
use crate::manifest_v2::ReleaseType;
#[cfg(any(feature = "downloads", feature = "assets"))]
use anyhow::Result;
#[cfg(feature = "downloads")]
use anyhow::anyhow;
use anyhow::bail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[cfg(feature = "downloads")]
use simple_download_utility::{DownloadProgress, download_and_validate_file, download_file};
use simple_download_utility::{FileDownloadArguments, MultiDownloadProgress, download_multiple_files, download_multiple_files_with_client};
use std::collections::HashMap;
#[cfg(feature = "downloads")]
use std::path::Path;
use tokio::sync::mpsc::Sender;

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
    pub libraries: Vec<LibraryItem>,
}

//#[derive(Serialize, Deserialize, Clone, Debug)]
//pub struct Libraries{
//    pub records: Vec<LibraryItem>
//}

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
    /// Regex pattern for OS version matching (used in older versions)
    pub version: Option<String>,
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
    /// Native library mappings (OS name -> classifier key)
    /// Used for platform-specific native libraries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,
    /// Extraction rules for native libraries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<ExtractRules>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractRules {
    /// Paths to exclude when extracting (e.g., ["META-INF/"])
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LibraryDownload {
    /// Main artifact download (may be absent for native-only libraries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Download>,
    /// Platform-specific native classifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, Download>>,
}

#[cfg(feature = "downloads")]
impl VersionManifest {
    pub async fn from_url(url: impl AsRef<str>) -> Result<Self> {
        let url = url.as_ref();
        let response = reqwest::get(url).await?;
        let text = response.text().await?;
        let json_result = serde_json::from_str::<Self>(&text);

        #[cfg(feature = "log")]
        if let Err(ref e) = json_result {
            let line = e.line();
            let column = e.column();
            error!("Failed to deserialize VersionManifest from {}: {}", url, e);
            error!("Error at line {}, column {}", line, column);

            // Show context around the error (60 chars before and after)
            let error_offset = text.lines().take(line - 1).map(|l| l.len() + 1).sum::<usize>() + column - 1;
            let start = error_offset.saturating_sub(60);
            let end = (error_offset + 60).min(text.len());
            let context = &text[start..end];

            error!("Context around error: {}", context);
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
}

pub trait LibraryItemDownloader {
    fn download(&self, directory: impl AsRef<Path>, parallel: u16, sender: Option<Sender<MultiDownloadProgress>>)
    -> impl Future<Output = Result<()>>;
    fn download_with_client(
        &self,
        client: &Client,
        directory: impl AsRef<Path>,
        parallel: u16,
        sender: Option<Sender<MultiDownloadProgress>>,
    ) -> impl Future<Output = Result<()>>;
}

#[cfg(feature = "downloads")]
impl LibraryItemDownloader for Vec<LibraryItem> {
    async fn download(&self, directory: impl AsRef<Path>, parallel: u16, sender: Option<Sender<MultiDownloadProgress>>) -> Result<()> {
        let client = Client::new();
        self.download_with_client(&client, directory, parallel, sender).await
    }

    async fn download_with_client(
        &self,
        client: &Client,
        directory: impl AsRef<Path>,
        parallel: u16,
        sender: Option<Sender<MultiDownloadProgress>>,
    ) -> Result<()> {
        let directory = directory.as_ref();
        if !directory.exists() {
            tokio::fs::create_dir_all(&directory).await?;
        }

        let download_items: Vec<FileDownloadArguments> = self
            .iter()
            .filter_map(|item| {
                let classifiers = item.downloads.classifiers.clone();
                let artifact = item.downloads.artifact.clone();

                let url: String = if let Some(classifiers) = &classifiers {
                    #[cfg(target_os = "windows")]
                    let name = "natives-windows";
                    #[cfg(target_os = "linux")]
                    let name = "natives-linux";
                    #[cfg(target_os = "macos")]
                    let name = "natives-osx";

                    if let Some(native) = classifiers.get(name) {
                        native.url.clone()
                    } else {
                        return None;
                    }
                } else if let Some(artifact) = &artifact {
                    artifact.url.clone()
                } else {
                    return None;
                };

                let sha: String = if let Some(classifiers) = &classifiers {
                    #[cfg(target_os = "windows")]
                    let name = "natives-windows";
                    #[cfg(target_os = "linux")]
                    let name = "natives-linux";
                    #[cfg(target_os = "macos")]
                    let name = "natives-osx";

                    if let Some(native) = classifiers.get(name) {
                        native.sha1.clone()
                    } else {
                        return None;
                    }
                } else if let Some(artifact) = &artifact {
                    artifact.sha1.clone()
                } else {
                    return None;
                };

                let path: String = if let Some(classifiers) = classifiers {
                    #[cfg(target_os = "windows")]
                    let name = "natives-windows";
                    #[cfg(target_os = "linux")]
                    let name = "natives-linux";
                    #[cfg(target_os = "macos")]
                    let name = "natives-osx";

                    if let Some(native) = classifiers.get(name)
                        && let Some(path) = &native.id
                    {
                        path.clone()
                    } else {
                        return None;
                    }
                } else if let Some(artifact) = artifact
                    && let Some(path) = artifact.id
                {
                    path.clone()
                } else {
                    return None;
                };

                Some(FileDownloadArguments { url, sha1: Some(sha), sender: None, path: directory.join(path).to_string_lossy().to_string() })
            })
            .collect();

        if let Err(e) = download_multiple_files_with_client(client, download_items, parallel, sender).await {
            bail!("Download failed: {}", e);
        }

        Ok(())
    }
}

#[cfg(feature = "assets")]
impl VersionManifest {
    pub async fn assets(&self) -> Result<Assets> {
        Assets::from_url(&self.asset_index.url).await
    }
}

#[cfg(test)]
#[cfg(feature = "downloads")]
mod test {
    use crate::manifest_v2::ManifestV2;
    #[cfg(feature = "log")]
    use crate::setup_logging;
    use futures_util::{StreamExt, stream};

    #[tokio::test]
    async fn download_libraries() {
        use crate::version_manifest::LibraryItemDownloader;
        #[cfg(feature = "log")]
        setup_logging();
        let manifest = ManifestV2::fetch().await.unwrap();
        let client = reqwest::Client::new();
        let client = std::sync::Arc::new(client);
        let results: Vec<anyhow::Result<()>> = stream::iter(manifest.versions)
            .map(|version| {
                let client = std::sync::Arc::clone(&client);
                async move {
                    let manifest = version.manifest().await?;
                    info!("Downloading libraries for minecraft {}", version.id);
                    manifest
                        .libraries
                        .download_with_client(&client, format!("target/tests/download_libraries/{}", version.id), 150, None)
                        .await
                        .unwrap_or_else(|e| panic!("Failed to download libraries for minecraft version {} - {}", version.id, e));
                    Ok(())
                }
            })
            .buffer_unordered(2)
            .collect()
            .await;

        for result in results {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn download_server() {
        use crate::manifest_v2::ManifestV2;
        use crate::version_manifest::VersionManifest;
        #[cfg(feature = "log")]
        setup_logging();

        let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");
        let release_id = &manifest.latest.release;
        let version: anyhow::Result<Option<VersionManifest>> = manifest.version(release_id).await;
        if let Ok(Some(version)) = version {
            let output = format!("target/test/server-{}.jar", release_id);
            version.download_server(output, true, None).await.unwrap();
        } else {
            panic!("Failed to fetch version.");
        }
    }

    #[tokio::test]
    async fn download_client() {
        use crate::manifest_v2::ManifestV2;
        use crate::version_manifest::VersionManifest;
        #[cfg(feature = "log")]
        setup_logging();

        let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");
        let release_id = &manifest.latest.release;
        let version: anyhow::Result<Option<VersionManifest>> = manifest.version(release_id).await;
        if let Ok(Some(version)) = version {
            let output = format!("target/test/client-{}.jar", release_id);
            version.download_client(output, true, None).await.unwrap();
        } else {
            panic!("Failed to fetch version.");
        }
    }
}
