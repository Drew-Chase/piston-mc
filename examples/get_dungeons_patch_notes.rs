use piston_mc::patch_notes::dungeons::DungeonsPatchNotes;

#[tokio::main]
async fn main() {
    // Fetch Minecraft Dungeons patch notes from Mojang's API
    let patch_notes = DungeonsPatchNotes::fetch().await.expect("Failed to fetch Dungeons patch notes");

    println!("Total Minecraft Dungeons patch notes: {}", patch_notes.entries.len());

    // Display the latest entry
    if let Some(latest) = patch_notes.latest() {
        println!("\n=== Latest Dungeons Update ===");
        println!("Title: {}", latest.title);
        println!("Version: {}", latest.version);
        println!("Date: {}", latest.date);
        println!("Image: {}", latest.image.url);
    }

    // List all updates
    println!("\n=== All Dungeons Updates ===");
    for entry in &patch_notes.entries {
        println!("{} - {} ({})", entry.version, entry.title, entry.date);
    }
}
