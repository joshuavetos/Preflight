use crate::models::{Issue, Severity};
use crate::risk_config::RiskConfig;

/// Compute weighted risk using the external config
pub fn risk_score(issue: &Issue, cfg: &RiskConfig) -> u32 {
    cfg.weight_for(&issue.code)
}

/// Summarize total risk across all issues
pub fn summarize_risk(issues: &[Issue], cfg: &RiskConfig) -> u32 {
    issues.iter().map(|i| risk_score(i, cfg)).sum()
}

/// Returns the possibly overridden severity based on config
pub fn severity_for(issue: &Issue, cfg: &RiskConfig) -> Severity {
    if let Some(ov) = cfg.severity_override(&issue.code) {
        match ov.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "warning" => Severity::Warning,
            _ => issue.severity.clone(),
        }
    } else {
        issue.severity.clone()
    }
}
