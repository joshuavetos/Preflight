mod graph;
mod models;
mod oracle;
mod scanner;
mod server;
mod utils;
mod doctor;

use clap::{Parser, Subcommand};
use graph::{derive_edges, summarize, DependencyGraph};
use models::SystemState;
use oracle::{evaluate, simulate_command};
use std::path::PathBuf;
use utils::write_state;

#[derive(Parser)]
#[command(author, version, about = "Preflight system scanner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan,
    Simulate { command: String },
    Dashboard,
    Doctor,
}

fn scan_command() -> Result<SystemState, String> {
    let mut state = scanner::perform_scan();
    derive_edges(&mut state);
    state.issues = evaluate(&state);
    state.assert_contract();
    let graph = DependencyGraph::from_state(&state);
    if graph.nodes.is_empty() {
        return Err("Graph invariant violated: no nodes generated".to_string());
    }
    let path = PathBuf::from(".preflight/scan.json");
    write_state(&path, &state).map_err(|e| format!("Failed to write scan file: {e}"))?;
    println!("Preflight scan complete. {}", summarize(&state));
    Ok(state)
}

fn simulate(command: &str) {
    if !std::path::PathBuf::from(".preflight/scan.json").exists() {
        println!("⚠️  No scan.json found. Run `preflight scan` first.");
    }
    let issues = simulate_command(command);
    if issues.is_empty() {
        println!("Simulation successful: no predicted issues for `{command}`.");
    } else {
        println!("Simulation detected potential issues for `{command}`:");
        for issue in issues {
            println!(
                "- [{}] {} ({}): {}",
                issue.severity.to_string(),
                issue.title,
                issue.code,
                issue.suggestion
            );
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan => {
            if let Err(e) = scan_command() {
                eprintln!("Scan failed: {e}");
                std::process::exit(1);
            }
        }
        Commands::Simulate { command } => simulate(&command),
        Commands::Dashboard => {
            if let Err(e) = server::run_dashboard_server().await {
                eprintln!("Dashboard failed: {e}");
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            if let Err(e) = doctor::doctor() {
                eprintln!("Doctor failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

impl ToString for models::Severity {
    fn to_string(&self) -> String {
        match self {
            models::Severity::Critical => "critical".to_string(),
            models::Severity::Warning => "warning".to_string(),
        }
    }
}
