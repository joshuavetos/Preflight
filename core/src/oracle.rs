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
                description: "Port 8000 appears to be bound and may conflict with local services.".to_string(),
                suggestion:
                    "Stop the service using port 8000 or reconfigure the workload to use a different port.".to_string(),
            });
        }
    }

    //-------------------------------
    // POSTGRES ISSUES
    //-------------------------------
    if state.nodes.iter().any(|n| n.id == "postgres") {
        let pg = state.nodes.iter().find(|n| n.id == "postgres").unwrap();

        if !pg.metadata.contains_key("version") {
            issues.push(Issue {
                code: "POSTGRES_NO_VERSION".into(),
                severity: Severity::Warning,
                title: "Postgres detected but version unknown".into(),
                description: "Could not determine PostgreSQL version".into(),
                suggestion: "Ensure `psql --version` returns a valid version".into(),
            });
        }

        // Port 5432 conflict
        if state
            .nodes
            .iter()
            .any(|n| n.id == "port5432" && n.status == Status::Active)
        {
            issues.push(Issue {
                code: "POSTGRES_PORT_5432_CONFLICT".into(),
                severity: Severity::Critical,
                title: "Port 5432 conflict".into(),
                description: "Port 5432 appears to be in use; Postgres will fail to start".into(),
                suggestion: "Stop the conflicting service or change Postgres port".into(),
            });
        }
    }

    //-------------------------------
    // REDIS ISSUES
    //-------------------------------
    if state.nodes.iter().any(|n| n.id == "redis") {
        let redis = state.nodes.iter().find(|n| n.id == "redis").unwrap();

        if !redis.metadata.contains_key("reachable") {
            issues.push(Issue {
                code: "REDIS_NOT_RESPONDING".into(),
                severity: Severity::Critical,
                title: "Redis unreachable".into(),
                description: "Redis CLI ping failed".into(),
                suggestion: "Ensure redis-server is running and responding to PING".into(),
            });
        }

        if state
            .nodes
            .iter()
            .any(|n| n.id == "port6379" && n.status == Status::Active)
        {
            issues.push(Issue {
                code: "REDIS_PORT_6379_CONFLICT".into(),
                severity: Severity::Critical,
                title: "Port 6379 conflict".into(),
                description: "Another process is bound to Redis default port".into(),
                suggestion: "Stop the conflicting process or reconfigure Redis".into(),
            });
        }
    }

    //-------------------------------
    // PYTHON ISSUES
    //-------------------------------
    if let Some(py) = state.nodes.iter().find(|n| n.id == "python") {
        if let Some(ver) = py.metadata.get("version") {
            if let Some(s) = ver.as_str() {
                if let Some(cap) = regex::Regex::new(r"3\.(\d{1,2})").unwrap().captures(s) {
                    if let Ok(minor) = cap[1].parse::<u32>() {
                        if minor < 9 {
                            issues.push(Issue {
                                code: "PYTHON_DEPRECATED".into(),
                                severity: Severity::Warning,
                                title: "Outdated Python version".into(),
                                description: format!(
                                    "Python {s} is below recommended minimum (3.9)"
                                ),
                                suggestion: "Upgrade to Python 3.9+ for security and compatibility"
                                    .into(),
                            });
                        }
                    }
                }
            }
        }
    }

    //-------------------------------
    // GPU ISSUES
    //-------------------------------
    if state.nodes.iter().any(|n| n.id == "gpu") {
        let gpu = state.nodes.iter().find(|n| n.id == "gpu").unwrap();

        if !gpu.metadata.contains_key("nvidia_smi") {
            issues.push(Issue {
                code: "GPU_DRIVER_MISSING".into(),
                severity: Severity::Warning,
                title: "GPU detected, but drivers missing".into(),
                description: "NVIDIA GPU detected but `nvidia-smi` did not provide driver output"
                    .into(),
                suggestion: "Install latest NVIDIA drivers".into(),
            });
        }
    }

    //-------------------------------
    // DOCKER IMAGES ISSUES
    //-------------------------------
    if let Some(imgs) = state.nodes.iter().find(|n| n.id == "docker_images") {
        let count = imgs
            .metadata
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if count == 0 {
            issues.push(Issue {
                code: "DOCKER_NO_IMAGES".into(),
                severity: Severity::Warning,
                title: "No Docker images installed".into(),
                description: "Docker is installed but contains no local images".into(),
                suggestion: "Pull or build required images before running workloads".into(),
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
                        suggestion: "Choose a different host port or stop the conflicting service before running the command."
                            .to_string(),
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

    //-------------------------------
    // POSTGRES SIMULATION
    //-------------------------------
    if normalized.contains("postgres") || normalized.contains("psql") {
        issues.push(Issue {
            code: "SIM_POSTGRES_REQUIRED".into(),
            severity: Severity::Warning,
            title: "Command references Postgres".into(),
            description: "Postgres-related command detected; ensure server reachable".into(),
            suggestion: "Start postgres or verify configuration".into(),
        });
    }

    if normalized.contains("5432") {
        issues.push(Issue {
            code: "SIM_PORT_5432_CONFLICT".into(),
            severity: Severity::Warning,
            title: "Potential Postgres port conflict".into(),
            description: "Command suggests binding or using port 5432".into(),
            suggestion: "Ensure 5432 is free".into(),
        });
    }

    //-------------------------------
    // REDIS SIMULATION
    //-------------------------------
    if normalized.contains("redis") {
        issues.push(Issue {
            code: "SIM_REDIS_REQUIRED".into(),
            severity: Severity::Warning,
            title: "Command references Redis".into(),
            description: "Redis-related command detected".into(),
            suggestion: "Ensure redis-server is running".into(),
        });
    }

    if normalized.contains("6379") {
        issues.push(Issue {
            code: "SIM_PORT_6379_CONFLICT".into(),
            severity: Severity::Warning,
            title: "Potential Redis port conflict".into(),
            description: "Command suggests binding or using port 6379".into(),
            suggestion: "Ensure 6379 is free".into(),
        });
    }

    issues
}
