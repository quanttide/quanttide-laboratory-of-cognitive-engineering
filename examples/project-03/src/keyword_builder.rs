use std::fs;
use std::path::Path;

use crate::models::{IntentYaml, KeywordEntry, KeywordTable};
use crate::tokenizer;

pub fn build_keyword_table(yaml_path: &str) -> Result<KeywordTable, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(yaml_path)?;
    let yaml: IntentYaml = serde_yaml::from_str(&content)?;

    let table: KeywordTable = yaml
        .clusters
        .into_iter()
        .map(|cluster| {
            let mut all_words: Vec<String> = Vec::new();

            let name_tokens = tokenizer::tokenize(&cluster.name);
            all_words.extend(name_tokens);

            for (_, intents) in &cluster.per_week {
                for intent in intents {
                    let tokens = tokenizer::tokenize(intent);
                    all_words.extend(tokens);
                }
            }

            all_words.sort();
            all_words.dedup();

            KeywordEntry {
                id: cluster.id,
                name: cluster.name,
                keywords: all_words,
            }
        })
        .collect();

    Ok(table)
}

pub fn save_keyword_table(table: &KeywordTable, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(table)?;
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, json)?;
    Ok(())
}
