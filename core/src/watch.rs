use crate::graph;
use crate::history;
use crate::json_diff::diff_states;
use crate::models::SystemState;
use crate::oracle;
use crate::scanner;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

async fn run_scan_cycle() -> Result<SystemState, String> {
    let mut state = scanner::perform_scan();
    graph::derive_edges(&mut state);
    state.issues = oracle::evaluate(&state);
    state.assert_contract();
    history::record_scan(&state)?;
    Ok(state)
}

pub async fn run() -> Result<(), String> {
    let mut previous: Option<SystemState> = None;
    loop {
        let state = run_scan_cycle()?;
        if let Some(prev) = &previous {
            let diff = diff_states(&json!(prev), &json!(state));
            println!("=== Diff since last scan ===");
            println!("{}", serde_json::to_string_pretty(&diff).unwrap());
        } else {
            println!("Initial scan recorded.");
        }
        previous = Some(state);
        sleep(Duration::from_secs(5)).await;
    }
}
