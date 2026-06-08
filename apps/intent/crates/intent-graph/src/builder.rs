use std::fs;
use std::path::Path;

use crate::models::*;
use crate::tokenizer;

fn parse_relation_name(name: &str) -> (String, String, bool) {
    for sep in &[" ⇄ ", " ↔ ", " → "] {
        if let Some(pos) = name.find(sep) {
            let source = name[..pos].trim().to_string();
            let target = name[pos + sep.len()..].trim().to_string();
            let bidirectional = *sep == " ⇄ " || *sep == " ↔ ";
            return (source, target, bidirectional);
        }
    }
    (String::new(), name.to_string(), false)
}

fn parse_relation_type(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(paren) = raw.find('（') {
        raw[..paren].trim().to_string()
    } else if let Some(paren) = raw.find('(') {
        raw[..paren].trim().to_string()
    } else if raw.contains("双向") {
        "支持".to_string()
    } else if raw.contains(" + ") {
        raw.split(" + ").next().unwrap_or(raw).trim().to_string()
    } else {
        raw.to_string()
    }
}

fn match_cluster_id(clusters: &[Cluster], reference: &str) -> Option<u32> {
    let ref_lower = reference.to_lowercase();

    for cluster in clusters {
        if cluster.name.to_lowercase().contains(&ref_lower) {
            return Some(cluster.id);
        }
    }

    for cluster in clusters {
        if let Some(ref evo) = cluster.evolution {
            if evo.to_lowercase().contains(&ref_lower) {
                return Some(cluster.id);
            }
        }
    }

    let words: Vec<String> = ref_lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect();
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        for word in &words {
            if word.len() >= 3 && name_lower.contains(word.as_str()) {
                return Some(cluster.id);
            }
        }
    }
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        for word in &words {
            if word.len() >= 2 && name_lower.contains(word.as_str()) {
                return Some(cluster.id);
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
    for cluster in clusters {
        let name_lower = cluster.name.to_lowercase();
        let name_bigrams: std::collections::HashSet<String> = name_lower
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
            best = Some((cluster.id, recall));
        }
    }
    best.map(|(id, _)| id)
}

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn from_yaml(
        intent_path: &str,
        relation_path: &str,
    ) -> Result<super::graph::IntentGraph, Box<dyn std::error::Error>> {
        let mut ig = super::graph::IntentGraph::new();

        let intent_content = fs::read_to_string(intent_path)?;
        let intent_yaml: IntentYaml = serde_yaml::from_str(&intent_content)?;
        let clusters = intent_yaml.clusters;

        for cluster in &clusters {
            let per_week_intents: Vec<String> = cluster
                .per_week
                .values()
                .flat_map(|v| v.clone())
                .collect();
            let node = NodeWeight {
                id: cluster.id,
                name: cluster.name.clone(),
                r#type: cluster.cluster_type.clone().unwrap_or_default(),
                evolution: cluster.evolution.clone().unwrap_or_default(),
                per_week_intents,
            };
            ig.add_node(node);
        }

        let relation_content = fs::read_to_string(relation_path)?;
        let relation_yaml: RelationYaml = serde_yaml::from_str(&relation_content)?;

        for entry in &relation_yaml.stable_relations {
            Self::add_edge_entry(&mut ig, entry, "stable", &clusters);
        }
        for entry in &relation_yaml.periodic_tensions {
            Self::add_edge_entry(&mut ig, entry, "periodic", &clusters);
        }
        for entry in &relation_yaml.situational_relations {
            Self::add_edge_entry(
                &mut ig,
                &RelationEntry {
                    name: entry.name.clone(),
                    relation_type: entry.relation_type.clone(),
                    weeks: entry.weeks.clone(),
                    logic: String::new(),
                },
                "situational",
                &clusters,
            );
        }

        Ok(ig)
    }

    fn add_edge_entry(
        ig: &mut super::graph::IntentGraph,
        entry: &RelationEntry,
        period_type: &str,
        clusters: &[Cluster],
    ) {
        let (source_ref, target_ref, bidirectional) = parse_relation_name(&entry.name);
        if source_ref.is_empty() {
            return;
        }
        let source_id = match_cluster_id(clusters, &source_ref);
        let target_id = match_cluster_id(clusters, &target_ref);
        if let (Some(sid), Some(tid)) = (source_id, target_id) {
            let rel_type = parse_relation_type(&entry.relation_type);
            let weight = EdgeWeight {
                relation_type: rel_type.clone(),
                logic: entry.logic.clone(),
                weeks: entry.weeks.clone(),
                period_type: period_type.to_string(),
            };
            ig.add_edge(sid, tid, weight);
            if bidirectional {
                let rev_weight = EdgeWeight {
                    relation_type: rel_type,
                    logic: entry.logic.clone(),
                    weeks: entry.weeks.clone(),
                    period_type: period_type.to_string(),
                };
                ig.add_edge(tid, sid, rev_weight);
            }
        }
    }

    pub fn build_keyword_table(clusters: &[Cluster]) -> KeywordTable {
        clusters
            .iter()
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
                    name: cluster.name.clone(),
                    keywords: all_words,
                }
            })
            .collect()
    }

    pub fn build_keyword_table_from_yaml(
        intent_path: &str,
    ) -> Result<KeywordTable, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(intent_path)?;
        let yaml: IntentYaml = serde_yaml::from_str(&content)?;
        Ok(Self::build_keyword_table(&yaml.clusters))
    }

    pub fn save_keyword_table(
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
}
