use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct IntentYaml {
    pub clusters: Vec<Cluster>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Cluster {
    pub id: u32,
    pub name: String,
    pub weeks: Vec<String>,
    #[serde(rename = "type")]
    pub cluster_type: Option<String>,
    pub evolution: Option<String>,
    pub per_week: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordEntry {
    pub id: u32,
    pub name: String,
    pub keywords: Vec<String>,
}

pub type KeywordTable = Vec<KeywordEntry>;

#[derive(Debug, Serialize)]
pub struct MatchResult {
    pub id: u32,
    pub name: String,
    pub score: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OutputResponse {
    pub matched: Vec<MatchResult>,
}
