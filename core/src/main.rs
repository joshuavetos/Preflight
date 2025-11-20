mod graph;
mod models;
mod oracle;
mod scanner;
mod server;
mod utils;

mod command_ast;
mod config;
mod doctor;
mod json_diff;
mod proposed_state;
mod risk;
mod risk_config;
mod system_provider;
mod tokenizer;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Preflight system scanner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan,
    Simulate { command: String },
    SimulateProposed { command: String },
    Dashboard,
    Doctor,
}

fn scan_command() -> Result<models::SystemState, String> {
    let mut state = scanner::perform_scan();
    graph::derive_edges(&mut state);
    state.issues = oracle::evaluate(&state);
    state.assert_contract();

    let path = PathBuf::from(".preflight/scan.json");
    utils::write_state(&path, &state).map_err(|e| format!("Failed to write scan: {}", e))?;

    println!("Preflight scan complete. {}", graph::summarize(&state));
    Ok(state)
}

fn simulate_simple(command: &str) {
    let result = oracle::simulate_command(command);
    if result.issues.is_empty() {
        println!("Simulation successful: no predicted issues.");
    } else {
        println!("Simulation detected potential issues:");
        for issue in result.issues {
            println!(
                "- [{}] {} ({})",
                issue.severity.to_string().to_uppercase(),
                issue.title,
                issue.code
            );
        }
    }
}

fn simulate_proposed(command: &str) {
    let result = oracle::simulate_command(command);

    println!("\n=== Predicted Issues ===");
    for issue in &result.issues {
        println!(
            "- [{}] {} ({})",
            issue.severity.to_string().to_uppercase(),
            issue.title,
            issue.code
        );
    }

    if let Some(ps) = result.proposed_state {
        let path = PathBuf::from(".preflight/scan_proposed.json");
        utils::write_state(&path, &ps).expect("write proposed");
        println!("\nProposed state written to .preflight/scan_proposed.json");
    }

    if let Some(diff) = result.diff {
        println!("\n=== Diff (current → proposed) ===");
        println!("{}", serde_json::to_string_pretty(&diff).unwrap());
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan => {
            if let Err(e) = scan_command() {
                eprintln!("Scan failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Simulate { command } => simulate_simple(&command),

        Commands::SimulateProposed { command } => simulate_proposed(&command),

        Commands::Dashboard => {
            if let Err(e) = server::run_dashboard_server().await {
                eprintln!("Dashboard failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            if let Err(e) = doctor::doctor() {
                eprintln!("Doctor failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
