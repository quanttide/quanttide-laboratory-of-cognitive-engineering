use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Situation {
    pub id: u32,
    pub title: String,
    pub weeks: Vec<String>,
    #[serde(rename = "type")]
    pub situation_type: Option<String>,
    pub evolution: Option<String>,
    pub per_week: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodSlice {
    #[serde(alias = "week")]
    pub label: String,
    pub intents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeWeight {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: String,
    pub r#type: String,
    pub evolution: String,
    #[serde(alias = "per_week_intents")]
    pub period_slices: Vec<PeriodSlice>,
}
