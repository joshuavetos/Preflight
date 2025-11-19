use crate::models::{Edge, Node, Relation, Status, SystemState};
use std::collections::HashMap;

pub struct DependencyGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl DependencyGraph {
    pub fn from_state(state: &SystemState) -> Self {
        DependencyGraph {
            nodes: state.nodes.clone(),
            edges: state.edges.clone(),
        }
    }

    pub fn node_map(&self) -> HashMap<String, &Node> {
        self.nodes.iter().map(|n| (n.id.clone(), n)).collect()
    }
}

pub fn summarize(state: &SystemState) -> String {
    let active = state
        .nodes
        .iter()
        .filter(|n| matches!(n.status, Status::Active))
        .count();
    let inactive = state
        .nodes
        .iter()
        .filter(|n| matches!(n.status, Status::Inactive))
        .count();
    let conflicts = state
        .nodes
        .iter()
        .filter(|n| matches!(n.status, Status::Conflict))
        .count();
    let issues = state.issues.len();
    format!(
        "Nodes: active={active}, inactive={inactive}, conflict={conflicts}. Issues detected: {issues}.",
    )
}

pub fn derive_edges(state: &mut SystemState) {
    if !state
        .edges
        .iter()
        .any(|e| e.from == "docker" && e.to == "port8000")
    {
        state.edges.push(Edge {
            from: "docker".to_string(),
            to: "port8000".to_string(),
            relation: Relation::BINDS,
        });
    }
}
