use piston_mc::manifest_v2::{ManifestV2, Version};
#[tokio::main]
async fn main() {
	// Fetch the Minecraft version manifest from Mojang's Piston API
	let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");

	// Display the most recent stable release version identifier
	println!("Latest Release: {}", manifest.latest.release);
	// Display the most recent development snapshot version identifier
	println!("Latest Snapshot: {}", manifest.latest.snapshot);

	// Filter and collect only stable release versions (excludes snapshots, beta, alpha)
	let releases: Vec<Version> = manifest.releases();
	// Iterate through and display each stable release version
	for release in releases {
		println!("Release: {}", release.id);
	}

	// Visual separator for the complete version listing
	println!("\n\n====== All Versions: ========");

	// Iterate through every version in the manifest
	// This includes all release types: stable releases, snapshots, old_beta, and old_alpha
	for version in manifest.versions {
		// Display the version type (e.g., "release", "snapshot") alongside its identifier
		println!("{}: {}", version.release_type, version.id);
	}

}