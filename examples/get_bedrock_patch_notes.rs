use piston_mc::patch_notes::bedrock::BedrockPatchNotes;

#[tokio::main]
async fn main() {
    // Fetch Bedrock Edition patch notes from Mojang's API
    let patch_notes = BedrockPatchNotes::fetch()
        .await
        .expect("Failed to fetch Bedrock patch notes");

    println!("Total Bedrock Edition patch notes: {}", patch_notes.entries.len());

    // Display retail (stable) releases only
    println!("\n=== Recent Retail Releases ===");
    for entry in patch_notes.retail().iter().take(5) {
        println!("{} - {} ({})", entry.version, entry.title, entry.date);
    }

    // Display beta releases
    println!("\n=== Recent Beta Releases ===");
    for entry in patch_notes.beta().iter().take(5) {
        println!("{} - {} ({})", entry.version, entry.title, entry.date);
    }

    // Show details of the first entry
    if let Some(first) = patch_notes.entries.first() {
        println!("\n=== Latest Entry Details ===");
        println!("ID: {}", first.id);
        println!("Title: {}", first.title);
        println!("Version: {}", first.version);
        println!("Type: {}", first.patch_note_type.as_deref().unwrap_or("unknown"));
        println!("Date: {}", first.date);
        println!("Image: {}", first.image.url);
    }
}
