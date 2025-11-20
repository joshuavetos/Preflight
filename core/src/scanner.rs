use crate::models::{Node, NodeType, Status, SystemState};
use crate::system_provider::{RealSystemProvider, SystemProvider};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::env;

fn check_port<P: SystemProvider>(provider: &P, port: u16) -> Status {
    let probe = format!(
        "ss -ltn sport = :{0} || (netstat -ltn 2>/dev/null | grep :{0})",
        port
    );
    if let Some(output) = provider.command_output("sh", &["-c", &probe]) {
        if !output.trim().is_empty() {
            return Status::Active;
        }
    }
    match std::net::TcpListener::bind(("0.0.0.0", port)) {
        Ok(_) => Status::Inactive,
        Err(_) => Status::Active,
    }
}

fn detect_python<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let version = provider.command_output("python", &["--version"]);
    let version3 = provider.command_output("python3", &["--version"]);
    let venv_active = env::var("VIRTUAL_ENV").is_ok();
    let pipenv_active = env::var("PIPENV_ACTIVE").is_ok();
    let poetry_active = env::var("POETRY_ACTIVE").is_ok();
    let conda_active = env::var("CONDA_DEFAULT_ENV").is_ok() || env::var("CONDA_PREFIX").is_ok();

    let mut metadata = HashMap::new();
    if let Some(v) = &version {
        metadata.insert("version".into(), json!(v));
    }
    if let Some(v) = &version3 {
        metadata.insert("python3_version".into(), json!(v));
    }
    metadata.insert("venv".into(), json!(venv_active));
    metadata.insert("pipenv".into(), json!(pipenv_active));
    metadata.insert("poetry".into(), json!(poetry_active));
    metadata.insert("conda".into(), json!(conda_active));

    let status = if version.is_some() {
        Status::Active
    } else {
        Status::Inactive
    };

    nodes.push(Node {
        id: "python".into(),
        node_type: NodeType::Runtime,
        label: "Python".into(),
        status,
        metadata,
    });
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

    let package_json_present = provider.file_exists("package.json");
    let node_modules_exists = provider.file_exists("node_modules");
    let package_lock_path = "package-lock.json";
    let mut lockfile_drift = false;
    if package_json_present && provider.file_exists(package_lock_path) {
        if let (Some(pkg_time), Some(lock_time)) = (
            provider.modification_time("package.json"),
            provider.modification_time(package_lock_path),
        ) {
            lockfile_drift = pkg_time > lock_time;
        }
    }

    metadata.insert("package_json_present".into(), json!(package_json_present));
    metadata.insert(
        "node_modules_mismatch".into(),
        json!(package_json_present && !node_modules_exists),
    );
    metadata.insert("lockfile_drift".into(), json!(lockfile_drift));

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
    let port_status = check_port(provider, 5432);
    let version = provider.command_output("psql", &["--version"]);
    let pg_processes = provider
        .command_output("ps", &["aux"])
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains("postgres"))
        .map(|l| l.to_string())
        .collect::<Vec<_>>();
    let versions = provider.list_dir("/usr/lib/postgresql").unwrap_or_default();
    let mut metadata = HashMap::new();
    metadata.insert("port".into(), json!(5432));
    metadata.insert(
        "port_bound".into(),
        json!(matches!(port_status, Status::Active)),
    );
    if let Some(v) = &version {
        metadata.insert("version".into(), json!(v));
    }
    metadata.insert("processes".into(), json!(pg_processes));
    metadata.insert("installed_versions".into(), json!(versions));

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
    let port_status = check_port(provider, 6379);
    let version = provider.command_output("redis-server", &["--version"]);
    let config_path_candidates = vec!["/etc/redis/redis.conf", "/usr/local/etc/redis/redis.conf"];
    let config_path = config_path_candidates
        .iter()
        .find(|p| provider.file_exists(p))
        .map(|p| p.to_string());
    let mut memory_limit: Option<String> = None;
    if let Some(conf) = &config_path {
        if let Some(content) = provider.read_file(conf) {
            for line in content.lines() {
                if line.trim_start().starts_with("maxmemory") {
                    memory_limit = line.split_whitespace().nth(1).map(|s| s.to_string());
                    break;
                }
            }
        }
    }
    let mut metadata = HashMap::new();
    metadata.insert("port".into(), json!(6379));
    metadata.insert(
        "port_bound".into(),
        json!(matches!(port_status, Status::Active)),
    );
    if let Some(v) = &version {
        metadata.insert("version".into(), json!(v));
    }
    metadata.insert("config_path".into(), json!(config_path));
    metadata.insert("maxmemory".into(), json!(memory_limit));

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
    let cuda_version = provider.command_output("nvcc", &["--version"]);
    let cudnn_version = [
        "/usr/include/cudnn_version.h",
        "/usr/local/cuda/include/cudnn_version.h",
    ]
    .iter()
    .find_map(|p| {
        provider.read_file(p).and_then(|contents| {
            contents
                .lines()
                .find(|l| l.contains("CUDNN_MAJOR"))
                .and_then(|major| {
                    let major_num = major.split_whitespace().last()?.to_string();
                    let minor = contents
                        .lines()
                        .find(|l| l.contains("CUDNN_MINOR"))
                        .and_then(|l| l.split_whitespace().last())
                        .unwrap_or("0");
                    let patch = contents
                        .lines()
                        .find(|l| l.contains("CUDNN_PATCHLEVEL"))
                        .and_then(|l| l.split_whitespace().last())
                        .unwrap_or("0");
                    Some(format!("{}.{}.{}", major_num, minor, patch))
                })
        })
    });
    let mut metadata = HashMap::new();
    if let Some(info) = &gpu_info {
        metadata.insert("nvidia_smi".into(), json!(info));
    }
    if let Some(cuda) = &cuda_version {
        metadata.insert("cuda_version".into(), json!(cuda));
    }
    if let Some(cudnn) = &cudnn_version {
        metadata.insert("cudnn_version".into(), json!(cudnn));
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

fn detect_port_8000<P: SystemProvider>(provider: &P, nodes: &mut Vec<Node>) {
    let mut metadata = HashMap::new();
    metadata.insert("protocol".into(), json!("tcp"));
    metadata.insert("port".into(), json!(8000));

    nodes.push(Node {
        id: "port8000".into(),
        node_type: NodeType::Port,
        label: "Port 8000".into(),
        status: check_port(provider, 8000),
        metadata,
    });
}

pub fn perform_scan() -> SystemState {
    let provider = RealSystemProvider;
    perform_scan_with_provider(&provider)
}

pub fn perform_scan_with_provider<P: SystemProvider>(provider: &P) -> SystemState {
    let timestamp = Utc::now().to_rfc3339();
    let mut nodes = vec![Node {
        id: "os".into(),
        node_type: NodeType::Os,
        label: std::env::consts::OS.into(),
        status: Status::Active,
        metadata: HashMap::new(),
    }];

    detect_docker(provider, &mut nodes);
    detect_python(provider, &mut nodes);
    detect_nodejs(provider, &mut nodes);
    detect_postgres(provider, &mut nodes);
    detect_redis(provider, &mut nodes);
    detect_gpu(provider, &mut nodes);
    detect_port_8000(provider, &mut nodes);

    SystemState::new(nodes, Vec::new(), Vec::new(), timestamp)
}
