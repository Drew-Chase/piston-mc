#![doc = include_str!("../.wiki/Java.md")]

use crate::download_util::{download_multiple_files, FileDownloadArguments, MultiDownloadProgress};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

const PISTON_URL: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaManifest {
    pub linux: Runtimes,
    #[serde(rename = "linux-i386")]
    pub linux_i386: Runtimes,
    #[serde(rename = "mac-os")]
    pub macos: Runtimes,
    #[serde(rename = "mac-os-arm64")]
    pub macos_arm64: Runtimes,
    #[serde(rename = "windows-arm64")]
    pub windows_arm64: Runtimes,
    #[serde(rename = "windows-x64")]
    pub windows_x64: Runtimes,
    #[serde(rename = "windows-x86")]
    pub windows_x86: Runtimes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Runtimes {
    #[serde(rename = "java-runtime-alpha")]
    pub alpha: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-beta")]
    pub beta: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-gamma")]
    pub gamma: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-delta")]
    pub delta: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-gamma-snapshot")]
    pub gamma_snapshot: Vec<JavaRuntime>,
    #[serde(rename = "java-runtime-epsilon")]
    pub epsilon: Vec<JavaRuntime>,
    #[serde(rename = "jre-legacy")]
    pub legacy: Vec<JavaRuntime>,
    #[serde(rename = "minecraft-java-exe")]
    pub minecraft_java_exe: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaRuntime {
    version: Version,
    manifest: Manifest,
    availability: Availability,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub sha1: String,
    pub size: usize,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub released: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Availability {
    pub group: u32,
    pub progress: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaInstallationFile {
    #[serde(skip)]
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: Option<FileType>,
    pub executable: Option<bool>,
    pub downloads: Option<Downloads>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
    pub lzma: Option<DownloadItem>,
    pub raw: DownloadItem,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadItem {
    pub sha1: String,
    pub size: usize,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "link")]
    Link,
}

impl JavaManifest {
    pub async fn fetch() -> Result<Self> {
        let response = reqwest::get(PISTON_URL).await?;
        let text = response.text().await?;
        let json_result = serde_json::from_str::<Self>(&text);
        #[cfg(feature = "log")]
        if let Err(ref e) = json_result {
            let line = e.line();
            let column = e.column();
            error!("Failed to deserialize VersionManifest from {}: {}", PISTON_URL, e);
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
}

impl JavaRuntime {
    pub async fn get_installation_files(&self) -> Result<Vec<JavaInstallationFile>> {
        let url = self.manifest.url.clone();
        let response = reqwest::get(&url).await?;
        let files: serde_json::Value = response.json().await?;
        let files = files.get("files").ok_or_else(|| anyhow!("Missing 'files' field in response"))?;
        let json_result = serde_json::from_value::<HashMap<String, JavaInstallationFile>>(files.clone());
        #[cfg(feature = "log")]
        if let Err(ref e) = json_result {
            let line = e.line();
            let column = e.column();
            error!("Failed to deserialize VersionManifest from {}: {}", url, e);
            error!("Error at line {}, column {}", line, column);
        }
        let map = json_result?;
        Ok(map
            .into_iter()
            .map(|(name, mut file)| {
                file.name = name;
                file
            })
            .collect())
    }

    pub async fn install(&self, directory: impl AsRef<Path>, parallel: u16, sender: Option<tokio::sync::mpsc::Sender<MultiDownloadProgress>>) -> Result<()> {
        let directory = directory.as_ref();
        let installation_files = self.get_installation_files().await?;

        let args: Vec<FileDownloadArguments> = installation_files
            .iter()
            .filter_map(|item| {
                item.downloads.as_ref().map(|download| FileDownloadArguments {
                    url: download.raw.url.clone(),
                    path: directory.join(&item.name).to_string_lossy().to_string(),
                    sender: None,
                    sha1: Some(download.raw.sha1.clone()),
                })
            })
            .collect();

        #[cfg(feature = "log")]
        info!("Downloading files: {:?}", installation_files);

        download_multiple_files(args, parallel, sender).await?;

        Ok(())
    }
}

impl Display for JavaRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version.name)
    }
}

#[cfg(test)]
mod test {
    use crate::java::JavaManifest;
    #[cfg(feature = "log")]
    use crate::setup_logging;
    use futures_util::{StreamExt, stream};

    #[tokio::test]
    async fn fetch() {
        #[cfg(feature = "log")]
        setup_logging();
        let manifest = JavaManifest::fetch().await.unwrap();
        info!("{:?}", manifest);
    }
    #[tokio::test]
    async fn get_installation_files() {
        #[cfg(feature = "log")]
        setup_logging();
        let manifest = JavaManifest::fetch().await.unwrap();
        let runtimes = [
            &manifest.linux.alpha,
            &manifest.linux.beta,
            &manifest.linux.gamma,
            &manifest.linux.delta,
            &manifest.linux.gamma_snapshot,
            &manifest.linux.epsilon,
            &manifest.linux.legacy,
            &manifest.linux_i386.alpha,
            &manifest.linux_i386.beta,
            &manifest.linux_i386.gamma,
            &manifest.linux_i386.delta,
            &manifest.linux_i386.gamma_snapshot,
            &manifest.linux_i386.epsilon,
            &manifest.linux_i386.legacy,
            &manifest.macos.alpha,
            &manifest.macos.beta,
            &manifest.macos.gamma,
            &manifest.macos.delta,
            &manifest.macos.gamma_snapshot,
            &manifest.macos.epsilon,
            &manifest.macos.legacy,
            &manifest.macos_arm64.alpha,
            &manifest.macos_arm64.beta,
            &manifest.macos_arm64.gamma,
            &manifest.macos_arm64.delta,
            &manifest.macos_arm64.gamma_snapshot,
            &manifest.macos_arm64.epsilon,
            &manifest.macos_arm64.legacy,
            &manifest.windows_arm64.alpha,
            &manifest.windows_arm64.beta,
            &manifest.windows_arm64.gamma,
            &manifest.windows_arm64.delta,
            &manifest.windows_arm64.gamma_snapshot,
            &manifest.windows_arm64.epsilon,
            &manifest.windows_arm64.legacy,
            &manifest.windows_x64.alpha,
            &manifest.windows_x64.beta,
            &manifest.windows_x64.gamma,
            &manifest.windows_x64.delta,
            &manifest.windows_x64.gamma_snapshot,
            &manifest.windows_x64.epsilon,
            &manifest.windows_x64.legacy,
            &manifest.windows_x86.alpha,
            &manifest.windows_x86.beta,
            &manifest.windows_x86.gamma,
            &manifest.windows_x86.delta,
            &manifest.windows_x86.gamma_snapshot,
            &manifest.windows_x86.epsilon,
            &manifest.windows_x86.legacy,
        ];

        let results: Vec<_> = stream::iter(runtimes)
            .enumerate()
            .map(|(idx, runtime_vec)| {
                let runtime = runtime_vec.first();
                async move {
                    if let Some(runtime) = runtime {
                        let files_result = runtime.get_installation_files().await;
                        #[cfg(feature = "log")]
                        if let Ok(ref files) = files_result {
                            info!("Runtime {}: {} - found {} installation files", idx, runtime, files.len());
                        }
                        files_result
                    } else {
                        #[cfg(feature = "log")]
                        info!("Runtime {}: empty runtime vector", idx);
                        Ok(vec![])
                    }
                }
            })
            .buffer_unordered(10usize)
            .collect()
            .await;

        for result in &results {
            if let Err(e) = result {
                #[cfg(feature = "log")]
                error!("Failed to get installation files: {}", e);
            }
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn install(){
        #[cfg(feature = "log")]
        setup_logging();
        let manifest = JavaManifest::fetch().await.unwrap();
        let directory = "target/test/";
        let runtimes = [
            &manifest.linux.alpha,
            &manifest.linux.beta,
            &manifest.linux.gamma,
            &manifest.linux.delta,
            &manifest.linux.gamma_snapshot,
            &manifest.linux.epsilon,
            &manifest.linux.legacy,
            &manifest.linux_i386.alpha,
            &manifest.linux_i386.beta,
            &manifest.linux_i386.gamma,
            &manifest.linux_i386.delta,
            &manifest.linux_i386.gamma_snapshot,
            &manifest.linux_i386.epsilon,
            &manifest.linux_i386.legacy,
            &manifest.macos.alpha,
            &manifest.macos.beta,
            &manifest.macos.gamma,
            &manifest.macos.delta,
            &manifest.macos.gamma_snapshot,
            &manifest.macos.epsilon,
            &manifest.macos.legacy,
            &manifest.macos_arm64.alpha,
            &manifest.macos_arm64.beta,
            &manifest.macos_arm64.gamma,
            &manifest.macos_arm64.delta,
            &manifest.macos_arm64.gamma_snapshot,
            &manifest.macos_arm64.epsilon,
            &manifest.macos_arm64.legacy,
            &manifest.windows_arm64.alpha,
            &manifest.windows_arm64.beta,
            &manifest.windows_arm64.gamma,
            &manifest.windows_arm64.delta,
            &manifest.windows_arm64.gamma_snapshot,
            &manifest.windows_arm64.epsilon,
            &manifest.windows_arm64.legacy,
            &manifest.windows_x64.alpha,
            &manifest.windows_x64.beta,
            &manifest.windows_x64.gamma,
            &manifest.windows_x64.delta,
            &manifest.windows_x64.gamma_snapshot,
            &manifest.windows_x64.epsilon,
            &manifest.windows_x64.legacy,
            &manifest.windows_x86.alpha,
            &manifest.windows_x86.beta,
            &manifest.windows_x86.gamma,
            &manifest.windows_x86.delta,
            &manifest.windows_x86.gamma_snapshot,
            &manifest.windows_x86.epsilon,
            &manifest.windows_x86.legacy,
        ];

        let results: Vec<_> = stream::iter(runtimes)
            .map(|runtime| async move {
                if let Some(runtime) = runtime.first() {
                    let directory = std::path::Path::new(directory).join(format!("{}-{}", runtime, runtime.manifest.sha1));
                    info!("Installing java {} to {}...", runtime, directory.display());
                    runtime.install(&directory, 20, None).await
                } else {
                    warn!("No runtime specified");
                    Ok(())
                }
            })
            .buffer_unordered(27)
            .collect()
            .await;

        for result in results {
            result.unwrap();
        }

    }

}
