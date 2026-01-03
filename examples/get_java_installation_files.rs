use piston_mc::java::JavaManifest;

#[tokio::main]
async fn main() {
    // Fetch the Java runtime manifest from Mojang's Piston API
    let manifest = JavaManifest::fetch().await.expect("Failed to fetch Java manifest.");

    // Get the gamma runtime (Java 17) for Windows x64
    // You can change this to match your platform:
    // - manifest.linux.gamma for Linux x64
    // - manifest.macos.gamma for macOS x64
    // - manifest.macos_arm64.gamma for macOS ARM64 (Apple Silicon)
    let runtime = manifest.windows_x64.gamma.first().expect("No gamma runtime available.");

    // Display the runtime version information
    println!("Java Runtime: {}", runtime);

    // Fetch the list of all files required for this Java runtime
    let files = runtime.get_installation_files().await.expect("Failed to get installation files.");

    // Display the total number of files
    println!("Total files: {}", files.len());

    // Count files by type
    let file_count = files.iter().filter(|f| f.downloads.is_some()).count();
    let directory_count = files.iter().filter(|f| f.downloads.is_none()).count();

    println!("Files to download: {}", file_count);
    println!("Directories to create: {}", directory_count);

    // Calculate total download size
    let total_size: usize = files.iter().filter_map(|f| f.downloads.as_ref()).map(|d| d.raw.size).sum();

    println!("Total download size: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);

    // Display first 20 files as a sample
    println!("\nFirst 20 files:");
    for file in files.iter().take(20) {
        if let Some(downloads) = &file.downloads {
            println!("  {} ({} bytes)", file.name, downloads.raw.size);
        } else {
            println!("  {} (directory)", file.name);
        }
    }
}
