use crate::models::{SystemState, DETERMINISTIC_TIMESTAMP};
use fs2::FileExt;
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeMap;
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
    let canonical = sort_json(serde_json::to_value(state).expect("state serializable"));
    let serialized = serde_json::to_string_pretty(&canonical)
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

pub fn json_envelope(command: &str, status: &str, data: Value) -> Value {
    json!({
        "command": command,
        "status": status,
        "timestamp": DETERMINISTIC_TIMESTAMP,
        "data": data
    })
}

fn sort_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut ordered = BTreeMap::new();
            for (k, v) in map {
                ordered.insert(k, sort_json(v));
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in ordered {
                new_map.insert(k, v);
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_json).collect()),
        other => other,
    }
}
