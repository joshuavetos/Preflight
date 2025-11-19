use crate::models::SystemState;
use fs2::FileExt;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn write_state(path: &Path, state: &SystemState) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("json.tmp");
    let mut file = File::create(&tmp_path)?;
    file.lock_exclusive()?;
    let serialized = serde_json::to_string_pretty(state)
        .expect("serialization invariant: SystemState must be serializable");
    file.write_all(serialized.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn which(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .is_ok()
}

pub fn ok(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn warn(msg: &str) {
    println!("\x1b[33m{}\x1b[0m", msg);
}

pub fn err(msg: &str) {
    eprintln!("\x1b[31m{}\x1b[0m", msg);
}
