use std::fs;

use serde::{Deserialize, Serialize};

use crate::tokenizer;

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
                    .find(|p| p.starts_with("2026-W"))
                    .unwrap_or("unknown")
                    .to_string();
                let source = file_path.split('/').last().unwrap_or("unknown").to_string();
                let quote = body.chars().take(500).collect::<String>().trim().to_string();
                results.push(Cooccurrence { source, week, quote });
            }
        }
        results
    }
}

pub fn find_raw_files(base_dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    for w in &["2026-W19", "2026-W20", "2026-W21", "2026-W22", "2026-W23"] {
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
