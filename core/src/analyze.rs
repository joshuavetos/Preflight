use crate::fix;
use crate::models::Issue;
use crate::utils::json_envelope;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct AnalysisItem {
    pub issue: String,
    pub root_cause: String,
    pub suggested_fix: String,
}

fn load_issues() -> Result<Vec<Issue>, String> {
    let state = fix::load_state()?;
    Ok(state.issues)
}

fn suggest_fix(code: &str, issue: &Issue) -> String {
    let commands = fix::commands();
    commands
        .get(code)
        .map(|c| c.to_string())
        .unwrap_or_else(|| issue.suggestion.clone())
}

pub fn run(json_output: bool) -> Result<(), String> {
    let issues = load_issues()?;
    let analysis: Vec<AnalysisItem> = issues
        .iter()
        .map(|issue| AnalysisItem {
            issue: issue.code.clone(),
            root_cause: issue.description.clone(),
            suggested_fix: suggest_fix(&issue.code, issue),
        })
        .collect();

    if json_output {
        let payload = json_envelope(
            "analyze",
            "ok",
            json!({
                "analysis": analysis
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
        println!("Root cause analysis ({} issues):", analysis.len());
        for item in analysis {
            println!(
                "- {} => {} | fix: {}",
                item.issue, item.root_cause, item.suggested_fix
            );
        }
    }

    Ok(())
}

pub fn write_json() -> Result<String, String> {
    let issues = load_issues()?;
    let analysis: Vec<AnalysisItem> = issues
        .iter()
        .map(|issue| AnalysisItem {
            issue: issue.code.clone(),
            root_cause: issue.description.clone(),
            suggested_fix: suggest_fix(&issue.code, issue),
        })
        .collect();
    let payload = json_envelope("analyze", "ok", json!({ "analysis": analysis }));
    let rendered = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(".preflight").map_err(|e| e.to_string())?;
    let path = ".preflight/analyze.json";
    std::fs::write(path, &rendered).map_err(|e| e.to_string())?;
    Ok(path.to_string())
}
