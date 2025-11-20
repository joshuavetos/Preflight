use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct RiskConfig {
    pub keyword_weights: Vec<(String, u32)>,
    pub severity_critical: u32,
    pub severity_warning: u32,
}

impl RiskConfig {
    pub fn load() -> RiskConfig {
        let path = ".preflight/risk_config.toml";
        if let Ok(toml_str) = fs::read_to_string(path) {
            toml::from_str(&toml_str).unwrap_or_else(|_| RiskConfig::default())
        } else {
            RiskConfig::default()
        }
    }

    pub fn default() -> RiskConfig {
        RiskConfig {
            keyword_weights: vec![
                ("port".into(), 20),
                ("bind".into(), 20),
                ("docker".into(), 10),
                ("gpu".into(), 10),
                ("compose".into(), 15),
                ("memory".into(), 15),
            ],
            severity_critical: 60,
            severity_warning: 30,
        }
    }
}
