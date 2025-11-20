use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs;

#[derive(Debug, serde::Deserialize)]
struct ReleaseManifest {
    version: String,
    linux_url: Option<String>,
    mac_url: Option<String>,
    windows_url: Option<String>,
    sha256: String,
}

const DEFAULT_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/PreflightHQ/Preflight/main/releases.json";

pub fn upgrade() -> Result<()> {
    println!("\n=== Preflight Updater ===");

    let manifest_url = std::env::var("PREFLIGHT_MANIFEST_URL")
        .unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_string());

    println!("Fetching manifest from {manifest_url}...");
    let client = Client::new();
    let manifest_text = client.get(&manifest_url).send()?.text()?;

    let manifest: ReleaseManifest =
        serde_json::from_str(&manifest_text).map_err(|e| anyhow!("Invalid manifest: {e}"))?;

    let current = env!("CARGO_PKG_VERSION");
    if manifest.version == current {
        println!("You're already on the latest version: {current}");
        return Ok(());
    }

    println!("Latest version available: {}", manifest.version);

    // Select URL for this OS
    let os = std::env::consts::OS;
    let url = match os {
        "linux" => manifest
            .linux_url
            .ok_or_else(|| anyhow!("No Linux build in manifest"))?,
        "macos" | "darwin" => manifest
            .mac_url
            .ok_or_else(|| anyhow!("No macOS build in manifest"))?,
        "windows" => manifest
            .windows_url
            .ok_or_else(|| anyhow!("No Windows build in manifest"))?,
        _ => return Err(anyhow!("Unsupported OS: {os}")),
    };

    println!("Downloading binary from: {url}");
    let bytes = client.get(&url).send()?.bytes()?;

    // Verify checksum
    println!("Verifying checksum...");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());

    if hash != manifest.sha256 {
        return Err(anyhow!("Checksum mismatch — aborting update"));
    }

    // Determine install path
    let install_path = std::env::current_exe()?;
    let tmp_path = install_path.with_extension("tmp");

    println!("Installing to: {install_path:?}");

    // Write tmp file
    fs::write(&tmp_path, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_path, perms)?;
    }

    // Atomic replace
    fs::rename(&tmp_path, &install_path)?;

    println!(
        "✔ Upgrade complete! Now running version {}",
        manifest.version
    );

    Ok(())
}
