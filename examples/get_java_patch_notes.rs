use piston_mc::patch_notes::java::JavaPatchNotes;

#[tokio::main]
async fn main() {
    // Fetch Java Edition patch notes from Mojang's API
    let patch_notes = JavaPatchNotes::fetch().await.expect("Failed to fetch Java patch notes");

    println!("Total Java Edition patch notes: {}", patch_notes.entries.len());

    // Display release versions only
    println!("\n=== Recent Releases ===");
    for entry in patch_notes.releases().iter().take(5) {
        let date = entry.date.as_deref().unwrap_or("unknown");
        println!("{} - {} ({})", entry.version, entry.title, date);
    }

    // Display snapshots
    println!("\n=== Recent Snapshots ===");
    for entry in patch_notes.snapshots().iter().take(5) {
        let date = entry.date.as_deref().unwrap_or("unknown");
        println!("{} - {} ({})", entry.version, entry.title, date);
    }

    // Show details of the first entry
    if let Some(first) = patch_notes.entries.first() {
        println!("\n=== Latest Entry Details ===");
        println!("ID: {}", first.id);
        println!("Title: {}", first.title);
        println!("Version: {}", first.version);
        println!("Type: {}", first.patch_type.as_deref().unwrap_or("unknown"));
        println!("Date: {}", first.date.as_deref().unwrap_or("unknown"));
        println!("Image: {}", first.image.url);
        println!("Content Path: {}", first.content_path);
    }
}
