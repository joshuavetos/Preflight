use anyhow::{anyhow, Result};
use duct::cmd;
use std::{env, path::PathBuf};

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
        cmd!("npm", "run", "build").dir(web).run()?;
    }
    println!("✔ Build complete");
    Ok(())
}

fn install() -> Result<()> {
    println!("📦 Installing Preflight CLI");
    cmd!("cargo", "install", "--path", "core", "--locked").run()?;

    let web = PathBuf::from("web");
    cmd!("npm", "install").dir(&web).run()?;
    cmd!("npm", "run", "build").dir(web).run()?;

    println!("✔ Installed. Run `preflight scan`.");
    Ok(())
}

fn dev_dashboard() -> Result<()> {
    let web = PathBuf::from("web");
    cmd!("npm", "install").dir(&web).run()?;
    cmd!("npm", "run", "dev").dir(web).run()?;
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
    println!("🚀 Creating release build");
    cmd!("cargo", "build", "--release").run()?;
    println!("✔ Release binary built at target/release/preflight");
    Ok(())
}
