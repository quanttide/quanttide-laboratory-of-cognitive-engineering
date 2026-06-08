use serde::{Deserialize, Serialize};

/// A keyword entry mapping a situation to its extracted keywords.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordEntry {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: String,
    pub keywords: Vec<String>,
}

pub type KeywordTable = Vec<KeywordEntry>;
