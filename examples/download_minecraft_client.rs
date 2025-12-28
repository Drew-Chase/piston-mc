use std::path::Path;
use piston_mc::{manifest_v2::ManifestV2, version_manifest::VersionManifest};

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

		// Download the client with SHA1 validation enabled and no progress reporting
		version.download_client(output, true, None).await.unwrap();
	}
}