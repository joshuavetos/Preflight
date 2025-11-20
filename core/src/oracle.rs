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
            "python" => {
                let env_flags = [
                    node.metadata
                        .get("venv")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    node.metadata
                        .get("pipenv")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    node.metadata
                        .get("poetry")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    node.metadata
                        .get("conda")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                ];
                let active_envs: usize = env_flags.iter().filter(|b| **b).count();
                if node.status == Status::Active {
                    if active_envs > 1 {
                        issues.push(Issue {
                            code: "PYTHON_MULTIPLE_ENV".into(),
                            severity: Severity::Warning,
                            title: "Multiple Python environments active".into(),
                            description:
                                "More than one Python environment tool detected simultaneously."
                                    .into(),
                            suggestion:
                                "Deactivate extra environments and keep a single manager active."
                                    .into(),
                        });
                    }
                    if active_envs == 0 {
                        issues.push(Issue {
                            code: "PYTHON_NO_ENV".into(),
                            severity: Severity::Warning,
                            title: "No Python environment detected".into(),
                            description: "Python is installed but no virtual environment manager is active.".into(),
                            suggestion: "Create and activate a virtual environment via venv, pipenv, poetry, or conda.".into(),
                        });
                    }
                    let missing = node
                        .metadata
                        .get("python_requirements_missing")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if !missing.is_empty() {
                        let missing_list: Vec<String> = missing
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        if !missing_list.is_empty() {
                            issues.push(Issue {
                                code: "PYTHON_PACKAGE_MISSING".into(),
                                severity: Severity::Warning,
                                title: "Python packages missing".into(),
                                description: format!(
                                    "requirements.txt lists missing packages: {}.",
                                    missing_list.join(", ")
                                ),
                                suggestion: "Install missing dependencies with pip install -r requirements.txt.".into(),
                            });
                        }
                    }
                    let drifts = node
                        .metadata
                        .get("python_requirements_drift")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if !drifts.is_empty() {
                        let details: Vec<String> = drifts
                            .iter()
                            .filter_map(|entry| {
                                let name = entry.get("name")?.as_str()?;
                                let required = entry.get("required")?.as_str().unwrap_or("");
                                let installed = entry.get("installed")?.as_str().unwrap_or("");
                                Some(format!("{} ({} -> {})", name, required, installed))
                            })
                            .collect();
                        if !details.is_empty() {
                            issues.push(Issue {
                                code: "PYTHON_REQUIREMENTS_DRIFT".into(),
                                severity: Severity::Warning,
                                title: "Python dependency drift".into(),
                                description: format!(
                                    "Installed packages do not satisfy requirements: {}.",
                                    details.join(", ")
                                ),
                                suggestion:
                                    "Reinstall dependencies with pip install -r requirements.txt or update pinned versions.".into(),
                            });
                        }
                    }
                    let lock_drift = node
                        .metadata
                        .get("python_lockfile_drift")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if lock_drift {
                        issues.push(Issue {
                            code: "PYTHON_LOCKFILE_DRIFT".into(),
                            severity: Severity::Warning,
                            title: "Python lockfile drift".into(),
                            description:
                                "Pipfile.lock or poetry.lock is older than its source manifest.".into(),
                            suggestion:
                                "Regenerate the lockfile with pipenv lock or poetry lock to capture current requirements.".into(),
                        });
                    }
                    let v = node.metadata.get("version").and_then(|v| v.as_str());
                    let v3 = node
                        .metadata
                        .get("python3_version")
                        .and_then(|v| v.as_str());
                    if let (Some(a), Some(b)) = (v, v3) {
                        if a != b {
                            issues.push(Issue {
                                code: "PYTHON_VERSION_DRIFT".into(),
                                severity: Severity::Warning,
                                title: "Python version drift".into(),
                                description: "python and python3 report different versions.".into(),
                                suggestion:
                                    "Align python and python3 to the same version or adjust PATH."
                                        .into(),
                            });
                        }
                    }
                }
            }
            "nodejs" if node.status != Status::Active => {
                issues.push(Issue {
                    code: "NODEJS_INACTIVE".into(),
                    severity: Severity::Warning,
                    title: "Node.js unavailable".into(),
                    description: "Node.js was not detected during the scan.".into(),
                    suggestion: "Install Node.js and ensure it is available on PATH.".into(),
                });
            }
            "nodejs" => {
                let has_package_json = node
                    .metadata
                    .get("package_json_present")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let node_modules_mismatch = node
                    .metadata
                    .get("node_modules_mismatch")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let lockfile_drift = node
                    .metadata
                    .get("lockfile_drift")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !has_package_json {
                    issues.push(Issue {
                        code: "NODE_PACKAGE_MISSING".into(),
                        severity: Severity::Warning,
                        title: "package.json missing".into(),
                        description: "No package.json found for the Node.js workspace.".into(),
                        suggestion:
                            "Initialize the project with npm init or ensure package.json exists."
                                .into(),
                    });
                }
                if node_modules_mismatch {
                    issues.push(Issue {
                        code: "NODE_LOCKFILE_DRIFT".into(),
                        severity: Severity::Warning,
                        title: "Dependencies not installed".into(),
                        description: "package.json present but node_modules missing.".into(),
                        suggestion: "Run npm install or your package manager to sync dependencies."
                            .into(),
                    });
                } else if lockfile_drift {
                    issues.push(Issue {
                        code: "NODE_LOCKFILE_DRIFT".into(),
                        severity: Severity::Warning,
                        title: "Lockfile out of date".into(),
                        description: "package.json is newer than package-lock.json.".into(),
                        suggestion: "Regenerate lockfile to reflect package.json changes.".into(),
                    });
                }
                let version_mismatches = node
                    .metadata
                    .get("node_version_mismatches")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if !version_mismatches.is_empty() {
                    let details: Vec<String> = version_mismatches
                        .iter()
                        .filter_map(|entry| {
                            let name = entry.get("name")?.as_str()?;
                            let required = entry.get("required")?.as_str().unwrap_or("");
                            let installed = entry
                                .get("installed")
                                .and_then(|v| v.as_str())
                                .unwrap_or("missing");
                            Some(format!("{} ({} -> {})", name, required, installed))
                        })
                        .collect();
                    if !details.is_empty() {
                        issues.push(Issue {
                            code: "NODE_VERSION_MISMATCH".into(),
                            severity: Severity::Warning,
                            title: "Node dependency drift".into(),
                            description: format!(
                                "Installed Node modules do not satisfy package.json: {}.",
                                details.join(", ")
                            ),
                            suggestion:
                                "Install or update dependencies to satisfy declared semantic versions.".into(),
                        });
                    }
                }
            }
            "postgres" => {
                let port_bound = node
                    .metadata
                    .get("port_bound")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let process_count = node
                    .metadata
                    .get("processes")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let installed_versions = node
                    .metadata
                    .get("installed_versions")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                if port_bound {
                    issues.push(Issue {
                        code: "POSTGRES_PORT_BOUND".into(),
                        severity: Severity::Warning,
                        title: "PostgreSQL port bound".into(),
                        description: "Port 5432 is currently bound.".into(),
                        suggestion: "Stop the conflicting PostgreSQL instance or update the port configuration.".into(),
                    });
                }
                if process_count > 1 {
                    issues.push(Issue {
                        code: "POSTGRES_MULTI_INSTANCE".into(),
                        severity: Severity::Warning,
                        title: "Multiple PostgreSQL processes".into(),
                        description: "More than one PostgreSQL process detected.".into(),
                        suggestion: "Consolidate to a single instance or ensure intentional multi-instance setup.".into(),
                    });
                }
                if installed_versions > 1 {
                    issues.push(Issue {
                        code: "POSTGRES_VERSION_DRIFT".into(),
                        severity: Severity::Warning,
                        title: "Multiple PostgreSQL versions installed".into(),
                        description: "Detected multiple PostgreSQL versions on the system.".into(),
                        suggestion: "Unify to a single supported PostgreSQL version.".into(),
                    });
                }
                if node.status != Status::Active {
                    issues.push(Issue {
                        code: "POSTGRES_INACTIVE".into(),
                        severity: Severity::Warning,
                        title: "PostgreSQL unavailable".into(),
                        description: "PostgreSQL was not detected during the scan.".into(),
                        suggestion: "Install or start PostgreSQL and verify psql is reachable."
                            .into(),
                    });
                }
            }
            "redis" => {
                let port_bound = node
                    .metadata
                    .get("port_bound")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let config_path = node.metadata.get("config_path").and_then(|v| v.as_str());
                let maxmemory = node
                    .metadata
                    .get("maxmemory")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if port_bound {
                    issues.push(Issue {
                        code: "REDIS_PORT_BOUND".into(),
                        severity: Severity::Warning,
                        title: "Redis port bound".into(),
                        description: "Port 6379 is currently bound.".into(),
                        suggestion:
                            "Stop the conflicting Redis instance or adjust the configured port."
                                .into(),
                    });
                }
                if config_path.is_none() {
                    issues.push(Issue {
                        code: "REDIS_CONFIG_MISSING".into(),
                        severity: Severity::Warning,
                        title: "redis.conf missing".into(),
                        description:
                            "Redis configuration file was not found in standard locations.".into(),
                        suggestion: "Create redis.conf under /etc/redis or /usr/local/etc/redis."
                            .into(),
                    });
                }
                if let Some(mem) = maxmemory {
                    let mem_lower = mem.to_lowercase();
                    let threshold_bytes = 256 * 1024 * 1024u64;
                    let parsed = if mem_lower.ends_with("mb") {
                        mem_lower
                            .trim_end_matches("mb")
                            .trim()
                            .parse::<u64>()
                            .ok()
                            .map(|m| m * 1024 * 1024)
                    } else if mem_lower.ends_with("gb") {
                        mem_lower
                            .trim_end_matches("gb")
                            .trim()
                            .parse::<u64>()
                            .ok()
                            .map(|g| g * 1024 * 1024 * 1024)
                    } else {
                        mem_lower.trim().parse::<u64>().ok()
                    };
                    if let Some(val) = parsed {
                        if val < threshold_bytes {
                            issues.push(Issue {
                                code: "REDIS_MEMORY_LOW".into(),
                                severity: Severity::Warning,
                                title: "Redis memory limit low".into(),
                                description: format!("Redis maxmemory is set to {} (<256MB).", mem),
                                suggestion:
                                    "Increase Redis maxmemory to at least 256MB for stability."
                                        .into(),
                            });
                        }
                    }
                }
                if node.status != Status::Active {
                    issues.push(Issue {
                        code: "REDIS_INACTIVE".into(),
                        severity: Severity::Warning,
                        title: "Redis unavailable".into(),
                        description: "Redis was not detected during the scan.".into(),
                        suggestion:
                            "Install or start Redis so redis-server or redis-cli are reachable."
                                .into(),
                    });
                }
            }
            "gpu" if node.status != Status::Active => {
                issues.push(Issue {
                    code: "GPU_MISSING".into(),
                    severity: Severity::Warning,
                    title: "GPU unavailable".into(),
                    description: "No GPU was detected via nvidia-smi.".into(),
                    suggestion:
                        "Install GPU drivers or ensure the GPU is accessible to this environment."
                            .into(),
                });
            }
            "gpu" => {
                let cuda_version = node.metadata.get("cuda_version").and_then(|v| v.as_str());
                let nvidia_smi = node.metadata.get("nvidia_smi").and_then(|v| v.as_str());
                let cudnn_version = node.metadata.get("cudnn_version").and_then(|v| v.as_str());
                if let (Some(cuda), Some(smi)) = (cuda_version, nvidia_smi) {
                    if let Some(smi_cuda) = smi.lines().find_map(|l| {
                        if l.contains("CUDA Version") {
                            l.split("CUDA Version:").nth(1).map(|s| {
                                s.trim().split_whitespace().next().unwrap_or("").to_string()
                            })
                        } else {
                            None
                        }
                    }) {
                        if !smi_cuda.is_empty() && !cuda.contains(&smi_cuda) {
                            issues.push(Issue {
                                code: "CUDA_VERSION_MISMATCH".into(),
                                severity: Severity::Warning,
                                title: "CUDA version mismatch".into(),
                                description: format!(
                                    "nvcc reports {} but nvidia-smi shows {}.",
                                    cuda, smi_cuda
                                ),
                                suggestion:
                                    "Align installed CUDA toolkit with driver-supported version."
                                        .into(),
                            });
                        }
                    }
                }
                if cudnn_version.is_none() && node.status == Status::Active {
                    issues.push(Issue {
                        code: "CUDNN_MISSING".into(),
                        severity: Severity::Warning,
                        title: "cuDNN missing".into(),
                        description: "No cuDNN headers found in common include paths.".into(),
                        suggestion: "Install cuDNN matching the installed CUDA toolkit.".into(),
                    });
                }
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
