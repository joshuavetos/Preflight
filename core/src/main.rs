mod graph;
mod models;
mod oracle;
mod scanner;
mod server;
mod utils;
mod validate;

mod analyze;
mod command_ast;
mod config;
mod deps;
mod doctor;
mod exporter;
mod fix;
mod history;
mod json_diff;
mod proposed_state;
mod remote;
mod risk;
mod risk_config;
mod security;
mod share;
mod snapshot;
mod spec;
mod system_provider;
mod tokenizer;
mod updater;
mod watch;

use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Preflight system scanner")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(long)]
        remote: Option<String>,
    },
    Simulate {
        command: String,
    },
    SimulateProposed {
        command: String,
    },
    Dashboard,
    Doctor,
    Upgrade,
    Fix,
    Diff,
    Watch,
    Deps,
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommand,
    },
    Export {
        #[arg(long)]
        format: String,
    },
    Validate,
    ValidateEnv,
    Analyze,
    Security,
    Share {
        #[arg(long)]
        out: String,
    },
}

#[derive(Subcommand)]
enum SnapshotCommand {
    Save { name: String },
    Restore { name: String },
}

fn scan_command(remote: Option<String>, json_output: bool) -> Result<models::SystemState, String> {
    let mut state = if let Some(target) = remote {
        remote::remote_scan(&target)?
    } else {
        let mut local_state = scanner::perform_scan();
        graph::derive_edges(&mut local_state);
        local_state.issues = oracle::evaluate(&local_state);
        local_state.assert_contract();
        let path = PathBuf::from(".preflight/scan.json");
        utils::write_state(&path, &local_state)
            .map_err(|e| format!("Failed to write scan: {}", e))?;
        local_state
    };

    history::record_scan(&state)?;
    if json_output {
        let payload = utils::json_envelope(
            "scan",
            "ok",
            json!({
                "issues": state.issues.clone(),
                "nodes": state.nodes.clone()
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
        println!("Preflight scan complete. {}", graph::summarize(&state));
    }
    Ok(state)
}

fn simulate_simple(command: &str, json_output: bool) {
    let result = oracle::simulate_command(command);
    if json_output {
        let simplified: Vec<_> = result
            .issues
            .iter()
            .map(|i| json!({"code": i.code, "severity": format!("{:?}", i.severity)}))
            .collect();
        let payload = utils::json_envelope(
            "simulate",
            "ok",
            json!({
                "input": command,
                "issues_after_simulation": simplified
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else if result.issues.is_empty() {
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

fn simulate_proposed(command: &str, json_output: bool) {
    let result = oracle::simulate_command(command);

    if json_output {
        let payload = utils::json_envelope(
            "simulate-proposed",
            "ok",
            json!({
                "input": command,
                "proposed_state_diff": {
                    "before": json!({}),
                    "after": result.diff.clone().unwrap_or_else(|| json!({}))
                }
            }),
        );
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { remote } => {
            if let Err(e) = scan_command(remote, cli.json) {
                eprintln!("Scan failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Simulate { command } => simulate_simple(&command, cli.json),

        Commands::SimulateProposed { command } => simulate_proposed(&command, cli.json),

        Commands::Dashboard => {
            if let Err(e) = server::run_dashboard_server().await {
                eprintln!("Dashboard failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Doctor => {
            if let Err(e) = doctor::doctor(cli.json) {
                eprintln!("Doctor failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Upgrade => {
            if let Err(e) = updater::upgrade() {
                eprintln!("Upgrade failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Deps => {
            if cli.json {
                match deps::collect_graph() {
                    Ok(graph) => {
                        let payload =
                            utils::json_envelope("deps", "ok", json!({ "graph": graph.0 }));
                        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                    }
                    Err(e) => {
                        eprintln!("Deps failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if let Err(e) = deps::run() {
                eprintln!("Deps failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Fix => {
            if let Err(e) = fix::run(cli.json) {
                eprintln!("Fix suggestions failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Diff => {
            if let Err(e) = history::diff_latest() {
                eprintln!("Diff failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Watch => {
            if let Err(e) = watch::run().await {
                eprintln!("Watch failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Snapshot { action } => match action {
            SnapshotCommand::Save { name } => {
                if let Err(e) = snapshot::save(&name) {
                    eprintln!("Snapshot save failed: {}", e);
                    std::process::exit(1);
                }
            }
            SnapshotCommand::Restore { name } => {
                if let Err(e) = snapshot::restore(&name) {
                    eprintln!("Snapshot restore failed: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Export { format } => {
            if let Err(e) = exporter::export(&format, cli.json) {
                eprintln!("Export failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Validate => {
            let code = validate::validate(cli.json);
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::ValidateEnv => match spec::run(cli.json) {
            Ok(code) => {
                if code != 0 {
                    std::process::exit(code);
                }
            }
            Err(e) => {
                eprintln!("validate-env failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Analyze => {
            if let Err(e) = analyze::run(cli.json) {
                eprintln!("Analyze failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Security => {
            if let Err(e) = security::run(cli.json) {
                eprintln!("Security scan failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Share { out } => {
            if let Err(e) = share::run(&out, cli.json) {
                eprintln!("Share failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
