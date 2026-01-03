use piston_mc::{manifest_v2::ManifestV2, version_manifest::VersionManifest};

#[tokio::main]
async fn main() {
    // Fetch the Minecraft version manifest from Mojang's Piston API
    let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");

    // Attempt to retrieve the specific version manifest for Minecraft 1.12.2
    let version: anyhow::Result<Option<VersionManifest>> = manifest.version("1.12.2").await;

    // Proceed only if the version was found successfully
    if let Ok(Some(version)) = version {
        // Display basic version information
        println!("Version: {}", version.id);
        println!("Release Type: {}", version.release_type);
        println!("Release Time: {}", version.release_time);
    }
}
