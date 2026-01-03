use piston_mc::java::JavaManifest;
use std::path::Path;
use simple_download_utility::MultiDownloadProgress;

#[tokio::main]
async fn main() {
    // Record the start time to measure total execution duration
    let start_time = std::time::Instant::now();

    // Fetch the Java runtime manifest from Mojang's Piston API
    let manifest = JavaManifest::fetch().await.expect("Failed to fetch Java manifest.");

    // Get the gamma runtime (Java 17) for Windows x64
    // You can change this to match your platform:
    // - manifest.linux.gamma for Linux x64
    // - manifest.macos.gamma for macOS x64
    // - manifest.macos_arm64.gamma for macOS ARM64 (Apple Silicon)
    let runtime = manifest.windows_x64.gamma.first().expect("No gamma runtime available.");

    // Display the runtime version being installed
    println!("Installing Java Runtime: {}", runtime);

    // Define the installation directory
    let install_dir = Path::new("target/examples/java-17");

    // Configure the maximum number of concurrent download tasks
    let number_of_parallel_downloads = 100;

    // Create a channel for receiving download progress updates
    // Buffer size of 16 allows some backpressure handling between producer and consumer
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<MultiDownloadProgress>(16);

    // Start the Java runtime installation task
    let task = runtime.install(install_dir, number_of_parallel_downloads, Some(sender));

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
            println!("Install progress: {:.2}% ({}/{} files) {:.2} MB/s", percent, files_downloaded, file_count, mb_per_sec);
        }
    });

    // Await the installation task completion and handle any errors
    task.await.expect("Failed to install Java runtime.");

    // Calculate the total elapsed time since the operation started
    let elapsed = start_time.elapsed();

    // Display success message and installation location
    println!("\nJava runtime installed successfully to: {}", install_dir.display());

    // Display the total execution time for benchmarking purposes
    println!("Time elapsed: {:?}", elapsed);
}
