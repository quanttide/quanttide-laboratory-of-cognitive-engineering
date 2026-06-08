use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TestSegment {
    pub id: String,
    pub week: String,
    pub segment: String,
    pub expected_clusters: Vec<u32>,
    pub expected_relations: Vec<ExpectedRelation>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedRelation {
    pub from: u32,
    pub to: u32,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct BaselineEntry {
    pub id: String,
    pub segment: Option<String>,
    pub clusters: Vec<u32>,
    pub relations: Vec<BaselineRelation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaselineRelation {
    pub from: u32,
    pub to: u32,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct OutputA {
    pub matched: Vec<MatchEntryA>,
}

#[derive(Debug, Deserialize)]
pub struct MatchEntryA {
    pub id: u32,
    pub name: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OutputB {
    pub match_nodes: Vec<MatchEntryB>,
    pub neighbors: Vec<NeighborEntry>,
    pub bfs_paths: Vec<Vec<PathStep>>,
    pub conflicts: Vec<ConflictEntry>,
    pub candidate_edges: Vec<CandidateEntry>,
}

#[derive(Debug, Deserialize)]
pub struct MatchEntryB {
    pub id: u32,
    pub name: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NeighborEntry {
    pub from: u32,
    pub to: u32,
    pub relation: String,
    pub logic: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathStep {
    pub from: u32,
    pub to: u32,
    pub relation: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConflictEntry {
    pub node_a: u32,
    pub node_b: u32,
    pub relation_type: String,
    pub via: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CandidateEntry {
    pub from: u32,
    pub to: u32,
    pub proposed_type: String,
    pub evidence: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct SampleOutput {
    pub id: String,
    pub baseline: BaselineSummary,
    pub approach_a: ApproachAOutput,
    pub approach_b: ApproachBOutput,
    pub metrics: Metrics,
    pub path_grades: Vec<PathGrade>,
}

#[derive(Debug, Serialize)]
pub struct BaselineSummary {
    pub clusters: Vec<u32>,
    pub relations: Vec<BaselineRelation>,
}

#[derive(Debug, Serialize)]
pub struct ApproachAOutput {
    pub matched: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct ApproachBOutput {
    pub matched: Vec<u32>,
    pub neighbors: Vec<NeighborEntry>,
    pub bfs_paths: Vec<Vec<PathStep>>,
    pub conflicts: Vec<ConflictEntry>,
    pub candidate_edges: Vec<CandidateEntry>,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub recall_a: f64,
    pub recall_b: f64,
    pub incremental_nodes: usize,
    pub incremental_relations: usize,
    pub false_positive_a: f64,
    pub false_positive_b: f64,
}

#[derive(Debug, Serialize)]
pub struct PathGrade {
    pub from: u32,
    pub to: u32,
    pub depth: usize,
    pub relation: String,
    pub grade: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_samples: usize,
    pub avg_recall_a: f64,
    pub avg_recall_b: f64,
    pub total_incremental_nodes: usize,
    pub total_incremental_relations: usize,
    pub avg_false_positive_a: f64,
    pub avg_false_positive_b: f64,
    pub avg_path_grade: Option<f64>,
    pub caveat: String,
}

#[derive(Debug, Serialize)]
pub struct EvaluationOutput {
    pub samples: Vec<SampleOutput>,
    pub summary: Summary,
}
