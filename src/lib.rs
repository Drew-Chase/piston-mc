pub mod assets;
pub mod download_util;
mod java;
pub mod manifest_v2;
pub mod sha_validation;
pub mod version_manifest;

#[cfg(test)]
pub(crate) fn setup_logging() {
    _ = pretty_env_logger::env_logger::builder().is_test(true).format_timestamp(None).filter_level(log::LevelFilter::Trace).try_init();
}
