use crate::graph;
use crate::models::SystemState;
use crate::oracle;
use crate::scanner;
use crate::schema;
use crate::system_provider::SystemProvider;
use crate::utils;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct SshSystemProvider {
    target: String,
}

impl SshSystemProvider {
    fn new(url: &str) -> Result<Self, String> {
        let target = url.trim_start_matches("ssh://").to_string();
        if target.is_empty() {
            return Err("Invalid remote target".into());
        }
        Ok(SshSystemProvider { target })
    }

    fn ssh_command(&self, command: &str) -> Option<String> {
        let output = Command::new("ssh")
            .arg(&self.target)
            .arg(command)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            }
        } else {
            Some(stdout)
        }
    }
}

impl SystemProvider for SshSystemProvider {
    fn file_exists(&self, path: &str) -> bool {
        self.ssh_command(&format!("test -e '{}' && echo ok", path))
            .is_some()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        self.ssh_command(&format!("cat '{}'", path))
    }

    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String> {
        let joined = args.join(" ");
        let command = if joined.is_empty() {
            cmd.to_string()
        } else {
            format!("{} {}", cmd, joined)
        };
        self.ssh_command(&command)
    }

    fn list_dir(&self, path: &str) -> Option<Vec<String>> {
        self.ssh_command(&format!("ls -1 {}", path))
            .map(|out| out.lines().map(|s| s.to_string()).collect())
    }

    fn modification_time(&self, path: &str) -> Option<SystemTime> {
        self.ssh_command(&format!("stat -c %Y {}", path))
            .and_then(|out| {
                out.trim()
                    .parse::<u64>()
                    .ok()
                    .map(|secs| UNIX_EPOCH + Duration::from_secs(secs))
            })
    }
}

pub fn remote_scan(remote: &str) -> Result<SystemState, String> {
    let provider = SshSystemProvider::new(remote)?;
    let mut state = scanner::perform_scan_with_provider(&provider);
    if let Some(os_name) = provider.command_output("uname", &["-s"]) {
        if let Some(os_node) = state.nodes.iter_mut().find(|n| n.id == "os") {
            os_node.label = os_name;
        }
    }
    graph::derive_edges(&mut state);
    state.issues = oracle::evaluate(&state);
    state.refresh_fingerprint();
    state.assert_contract();
    schema::validate_against_contract(&state)
        .map_err(|e| format!("Schema validation failed: {e}"))?;
    let path = std::path::PathBuf::from(".preflight/scan.json");
    utils::write_state(&path, &state).map_err(|e| format!("Failed to write scan: {}", e))?;
    Ok(state)
}
