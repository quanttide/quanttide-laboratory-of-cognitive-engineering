use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
struct EvidenceCitation {
    source: String,
    quote: String,
    week: String,
}

#[derive(Serialize)]
struct CompletedRelation {
    from: u32,
    to: u32,
    relation_type: String,
    direction: String,
    confidence: f64,
    from_name: String,
    to_name: String,
    evidence_chain: Vec<EvidenceCitation>,
    llm_reasoning: String,
}

#[derive(Serialize)]
struct CompleteOutput {
    total_pairs: usize,
    completed_pairs: usize,
    relation_type_distribution: HashMap<String, usize>,
    relations: Vec<CompletedRelation>,
}

fn collect_keywords(yaml_path: &str) -> Vec<(u32, String, Vec<String>)> {
    let content = fs::read_to_string(yaml_path).expect("Failed to read YAML");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("Failed to parse");
    let mut result = Vec::new();

    if let Some(arr) = yaml["clusters"].as_sequence() {
        for item in arr {
            let id = item["id"].as_u64().unwrap_or(0) as u32;
            let name = item["name"].as_str().unwrap_or("").to_string();
            let evolution = item["evolution"].as_str().unwrap_or("").to_string();
            let mut keywords: Vec<String> = Vec::new();

            // Keywords from name
            for word in name.chars().collect::<Vec<_>>().windows(2) {
                let bigram: String = word.iter().collect();
                if !is_stopword(&bigram) {
                    keywords.push(bigram);
                }
            }
            // Keywords from evolution
            for part in evolution.split("→") {
                let trimmed = part.trim();
                if trimmed.len() >= 2 {
                    for word in trimmed.chars().collect::<Vec<_>>().windows(2) {
                        let bigram: String = word.iter().collect();
                        if !is_stopword(&bigram) {
                            keywords.push(bigram);
                        }
                    }
                }
            }
            // Keywords from per_week
            if let Some(pw) = item["per_week"].as_mapping() {
                for (_, v) in pw {
                    if let Some(intents) = v.as_sequence() {
                        for intent in intents {
                            if let Some(text) = intent.as_str() {
                                for word in text.chars().collect::<Vec<_>>().windows(2) {
                                    let bigram: String = word.iter().collect();
                                    if !is_stopword(&bigram) {
                                        keywords.push(bigram);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Deduplicate
            keywords.sort();
            keywords.dedup();
            result.push((id, name, keywords));
        }
    }
    result
}

fn is_stopword(s: &str) -> bool {
    matches!(s, "的" | "了" | "与" | "以" | "在" | "有" | "和" | "是" | "不" | "为" | "之" | "到" | "要" | "而" | "从" | "对" | "也" | "就" | "都" | "及" | "或" | "把" | "被" | "让" | "将" | "并" | "所" | "化" | "性" | "力" | "法" | "式")
}

fn find_raw_files(base: &str) -> Vec<String> {
    let mut files = Vec::new();
    for w in &["2026-W19", "2026-W20", "2026-W21", "2026-W22", "2026-W23"] {
        let dir_path = format!("{}/{}", base, w);
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

fn cluster_overlap(text: &str, keywords: &[String]) -> f64 {
    let text_chars: Vec<char> = text.chars().collect();
    if text_chars.len() < 3 {
        return 0.0;
    }
    let text_bigrams: Vec<String> = text_chars.windows(2).map(|w| w.iter().collect()).collect();
    let mut matches = 0usize;
    for kw in keywords {
        if text_bigrams.contains(kw) {
            matches += 1;
        }
    }
    if keywords.is_empty() {
        return 0.0;
    }
    matches as f64 / keywords.len() as f64
}

fn find_cooccurrences(
    files: &[String],
    clusters: &[(u32, String, Vec<String>)],
    pair: (u32, u32),
    threshold: f64,
) -> Vec<EvidenceCitation> {
    let (a, b) = pair;
    let ka = clusters.iter().find(|(id, _, _)| *id == a).unwrap();
    let kb = clusters.iter().find(|(id, _, _)| *id == b).unwrap();
    let mut citations = Vec::new();

    for file_path in files {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        // Skip headers and metadata
        let body = if let Some(idx) = content.find("\n#") {
            &content[idx..]
        } else {
            &content
        };

        let score_a = cluster_overlap(body, &ka.2);
        let score_b = cluster_overlap(body, &kb.2);

        if score_a > threshold && score_b > threshold {
            let week = file_path
                .split('/')
                .find(|p| p.starts_with("2026-W"))
                .unwrap_or("unknown")
                .to_string();
            let date = file_path
                .split('/')
                .last()
                .unwrap_or("unknown")
                .to_string();
            // Take first 500 chars as evidence
            let quote = body.chars().take(500).collect::<String>().trim().to_string();
            citations.push(EvidenceCitation {
                source: date,
                quote,
                week,
            });
        }
    }
    citations
}

fn call_llm_for_pair(
    pair: (u32, u32),
    name_a: &str,
    name_b: &str,
    evidence: &[EvidenceCitation],
) -> Result<CompletedRelation, Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY not set")?;

    let evidence_text: String = if evidence.is_empty() {
        "没有找到同时提及这两个簇的原始日记段落。请基于它们的周次分布和主题描述，推断最可能的关系类型（如同框、时序、类比）。".to_string()
    } else {
        let mut s = String::new();
        for e in evidence.iter().take(3) {
            s.push_str(&format!("[{} {}]\n{}\n\n", e.week, e.source, e.quote));
        }
        s
    };

    let prompt = format!(
        r#"你是一个意图关系推理引擎。假设"所有想法都有内在关联"，请推断以下两个意图簇之间的关系。

## 簇 A（{}）：{}
## 簇 B（{}）：{}

## 共现证据

{}

任务：基于以上证据，判断两簇之间最具体的关系类型。
选择范围（从最具体到最通用）：
1. 支持 - A 的存在/演进促进了 B
2. 冲突 - A 与 B 之间存在内在矛盾
3. 触发 - A 的事件/变化激活了 B
4. 演化 - A 的演进逻辑推动 B 的演进
5. 情感补给 - A 为 B 提供情绪能量
6. 同框 - 在同一思考时空出现，无直接互动
7. 时序 - A 和 B 先后出现，逻辑上相关
8. 类比 - A 和 B 用同一套隐喻框架描述
9. 组件 - A 是 B 的组成部分或具体实例

如果证据充分选择最具体的类型，否则选择较通用的类型。

输出 JSON：
{{
  "relation_type": "类型",
  "direction": "单向/双向",
  "confidence": 0.0-1.0,
  "reasoning": "简要推理过程"
}}"#,
        name_a, pair.0, name_b, pair.1, evidence_text
    );

    let body = serde_json::json!({
        "model": "deepseek-chat",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.3,
        "max_tokens": 1024
    });

    let resp = ureq::post("https://api.deepseek.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(&body)?;

    let resp_json: serde_json::Value = resp.into_json()?;
    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content")?
        .to_string();

    // Parse JSON from response
    let json_str = if let Some(start) = content.find('{') {
        let end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        content[start..end].to_string()
    } else {
        content
    };

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    let rtype = parsed["relation_type"].as_str().unwrap_or("同框").to_string();
    let dir = parsed["direction"].as_str().unwrap_or("单向").to_string();
    let conf = parsed["confidence"].as_f64().unwrap_or(0.3);
    let reasoning = parsed["reasoning"].as_str().unwrap_or("").to_string();

    Ok(CompletedRelation {
        from: pair.0.min(pair.1),
        to: pair.0.max(pair.1),
        relation_type: rtype,
        direction: dir,
        confidence: conf,
        from_name: name_a.to_string(),
        to_name: name_b.to_string(),
        evidence_chain: evidence.to_vec(),
        llm_reasoning: reasoning,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let intent_yaml = "assets/refined/intent.yaml";
    let raw_dir = "assets/raw";

    let clusters = collect_keywords(intent_yaml);
    let raw_files = find_raw_files(raw_dir);

    println!("Raw files found: {}", raw_files.len());
    for (id, name, kws) in &clusters {
        println!("  Cluster {} ({}): {} keywords", id, name, kws.len());
    }

    let pairs: Vec<(u32, u32)> = {
        let mut v = Vec::new();
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                v.push((clusters[i].0, clusters[j].0));
            }
        }
        v
    };
    println!("Cluster pairs to analyze: {}", pairs.len());

    let mut relations = Vec::new();
    let n = pairs.len();
    for (idx, (a, b)) in pairs.iter().enumerate() {
        let ca = clusters.iter().find(|(id, _, _)| *id == *a).unwrap();
        let cb = clusters.iter().find(|(id, _, _)| *id == *b).unwrap();

        println!("[{}/{}] Analyzing pair ({} {}) ↔ ({} {})...",
            idx + 1, n, a, ca.1, b, cb.1);

        let evidence = find_cooccurrences(&raw_files, &clusters, (*a, *b), 0.08);

        match call_llm_for_pair((*a, *b), &ca.1, &cb.1, &evidence) {
            Ok(rel) => {
                println!("  → {} (conf={}) [{} citations]", rel.relation_type, rel.confidence, evidence.len());
                relations.push(rel);
            }
            Err(e) => {
                eprintln!("  → Error: {}", e);
            }
        }

        // Rate limit: sleep 1 second between calls
        if idx < n - 1 {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }

    let mut dist: HashMap<String, usize> = HashMap::new();
    for r in &relations {
        *dist.entry(r.relation_type.clone()).or_insert(0) += 1;
    }

    let output = CompleteOutput {
        total_pairs: pairs.len(),
        completed_pairs: relations.len(),
        relation_type_distribution: dist,
        relations,
    };

    fs::create_dir_all("examples/project-07/data/output").ok();
    fs::write(
        "examples/project-07/data/output/completed-relations.json",
        serde_json::to_string_pretty(&output)?,
    )?;

    println!("\nDone. {} pairs completed.", output.completed_pairs);
    println!("Relation type distribution:");
    for (t, c) in &output.relation_type_distribution {
        println!("  {}: {}", t, c);
    }

    Ok(())
}
