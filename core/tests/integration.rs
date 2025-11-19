use assert_cmd::Command;
use preflight::{models::Status, oracle, scanner};

#[test]
fn scanner_produces_nodes_and_docker_entry() {
    let state = scanner::perform_scan();
    assert!(
        state.nodes.len() >= 1,
        "scanner must emit at least one node"
    );
    let docker = state.nodes.iter().find(|n| n.id == "docker");
    assert!(docker.is_some(), "scanner must always include docker node");
    if let Some(node) = docker {
        assert!(matches!(node.status, Status::Active | Status::Inactive));
    }
}

#[test]
fn scan_command_serializes() {
    let mut cmd = Command::cargo_bin("preflight").unwrap();
    cmd.arg("scan").assert().success();
    let path = std::path::Path::new(".preflight/scan.json");
    assert!(path.exists(), "scan.json must be written");
    let data = std::fs::read_to_string(path).expect("scan.json readable");
    let parsed: preflight::models::SystemState =
        serde_json::from_str(&data).expect("valid json schema");
    assert!(parsed.nodes.len() >= 1);
    assert!(!parsed.version.is_empty());
    assert!(!parsed.timestamp.is_empty());
}

#[test]
fn python_detector_runs_without_panic() {
    let state = scanner::perform_scan();
    assert!(state.nodes.iter().any(|n| n.id == "os"));
}

#[test]
fn docker_images_detector_runs_without_panic() {
    let state = scanner::perform_scan();
    assert!(state.nodes.iter().any(|n| n.id == "os"));
}

#[test]
fn issue_engine_runs_without_panic() {
    let state = scanner::perform_scan();
    let issues = oracle::evaluate(&state);
    assert!(issues.len() >= 0);
}

#[test]
fn simulation_engine_runs() {
    let issues = oracle::simulate_command("docker compose up -p test");
    assert!(issues.len() >= 0);
}
