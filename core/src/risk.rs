use crate::models::{Issue, Severity};
use crate::risk_config::RiskConfig;

pub fn risk_score(issue: &Issue) -> u32 {
    let cfg = RiskConfig::load();
    let sev = match issue.severity {
        Severity::Critical => cfg.severity_critical,
        Severity::Warning => cfg.severity_warning,
    };

    let mut score = sev;
    let text = format!("{} {} {}", issue.title, issue.description, issue.suggestion).to_lowercase();

    for (keyword, weight) in cfg.keyword_weights {
        if text.contains(&keyword.to_lowercase()) {
            score += weight;
        }
    }

    score.min(100)
}

pub fn summarize_risk(issues: &[Issue]) -> u32 {
    if issues.is_empty() {
        return 0;
    }
    let total: u32 = issues.iter().map(|i| risk_score(i)).sum();
    (total / issues.len() as u32).min(100)
}
