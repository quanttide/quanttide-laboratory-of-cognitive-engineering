use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ClusterDef {
    id: u32,
    name: String,
    #[serde(rename = "type")]
    cluster_type: Option<String>,
    weeks: Vec<String>,
    evolution: Option<String>,
}

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

fn build_cluster_desc(yaml_path: &str) -> (Vec<ClusterDef>, String) {
    let content = fs::read_to_string(yaml_path).expect("Failed to read intent.yaml");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("Failed to parse YAML");
    let mut clusters = Vec::new();
    let mut desc_lines = Vec::new();

    if let Some(arr) = yaml["clusters"].as_sequence() {
        for item in arr {
            let id = item["id"].as_u64().unwrap_or(0) as u32;
            let name = item["name"].as_str().unwrap_or("").to_string();
            let ctype = item["type"].as_str().map(|s| s.to_string());
            let weeks: Vec<String> = item["weeks"]
                .as_sequence()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let evolution = item["evolution"].as_str().map(|s| s.to_string());

            clusters.push(ClusterDef {
                id,
                name: name.clone(),
                cluster_type: ctype.clone(),
                weeks: weeks.clone(),
                evolution: evolution.clone(),
            });

            let mut per_week_strs: Vec<String> = Vec::new();
            if let Some(pw) = item["per_week"].as_mapping() {
                for (k, v) in pw {
                    let week = k.as_str().unwrap_or("");
                    if let Some(intents) = v.as_sequence() {
                        for intent in intents {
                            if let Some(text) = intent.as_str() {
                                per_week_strs.push(format!("  {}: {}", week, text));
                            }
                        }
                    }
                }
            }
            desc_lines.push(format!(
                "簇{}（{}）类型:{} 周次:{} 演化:{}",
                id,
                name,
                ctype.as_deref().unwrap_or("未知"),
                weeks.join(","),
                evolution.as_deref().unwrap_or("无"),
            ));
            for l in &per_week_strs {
                desc_lines.push(l.clone());
            }
            desc_lines.push(String::new());
        }
    }
    (clusters, desc_lines.join("\n"))
}

fn build_prompt(
    cluster_desc: &str,
    intent_md: &str,
    relation_md: &str,
) -> String {
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

注意：
- 寻找所有可能的隐含关系，不仅仅是显式的（YAML 中已有的边）
- 关注事件触发关系（一个簇的事件触发另一个簇）
- 关注演化关系（一个簇的演进推动另一个簇的演进）
- 关注时序前后关系
- 关注间接影响链
- 关注"潜行期"的关系——一个簇在休眠时如何被另一个簇影响
- 置信度 < 0.4 的请标记为"无关系"

输出必须是以下 JSON 格式（不要有任何其他文字）：

```json
{{
  "relations": [
    {{
      "from": 1,
      "to": 2,
      "relation_type": "支持/冲突/情感补给/触发/演化/时序先后/无关系",
      "direction": "单向/双向",
      "evidence": "简要的推理依据",
      "confidence": 0.85
    }}
  ]
}}
```

输出所有 45 对 (C(10,2)) 的关系推断。"#,
        cluster_desc, intent_md, relation_md
    )
}

fn call_llm(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY not set")?;

    let body = serde_json::json!({
        "model": "deepseek-chat",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 8192
    });

    let resp = ureq::post("https://api.deepseek.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(&body)?;

    let resp_json: serde_json::Value = resp.into_json()?;
    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();

    Ok(content)
}

fn parse_relations_from_response(response: &str) -> Vec<InferredRelation> {
    // Try to extract JSON from markdown code block
    let json_str = if let Some(start) = response.find("```json") {
        let start = start + 7;
        let end = response[start..].find("```").map(|i| start + i).unwrap_or(response.len());
        response[start..end].trim().to_string()
    } else if let Some(start) = response.find('{') {
        let end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        response[start..end].to_string()
    } else {
        response.to_string()
    };

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse JSON response: {}", e);
            eprintln!("Raw response (first 500 chars): {}", &response[..response.len().min(500)]);
            return vec![];
        }
    };

    let mut relations = Vec::new();
    if let Some(arr) = parsed["relations"].as_array() {
        for item in arr {
            let from = item["from"].as_u64().unwrap_or(0) as u32;
            let to = item["to"].as_u64().unwrap_or(0) as u32;
            if from == 0 || to == 0 || from == to {
                continue;
            }
            relations.push(InferredRelation {
                from: from.min(to),
                to: from.max(to),
                relation_type: item["relation_type"].as_str().unwrap_or("无关系").to_string(),
                direction: item["direction"].as_str().unwrap_or("单向").to_string(),
                evidence: item["evidence"].as_str().unwrap_or("").to_string(),
                confidence: item["confidence"].as_f64().unwrap_or(0.0),
                from_name: String::new(),
                to_name: String::new(),
            });
        }
    }
    relations
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let intent_yaml = "data/refined/intent.yaml";
    let intent_md = "data/analysis/intent.md";
    let relation_md = "data/analysis/intent-relation.md";

    let (clusters, cluster_desc) = build_cluster_desc(intent_yaml);
    let intent_analysis = read_file(intent_md);
    let relation_analysis = read_file(relation_md);

    let prompt = build_prompt(&cluster_desc, &intent_analysis, &relation_analysis);

    println!("Calling DeepSeek API...");
    let response = call_llm(&prompt)?;
    println!("Response received ({} chars)", response.len());

    let mut relations = parse_relations_from_response(&response);

    // Fill in cluster names
    let name_map: std::collections::HashMap<u32, String> = clusters
        .iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();
    for r in &mut relations {
        r.from_name = name_map.get(&r.from).cloned().unwrap_or_default();
        r.to_name = name_map.get(&r.to).cloned().unwrap_or_default();
    }

    let total_pairs = 45;
    let with_relations = relations.len();
    let coverage_rate = with_relations as f64 / total_pairs as f64;
    let new_edges = relations.iter().filter(|r| r.relation_type != "无关系").count();

    let mut fixed_clusters = Vec::new();
    let previously_isolated = [10u32];
    for &cid in &previously_isolated {
        let has_relation = relations.iter().any(|r| {
            (r.from == cid || r.to == cid) && r.relation_type != "无关系"
        });
        if has_relation {
            fixed_clusters.push(cid);
        }
    }

    let output = InferredRelationsOutput {
        total_pairs,
        with_relations,
        coverage_rate,
        new_edges_discovered: new_edges,
        previously_isolated_clusters_fixed: fixed_clusters,
        relations,
    };

    fs::create_dir_all("examples/project-07/data/output").ok();
    fs::write(
        "examples/project-07/data/output/inferred-relations.json",
        serde_json::to_string_pretty(&output)?,
    )?;

    println!("Inferred {} relations ({} non-none)", output.with_relations, output.new_edges_discovered);
    println!("Coverage: {:.1}%", output.coverage_rate * 100.0);
    if !output.previously_isolated_clusters_fixed.is_empty() {
        println!("Previously isolated clusters now connected: {:?}", output.previously_isolated_clusters_fixed);
    }
    println!("Output: data/output/inferred-relations.json");

    Ok(())
}
