use std::fs;
use std::path::Path;

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

pub fn save_table(
    table: &KeywordTable,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(table)?;
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, json)?;
    Ok(())
}
