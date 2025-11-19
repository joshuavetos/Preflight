use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const CONTRACT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Os,
    Service,
    Runtime,
    Application,
    Port,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub label: String,
    pub status: Status,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Relation {
    REQUIRES,
    BINDS,
    CONFLICTS,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relation: Relation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    pub code: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemState {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub issues: Vec<Issue>,
    pub version: String,
    pub timestamp: String,
}

impl SystemState {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, issues: Vec<Issue>, timestamp: String) -> Self {
        SystemState {
            nodes,
            edges,
            issues,
            version: CONTRACT_VERSION.to_string(),
            timestamp,
        }
    }

    pub fn assert_contract(&self) {
        assert!(
            !self.nodes.is_empty(),
            "SystemState invariant violated: nodes must not be empty"
        );
        assert!(
            !self.timestamp.is_empty(),
            "SystemState invariant violated: timestamp missing"
        );
        assert!(
            !self.version.is_empty(),
            "SystemState invariant violated: version missing"
        );
        assert!(
            self.version == CONTRACT_VERSION,
            "SystemState invariant violated: version mismatch"
        );
        let mut ids = std::collections::HashSet::new();
        for node in &self.nodes {
            assert!(
                ids.insert(&node.id),
                "duplicate node id detected: {}",
                node.id
            );
        }
        for issue in &self.issues {
            assert!(
                !issue.code.is_empty(),
                "Issue invariant violated: code missing"
            );
            assert!(
                !issue.title.is_empty(),
                "Issue invariant violated: title missing"
            );
        }
    }
}
