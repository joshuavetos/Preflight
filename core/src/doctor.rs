use crate::utils::{ok, warn, which};
use std::path::Path;

pub fn doctor() -> Result<(), String> {
    println!("\n=== Preflight Diagnostics ===");

    if Path::new(".preflight").exists() {
        ok(".preflight directory: OK");
    } else {
        warn(".preflight directory missing");
    }

    if which("docker") {
        ok("Docker CLI: OK");
    } else {
        warn("Docker CLI not found");
    }

    if which("node") {
        ok("Node.js: OK");
    } else {
        warn("Node.js not found");
    }

    if Path::new("web/dist").exists() {
        ok("Dashboard build (web/dist): OK");
    } else {
        warn("Dashboard build missing — run `npm run build` inside /web");
    }

    Ok(())
}
