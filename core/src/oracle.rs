use crate::command_ast::parse_command;
use crate::json_diff::diff_states;
use crate::models::{Issue, Severity, SystemState};
use crate::proposed_state::{apply_predicted_changes, clone_state};
use serde_json::json;
use serde_json::Value;

pub struct SimulationResult {
    pub issues: Vec<Issue>,
    pub proposed_state: Option<SystemState>,
    pub diff: Option<Value>,
}

pub fn evaluate(state: &SystemState) -> Vec<Issue> {
    // unchanged from Drop 2 — left intact intentionally
    let mut issues = Vec::new();

    for node in &state.nodes {
        if node.id == "docker" && node.status != crate::models::Status::Active {
            issues.push(Issue {
                code: "DOCKER_INACTIVE".into(),
                severity: Severity::Warning,
                title: "Docker daemon inactive".into(),
                description: "Docker was unreachable during the scan.".into(),
                suggestion: "Start the Docker service.".into(),
            });
        }

        if node.id == "port8000" && node.status == crate::models::Status::Active {
            issues.push(Issue {
                code: "PORT_8000_BOUND".into(),
                severity: Severity::Critical,
                title: "Port 8000 conflict".into(),
                description: "Port 8000 appears to be bound.".into(),
                suggestion: "Stop the conflicting service or select another port.".into(),
            });
        }
    }

    issues
}

pub fn simulate_command(raw: &str) -> SimulationResult {
    let parsed = parse_command(raw);

    // Same issue logic as before
    let mut issues = Vec::new();

    for p in parsed.ports.iter() {
        if *p == 8000 {
            issues.push(Issue {
                code: "SIM_PORT_8000_CONFLICT".into(),
                severity: Severity::Warning,
                title: "Potential port conflict".into(),
                description: format!("Command `{}` may bind port 8000.", raw),
                suggestion: "Choose another port or stop the conflicting workload.".into(),
            });
        }
    }

    if parsed.docker_compose {
        issues.push(Issue {
            code: "SIM_DOCKER_COMPOSE".into(),
            severity: Severity::Warning,
            title: "Docker Compose workload".into(),
            description: "Requires Docker daemon running.".into(),
            suggestion: "Ensure Docker is active.".into(),
        });
    }

    // Build proposed state
    let current_state = match std::fs::read_to_string(".preflight/scan.json") {
        Ok(s) => serde_json::from_str::<SystemState>(&s).unwrap(),
        Err(_) => {
            return SimulationResult {
                issues,
                proposed_state: None,
                diff: None,
            };
        }
    };

    let proposed = apply_predicted_changes(clone_state(&current_state), &parsed);

    // Compute JSON diff
    let current_json = json!(current_state);
    let proposed_json = json!(proposed);
    let diff = diff_states(&current_json, &proposed_json);

    SimulationResult {
        issues,
        proposed_state: Some(proposed),
        diff: Some(diff),
    }
}
