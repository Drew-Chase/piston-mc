use crate::version_manifest::VersionManifest;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

const PISTON_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManifestV2 {
    pub latest: LatestManifest,
    pub versions: Vec<Version>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LatestManifest {
    pub release: String,
    pub snapshot: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub release_type: ReleaseType,
    pub url: String,
    pub time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: chrono::DateTime<chrono::Utc>,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ReleaseType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "old_alpha")]
    OldAlpha,
}

impl ManifestV2 {
    pub async fn fetch() -> Result<ManifestV2> {
        debug!("Fetching versions manifest");
        let manifest = reqwest::get(PISTON_URL).await?.json::<Self>().await?;
        debug!("Found {} versions in manifest", manifest.versions.len());
        Ok(manifest)
    }

    pub async fn version(&self, id: impl AsRef<str>) -> Result<Option<VersionManifest>> {
        let id = id.as_ref();
        match self.versions.iter().find(|version| version.id == id) {
            Some(version) => Ok(Some(version.manifest().await?)),
            None => Ok(None),
        }
    }
}

impl Version {
    pub async fn manifest(&self) -> Result<VersionManifest> {
        debug!("Getting manifest version: {}", self.id);
        VersionManifest::from_url(&self.url).await
    }
}

mod test {

    #[tokio::test]
    async fn fetch_manifest() {
        use crate::manifest_v2::ManifestV2;
        setup_logging();
        let manifest = ManifestV2::fetch().await.unwrap();
        assert!(!manifest.versions.is_empty());
    }

    #[tokio::test]
    async fn version_manifest() {
        use crate::manifest_v2::ManifestV2;
        setup_logging();
        let manifest = ManifestV2::fetch().await.unwrap();
        for version in manifest.versions {
            let version = version.manifest().await;
            assert!(version.is_ok());
        }
    }

    fn setup_logging() {
        pretty_env_logger::env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .filter_level(log::LevelFilter::Trace)
            .init();
    }
}
