#![doc = include_str!("../.wiki/Home.md")]

// Logging setup: use real log crate when feature enabled, otherwise use no-op stubs
#[cfg(feature = "log")]
#[macro_use]
extern crate log;

#[cfg(not(feature = "log"))]
#[macro_use]
mod log_stub;

#[cfg(feature = "assets")]
pub mod assets;
//#[cfg(feature = "downloads")]
//pub mod download_util;
#[cfg(feature = "java")]
pub mod java;
pub mod manifest_v2;
#[cfg(feature = "news")]
pub mod news;
#[cfg(any(
    feature = "patch-notes",
    feature = "java-patch-notes",
    feature = "bedrock-patch-notes",
    feature = "dungeons-patch-notes",
    feature = "launcher-patch-notes"
))]
pub mod patch_notes;
pub mod sha_validation;
pub mod version_manifest;

#[cfg(test)]
#[cfg(feature = "log")]
pub(crate) fn setup_logging() {
    _ = pretty_env_logger::env_logger::builder().is_test(true).format_timestamp(None).filter_level(log::LevelFilter::Trace).try_init();
}
