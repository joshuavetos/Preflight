use crate::graph;
use crate::models::{Relation, SystemState};
use std::fs;

fn load_state() -> Result<SystemState, String> {
    let raw = fs::read_to_string(".preflight/scan.json")
        .map_err(|e| format!("Unable to read scan.json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid scan data: {e}"))
}

fn mermaid(state: &SystemState) -> String {
    let mut out = String::from("graph TD\n");
    for node in &state.nodes {
        out.push_str(&format!(
            "    {}[\"{} ({:?})\"]\n",
            node.id, node.label, node.status
        ));
    }
    for edge in &state.edges {
        let label = match edge.relation {
            Relation::REQUIRES => "requires",
            Relation::BINDS => "binds",
            Relation::CONFLICTS => "conflicts",
        };
        out.push_str(&format!("    {} -- {} --> {}\n", edge.from, label, edge.to));
    }
    out
}

fn graphviz(state: &SystemState) -> String {
    let mut out = String::from("digraph preflight {\n");
    for node in &state.nodes {
        out.push_str(&format!(
            "    {} [label=\"{} ({:?})\"]\n",
            node.id, node.label, node.status
        ));
    }
    for edge in &state.edges {
        let label = match edge.relation {
            Relation::REQUIRES => "requires",
            Relation::BINDS => "binds",
            Relation::CONFLICTS => "conflicts",
        };
        out.push_str(&format!(
            "    {} -> {} [label=\"{}\"]\n",
            edge.from, edge.to, label
        ));
    }
    out.push_str("}\n");
    out
}

pub fn export(format: &str) -> Result<(), String> {
    let mut state = load_state()?;
    graph::derive_edges(&mut state);
    let rendered = match format {
        "mermaid" => mermaid(&state),
        "graphviz" => graphviz(&state),
        other => return Err(format!("Unsupported export format: {}", other)),
    };
    println!("{}", rendered);
    Ok(())
}
