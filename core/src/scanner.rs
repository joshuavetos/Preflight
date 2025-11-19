use crate::models::{Node, NodeType, Status, SystemState};
use chrono::Utc;
use std::collections::HashMap;
use std::net::TcpListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use sysinfo::{System, SystemExt};

fn docker_active() -> Status {
    let path = "/var/run/docker.sock";
    if Path::new(path).exists() {
        match UnixStream::connect(path) {
            Ok(_) => Status::Active,
            Err(_) => Status::Inactive,
        }
    } else {
        Status::Inactive
    }
}

fn port_8000_status() -> Status {
    match TcpListener::bind(("0.0.0.0", 8000)) {
        Ok(_) => Status::Inactive,
        Err(_) => Status::Active,
    }
}

pub fn perform_scan() -> SystemState {
    let mut sys = System::new();
    sys.refresh_system();
    let os_name = sys.name().unwrap_or_else(|| "Unknown".to_string());
    let kernel = sys
        .kernel_version()
        .unwrap_or_else(|| "unknown".to_string());

    let mut os_metadata = HashMap::new();
    os_metadata.insert("kernel".to_string(), kernel.clone());
    os_metadata.insert("timestamp".to_string(), Utc::now().to_rfc3339());
    os_metadata.insert(
        "auditor".to_string(),
        "Tessrax Governance Kernel v16".to_string(),
    );

    let mut nodes = vec![Node {
        id: "os".to_string(),
        node_type: NodeType::OS,
        label: os_name,
        status: Status::Active,
        metadata: os_metadata,
    }];

    let mut docker_metadata = HashMap::new();
    docker_metadata.insert("socket".to_string(), "/var/run/docker.sock".to_string());
    let docker_status = docker_active();
    nodes.push(Node {
        id: "docker".to_string(),
        node_type: NodeType::Service,
        label: "Docker Daemon".to_string(),
        status: docker_status,
        metadata: docker_metadata,
    });

    let mut port_metadata = HashMap::new();
    port_metadata.insert("protocol".to_string(), "tcp".to_string());
    let port_status = port_8000_status();
    nodes.push(Node {
        id: "port8000".to_string(),
        node_type: NodeType::Port,
        label: "Port 8000".to_string(),
        status: port_status,
        metadata: port_metadata,
    });

    SystemState {
        nodes,
        edges: Vec::new(),
        issues: Vec::new(),
    }
}
