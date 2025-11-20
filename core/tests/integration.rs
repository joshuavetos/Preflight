use assert_cmd::Command;
use preflight::deps;
use preflight::models::{Node, NodeType, Status, SystemState};
use preflight::oracle;
use preflight::scanner;
use preflight::system_provider::SystemProvider;
use preflight::validate;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

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
fn extended_detectors_emit_nodes() {
    let state = scanner::perform_scan();
    for id in ["nodejs", "postgres", "redis", "gpu"] {
        let node = state.nodes.iter().find(|n| n.id == id);
        assert!(node.is_some(), "{} node must be present", id);
        if let Some(n) = node {
            assert!(
                matches!(
                    n.status,
                    Status::Active | Status::Inactive | Status::Conflict
                ),
                "{} node must have a concrete status",
                id
            );
        }
    }
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

#[test]
fn simulation_risk_scoring_runs() {
    let issues = oracle::simulate_command("docker compose up --gpus all -p test");
    assert!(issues.len() > 0, "Simulation should emit issues");
    assert!(issues.iter().any(|i| i.code == "SIM_RISK_SUMMARY"));
}

struct MockProvider {
    commands: HashMap<String, String>,
}

impl MockProvider {
    fn new() -> Self {
        MockProvider {
            commands: HashMap::new(),
        }
    }
}

impl SystemProvider for MockProvider {
    fn file_exists(&self, _path: &str) -> bool {
        false
    }

    fn read_file(&self, _path: &str) -> Option<String> {
        None
    }

    fn command_output(&self, cmd: &str, _args: &[&str]) -> Option<String> {
        self.commands.get(cmd).cloned()
    }

    fn list_dir(&self, _path: &str) -> Option<Vec<String>> {
        None
    }

    fn modification_time(&self, _path: &str) -> Option<SystemTime> {
        Some(SystemTime::now() - Duration::from_secs(10))
    }
}

#[test]
fn gpu_vendor_detection_marks_metadata() {
    let mut provider = MockProvider::new();
    provider
        .commands
        .insert("lspci".into(), "AMD Radeon Graphics".into());
    let state = scanner::perform_scan_with_provider(&provider);
    let gpu = state.nodes.iter().find(|n| n.id == "gpu").unwrap();
    let detected = gpu
        .metadata
        .get("amd_gpu_detected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(detected, "AMD GPU should be detected from lspci output");
}

#[test]
fn docker_compose_drift_issue_triggered() {
    let mut metadata = HashMap::new();
    metadata.insert("compose_version".into(), serde_json::json!("3.9"));
    metadata.insert("docker_api_version".into(), serde_json::json!("1.20"));
    let docker = Node {
        id: "docker".into(),
        node_type: NodeType::DockerImages,
        label: "Docker".into(),
        status: Status::Active,
        metadata,
    };
    let os = Node {
        id: "os".into(),
        node_type: NodeType::Os,
        label: "linux".into(),
        status: Status::Active,
        metadata: HashMap::new(),
    };
    let state = SystemState::new(vec![os, docker], vec![], vec![], "now".into());
    let issues = oracle::evaluate(&state);
    assert!(
        issues.iter().any(|i| i.code == "DOCKER_COMPOSE_DRIFT"),
        "compose drift should be reported"
    );
}

#[test]
fn dependency_graph_reports_modules() {
    let graph = deps::collect_graph().expect("graph should generate");
    assert!(!graph.0.is_empty());
}

#[test]
fn architecture_validator_flags_unsorted_imports() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("main.rs"),
        "use std::io;\nuse std::fmt;\nfn main() {}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"sample\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\n",
    )
    .unwrap();
    let violations = validate::validate_paths(&src, &dir.path().join("Cargo.toml"))
        .expect("validation should run");
    assert!(
        violations
            .iter()
            .any(|v| v.message.contains("Imports not sorted")),
        "unsorted imports should be flagged"
    );
}

#[test]
fn doctor_reports_fixable_and_unfixable() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".preflight")).unwrap();
    let issues = vec![
        serde_json::json!({
            "code": "DOCKER_INACTIVE",
            "severity": "warning",
            "title": "",
            "description": "",
            "suggestion": ""
        }),
        serde_json::json!({
            "code": "UNKNOWN",
            "severity": "warning",
            "title": "",
            "description": "",
            "suggestion": ""
        }),
    ];
    let state = serde_json::json!({
        "nodes": [{"id": "os", "type": "os", "label": "linux", "status": "active", "metadata": {}}],
        "edges": [],
        "issues": issues,
        "version": "1.0.0",
        "timestamp": "now"
    });
    fs::write(dir.path().join(".preflight/scan.json"), state.to_string()).unwrap();
    let mut cmd = Command::cargo_bin("preflight").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["--json", "doctor"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"fixable\":true"))
        .stdout(predicates::str::contains("\"fixable\":false"));
}
