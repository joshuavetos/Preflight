use crate::models::{Issue, Severity, Status, SystemState};
use crate::risk::summarize_risk;
use crate::risk_config::RiskConfig;
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

    //-------------------------------------------
    // ADVANCED PORT HEURISTICS
    //-------------------------------------------
    let port_regex = Regex::new(r"(?P<port>\d{2,5})").unwrap();
    for cap in port_regex.captures_iter(&normalized) {
        if let Some(p) = cap.name("port") {
            if let Ok(port) = p.as_str().parse::<u16>() {
                match port {
                    5432 => issues.push(Issue {
                        code: "SIM_PG_PORT".into(),
                        severity: Severity::Warning,
                        title: "Postgres port usage predicted".into(),
                        description: "Command indicates an intention to bind/use port 5432".into(),
                        suggestion: "Ensure postgres or workload does not conflict".into(),
                    }),
                    6379 => issues.push(Issue {
                        code: "SIM_REDIS_PORT".into(),
                        severity: Severity::Warning,
                        title: "Redis port usage predicted".into(),
                        description: "Command suggests binding redis default port".into(),
                        suggestion: "Verify redis-server status".into(),
                    }),
                    8000 => issues.push(Issue {
                        code: "SIM_PORT_8000".into(),
                        severity: Severity::Warning,
                        title: "Predicted port 8000 binding".into(),
                        description: "Workload likely to conflict with local dev servers".into(),
                        suggestion: "Free 8000 or switch service port".into(),
                    }),
                    _ => {}
                }
            }
        }
    }

    //-------------------------------------------
    // GPU HEURISTICS
    //-------------------------------------------
    if normalized.contains("--gpus") || normalized.contains("gpu") {
        issues.push(Issue {
            code: "SIM_GPU_ACCESS".into(),
            severity: Severity::Warning,
            title: "GPU passthrough requested".into(),
            description: "The command uses GPU flags which require nvidia-container-runtime".into(),
            suggestion: "Ensure nvidia drivers + container toolkit installed".into(),
        });
    }

    //-------------------------------------------
    // MEMORY + RESOURCE HEURISTICS
    //-------------------------------------------
    if normalized.contains("docker compose") && normalized.contains("up") {
        issues.push(Issue {
            code: "SIM_COMPOSE_RESOURCE".into(),
            severity: Severity::Warning,
            title: "Docker Compose workload may exceed system memory".into(),
            description: "Multiple containers may exceed available RAM or swap".into(),
            suggestion: "Check compose yaml for memory limits".into(),
        });
    }

    //-------------------------------------------
    // PYTHON / VENV HEURISTICS
    //-------------------------------------------
    if normalized.contains("python") && normalized.contains("-m") {
        issues.push(Issue {
            code: "SIM_PYTHON_VENV".into(),
            severity: Severity::Warning,
            title: "Python module execution predicted".into(),
            description: "Command may require an activated virtualenv".into(),
            suggestion: "Ensure `.venv` is activated or dependencies installed".into(),
        });
    }

    //-------------------------------------------
    // DOCKER IMAGE CHECKS
    //-------------------------------------------
    if normalized.contains("docker build") {
        issues.push(Issue {
            code: "SIM_DOCKER_BUILD".into(),
            severity: Severity::Warning,
            title: "Docker build predicted".into(),
            description: "Build operations may fail without proper Dockerfile context".into(),
            suggestion: "Ensure Dockerfile exists in build directory".into(),
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

    //-------------------------------------------
    // RISK SUMMARY (ADDED AS SYNTHETIC ISSUE)
    //-------------------------------------------
    if !issues.is_empty() {
        let cfg = RiskConfig::load();
        let score = summarize_risk(&issues, &cfg);
        issues.push(Issue {
            code: "SIM_RISK_SUMMARY".into(),
            severity: if score >= 70 {
                Severity::Critical
            } else {
                Severity::Warning
            },
            title: format!("Overall risk score: {score}"),
            description: format!("Based on predicted behaviors in `{command}`."),
            suggestion: "Review issues above to lower operational risk".into(),
        });
    }

    issues
}
