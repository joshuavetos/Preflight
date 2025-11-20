use crate::fix;
use crate::utils::{ok, warn, which};
use std::path::Path;

fn print_issue_overview() -> Result<(), String> {
    let state = fix::load_state()?;
    let fixes = fix::commands();

    println!("\n=== Scan Issues (fix integration) ===");
    for issue in state.issues {
        let fix_command = fixes.get(issue.code.as_str());
        let fixable = if fix_command.is_some() {
            "fixable"
        } else {
            "unfixable"
        };
        println!(
            "- {code} | severity: {severity:?} | {fixable}{cmd}",
            code = issue.code,
            severity = issue.severity,
            fixable = fixable,
            cmd = fix_command
                .map(|c| format!(" | fix: {}", c))
                .unwrap_or_else(|| String::from(""))
        );
    }

    Ok(())
}

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

    if Path::new(".preflight/scan.json").exists() {
        print_issue_overview()?;
    } else {
        warn("No scan data available — run `preflight scan` to populate issues.");
    }

    Ok(())
}
