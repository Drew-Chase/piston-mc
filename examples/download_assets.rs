use piston_mc::download_util::MultiDownloadProgress;
use piston_mc::manifest_v2::ManifestV2;
use piston_mc::version_manifest::VersionManifest;

#[tokio::main]
async fn main() {
	// Record the start time to measure total execution duration
	let start_time = std::time::Instant::now();

	// Fetch the Minecraft version manifest from Mojang's Piston API
	let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");

	// Attempt to retrieve the specific version manifest for Minecraft 1.12.2
	let version: anyhow::Result<Option<VersionManifest>> = manifest.version("1.12.2").await;

	// Proceed only if the version was found successfully
	if let Ok(Some(version)) = version {
		// Fetch the asset index for this Minecraft version
		let mut assets = version.assets().await.expect("Failed to fetch assets.");

		// Configure the maximum number of concurrent download tasks
		let number_of_parallel_downloads = 100;

		// Create a channel for receiving download progress updates
		// Buffer size of 16 allows some backpressure handling between producer and consumer
		let (sender, mut receiver) = tokio::sync::mpsc::channel::<MultiDownloadProgress>(16);

		// Start the asset download task, saving files to the specified directory
		let task = assets.download("target/examples/assets", number_of_parallel_downloads, Some(sender));

		// Spawn a separate async task to handle progress updates without blocking the download
		tokio::spawn(async move {
			// Continuously receive and display progress until the channel closes
			while let Some(progress) = receiver.recv().await {
				// Convert download speed from bytes/sec to megabytes/sec for readability
				let mb_per_sec = progress.bytes_per_second as f32 / 1024.0 / 1024.0;

				// Get the total number of files to download
				let file_count = progress.files_total;

				// Get the number of files that have been downloaded so far
				let files_downloaded = progress.files_downloaded;

				// Calculate download completion percentage based on file count
				let percent = (files_downloaded as f32 / file_count as f32) * 100.0;

				// Display formatted progress: percentage, file counts, and transfer speed
				println!("Download progress: {:.2}% ({}/{}) {:.2} MB/s", percent, files_downloaded, file_count, mb_per_sec);
			}
		});

		// Await the download task completion and handle any errors
		task.await.expect("Failed to download assets.");
	}

	// Calculate the total elapsed time since the operation started
	let elapsed = start_time.elapsed();

	// Display the total execution time for benchmarking purposes
	println!("Time elapsed: {:?}", elapsed);
}