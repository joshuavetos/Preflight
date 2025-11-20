use crate::models::SystemState;
use crate::utils::json_envelope;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Clone)]
pub struct FixCommand {
    pub code: String,
    pub command: String,
}

pub fn load_state() -> Result<SystemState, String> {
    let raw = fs::read_to_string(".preflight/scan.json")
        .map_err(|e| format!("Unable to read scan.json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid scan.json: {e}"))
}

pub fn commands() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("DOCKER_INACTIVE", "sudo systemctl start docker"),
        ("PORT_8000_BOUND", "sudo fuser -k 8000/tcp"),
        (
            "NODEJS_INACTIVE",
            "sudo apt-get update && sudo apt-get install -y nodejs npm",
        ),
        ("POSTGRES_PORT_BOUND", "sudo systemctl restart postgresql"),
        ("POSTGRES_INACTIVE", "sudo systemctl start postgresql"),
        (
            "POSTGRES_MULTI_INSTANCE",
            "sudo systemctl list-units | grep postgresql && sudo systemctl stop postgresql@*",
        ),
        (
            "POSTGRES_VERSION_DRIFT",
            "sudo apt-get autoremove 'postgresql-*'",
        ),
        ("REDIS_PORT_BOUND", "sudo systemctl restart redis-server"),
        ("REDIS_INACTIVE", "sudo systemctl start redis-server"),
        (
            "REDIS_MEMORY_LOW",
            "redis-cli CONFIG SET maxmemory 268435456",
        ),
        (
            "REDIS_CONFIG_MISSING",
            "sudo cp /etc/redis/redis.conf.default /etc/redis/redis.conf",
        ),
        ("GPU_MISSING", "sudo apt-get install -y nvidia-driver-535"),
        (
            "CUDA_VERSION_MISMATCH",
            "sudo apt-get install -y cuda-toolkit",
        ),
        ("CUDNN_MISSING", "sudo apt-get install -y libcudnn8"),
        ("PYTHON_MULTIPLE_ENV", "conda deactivate && deactivate"),
        (
            "PYTHON_NO_ENV",
            "python -m venv .venv && source .venv/bin/activate",
        ),
        (
            "PYTHON_VERSION_DRIFT",
            "sudo update-alternatives --config python",
        ),
        ("NODE_PACKAGE_MISSING", "npm init -y"),
        ("NODE_LOCKFILE_DRIFT", "npm install"),
    ])
}

pub fn run(json_output: bool) -> Result<(), String> {
    let state = load_state()?;
    let fixes = commands();
    let mut rendered: Vec<FixCommand> = Vec::new();
    println!("Suggested fixes ({} issues):", state.issues.len());
    for issue in state.issues {
        if let Some(cmd) = fixes.get(issue.code.as_str()) {
            rendered.push(FixCommand {
                code: issue.code.clone(),
                command: cmd.to_string(),
            });
            println!("- {}: {}", issue.code, cmd);
        } else {
            rendered.push(FixCommand {
                code: issue.code.clone(),
                command: issue.suggestion.clone(),
            });
            println!("- {}: {}", issue.code, issue.suggestion);
        }
    }

    if json_output {
        let payload = json_envelope(
            "fix",
            "ok",
            json!({
                "fixes": rendered
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
    Ok(())
}
