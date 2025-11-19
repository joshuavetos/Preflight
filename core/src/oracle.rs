use crate::models::{Issue, Severity, Status, SystemState};

pub fn evaluate(state: &SystemState) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut id_counter = 1;
    for node in &state.nodes {
        if node.id == "docker" && node.status != Status::Active {
            issues.push(Issue {
                id: id_counter,
                severity: Severity::Warning,
                title: "Docker daemon inactive".to_string(),
                description: "Docker socket was not reachable during the scan.".to_string(),
                suggestion: "Start the Docker service if container workloads are required."
                    .to_string(),
            });
            id_counter += 1;
        }
        if node.id == "port8000" && node.status == Status::Active {
            issues.push(Issue {
                id: id_counter,
                severity: Severity::Critical,
                title: "Port 8000 conflict".to_string(),
                description: "Port 8000 appears to be bound and may conflict with local services.".to_string(),
                suggestion: "Stop the service using port 8000 or reconfigure the workload to use a different port.".to_string(),
            });
            id_counter += 1;
        }
    }
    issues
}

pub fn simulate_command(command: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut id_counter = 1000;
    let normalized = command.to_lowercase();
    if normalized.contains("8000") || normalized.contains("port 8000") {
        issues.push(Issue {
            id: id_counter,
            severity: Severity::Warning,
            title: "Potential port 8000 conflict".to_string(),
            description: format!(
                "The simulated command `{command}` is expected to bind port 8000, which may already be in use."
            ),
            suggestion: "Choose a different host port or stop the conflicting service before running the command.".to_string(),
        });
        id_counter += 1;
    }
    if normalized.contains("docker-compose") || normalized.contains("docker compose") {
        issues.push(Issue {
            id: id_counter,
            severity: Severity::Warning,
            title: "Docker Compose simulation".to_string(),
            description: "Docker Compose workloads were simulated; ensure Docker is running before execution.".to_string(),
            suggestion: "Start Docker and confirm required images are available locally.".to_string(),
        });
    }
    issues
}
