use std::fs;
use std::path::Path;
use std::process::Command;

pub trait SystemProvider: Send + Sync {
    fn file_exists(&self, path: &str) -> bool;
    fn read_file(&self, path: &str) -> Option<String>;
    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String>;
}

pub struct RealSystemProvider;

impl SystemProvider for RealSystemProvider {
    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(cmd).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
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
