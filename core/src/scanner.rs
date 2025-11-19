use crate::models::{Node, NodeType, Status, SystemState};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::Path;
use sysinfo::{System, SystemExt};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::OpenOptions;

#[cfg(windows)]
use std::net::TcpStream;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

fn docker_active_with_metadata() -> (Status, HashMap<String, serde_json::Value>) {
    #[cfg(unix)]
    {
        let path = "/var/run/docker.sock";
        let mut metadata = HashMap::new();
        metadata.insert("socket".to_string(), json!(path));
        if Path::new(path).exists() {
            match UnixStream::connect(path) {
                Ok(_) => (Status::Active, metadata),
                Err(e) => {
                    metadata.insert("error".to_string(), json!(e.to_string()));
                    (Status::Inactive, metadata)
                }
            }
        } else {
            (Status::Inactive, metadata)
        }
    }

    #[cfg(windows)]
    {
        let pipe_path = Path::new(r"\\.\\pipe\\docker_engine");
        let mut metadata = HashMap::new();
        metadata.insert("pipe".to_string(), json!(pipe_path.to_string_lossy()));
        if pipe_path.exists() {
            let mut options = OpenOptions::new();
            options.read(true).write(true);
            options.custom_flags(0);
            match options.open(pipe_path) {
                Ok(_) => (Status::Active, metadata),
                Err(e) => {
                    metadata.insert("error".to_string(), json!(e.to_string()));
                    (Status::Inactive, metadata)
                }
            }
        } else if TcpStream::connect("127.0.0.1:2375").is_ok() {
            metadata.insert("tcp".to_string(), json!("127.0.0.1:2375"));
            (Status::Active, metadata)
        } else {
            (Status::Inactive, metadata)
        }
    }
}

fn port_status(port: u16) -> Status {
    match TcpListener::bind(("0.0.0.0", port)) {
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

    let timestamp = Utc::now().to_rfc3339();
    let mut os_metadata = HashMap::new();
    os_metadata.insert("kernel".to_string(), json!(kernel.clone()));
    os_metadata.insert("timestamp".to_string(), json!(timestamp.clone()));
    os_metadata.insert(
        "auditor".to_string(),
        json!("Tessrax Governance Kernel v16"),
    );

    let mut nodes = vec![Node {
        id: "os".to_string(),
        node_type: NodeType::Os,
        label: os_name,
        status: Status::Active,
        metadata: os_metadata,
    }];

    let (docker_status, mut docker_metadata) = docker_active_with_metadata();
    docker_metadata.insert(
        "platform".to_string(),
        json!(std::env::consts::OS.to_string()),
    );
    nodes.push(Node {
        id: "docker".to_string(),
        node_type: NodeType::Service,
        label: "Docker Daemon".to_string(),
        status: docker_status,
        metadata: docker_metadata,
    });

    let mut port_metadata = HashMap::new();
    port_metadata.insert("protocol".to_string(), json!("tcp"));
    port_metadata.insert("port".to_string(), json!(8000));
    let port_status = port_status(8000);
    nodes.push(Node {
        id: "port8000".to_string(),
        node_type: NodeType::Port,
        label: "Port 8000".to_string(),
        status: port_status,
        metadata: port_metadata,
    });

    SystemState::new(nodes, Vec::new(), Vec::new(), timestamp)
}
