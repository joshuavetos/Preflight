use crate::models::SystemState;
use crate::utils;
use std::fs;
use std::path::PathBuf;

fn snapshot_dir() -> PathBuf {
    PathBuf::from(".preflight/snapshots")
}

pub fn save(name: &str) -> Result<(), String> {
    let data = fs::read_to_string(".preflight/scan.json")
        .map_err(|e| format!("Unable to read current scan: {e}"))?;
    let state: SystemState =
        serde_json::from_str(&data).map_err(|e| format!("Invalid scan data: {e}"))?;
    let dir = snapshot_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Unable to create snapshot dir: {e}"))?;
    let path = dir.join(format!("{}.json", name));
    utils::write_state(&path, &state).map_err(|e| format!("Unable to save snapshot: {e}"))?;
    println!("Snapshot '{}' saved.", name);
    Ok(())
}

pub fn restore(name: &str) -> Result<(), String> {
    let path = snapshot_dir().join(format!("{}.json", name));
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Unable to read snapshot {}: {}", name, e))?;
    let state: SystemState =
        serde_json::from_str(&data).map_err(|e| format!("Invalid snapshot data: {e}"))?;
    utils::write_state(&PathBuf::from(".preflight/scan.json"), &state)
        .map_err(|e| format!("Unable to restore snapshot: {e}"))?;
    println!("Snapshot '{}' restored to .preflight/scan.json.", name);
    Ok(())
}
