use crate::fix;
use crate::models::SystemState;
use crate::utils::json_envelope;
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct DockerSpec {
    pub required_api: Option<String>,
    pub required_compose: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeSpec {
    pub min_version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GpuSpec {
    pub allow_amd: Option<bool>,
    pub allow_intel: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EnvSpec {
    pub docker: Option<DockerSpec>,
    pub node: Option<NodeSpec>,
    pub gpu: Option<GpuSpec>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnvViolation {
    pub r#type: String,
    pub expected: String,
    pub actual: String,
}

fn parse_spec() -> Result<EnvSpec, String> {
    let raw = fs::read_to_string(".preflight.yml")
        .map_err(|e| format!("Failed to read .preflight.yml: {e}"))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("Invalid YAML: {e}"))
}

fn find_node(state: &SystemState, id: &str) -> Option<&crate::models::Node> {
    state.nodes.iter().find(|n| n.id == id)
}

fn parse_version(raw: &str) -> Option<Version> {
    let cleaned = raw.trim().trim_start_matches('v');
    Version::parse(cleaned).ok()
}

fn compare_versions(requirement: &str, actual: &str) -> bool {
    if let (Ok(req), Some(actual_ver)) = (VersionReq::parse(requirement), parse_version(actual)) {
        req.matches(&actual_ver)
    } else {
        false
    }
}

fn validate_docker(spec: &DockerSpec, state: &SystemState, violations: &mut Vec<EnvViolation>) {
    if let Some(docker) = find_node(state, "docker") {
        if let Some(required_api) = &spec.required_api {
            let actual = docker
                .metadata
                .get("docker_api_version")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if actual.is_empty() || !compare_versions(required_api, actual) {
                violations.push(EnvViolation {
                    r#type: "docker_api".into(),
                    expected: required_api.clone(),
                    actual: actual.to_string(),
                });
            }
        }

        if let Some(required_compose) = &spec.required_compose {
            let actual = docker
                .metadata
                .get("compose_version")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if actual.is_empty() || !compare_versions(required_compose, actual) {
                violations.push(EnvViolation {
                    r#type: "docker_compose".into(),
                    expected: required_compose.clone(),
                    actual: actual.to_string(),
                });
            }
        }
    } else {
        if spec.required_api.is_some() {
            violations.push(EnvViolation {
                r#type: "docker_api".into(),
                expected: spec.required_api.clone().unwrap_or_default(),
                actual: "missing".into(),
            });
        }
        if spec.required_compose.is_some() {
            violations.push(EnvViolation {
                r#type: "docker_compose".into(),
                expected: spec.required_compose.clone().unwrap_or_default(),
                actual: "missing".into(),
            });
        }
    }
}

fn validate_node(spec: &NodeSpec, state: &SystemState, violations: &mut Vec<EnvViolation>) {
    if let Some(min_version) = &spec.min_version {
        let actual = find_node(state, "nodejs")
            .and_then(|n| n.metadata.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if actual.is_empty() || !compare_versions(min_version, actual) {
            violations.push(EnvViolation {
                r#type: "node_version".into(),
                expected: min_version.clone(),
                actual: actual.to_string(),
            });
        }
    }
}

fn validate_gpu(spec: &GpuSpec, state: &SystemState, violations: &mut Vec<EnvViolation>) {
    let node = find_node(state, "gpu");
    if let Some(allow_amd) = spec.allow_amd {
        let amd_detected = node
            .and_then(|n| n.metadata.get("amd_gpu_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if amd_detected && !allow_amd {
            violations.push(EnvViolation {
                r#type: "gpu_vendor".into(),
                expected: "amd disallowed".into(),
                actual: "amd detected".into(),
            });
        }
    }

    if let Some(allow_intel) = spec.allow_intel {
        let intel_detected = node
            .and_then(|n| n.metadata.get("intel_gpu_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if intel_detected && !allow_intel {
            violations.push(EnvViolation {
                r#type: "gpu_vendor".into(),
                expected: "intel disallowed".into(),
                actual: "intel detected".into(),
            });
        }
    }
}

fn evaluate(spec: &EnvSpec, state: &SystemState) -> Vec<EnvViolation> {
    let mut violations = Vec::new();
    if let Some(docker) = &spec.docker {
        validate_docker(docker, state, &mut violations);
    }
    if let Some(node) = &spec.node {
        validate_node(node, state, &mut violations);
    }
    if let Some(gpu) = &spec.gpu {
        validate_gpu(gpu, state, &mut violations);
    }
    violations
}

pub fn run(json_output: bool) -> Result<i32, String> {
    let spec = parse_spec()?;
    let state = fix::load_state()?;
    let violations = evaluate(&spec, &state);
    let status = if violations.is_empty() {
        "ok"
    } else {
        "violation"
    };

    if json_output {
        let payload = json_envelope(
            "validate-env",
            status,
            json!({
                "spec_file": ".preflight.yml",
                "violations": violations
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else if violations.is_empty() {
        println!("Environment matches .preflight.yml");
    } else {
        println!("Environment violations ({}):", violations.len());
        for v in &violations {
            println!("- {} expected {} got {}", v.r#type, v.expected, v.actual);
        }
    }

    Ok(if violations.is_empty() { 0 } else { 1 })
}

pub fn write_json() -> Result<String, String> {
    let spec = parse_spec()?;
    let state = fix::load_state()?;
    let violations = evaluate(&spec, &state);
    let status = if violations.is_empty() {
        "ok"
    } else {
        "violation"
    };
    let payload = json_envelope(
        "validate-env",
        status,
        json!({
            "spec_file": ".preflight.yml",
            "violations": violations
        }),
    );
    let rendered = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(".preflight").map_err(|e| e.to_string())?;
    let path = ".preflight/validate_env.json";
    std::fs::write(path, &rendered).map_err(|e| e.to_string())?;
    Ok(path.to_string())
}
