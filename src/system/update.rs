use serde::Deserialize;
use std::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_CHECK_URL: &str = "https://github.com/Yasar195/rust_cli_todo/releases/latest/download/version.json";

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_date: String,
    pub downloads: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
}

/// Check if a new version is available
pub fn check_for_updates() -> Result<UpdateInfo, Box<dyn Error>> {
    let response = reqwest::blocking::get(UPDATE_CHECK_URL)?;
    let version_info: VersionInfo = response.json()?;
    
    let current_version = semver::Version::parse(VERSION.trim_start_matches('v'))?;
    let latest_version = semver::Version::parse(version_info.version.trim_start_matches('v'))?;
    
    let update_available = latest_version > current_version;
    let download_url = if update_available {
        Some(get_download_url_from_info(&version_info))
    } else {
        None
    };
    
    Ok(UpdateInfo {
        current_version: format!("v{}", current_version),
        latest_version: version_info.version,
        update_available,
        download_url,
    })
}

/// Get the appropriate download URL for the current platform
fn get_download_url_from_info(info: &VersionInfo) -> String {
    let platform = get_platform_identifier();
    info.downloads
        .get(platform)
        .cloned()
        .unwrap_or_else(|| panic!("No download available for platform: {}", platform))
}

/// Get platform identifier matching your release naming
fn get_platform_identifier() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64", not(target_env = "musl")))]
    return "linux-amd64";
    
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
    return "linux-musl-amd64";
    
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "macos-amd64";
    
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "macos-arm64";
    
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-amd64";
    
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    return "unknown";
}

/// Perform self-update using the self_update crate (easiest method)
pub fn perform_update() -> Result<(), Box<dyn Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("Yasar195")
        .repo_name("rust_cli_todo")
        .bin_name("todo")
        .current_version(VERSION)
        .build()?
        .update()?;

    println!("Update status: `{}`", status.version());
    Ok(())
}
