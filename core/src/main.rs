mod graph;
mod models;
mod oracle;
mod scanner;
mod server;
mod utils;
mod config;
mod risk;
mod doctor;

use clap::{Parser, Subcommand};

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
    Dashboard,
    Doctor,
}

fn scan_command() -> Result<models::SystemState, String> {
    let mut state = scanner::perform_scan();
    graph::derive_edges(&mut state);
    state.issues = oracle::evaluate(&state);
    state.assert_contract();

    let path = std::path::PathBuf::from(".preflight/scan.json");
    utils::write_state(&path, &state)
        .map_err(|e| format!("Failed to write scan: {e}"))?;

    println!(
        "Preflight scan complete: {} nodes, {} edges, {} issues",
        state.nodes.len(),
        state.edges.len(),
        state.issues.len()
    );

    Ok(state)
}

fn simulate(command: &str) {
    if !std::path::PathBuf::from(".preflight/scan.json").exists() {
        println!("⚠️  No scan.json found. Run `preflight scan` first.");
    }

    let issues = oracle::simulate_command(command);
    if issues.is_empty() {
        println!("Simulation successful: no predicted issues.");
    } else {
        println!("Simulation detected {} potential issues:", issues.len());
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
