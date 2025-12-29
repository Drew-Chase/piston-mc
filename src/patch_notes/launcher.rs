//! Minecraft Launcher patch notes module.
//!
//! Provides access to Minecraft Launcher release notes.
//!
//! # Example
//! ```no_run
//! use piston_mc::patch_notes::launcher::LauncherPatchNotes;
//!
//! #[tokio::main]
//! async fn main() {
//!     let patch_notes = LauncherPatchNotes::fetch().await.unwrap();
//!     for entry in patch_notes.entries.iter().take(5) {
//!         println!("Windows: {} - {}", entry.versions.windows, entry.date);
//!     }
//! }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PatchImage;

const PISTON_URL: &str = "https://launchercontent.mojang.com/launcherPatchNotes_v2.json";

/// Container for Minecraft Launcher patch notes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherPatchNotes {
    /// Schema version of the patch notes format.
    pub version: u8,
    /// List of patch note entries.
    pub entries: Vec<LauncherPatchEntry>,
}

/// A single Minecraft Launcher patch note entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherPatchEntry {
    /// Unique identifier for this patch note.
    pub id: String,
    /// Release date in YYYY-MM-DD format.
    pub date: String,
    /// HTML-formatted body content of the patch notes.
    pub body: String,
    /// Platform-specific version numbers.
    pub versions: LauncherVersions,
    /// Optional highlight/announcement for this release.
    pub highlight: Option<LauncherHighlight>,
}

/// Platform-specific version numbers for the launcher.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct LauncherVersions {
    /// Windows version string.
    #[serde(default)]
    pub windows: Option<String>,
    /// macOS version string.
    #[serde(default)]
    pub osx: Option<String>,
    /// Linux version string.
    #[serde(default)]
    pub linux: Option<String>,
    /// Xbox/GameCore version string (not present in older entries).
    #[serde(default)]
    pub gamecore: Option<String>,
}

/// Highlighted announcement for a launcher release.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherHighlight {
    /// Date until which the highlight is active (YYYY-MM-DD).
    pub until: String,
    /// Title of the highlighted feature.
    pub title: String,
    /// Description of the highlighted feature.
    pub description: String,
    /// Image associated with the highlight.
    pub image: PatchImage,
}

impl LauncherPatchNotes {
    /// Fetches Minecraft Launcher patch notes from the Mojang API.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or if deserialization fails.
    pub async fn fetch() -> Result<LauncherPatchNotes> {
        let response = reqwest::get(PISTON_URL).await?;
        Ok(response.json().await?)
    }

    /// Returns the most recent patch note entry.
    pub fn latest(&self) -> Option<&LauncherPatchEntry> {
        self.entries.first()
    }

    /// Returns entries that have an active highlight.
    pub fn with_highlights(&self) -> Vec<&LauncherPatchEntry> {
        self.entries.iter().filter(|e| e.highlight.is_some()).collect()
    }
}

impl LauncherPatchEntry {
    /// Returns the version for a specific platform.
    pub fn version_for_platform(&self, platform: &str) -> Option<&str> {
        match platform.to_lowercase().as_str() {
            "windows" | "win" => self.versions.windows.as_deref(),
            "osx" | "macos" | "mac" => self.versions.osx.as_deref(),
            "linux" => self.versions.linux.as_deref(),
            "gamecore" | "xbox" => self.versions.gamecore.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn fetch_launcher_patch_notes() {
        let patch_notes = LauncherPatchNotes::fetch().await.unwrap();
        assert!(!patch_notes.entries.is_empty());

        let first = &patch_notes.entries[0];
        assert!(!first.id.is_empty());
        assert!(!first.date.is_empty());
        // At least one platform version should be present
        assert!(
            first.versions.windows.is_some()
                || first.versions.osx.is_some()
                || first.versions.linux.is_some()
        );
    }
}
