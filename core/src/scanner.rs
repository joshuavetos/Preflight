use crate::models::{Node, NodeType, Status, SystemState};
use crate::system_provider::{RealSystemProvider, SystemProvider};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::net::TcpListener;

fn check_port(port: u16) -> Status {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(_) => Status::Inactive,
        Err(_) => Status::Active,
    }
}

fn detect_python<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    if let Some(version) = provider.command_output("python", &["--version"]) {
        let mut metadata = HashMap::new();
        metadata.insert("version".into(), json!(version));

        nodes.push(Node {
            id: "python".into(),
            node_type: NodeType::Runtime,
            label: "Python".into(),
            status: Status::Active,
            metadata,
        });
    }
}

fn detect_docker<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let mut metadata = HashMap::new();
    let socket_path = "/var/run/docker.sock";
    metadata.insert("socket".into(), json!(socket_path));

    let docker_ok =
        provider.file_exists(socket_path) || provider.command_output("docker", &["info"]).is_some();

    let status = if docker_ok {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "docker".into(),
        node_type: NodeType::Service,
        label: "Docker Daemon".into(),
        status,
        metadata,
    });
}

fn detect_port_8000(nodes: &mut Vec<Node>) {
    let mut metadata = HashMap::new();
    metadata.insert("protocol".into(), json!("tcp"));
    metadata.insert("port".into(), json!(8000));

    nodes.push(Node {
        id: "port8000".into(),
        node_type: NodeType::Port,
        label: "Port 8000".into(),
        status: check_port(8000),
        metadata,
    });
}

pub fn perform_scan() -> SystemState {
    let provider = RealSystemProvider;

    let timestamp = Utc::now().to_rfc3339();
    let mut nodes = vec![Node {
        id: "os".into(),
        node_type: NodeType::Os,
        label: std::env::consts::OS.into(),
        status: Status::Active,
        metadata: HashMap::new(),
    }];

    detect_docker(&provider, &mut nodes);
    detect_python(&provider, &mut nodes);
    detect_port_8000(&mut nodes);

    SystemState::new(nodes, Vec::new(), Vec::new(), timestamp)
}
