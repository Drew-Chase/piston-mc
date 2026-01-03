use piston_mc::{manifest_v2::ManifestV2, version_manifest::VersionManifest};
use std::path::Path;
use simple_download_utility::DownloadProgress;

#[tokio::main]
async fn main() {
    // Fetch the Minecraft version manifest from Mojang's Piston API
    let manifest = ManifestV2::fetch().await.expect("Failed to fetch assets.");

    // Attempt to retrieve the specific version manifest for Minecraft 1.12.2
    let version: anyhow::Result<Option<VersionManifest>> = manifest.version("1.12.2").await;

    // Proceed only if the version was found successfully
    if let Ok(Some(version)) = version {
        // Define the output path for the downloaded server JAR file
        let output = Path::new("target/examples/server-1.12.2.jar");
        // If the downloader should attempt to validate the file using the file hash
        let validate = true;

        // Create a channel for receiving download progress updates
        // Buffer size of 10 allows some backpressure handling
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<DownloadProgress>(10);

        // Start the server download task with SHA1 validation enabled
        let task = version.download_server(output, validate, Some(sender));

        // Spawn a separate task to handle progress updates asynchronously
        tokio::spawn(async move {
            // Continuously receive and display progress until the channel closes
            while let Some(progress) = receiver.recv().await {
                // Calculate download completion percentage
                let percent = (progress.bytes_downloaded as f32 / progress.bytes_to_download as f32) * 100.0;
                // Convert download speed from bytes/sec to megabytes/sec
                let mb_per_sec = progress.bytes_per_second as f32 / 1024.0 / 1024.0;

                println!(
                    "Download progress: {:.2}% ({}/{} bytes) {:.2} MB/s",
                    percent, progress.bytes_downloaded, progress.bytes_to_download, mb_per_sec
                );
            }
        });

        // Await the download task completion
        task.await.unwrap();
    }
}
