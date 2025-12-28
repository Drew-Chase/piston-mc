use piston_mc::{manifest_v2::ManifestV2, version_manifest::VersionManifest};
use std::path::Path;

#[tokio::main]
async fn main() {
    // Fetch the Minecraft version manifest from Mojang's Piston API
    let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");

    // Attempt to retrieve the specific version manifest for Minecraft 1.12.2
    let version: anyhow::Result<Option<VersionManifest>> = manifest.version("1.12.2").await;

    // Proceed only if the version was found successfully
    if let Ok(Some(version)) = version {
        // Define the output path for the downloaded client JAR file
        let output = Path::new("target/examples/client-1.12.2.jar");

        // If the downloader should attempt to validate the file using the file hash
        let validate = true;

        // Download the client with SHA1 validation enabled and no progress reporting
        version.download_client(output, validate, None).await.unwrap();
    }
}
