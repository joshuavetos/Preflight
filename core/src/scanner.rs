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
    } else {
        nodes.push(Node {
            id: "python".into(),
            node_type: NodeType::Runtime,
            label: "Python".into(),
            status: Status::Inactive,
            metadata: HashMap::new(),
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

fn detect_nodejs<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let node_version = provider.command_output("node", &["--version"]);
    let npm_version = provider.command_output("npm", &["--version"]);
    let mut metadata = HashMap::new();

    if let Some(v) = &node_version {
        metadata.insert("version".into(), json!(v));
    }
    if let Some(v) = &npm_version {
        metadata.insert("npm".into(), json!(v));
    }

    let status = if node_version.is_some() {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "nodejs".into(),
        node_type: NodeType::Runtime,
        label: "Node.js".into(),
        status,
        metadata,
    });
}

fn detect_postgres<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let port_status = check_port(5432);
    let version = provider.command_output("psql", &["--version"]);
    let mut metadata = HashMap::new();
    metadata.insert("port".into(), json!(5432));
    metadata.insert(
        "port_bound".into(),
        json!(matches!(port_status, Status::Active)),
    );
    if let Some(v) = &version {
        metadata.insert("version".into(), json!(v));
    }

    let status = if version.is_some() || matches!(port_status, Status::Active) {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "postgres".into(),
        node_type: NodeType::Postgres,
        label: "PostgreSQL".into(),
        status,
        metadata,
    });
}

fn detect_redis<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let port_status = check_port(6379);
    let version = provider.command_output("redis-server", &["--version"]);
    let mut metadata = HashMap::new();
    metadata.insert("port".into(), json!(6379));
    metadata.insert(
        "port_bound".into(),
        json!(matches!(port_status, Status::Active)),
    );
    if let Some(v) = &version {
        metadata.insert("version".into(), json!(v));
    }

    let status = if version.is_some() || matches!(port_status, Status::Active) {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "redis".into(),
        node_type: NodeType::Redis,
        label: "Redis".into(),
        status,
        metadata,
    });
}

fn detect_gpu<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let gpu_info = provider.command_output("nvidia-smi", &[]);
    let mut metadata = HashMap::new();
    if let Some(info) = &gpu_info {
        metadata.insert("nvidia_smi".into(), json!(info));
    }

    let status = if gpu_info.is_some() {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "gpu".into(),
        node_type: NodeType::Gpu,
        label: "GPU".into(),
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
    detect_nodejs(&provider, &mut nodes);
    detect_postgres(&provider, &mut nodes);
    detect_redis(&provider, &mut nodes);
    detect_gpu(&provider, &mut nodes);
    detect_port_8000(&mut nodes);

    SystemState::new(nodes, Vec::new(), Vec::new(), timestamp)
}
