use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::keyword::{KeywordEntry, KeywordTable};
use crate::tokenizer;

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

pub(crate) fn match_situation_id(situations: &[Situation], reference: &str) -> Option<u32> {
    let ref_lower = reference.to_lowercase();

    for s in situations {
        if s.title.to_lowercase().contains(&ref_lower) {
            return Some(s.id);
        }
    }

    for s in situations {
        if let Some(ref evo) = s.evolution {
            if evo.to_lowercase().contains(&ref_lower) {
                return Some(s.id);
            }
        }
    }

    let words: Vec<String> = ref_lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect();
    for s in situations {
        let title_lower = s.title.to_lowercase();
        for word in &words {
            if word.len() >= 3 && title_lower.contains(word.as_str()) {
                return Some(s.id);
            }
        }
    }
    for s in situations {
        let title_lower = s.title.to_lowercase();
        for word in &words {
            if word.len() >= 2 && title_lower.contains(word.as_str()) {
                return Some(s.id);
            }
        }
    }

    let ref_bigrams: std::collections::HashSet<String> = ref_lower
        .chars()
        .collect::<Vec<char>>()
        .windows(2)
        .map(|w| w.iter().collect::<String>())
        .filter(|b| b.chars().all(|c| c.is_alphanumeric()))
        .collect();
    let mut best: Option<(u32, f64)> = None;
    for s in situations {
        let title_lower = s.title.to_lowercase();
        let name_bigrams: std::collections::HashSet<String> = title_lower
            .chars()
            .collect::<Vec<char>>()
            .windows(2)
            .map(|w| w.iter().collect::<String>())
            .filter(|b| b.chars().all(|c| c.is_alphanumeric()))
            .collect();
        if ref_bigrams.is_empty() || name_bigrams.is_empty() {
            continue;
        }
        let intersection = ref_bigrams.intersection(&name_bigrams).count();
        let recall = intersection as f64 / ref_bigrams.len() as f64;
        if recall >= 0.3 && best.map_or(true, |(_, b)| recall > b) {
            best = Some((s.id, recall));
        }
    }
    best.map(|(id, _)| id)
}

pub fn build_keyword_table(situations: &[Situation]) -> KeywordTable {
    situations
        .iter()
        .map(|s| {
            let mut all_words: Vec<String> = Vec::new();
            let name_tokens = tokenizer::tokenize(&s.title);
            all_words.extend(name_tokens);
            for (_, intents) in &s.per_week {
                for intent in intents {
                    let tokens = tokenizer::tokenize(intent);
                    all_words.extend(tokens);
                }
            }
            all_words.sort();
            all_words.dedup();
            KeywordEntry {
                id: s.id,
                title: s.title.clone(),
                keywords: all_words,
            }
        })
        .collect()
}

pub fn build_keyword_table_from_yaml(
    intent_path: &str,
) -> Result<KeywordTable, Box<dyn std::error::Error>> {
    use crate::yaml::GraphDefinition;
    let content = fs::read_to_string(intent_path)?;
    let yaml_root: GraphDefinition = serde_yaml::from_str(&content)?;
    Ok(build_keyword_table(&yaml_root.situations))
}
