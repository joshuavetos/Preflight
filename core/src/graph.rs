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

    if state.nodes.iter().any(|n| n.id == "python") {
        state.edges.push(Edge {
            from: "python".to_string(),
            to: "os".to_string(),
            relation: Relation::REQUIRES,
        });
    }

    if state.nodes.iter().any(|n| n.id == "postgres") {
        state.edges.push(Edge {
            from: "postgres".to_string(),
            to: "os".to_string(),
            relation: Relation::REQUIRES,
        });
    }

    if state.nodes.iter().any(|n| n.id == "redis") {
        state.edges.push(Edge {
            from: "redis".to_string(),
            to: "os".to_string(),
            relation: Relation::REQUIRES,
        });
    }

    if state.nodes.iter().any(|n| n.id == "gpu") {
        state.edges.push(Edge {
            from: "gpu".to_string(),
            to: "os".to_string(),
            relation: Relation::REQUIRES,
        });
    }

    if state.nodes.iter().any(|n| n.id == "docker_images") {
        state.edges.push(Edge {
            from: "docker_images".to_string(),
            to: "docker".to_string(),
            relation: Relation::REQUIRES,
        });
    }
}
