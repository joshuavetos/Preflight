use serde::Deserialize;
use std::{collections::HashMap, fs};

/// Fallback config if file missing or corrupt
fn default_config() -> RiskConfig {
    RiskConfig {
        issue_weights: HashMap::from([
            ("DOCKER_INACTIVE".to_string(), 10),
            ("PORT_8000_BOUND".to_string(), 100),
            ("SIM_PORT_8000_CONFLICT".to_string(), 20),
            ("SIM_DOCKER_COMPOSE".to_string(), 15),
        ]),
        severity_overrides: HashMap::new(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    pub issue_weights: HashMap<String, u32>,
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
}

impl RiskConfig {
    pub fn load() -> Self {
        match fs::read_to_string(".preflight/risk_config.json") {
            Ok(raw) => match serde_json::from_str::<RiskConfig>(&raw) {
                Ok(cfg) => {
                    if cfg.issue_weights.is_empty() {
                        eprintln!("⚠️ Empty issue_weights in risk_config.json — using defaults");
                        default_config()
                    } else {
                        cfg
                    }
                }
                Err(_) => {
                    eprintln!("⚠️ Invalid risk_config.json — using defaults");
                    default_config()
                }
            },
            Err(_) => default_config(),
        }
    }

    pub fn weight_for(&self, code: &str) -> u32 {
        self.issue_weights.get(code).cloned().unwrap_or(0)
    }

    pub fn severity_override(&self, code: &str) -> Option<String> {
        self.severity_overrides.get(code).cloned()
    }
}
