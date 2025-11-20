use crate::command_ast::parse_command;
use crate::json_diff::diff_states;
use crate::models::{Issue, Severity, Status, SystemState};
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

        match node.id.as_str() {
            "nodejs" if node.status != Status::Active => {
                issues.push(Issue {
                    code: "NODEJS_INACTIVE".into(),
                    severity: Severity::Warning,
                    title: "Node.js unavailable".into(),
                    description: "Node.js was not detected during the scan.".into(),
                    suggestion: "Install Node.js and ensure it is available on PATH.".into(),
                });
            }
            "postgres" => {
                let port_bound = node
                    .metadata
                    .get("port_bound")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if port_bound {
                    issues.push(Issue {
                        code: "POSTGRES_PORT_BOUND".into(),
                        severity: Severity::Warning,
                        title: "PostgreSQL port bound".into(),
                        description: "Port 5432 is currently bound.".into(),
                        suggestion: "Stop the conflicting PostgreSQL instance or update the port configuration.".into(),
                    });
                }
                if node.status != Status::Active {
                    issues.push(Issue {
                        code: "POSTGRES_INACTIVE".into(),
                        severity: Severity::Warning,
                        title: "PostgreSQL unavailable".into(),
                        description: "PostgreSQL was not detected during the scan.".into(),
                        suggestion: "Install or start PostgreSQL and verify psql is reachable.".into(),
                    });
                }
            }
            "redis" => {
                let port_bound = node
                    .metadata
                    .get("port_bound")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if port_bound {
                    issues.push(Issue {
                        code: "REDIS_PORT_BOUND".into(),
                        severity: Severity::Warning,
                        title: "Redis port bound".into(),
                        description: "Port 6379 is currently bound.".into(),
                        suggestion: "Stop the conflicting Redis instance or adjust the configured port.".into(),
                    });
                }
                if node.status != Status::Active {
                    issues.push(Issue {
                        code: "REDIS_INACTIVE".into(),
                        severity: Severity::Warning,
                        title: "Redis unavailable".into(),
                        description: "Redis was not detected during the scan.".into(),
                        suggestion: "Install or start Redis so redis-server or redis-cli are reachable.".into(),
                    });
                }
            }
            "gpu" if node.status != Status::Active => {
                issues.push(Issue {
                    code: "GPU_MISSING".into(),
                    severity: Severity::Warning,
                    title: "GPU unavailable".into(),
                    description: "No GPU was detected via nvidia-smi.".into(),
                    suggestion: "Install GPU drivers or ensure the GPU is accessible to this environment.".into(),
                });
            }
            _ => {}
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
