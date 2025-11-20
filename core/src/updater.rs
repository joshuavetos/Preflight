use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;

use crate::utils;

const ORG: &str = "PreflightHQ";
const REPO: &str = "Preflight";
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/PreflightHQ/Preflight/releases/latest";

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

pub struct ReleaseInfo {
    pub version: Version,
    pub download_url: String,
}

pub fn fetch_latest_release() -> Result<ReleaseInfo> {
    let client = Client::new();
    let release: GitHubRelease = client
        .get(LATEST_RELEASE_URL)
        .header(
            reqwest::header::USER_AGENT,
            format!("preflight-updater ({ORG}/{REPO})"),
        )
        .send()
        .context("failed to fetch latest release metadata")?
        .json()
        .context("failed to parse GitHub release response")?;

    let version_str = release.tag_name.trim_start_matches('v');
    let version = Version::parse(version_str)
        .with_context(|| format!("invalid semver tag in release: {}", release.tag_name))?;

    let os = std::env::consts::OS;
    let download_url = release
        .assets
        .iter()
        .find(|asset| match os {
            "linux" => asset.name.to_lowercase().contains("linux"),
            "macos" | "darwin" => {
                let name = asset.name.to_lowercase();
                name.contains("macos") || name.contains("darwin") || name.contains("apple")
            }
            "windows" => asset.name.to_lowercase().contains("win"),
            _ => false,
        })
        .map(|asset| asset.browser_download_url.clone())
        .ok_or_else(|| anyhow!("no downloadable asset found for {os}"))?;

    Ok(ReleaseInfo {
        version,
        download_url,
    })
}

pub fn download_binary(url: &str, dest_tmp: &Path) -> Result<()> {
    let client = Client::new();
    let bytes = client
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            format!("preflight-updater ({ORG}/{REPO})"),
        )
        .send()
        .context("failed to download binary")?
        .bytes()
        .context("failed to read binary stream")?;

    if let Some(parent) = dest_tmp.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(dest_tmp, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest_tmp)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest_tmp, perms)?;
    }

    Ok(())
}

pub fn atomic_swap(tmp: &Path, current_path: &Path) -> Result<()> {
    fs::rename(tmp, current_path).context("failed to replace current binary")
}

pub fn upgrade(json_output: bool) -> Result<()> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let release = fetch_latest_release()?;

    if release.version <= current_version {
        if json_output {
            let payload = utils::json_envelope(
                "upgrade",
                "ok",
                serde_json::json!({
                    "message": "already up-to-date"
                }),
            );
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("You're already on the latest version: {}", current_version);
        }
        return Ok(());
    }

    let tmp_path = Path::new(".preflight").join("preflight.tmp");
    download_binary(&release.download_url, &tmp_path)?;

    let current_path = std::env::current_exe()?;
    atomic_swap(&tmp_path, &current_path)?;

    if json_output {
        let payload = utils::json_envelope(
            "upgrade",
            "ok",
            serde_json::json!({
                "from": current_version.to_string(),
                "to": release.version.to_string()
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "✔ Upgrade complete! {} → {}",
            current_version, release.version
        );
    }

    Ok(())
}
