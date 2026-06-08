use std::collections::HashMap;
use std::fs;

use serde::Serialize;

use intent_llm::{extract_json, DeepSeekClient};

#[derive(Serialize)]
struct InferredRelation {
    from: u32,
    to: u32,
    relation_type: String,
    direction: String,
    evidence: String,
    confidence: f64,
    from_name: String,
    to_name: String,
}

#[derive(Serialize)]
struct InferredRelationsOutput {
    total_pairs: usize,
    with_relations: usize,
    coverage_rate: f64,
    new_edges_discovered: usize,
    previously_isolated_clusters_fixed: Vec<u32>,
    relations: Vec<InferredRelation>,
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Warning: cannot read {}: {}", path, e);
        String::new()
    })
}

fn build_cluster_desc(intent_yaml: &str) -> (Vec<(u32, String)>, String) {
    let content = fs::read_to_string(intent_yaml).expect("Failed to read intent.yaml");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("Failed to parse YAML");
    let mut clusters = Vec::new();
    let mut desc_lines = Vec::new();

    if let Some(arr) = yaml["clusters"].as_sequence() {
        for item in arr {
            let id = item["id"].as_u64().unwrap_or(0) as u32;
            let name = item["name"].as_str().unwrap_or("").to_string();
            let ctype = item["type"].as_str().unwrap_or("未知");
            let weeks: Vec<String> = item["weeks"]
                .as_sequence()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let evolution = item["evolution"].as_str().unwrap_or("无");

            clusters.push((id, name.clone()));

            desc_lines.push(format!(
                "簇{}（{}）类型:{} 周次:{} 演化:{}",
                id, name, ctype, weeks.join(","), evolution,
            ));
            if let Some(pw) = item["per_week"].as_mapping() {
                for (k, v) in pw {
                    let week = k.as_str().unwrap_or("");
                    if let Some(intents) = v.as_sequence() {
                        for intent in intents {
                            if let Some(text) = intent.as_str() {
                                desc_lines.push(format!("  {}: {}", week, text));
                            }
                        }
                    }
                }
            }
            desc_lines.push(String::new());
        }
    }
    (clusters, desc_lines.join("\n"))
}

fn build_prompt(cluster_desc: &str, intent_md: &str, relation_md: &str) -> String {
    format!(
        r#"你是一个意图关系推理引擎。以下是思考者的 10 个意图簇和分析报告。

## 簇定义

{}

## 五周意图演化报告

{}

## 意图关系分析

{}

---

任务：分析以上材料，推断每对意图簇之间的隐含关系。
注意寻找所有可能的隐含关系，包括事件触发、演化、时序、间接影响链。
置信度 < 0.4 的标记为"无关系"。

输出 JSON：{{"relations": [{{"from":1,"to":2,"relation_type":"支持/冲突/...","direction":"单向/双向","evidence":"...","confidence":0.85}}]}}
输出所有 45 对。"#,
        cluster_desc, intent_md, relation_md
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (clusters, cluster_desc) = build_cluster_desc("data/refined/intent.yaml");
    let prompt = build_prompt(
        &cluster_desc,
        &read_file("data/analysis/intent.md"),
        &read_file("data/analysis/intent-relation.md"),
    );

    let client = DeepSeekClient::from_env().map_err(|e| e.to_string())?;
    println!("Calling DeepSeek API...");
    let response = client.chat(&prompt).map_err(|e| e.to_string())?;
    println!("Response received ({} chars)", response.len());

    let parsed = extract_json(&response).map_err(|e| e.to_string())?;
    let name_map: HashMap<u32, String> = clusters.iter().map(|(id, n)| (*id, n.clone())).collect();

    let relations: Vec<InferredRelation> = parsed["relations"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let from = item["from"].as_u64()? as u32;
                    let to = item["to"].as_u64()? as u32;
                    if from == 0 || to == 0 || from == to {
                        return None;
                    }
                    Some(InferredRelation {
                        from: from.min(to),
                        to: from.max(to),
                        relation_type: item["relation_type"].as_str().unwrap_or("无关系").to_string(),
                        direction: item["direction"].as_str().unwrap_or("单向").to_string(),
                        evidence: item["evidence"].as_str().unwrap_or("").to_string(),
                        confidence: item["confidence"].as_f64().unwrap_or(0.0),
                        from_name: name_map.get(&from).cloned().unwrap_or_default(),
                        to_name: name_map.get(&to).cloned().unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let total_pairs = 45usize;
    let new_edges = relations.iter().filter(|r| r.relation_type != "无关系").count();
    let fixed = if relations.iter().any(|r| (r.from == 10 || r.to == 10) && r.relation_type != "无关系") {
        vec![10]
    } else {
        vec![]
    };

    let output = InferredRelationsOutput {
        total_pairs,
        with_relations: relations.len(),
        coverage_rate: relations.len() as f64 / total_pairs as f64,
        new_edges_discovered: new_edges,
        previously_isolated_clusters_fixed: fixed,
        relations,
    };

    fs::create_dir_all("examples/project-07/data/output").ok();
    fs::write(
        "examples/project-07/data/output/inferred-relations.json",
        serde_json::to_string_pretty(&output)?,
    )?;

    println!("Inferred {} relations ({} non-none)", output.with_relations, output.new_edges_discovered);
    println!("Coverage: {:.1}%", output.coverage_rate * 100.0);
    Ok(())
}
