//! Minecraft Dungeons patch notes module.
//!
//! Provides access to Minecraft Dungeons release notes.
//!
//! # Example
//! ```no_run
//! use piston_mc::patch_notes::dungeons::DungeonsPatchNotes;
//!
//! #[tokio::main]
//! async fn main() {
//!     let patch_notes = DungeonsPatchNotes::fetch().await.unwrap();
//!     for entry in patch_notes.entries.iter().take(5) {
//!         println!("{} - {}", entry.version, entry.title);
//!     }
//! }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PatchImage;

const PISTON_URL: &str = "https://launchercontent.mojang.com/dungeonsPatchNotes.json";

/// Container for Minecraft Dungeons patch notes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DungeonsPatchNotes {
    /// Schema version of the patch notes format.
    pub version: u8,
    /// List of patch note entries.
    pub entries: Vec<DungeonsPatchEntry>,
}

/// A single Minecraft Dungeons patch note entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DungeonsPatchEntry {
    /// Unique identifier for this patch note.
    pub id: String,
    /// Title of the release.
    pub title: String,
    /// Version string.
    pub version: String,
    /// Release date in YYYY-MM-DD format.
    pub date: String,
    /// HTML-formatted body content of the patch notes.
    pub body: String,
    /// Path to the full content JSON.
    #[serde(rename = "contentPath")]
    pub content_path: String,
    /// Image associated with this patch note.
    pub image: PatchImage,
}

impl DungeonsPatchNotes {
    /// Fetches Minecraft Dungeons patch notes from the Mojang API.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if deserialization fails.
    pub async fn fetch() -> Result<DungeonsPatchNotes> {
        let response = reqwest::get(PISTON_URL).await?;
        Ok(response.json().await?)
    }

    /// Returns the most recent patch note entry.
    pub fn latest(&self) -> Option<&DungeonsPatchEntry> {
        self.entries.first()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch_dungeons_patch_notes() {
        let patch_notes = DungeonsPatchNotes::fetch().await.unwrap();
        assert!(!patch_notes.entries.is_empty());

        let first = &patch_notes.entries[0];
        assert!(!first.id.is_empty());
        assert!(!first.title.is_empty());
        assert!(!first.version.is_empty());
    }
}
