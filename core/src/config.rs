use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub weights: RiskWeights,
    #[serde(default)]
    pub issue_overrides: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskWeights {
    pub critical: u32,
    pub warning: u32,
}

impl RiskConfig {
    pub fn load() -> Result<Self, String> {
        let path = ".preflight/risk_config.json";

        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to load {path}: {e}"))?;

        serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid risk_config.json: {e}"))
    }
}
