//! Unified risk scoring for Preflight simulation.
//! Each predicted issue can include a numerical score (0–100).

use crate::models::{Issue, Severity};

/// Compute a numeric risk score for simulation issues.
/// Severity + contextual keywords drive the weighting.
pub fn risk_score(issue: &Issue) -> u32 {
    let sev = match issue.severity {
        Severity::Critical => 60,
        Severity::Warning => 30,
    };

    let mut score = sev;
    let text = format!("{} {} {}", issue.title, issue.description, issue.suggestion).to_lowercase();

    // Keyword amplification
    if text.contains("bind") || text.contains("port") {
        score += 20;
    }
    if text.contains("docker") {
        score += 10;
    }
    if text.contains("gpu") {
        score += 10;
    }
    if text.contains("compose") {
        score += 15;
    }
    if text.contains("memory") {
        score += 15;
    }

    // Cap at 100
    score.min(100)
}

/// Summarizes multiple risk scores into a single bucket.
pub fn summarize_risk(all: &[Issue]) -> u32 {
    if all.is_empty() {
        return 0;
    }
    let total: u32 = all.iter().map(risk_score).sum();
    (total / all.len() as u32).min(100)
}
