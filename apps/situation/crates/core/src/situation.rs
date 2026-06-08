use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::intent::{IntentId, Intent};
use crate::tokenizer;

/// Full definition of the situation graph (nodes) in YAML form.
#[derive(Debug, Deserialize)]
pub struct GraphDefinition {
    pub situations: Vec<Situation>,
}

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
    pub intents: Vec<IntentId>,
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

// --- Keyword table (derived from situations) ---

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordEntry {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: String,
    pub keywords: Vec<String>,
}

pub type KeywordTable = Vec<KeywordEntry>;

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

pub fn build_keyword_table(nodes: &[NodeWeight], store: &Intent) -> KeywordTable {
    nodes
        .iter()
        .map(|n| {
            let mut all_words: Vec<String> = Vec::new();
            let name_tokens = tokenizer::tokenize(&n.title);
            all_words.extend(name_tokens);
            for slice in &n.period_slices {
                for id in &slice.intents {
                    if let Some(content) = store.get(*id) {
                        let tokens = tokenizer::tokenize(content);
                        all_words.extend(tokens);
                    }
                }
            }
            all_words.sort();
            all_words.dedup();
            KeywordEntry {
                id: n.id as u32,
                title: n.title.clone(),
                keywords: all_words,
            }
        })
        .collect()
}

pub fn build_keyword_table_from_yaml(
    intent_path: &str,
) -> Result<KeywordTable, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(intent_path)?;
    let yaml_root: GraphDefinition = serde_yaml::from_str(&content)?;
    let situations = &yaml_root.situations;

    let mut store = Intent::new();
    let nodes: Vec<NodeWeight> = situations
        .iter()
        .map(|s| {
            let mut period_slices: Vec<PeriodSlice> = Vec::new();
            let mut weeks: Vec<&str> = s.per_week.keys().map(|s| s.as_str()).collect();
            weeks.sort();
            for week in weeks {
                if let Some(intents) = s.per_week.get(week) {
                    let ids: Vec<IntentId> = intents
                        .iter()
                        .map(|content| store.add(content.clone()))
                        .collect();
                    period_slices.push(PeriodSlice {
                        label: week.to_string(),
                        intents: ids,
                    });
                }
            }
            NodeWeight {
                id: s.id,
                title: s.title.clone(),
                r#type: s.situation_type.clone().unwrap_or_default(),
                evolution: s.evolution.clone().unwrap_or_default(),
                period_slices,
            }
        })
        .collect();

    Ok(build_keyword_table(&nodes, &store))
}

// --- Situation index (text analysis) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SituationEntry {
    pub id: u32,
    pub title: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooccurrence {
    pub source: String,
    pub week: String,
    pub quote: String,
}

pub struct SituationIndex {
    pub situations: Vec<SituationEntry>,
}

const STOPWORDS: &[&str] = &[
    "的", "了", "与", "以", "在", "有", "和", "是", "不", "为",
    "之", "到", "要", "而", "从", "对", "也", "就", "都", "及",
    "或", "把", "被", "让", "将", "并", "所", "化", "性", "力",
    "法", "式", "个", "这", "那", "上", "下", "中", "出", "去",
    "能", "会", "可", "但", "还", "没", "很", "太", "更", "最",
];

fn is_stopword(s: &str) -> bool {
    STOPWORDS.contains(&s)
}

impl SituationIndex {
    pub fn from_yaml(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let mut situations = Vec::new();
        if let Some(arr) = yaml["situations"].as_sequence() {
            for item in arr {
                let id = item["id"].as_u64().unwrap_or(0) as u32;
                let title = item["title"].as_str().unwrap_or("").to_string();
                let mut keywords: Vec<String> = Vec::new();

                for t in tokenizer::tokenize(&title) {
                    if !is_stopword(&t) {
                        keywords.push(t);
                    }
                }
                if let Some(evolution) = item["evolution"].as_str() {
                    for t in tokenizer::tokenize(evolution) {
                        if !is_stopword(&t) {
                            keywords.push(t);
                        }
                    }
                }
                if let Some(pw) = item["per_week"].as_mapping() {
                    for (_, v) in pw {
                        if let Some(intents) = v.as_sequence() {
                            for intent in intents {
                                if let Some(text) = intent.as_str() {
                                    for t in tokenizer::tokenize(text) {
                                        if !is_stopword(&t) {
                                            keywords.push(t);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                keywords.sort();
                keywords.dedup();
                situations.push(SituationEntry { id, title, keywords });
            }
        }
        Ok(SituationIndex { situations })
    }

    pub fn get(&self, id: u32) -> Option<&SituationEntry> {
        self.situations.iter().find(|c| c.id == id)
    }

    pub fn overlap(&self, text: &str, situation_id: u32) -> f64 {
        let entry = match self.situations.iter().find(|c| c.id == situation_id) {
            Some(e) => e,
            None => return 0.0,
        };
        let text_tokens = tokenizer::tokenize(text);
        if text_tokens.is_empty() || entry.keywords.is_empty() {
            return 0.0;
        }
        let matches = entry
            .keywords
            .iter()
            .filter(|kw| text_tokens.contains(kw))
            .count();
        matches as f64 / entry.keywords.len() as f64
    }

    pub fn find_cooccurrences(
        &self,
        files: &[String],
        situation_a: u32,
        situation_b: u32,
        threshold: f64,
        week_prefix: &str,
        max_quote_len: usize,
    ) -> Vec<Cooccurrence> {
        let mut results = Vec::new();
        for file_path in files {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                _ => continue,
            };
            let body = if let Some(idx) = content.find("\n#") {
                &content[idx..]
            } else {
                &content
            };
            let score_a = self.overlap(body, situation_a);
            let score_b = self.overlap(body, situation_b);
            if score_a > threshold && score_b > threshold {
                let week = file_path
                    .split('/')
                    .find(|p| p.starts_with(week_prefix))
                    .unwrap_or("unknown")
                    .to_string();
                let source = file_path.split('/').last().unwrap_or("unknown").to_string();
                let quote = body.chars().take(max_quote_len).collect::<String>().trim().to_string();
                results.push(Cooccurrence { source, week, quote });
            }
        }
        results
    }
}

pub fn find_raw_files(base_dir: &str, weeks: &[&str]) -> Vec<String> {
    let mut files = Vec::new();
    for w in weeks {
        let dir_path = format!("{}/{}", base_dir, w);
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    files
}
