use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const CONTRACT_VERSION: &str = "1.0.0";
pub const DETERMINISTIC_TIMESTAMP: &str = "1970-01-01T00:00:00Z";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Os,
    Service,
    Runtime,
    Application,
    Port,
    File,
    Python,
    Postgres,
    Mysql,
    Redis,
    Gpu,
    DockerImages,
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
    pub metadata: BTreeMap<String, Value>,
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
    pub fingerprint: String,
}

impl SystemState {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, issues: Vec<Issue>, timestamp: String) -> Self {
        let mut state = SystemState {
            nodes,
            edges,
            issues,
            version: CONTRACT_VERSION.to_string(),
            timestamp,
            fingerprint: String::new(),
        };
        state.normalize();
        state.fingerprint = state.compute_fingerprint();
        state
    }

    pub fn refresh_fingerprint(&mut self) {
        self.normalize();
        self.fingerprint = self.compute_fingerprint();
    }

    pub fn normalize(&mut self) {
        self.nodes
            .sort_by(|a, b| a.id.to_lowercase().cmp(&b.id.to_lowercase()));
        self.edges.sort_by(|a, b| {
            let left = (&a.from, &a.to, format!("{:?}", a.relation));
            let right = (&b.from, &b.to, format!("{:?}", b.relation));
            left.cmp(&right)
        });
        self.issues.sort_by(|a, b| a.code.cmp(&b.code));
    }

    fn compute_fingerprint(&self) -> String {
        let payload = serde_json::json!({
            "version": self.version,
            "timestamp": self.timestamp,
            "nodes": self.nodes,
            "edges": self.edges,
            "issues": self.issues,
        });
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(&payload).unwrap_or_default());
        let digest = hasher.finalize();
        format!("{:x}", digest)
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
            !self.fingerprint.is_empty(),
            "SystemState invariant violated: fingerprint missing"
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
