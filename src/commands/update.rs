use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use serde::Deserialize;

use crate::error::{Error, Result};

const GITHUB_REPO: &str = "nickcramaro/linear-cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn get_asset_name() -> Result<&'static str> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("macos", "aarch64") => Ok("linear-macos-aarch64"),
        ("macos", "x86_64") => Ok("linear-macos-x86_64"),
        ("linux", "x86_64") => Ok("linear-linux-x86_64"),
        ("linux", "aarch64") => Ok("linear-linux-aarch64"),
        _ => Err(Error::GraphQL(format!(
            "Unsupported platform: {}-{}",
            os, arch
        ))),
    }
}

pub async fn handle_update() -> Result<()> {
    println!("Current version: v{}", CURRENT_VERSION);
    println!("Checking for updates...");

    // Fetch latest release from GitHub
    let client = reqwest::Client::new();
    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ))
        .header("User-Agent", "linear-cli")
        .send()
        .await?
        .json()
        .await?;

    let latest_version = release.tag_name.trim_start_matches('v');
    println!("Latest version: v{}", latest_version);

    if latest_version == CURRENT_VERSION {
        println!("Already up to date!");
        return Ok(());
    }

    // Find the right asset for this platform
    let asset_name = get_asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| Error::GraphQL(format!("No release asset found for {}", asset_name)))?;

    println!("Downloading {}...", asset_name);

    // Download the binary
    let binary_data = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "linear-cli")
        .send()
        .await?
        .bytes()
        .await?;

    // Get current executable path
    let current_exe = env::current_exe().map_err(|e| Error::GraphQL(e.to_string()))?;

    // Write to a temp file first
    let temp_path = current_exe.with_extension("new");
    let mut temp_file = fs::File::create(&temp_path).map_err(|e| Error::GraphQL(e.to_string()))?;
    temp_file
        .write_all(&binary_data)
        .map_err(|e| Error::GraphQL(e.to_string()))?;

    // Set executable permissions
    let mut perms = fs::metadata(&temp_path)
        .map_err(|e| Error::GraphQL(e.to_string()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_path, perms).map_err(|e| Error::GraphQL(e.to_string()))?;

    // Replace the current binary
    fs::rename(&temp_path, &current_exe).map_err(|e| Error::GraphQL(e.to_string()))?;

    println!("Updated to v{}!", latest_version);

    Ok(())
}
