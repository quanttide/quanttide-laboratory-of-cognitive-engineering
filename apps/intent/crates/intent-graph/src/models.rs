pub use crate::situation::{NodeWeight, PerWeek, Situation};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct IntentYaml {
    pub situations: Vec<Situation>,
}

#[derive(Debug, Deserialize)]
pub struct RelationYaml {
    pub stable_relations: Vec<RelationEntry>,
    pub periodic_tensions: Vec<RelationEntry>,
    pub situational_relations: Vec<SituationalRelationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RelationEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub weeks: Vec<String>,
    pub logic: String,
}

#[derive(Debug, Deserialize)]
pub struct SituationalRelationEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub weeks: Vec<String>,
    pub trigger: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeWeight {
    pub relation_type: String,
    pub logic: String,
    pub weeks: Vec<String>,
    pub period_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchedNode {
    pub id: u32,
    pub title: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NeighborInfo {
    pub from: u32,
    pub to: u32,
    pub relation: String,
    pub logic: String,
    pub direction: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PathStep {
    pub from: u32,
    pub to: u32,
    pub relation: String,
}

#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub node_a: u32,
    pub node_b: u32,
    pub relation_type: String,
    pub via: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct CandidateEdge {
    pub from: u32,
    pub to: u32,
    pub proposed_type: String,
    pub evidence: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct InferenceOutput {
    pub match_nodes: Vec<MatchedNode>,
    pub neighbors: Vec<NeighborInfo>,
    pub bfs_paths: Vec<Vec<PathStep>>,
    pub conflicts: Vec<ConflictInfo>,
    pub candidate_edges: Vec<CandidateEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordEntry {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: String,
    pub keywords: Vec<String>,
}

pub type KeywordTable = Vec<KeywordEntry>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<NodeWeight>,
    pub edges: Vec<EdgeData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeData {
    pub source: u32,
    pub target: u32,
    pub weight: EdgeWeight,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejectLog {
    pub rejected: Vec<(u32, u32)>,
}
