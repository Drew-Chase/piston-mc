//! Java Edition patch notes module.
//!
//! Provides access to Minecraft: Java Edition release notes.
//!
//! # Example
//! ```no_run
//! use piston_mc::patch_notes::java::JavaPatchNotes;
//!
//! #[tokio::main]
//! async fn main() {
//!     let patch_notes = JavaPatchNotes::fetch().await.unwrap();
//!     for entry in patch_notes.entries.iter().take(5) {
//!         println!("{} - {}", entry.version, entry.title);
//!     }
//! }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PatchImage;

const PISTON_URL: &str = "https://launchercontent.mojang.com/javaPatchNotes.json";

/// Container for Java Edition patch notes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaPatchNotes {
    /// Schema version of the patch notes format.
    pub version: u8,
    /// List of patch note entries.
    pub entries: Vec<JavaPatchEntry>,
}

/// A single Java Edition patch note entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaPatchEntry {
    /// Unique identifier for this patch note.
    pub id: String,
    /// Title of the release (e.g., "1.20.4").
    pub title: String,
    /// Version string.
    pub version: String,
    /// Type of patch (e.g., "release", "snapshot").
    #[serde(rename = "type", default)]
    pub patch_type: Option<String>,
    /// Release date in YYYY-MM-DD format.
    #[serde(default)]
    pub date: Option<String>,
    /// HTML-formatted body content of the patch notes.
    pub body: String,
    /// Path to the full content JSON.
    #[serde(rename = "contentPath")]
    pub content_path: String,
    /// Image associated with this patch note.
    pub image: PatchImage,
}

impl JavaPatchNotes {
    /// Fetches Java Edition patch notes from the Mojang API.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if deserialization fails.
    pub async fn fetch() -> Result<JavaPatchNotes> {
        let response = reqwest::get(PISTON_URL).await?;
        Ok(response.json().await?)
    }

    /// Returns patch notes filtered by type (e.g., "release", "snapshot").
    pub fn by_type(&self, patch_type: &str) -> Vec<&JavaPatchEntry> {
        self.entries.iter().filter(|e| e.patch_type.as_deref() == Some(patch_type)).collect()
    }

    /// Returns only release patch notes.
    pub fn releases(&self) -> Vec<&JavaPatchEntry> {
        self.by_type("release")
    }

    /// Returns only snapshot patch notes.
    pub fn snapshots(&self) -> Vec<&JavaPatchEntry> {
        self.by_type("snapshot")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch_java_patch_notes() {
        let patch_notes = JavaPatchNotes::fetch().await.unwrap();
        assert!(!patch_notes.entries.is_empty());

        let first = &patch_notes.entries[0];
        assert!(!first.id.is_empty());
        assert!(!first.title.is_empty());
        assert!(!first.version.is_empty());
    }
}
