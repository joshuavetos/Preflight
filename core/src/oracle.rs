use crate::models::{Issue, Severity, Status, SystemState};
use regex::Regex;

pub fn evaluate(state: &SystemState) -> Vec<Issue> {
    let mut issues = Vec::new();
    for node in &state.nodes {
        if node.id == "docker" && node.status != Status::Active {
            issues.push(Issue {
                code: "DOCKER_INACTIVE".to_string(),
                severity: Severity::Warning,
                title: "Docker daemon inactive".to_string(),
                description: "Docker socket was not reachable during the scan.".to_string(),
                suggestion: "Start the Docker service if container workloads are required."
                    .to_string(),
            });
        }
        if node.id == "port8000" && node.status == Status::Active {
            issues.push(Issue {
                code: "PORT_8000_BOUND".to_string(),
                severity: Severity::Critical,
                title: "Port 8000 conflict".to_string(),
                description:
                    "Port 8000 appears to be bound and may conflict with local services.".to_string(),
                suggestion: "Stop the service using port 8000 or reconfigure the workload to use a different port.".to_string(),
            });
        }
    }
    issues
}

pub fn simulate_command(command: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let normalized = command.to_lowercase();

    let port_regex = Regex::new(r"(?P<port>\d{2,5})").expect("regex compilation cannot fail");
    for cap in port_regex.captures_iter(&normalized) {
        if let Some(port_str) = cap.name("port") {
            if let Ok(port) = port_str.as_str().parse::<u16>() {
                if port == 8000 {
                    issues.push(Issue {
                        code: "SIM_PORT_8000_CONFLICT".to_string(),
                        severity: Severity::Warning,
                        title: "Potential port 8000 conflict".to_string(),
                        description: format!(
                            "The simulated command `{command}` is expected to bind port 8000, which may already be in use."
                        ),
                        suggestion: "Choose a different host port or stop the conflicting service before running the command.".to_string(),
                    });
                }
            }
        }
    }

    if normalized.contains("docker-compose") || normalized.contains("docker compose") {
        issues.push(Issue {
            code: "SIM_DOCKER_COMPOSE".to_string(),
            severity: Severity::Warning,
            title: "Docker Compose simulation".to_string(),
            description:
                "Docker Compose workloads were simulated; ensure Docker is running before execution.".to_string(),
            suggestion: "Start Docker and confirm required images are available locally.".to_string(),
        });
    }

    issues
}
