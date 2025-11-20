use crate::fix;
use crate::models::Issue;
use crate::utils::json_envelope;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize, Clone)]
pub struct SecurityIssue {
    pub code: String,
    pub severity: String,
    pub description: String,
}

fn load_security_issues() -> Result<Vec<SecurityIssue>, String> {
    let state = fix::load_state()?;
    let issues: Vec<SecurityIssue> = state
        .issues
        .into_iter()
        .filter(|i| i.code.starts_with("SEC_"))
        .map(|issue| SecurityIssue {
            code: issue.code,
            severity: format!("{:?}", issue.severity),
            description: issue.description,
        })
        .collect();
    Ok(issues)
}

pub fn run(json_output: bool) -> Result<(), String> {
    let issues = load_security_issues()?;
    let status = if issues.is_empty() { "ok" } else { "violation" };

    if json_output {
        let payload = json_envelope(
            "security",
            status,
            json!({
                "security_issues": issues
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else if issues.is_empty() {
        println!("No security issues detected");
    } else {
        println!("Security findings ({}):", issues.len());
        for issue in &issues {
            println!("- {} ({})", issue.code, issue.severity);
        }
    }
    Ok(())
}
