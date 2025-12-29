//! Patch notes modules for fetching release notes from Mojang's launcher content API.
//!
//! This module provides access to patch notes for:
//! - Java Edition (`java`)
//! - Bedrock Edition (`bedrock`)
//! - Minecraft Dungeons (`dungeons`)
//! - Minecraft Launcher (`launcher`)
//!
//! Each submodule is feature-gated and can be enabled independently.

use serde::{Deserialize, Serialize};

#[cfg(feature = "java-patch-notes")]
pub mod java;

#[cfg(feature = "bedrock-patch-notes")]
pub mod bedrock;

#[cfg(feature = "dungeons-patch-notes")]
pub mod dungeons;

#[cfg(feature = "launcher-patch-notes")]
pub mod launcher;

/// Image data associated with a patch note entry.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchImage {
    /// The URL path to the image (relative to launchercontent.mojang.com).
    pub url: String,
    /// The title/alt text of the image.
    pub title: String,
}
