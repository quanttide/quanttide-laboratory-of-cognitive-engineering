use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TestSegment {
    pub id: String,
    pub week: String,
    pub segment: String,
    pub clusters: Vec<u32>,
}

#[derive(Deserialize)]
pub struct ReasonTestCase {
    pub id: String,
    #[allow(dead_code)]
    pub description: String,
    pub input_clusters: Vec<u32>,
    pub query_type: String,
    pub expected_direct_edges: Vec<ExpectedEdge>,
    pub expected_path: Option<ExpectedPath>,
    #[allow(dead_code)]
    pub note: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ExpectedEdge {
    pub from: u32,
    pub to: u32,
    #[serde(rename = "type")]
    pub relation_type: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ExpectedPath {
    pub hops: usize,
    pub path_clusters: Vec<u32>,
}

#[derive(Deserialize)]
pub struct FeedbackData {
    pub weekly_snapshots: Vec<WeeklySnapshot>,
    pub timeline: Vec<FeedbackWeek>,
}

#[derive(Deserialize)]
pub struct WeeklySnapshot {
    pub week: String,
    pub active_clusters: Vec<u32>,
    #[serde(default)]
    pub new_clusters: Vec<u32>,
    #[serde(default)]
    pub stable_relations: Vec<IncrementalEdge>,
    #[serde(default)]
    pub periodic_relations: Vec<IncrementalEdge>,
    #[serde(default)]
    pub situational_relations: Vec<IncrementalEdge>,
    #[serde(default)]
    pub incremental_relations: Vec<IncrementalEdge>,
}

#[derive(Deserialize)]
pub struct IncrementalEdge {
    pub from: u32,
    pub to: u32,
    #[serde(rename = "type")]
    pub relation_type: String,
}

#[derive(Deserialize)]
pub struct FeedbackWeek {
    pub week: String,
    pub segments: Vec<FeedbackSegment>,
    #[allow(dead_code)]
    pub evaluation_summary: EvalSummary,
}

#[derive(Deserialize)]
pub struct FeedbackSegment {
    pub id: String,
    pub clusters: Vec<u32>,
    pub expected_relations: Vec<ExpectedRelation>,
    pub segment: String,
    pub evidence: String,
}

#[derive(Deserialize)]
pub struct ExpectedRelation {
    pub from: u32,
    pub to: u32,
    #[serde(rename = "type")]
    pub relation_type: String,
    #[allow(dead_code)]
    pub involves_new_clusters: bool,
}

#[derive(Deserialize)]
pub struct EvalSummary {
    #[allow(dead_code)]
    pub stable_count: usize,
    #[allow(dead_code)]
    pub new_cluster_count: usize,
    #[allow(dead_code)]
    pub total_count: usize,
}
