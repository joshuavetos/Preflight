use crate::config::RiskConfig;
use crate::models::{Issue, Severity};

pub fn risk_score(issue: &Issue, cfg: &RiskConfig) -> u32 {
    if let Some(override_val) = cfg.issue_overrides.get(&issue.code) {
        return *override_val;
    }

    match issue.severity {
        Severity::Critical => cfg.weights.critical,
        Severity::Warning => cfg.weights.warning,
    }
}

pub fn summarize_risk(issues: &[Issue], cfg: &RiskConfig) -> u32 {
    issues.iter().map(|i| risk_score(i, cfg)).sum()
}
