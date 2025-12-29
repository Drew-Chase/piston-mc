use piston_mc::patch_notes::launcher::LauncherPatchNotes;

#[tokio::main]
async fn main() {
    // Fetch Minecraft Launcher patch notes from Mojang's API
    let patch_notes = LauncherPatchNotes::fetch()
        .await
        .expect("Failed to fetch Launcher patch notes");

    println!("Total Launcher patch notes: {}", patch_notes.entries.len());

    // Display the latest entry with platform versions
    if let Some(latest) = patch_notes.latest() {
        println!("\n=== Latest Launcher Update ===");
        println!("Date: {}", latest.date);
        println!("Platform Versions:");
        if let Some(windows) = &latest.versions.windows {
            println!("  Windows:  {}", windows);
        }
        if let Some(osx) = &latest.versions.osx {
            println!("  macOS:    {}", osx);
        }
        if let Some(linux) = &latest.versions.linux {
            println!("  Linux:    {}", linux);
        }
        if let Some(gamecore) = &latest.versions.gamecore {
            println!("  GameCore: {}", gamecore);
        }

        // Check for highlight
        if let Some(highlight) = &latest.highlight {
            println!("\nHighlight:");
            println!("  Title: {}", highlight.title);
            println!("  Description: {}", highlight.description);
            println!("  Until: {}", highlight.until);
        }
    }

    // Show entries with highlights
    let highlighted = patch_notes.with_highlights();
    if !highlighted.is_empty() {
        println!("\n=== Entries with Highlights ===");
        for entry in highlighted.iter().take(3) {
            if let Some(h) = &entry.highlight {
                println!("{} - {}", entry.date, h.title);
            }
        }
    }

    // List recent updates
    println!("\n=== Recent Launcher Updates ===");
    for entry in patch_notes.entries.iter().take(10) {
        let version = entry.versions.windows.as_deref().unwrap_or("N/A");
        println!("{} - Windows v{}", entry.date, version);
    }
}
