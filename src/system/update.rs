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

/// Alternative: Manual self-update implementation for more control
// #[allow(dead_code)]
// pub fn perform_update_manual(download_url: &str) -> Result<(), Box<dyn Error>> {
//     use std::env;
//     use std::fs;
//     use std::io::Write;
    
//     println!("Downloading update from: {}", download_url);
    
//     let response = reqwest::blocking::get(download_url)?;
//     let bytes = response.bytes()?;
    
//     let current_exe = env::current_exe()?;
//     let temp_path = current_exe.with_extension("new");
    
//     // Extract based on platform
//     #[cfg(not(windows))]
//     {
//         use flate2::read::GzDecoder;
//         use tar::Archive;
//         use std::os::unix::fs::PermissionsExt;
        
//         let tar = GzDecoder::new(&bytes[..]);
//         let mut archive = Archive::new(tar);
        
//         for entry in archive.entries()? {
//             let mut entry = entry?;
//             let path = entry.path()?;
//             if path.file_name().and_then(|n| n.to_str()) == Some("todo") {
//                 let mut file = fs::File::create(&temp_path)?;
//                 std::io::copy(&mut entry, &mut file)?;
                
//                 // Set executable permissions
//                 let mut perms = fs::metadata(&temp_path)?.permissions();
//                 perms.set_mode(0o755);
//                 fs::set_permissions(&temp_path, perms)?;
//                 break;
//             }
//         }
//     }
    
//     #[cfg(windows)]
//     {
//         use zip::ZipArchive;
//         use std::io::Cursor;
        
//         let cursor = Cursor::new(bytes);
//         let mut archive = ZipArchive::new(cursor)?;
        
//         for i in 0..archive.len() {
//             let mut file = archive.by_index(i)?;
//             if file.name().ends_with("todo.exe") {
//                 let mut outfile = fs::File::create(&temp_path)?;
//                 std::io::copy(&mut file, &mut outfile)?;
//                 break;
//             }
//         }
//     }
    
//     // Replace current binary
//     let backup_path = current_exe.with_extension("old");
    
//     // On Windows, we might need to handle file locks differently
//     #[cfg(windows)]
//     {
//         // Create a batch script to replace the file after exit
//         let batch_script = format!(
//             r#"@echo off
// timeout /t 2 /nobreak > NUL
// del "{old}"
// move /y "{new}" "{current}"
// del "{backup}"
// start "" "{current}"
// "#,
//             old = backup_path.display(),
//             new = temp_path.display(),
//             current = current_exe.display(),
//             backup = current_exe.with_extension("bat").display()
//         );
        
//         let script_path = current_exe.with_extension("bat");
//         let mut file = fs::File::create(&script_path)?;
//         file.write_all(batch_script.as_bytes())?;
        
//         println!("Update downloaded. The application will restart...");
//         std::process::Command::new("cmd")
//             .args(&["/C", "start", "", script_path.to_str().unwrap()])
//             .spawn()?;
        
//         std::process::exit(0);
//     }
    
//     #[cfg(not(windows))]
//     {
//         fs::rename(&current_exe, &backup_path)?;
//         fs::rename(&temp_path, &current_exe)?;
        
//         println!("✓ Update successful!");
//         println!("Please restart the application to use the new version.");
        
//         // Clean up backup
//         let _ = fs::remove_file(backup_path);
//     }
    
//     Ok(())
// }

/// Get current version
pub fn get_current_version() -> &'static str {
    VERSION
}