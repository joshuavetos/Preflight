use crate::models::SystemState;
use crate::utils;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const HISTORY_DIR: &str = ".preflight/history";
const MAX_HISTORY: usize = 10;

fn history_dir() -> PathBuf {
    PathBuf::from(HISTORY_DIR)
}

pub fn record_scan(state: &SystemState) -> Result<(), String> {
    let dir = history_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Unable to create history dir: {e}"))?;
    let filename = format!("scan-{}.json", state.timestamp.replace(':', "-"));
    let path = dir.join(filename);
    utils::write_state(&path, state).map_err(|e| format!("Unable to persist history: {e}"))?;
    prune_history()?;
    Ok(())
}

fn prune_history() -> Result<(), String> {
    let dir = history_dir();
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("Unable to list history: {e}"))?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
    while entries.len() > MAX_HISTORY {
        if let Some(entry) = entries.first() {
            let _ = fs::remove_file(entry.path());
        }
        entries.remove(0);
    }
    Ok(())
}

fn load_scans() -> Result<Vec<SystemState>, String> {
    if !history_dir().exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(history_dir())
        .map_err(|e| format!("Unable to read history: {e}"))?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
    let mut states = Vec::new();
    for entry in entries {
        let data = fs::read_to_string(entry.path())
            .map_err(|e| format!("Unable to read history file: {e}"))?;
        if let Ok(state) = serde_json::from_str::<SystemState>(&data) {
            states.push(state);
        }
    }
    Ok(states)
}

pub fn diff_latest() -> Result<(), String> {
    let states = load_scans()?;
    if states.len() < 2 {
        println!("Not enough history to diff. Run more scans.");
        return Ok(());
    }
    let prev = &states[states.len() - 2];
    let current = &states[states.len() - 1];

    let prev_nodes: HashMap<_, _> = prev.nodes.iter().map(|n| (&n.id, n)).collect();
    let curr_nodes: HashMap<_, _> = current.nodes.iter().map(|n| (&n.id, n)).collect();

    let mut added_nodes = Vec::new();
    let mut removed_nodes = Vec::new();
    let mut changed_nodes = Vec::new();

    for id in curr_nodes.keys() {
        if !prev_nodes.contains_key(id) {
            added_nodes.push(*id);
        }
    }
    for id in prev_nodes.keys() {
        if !curr_nodes.contains_key(id) {
            removed_nodes.push(*id);
        }
    }
    for (id, node) in &curr_nodes {
        if let Some(prev_node) = prev_nodes.get(id) {
            if *prev_node != node {
                changed_nodes.push(*id);
            }
        }
    }

    let prev_issues: Vec<_> = prev.issues.iter().map(|i| i.code.as_str()).collect();
    let curr_issues: Vec<_> = current.issues.iter().map(|i| i.code.as_str()).collect();

    let mut added_issues = Vec::new();
    let mut removed_issues = Vec::new();

    for code in &curr_issues {
        if !prev_issues.contains(code) {
            added_issues.push(*code);
        }
    }
    for code in &prev_issues {
        if !curr_issues.contains(code) {
            removed_issues.push(*code);
        }
    }

    println!("=== Diff between last two scans ===");
    println!("Added nodes: {:?}", added_nodes);
    println!("Removed nodes: {:?}", removed_nodes);
    println!("Changed nodes: {:?}", changed_nodes);
    println!("Added issues: {:?}", added_issues);
    println!("Removed issues: {:?}", removed_issues);
    Ok(())
}
