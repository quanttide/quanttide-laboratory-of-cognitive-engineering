use serde::{Deserialize, Serialize};

/// A situation matched against user input.
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchedNode {
    pub id: u32,
    pub title: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

/// A neighboring node reachable via one edge.
#[derive(Debug, Serialize)]
pub struct NeighborInfo {
    pub from: u32,
    pub to: u32,
    pub relation: String,
    pub logic: String,
    pub direction: String,
}

/// A single step in a BFS path.
#[derive(Debug, Serialize, Clone)]
pub struct PathStep {
    pub from: u32,
    pub to: u32,
    pub relation: String,
}

/// A conflict detected between two nodes.
#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub node_a: u32,
    pub node_b: u32,
    pub relation_type: String,
    pub via: Vec<u32>,
}

/// A proposed new edge from inference.
#[derive(Debug, Serialize)]
pub struct CandidateEdge {
    pub from: u32,
    pub to: u32,
    pub proposed_type: String,
    pub evidence: String,
    pub confidence: f64,
}

/// Combined output of the infer method.
#[derive(Debug, Serialize)]
pub struct InferenceOutput {
    pub match_nodes: Vec<MatchedNode>,
    pub neighbors: Vec<NeighborInfo>,
    pub bfs_paths: Vec<Vec<PathStep>>,
    pub conflicts: Vec<ConflictInfo>,
    pub candidate_edges: Vec<CandidateEdge>,
}
