//! Bedrock Edition patch notes module.
//!
//! Provides access to Minecraft: Bedrock Edition release notes.
//!
//! # Example
//! ```no_run
//! use piston_mc::patch_notes::bedrock::BedrockPatchNotes;
//!
//! #[tokio::main]
//! async fn main() {
//!     let patch_notes = BedrockPatchNotes::fetch().await.unwrap();
//!     for entry in patch_notes.entries.iter().take(5) {
//!         println!("{} - {}", entry.version, entry.title);
//!     }
//! }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PatchImage;

const PISTON_URL: &str = "https://launchercontent.mojang.com/bedrockPatchNotes.json";

/// Container for Bedrock Edition patch notes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BedrockPatchNotes {
    /// Schema version of the patch notes format.
    pub version: u8,
    /// List of patch note entries.
    pub entries: Vec<BedrockPatchEntry>,
}

/// A single Bedrock Edition patch note entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BedrockPatchEntry {
    /// Unique identifier for this patch note.
    pub id: String,
    /// Title of the release.
    pub title: String,
    /// Version string.
    pub version: String,
    /// Type of patch note (e.g., "retail", "beta").
    #[serde(rename = "patchNoteType", default)]
    pub patch_note_type: Option<String>,
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

impl BedrockPatchNotes {
    /// Fetches Bedrock Edition patch notes from the Mojang API.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if deserialization fails.
    pub async fn fetch() -> Result<BedrockPatchNotes> {
        let response = reqwest::get(PISTON_URL).await?;
        Ok(response.json().await?)
    }

    /// Returns patch notes filtered by type (e.g., "retail", "beta").
    pub fn by_type(&self, patch_note_type: &str) -> Vec<&BedrockPatchEntry> {
        self.entries
            .iter()
            .filter(|e| e.patch_note_type.as_deref() == Some(patch_note_type))
            .collect()
    }

    /// Returns only retail (stable) patch notes.
    pub fn retail(&self) -> Vec<&BedrockPatchEntry> {
        self.by_type("retail")
    }

    /// Returns only beta patch notes.
    pub fn beta(&self) -> Vec<&BedrockPatchEntry> {
        self.by_type("beta")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch_bedrock_patch_notes() {
        let patch_notes = BedrockPatchNotes::fetch().await.unwrap();
        assert!(!patch_notes.entries.is_empty());

        let first = &patch_notes.entries[0];
        assert!(!first.id.is_empty());
        assert!(!first.title.is_empty());
        assert!(!first.version.is_empty());
    }
}
