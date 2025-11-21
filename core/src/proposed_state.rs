use crate::command_ast::ParsedCommand;
use crate::models::{Status, SystemState};
use serde_json::json;

/// Deep clone of SystemState
pub fn clone_state(original: &SystemState) -> SystemState {
    let mut cloned = SystemState {
        nodes: original.nodes.clone(),
        edges: original.edges.clone(),
        issues: original.issues.clone(),
        version: original.version.clone(),
        timestamp: original.timestamp.clone(),
        fingerprint: original.fingerprint.clone(),
    };

    cloned.refresh_fingerprint();
    cloned
}

/// Apply "predicted changes" based on ParsedCommand.
/// This **never** mutates the original state.
/// This logic is intentionally minimal — safe extensions can follow.
pub fn apply_predicted_changes(mut proposed: SystemState, parsed: &ParsedCommand) -> SystemState {
    // Port binding simulation
    for p in &parsed.ports {
        if *p == 8000 {
            if let Some(node) = proposed.nodes.iter_mut().find(|n| n.id == "port8000") {
                node.status = Status::Active;
                node.metadata
                    .insert("predicted_bind".to_string(), json!(true));
            }
        }
    }

    // Docker compose → assume "docker" will be required
    if parsed.docker_compose || parsed.docker_run || parsed.docker_build {
        if let Some(docker) = proposed.nodes.iter_mut().find(|n| n.id == "docker") {
            docker.status = Status::Active;
            docker
                .metadata
                .insert("predict_used".into(), json!(parsed.raw.clone()));
        }
    }

    proposed
}
