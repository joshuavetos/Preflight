use crate::fix;
use crate::utils::{json_envelope, ok, warn, which};
use serde::Serialize;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub struct DoctorIssue {
    pub code: String,
    pub fixable: bool,
    pub fix_command: Option<String>,
}

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

fn collect_issues() -> Result<Vec<DoctorIssue>, String> {
    let state = fix::load_state()?;
    let fixes = fix::commands();
    Ok(state
        .issues
        .into_iter()
        .map(|issue| DoctorIssue {
            code: issue.code.clone(),
            fixable: fixes.contains_key(issue.code.as_str()),
            fix_command: fixes.get(issue.code.as_str()).map(|c| c.to_string()),
        })
        .collect())
}

pub fn doctor(json_output: bool) -> Result<(), String> {
    if json_output {
        if Path::new(".preflight/scan.json").exists() {
            let issues = collect_issues()?;
            let payload = json_envelope(
                "doctor",
                "ok",
                json!({
                    "issues": issues
                }),
            );
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(())
        } else {
            Err("No scan data available — run `preflight scan` first.".into())
        }
    } else {
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
}
