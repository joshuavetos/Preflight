use anyhow::{anyhow, Result};
use duct::cmd;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let task = args.next().unwrap_or_else(|| {
        eprintln!("Usage: cargo xtask <command>");
        std::process::exit(1);
    });

    match task.as_str() {
        "build" => build(),
        "install" => install(),
        "dev-dashboard" => dev_dashboard(),
        "scan" => run_scan(),
        "check" => check(),
        "release" => release(),
        other => Err(anyhow!("Unknown xtask command `{}`", other)),
    }
}

fn build() -> Result<()> {
    println!("🔧 Building Preflight (Rust + Dashboard)");
    cmd!("cargo", "build", "--release").run()?;

    let web = PathBuf::from("web");
    if web.exists() {
        cmd!("npm", "install").dir(&web).run()?;
        cmd!("npm", "run", "build").dir(&web).run()?;
    }
    println!("✔ Build complete");
    Ok(())
}

fn install() -> Result<()> {
    println!("📦 Installing Preflight CLI");
    cmd!("cargo", "install", "--path", "core", "--locked").run()?;

    let web = PathBuf::from("web");
    cmd!("npm", "install").dir(&web).run()?;
    cmd!("npm", "run", "build").dir(&web).run()?;

    println!("✔ Installed. Run `preflight scan`.");
    Ok(())
}

fn dev_dashboard() -> Result<()> {
    let web = PathBuf::from("web");
    cmd!("npm", "install").dir(&web).run()?;
    cmd!("npm", "run", "dev").dir(&web).run()?;
    Ok(())
}

fn run_scan() -> Result<()> {
    println!("🛫 Running preflight scan");
    cmd!("cargo", "run", "--bin", "preflight", "--", "scan").run()?;
    Ok(())
}

fn check() -> Result<()> {
    println!("🔍 Running fmt, clippy, and tests…");

    cmd!("cargo", "fmt", "--all", "--check").run()?;
    cmd!("cargo", "clippy", "--", "-D", "warnings").run()?;
    cmd!("cargo", "test", "--workspace").run()?;

    println!("✔ Checks passed");
    Ok(())
}

fn release() -> Result<()> {
    println!("🚀 Building release artifacts");

    // 1. Build Rust binary
    cmd!("cargo", "build", "--release").run()?;

    // 2. Build dashboard
    let web = PathBuf::from("web");
    cmd!("npm", "install").dir(&web).run()?;
    cmd!("npm", "run", "build").dir(&web).run()?;

    // 3. Prepare release directory
    let release_root = PathBuf::from("target/release-bundle");
    if release_root.exists() {
        fs::remove_dir_all(&release_root)?;
    }
    fs::create_dir_all(&release_root)?;

    // 4. Copy binary
    let bin_name = if cfg!(windows) {
        "preflight.exe"
    } else {
        "preflight"
    };
    let bin_src = Path::new("target/release").join(bin_name);
    let bin_dst = release_root.join(bin_name);
    fs::copy(&bin_src, &bin_dst)?;

    let dist_dir = PathBuf::from("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)?;
    }
    fs::create_dir_all(&dist_dir)?;
    fs::copy(&bin_dst, dist_dir.join(bin_name))?;

    // 5. Copy dashboard
    let dashboard_src = PathBuf::from("web/dist");
    let dashboard_dst = release_root.join("dashboard");
    copy_dir_all(&dashboard_src, &dashboard_dst)?;

    // 6. Generate manifest.json
    let manifest = serde_json::json!({
      "name": "preflight",
      "version": env!("CARGO_PKG_VERSION"),
      "binary": bin_name,
      "dashboard": "dashboard/",
      "os": std::env::consts::OS,
      "arch": std::env::consts::ARCH,
    });
    fs::write(release_root.join("manifest.json"), manifest.to_string())?;

    // 7. Create archive (.zip on Windows, .tar.gz elsewhere)
    if cfg!(windows) {
        cmd!(
            "powershell",
            "-Command",
            "Compress-Archive",
            "-Path",
            "target/release-bundle/*",
            "-DestinationPath",
            "target/preflight.zip"
        )
        .run()?;
        println!("📦 Created target/preflight.zip");
    } else {
        cmd!(
            "tar",
            "-czf",
            "target/preflight.tar.gz",
            "-C",
            "target",
            "release-bundle"
        )
        .run()?;
        println!("📦 Created target/preflight.tar.gz");
    }

    println!("✔ Release artifacts ready");
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
