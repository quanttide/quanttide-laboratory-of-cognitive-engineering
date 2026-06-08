use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};

use intent_llm::{extract_json, DeepSeekClient};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ScaffoldData {
    cluster_descriptions: Vec<ClusterDescription>,
    graph: GraphData,
    keyword_index: Vec<KeywordEntry>,
    relation_types: Vec<RelationType>,
}

#[derive(Deserialize)]
struct ClusterDescription {
    id: u32,
    name: String,
    r#type: String,
    weeks: Vec<String>,
    evolution: String,
    per_week_intents: Vec<PerWeekIntent>,
}

#[derive(Deserialize)]
struct PerWeekIntent {
    week: String,
    intents: Vec<String>,
}

#[derive(Deserialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Deserialize, Clone, Serialize)]
struct GraphNode {
    id: u32,
    name: String,
}

#[derive(Deserialize, Clone, Serialize)]
struct GraphEdge {
    source: u32,
    target: u32,
    relation_type: String,
    category: String,
    weeks: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct KeywordEntry {
    id: u32,
    name: String,
    keywords: Vec<String>,
}

#[derive(Deserialize)]
struct RelationType {
    name: String,
    description: String,
}

// --- Session types ---

#[derive(Deserialize, Serialize, Clone)]
struct SessionFile {
    id: String,
    created: String,
    turns: Vec<Turn>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Turn {
    id: String,
    timestamp: String,
    input: TurnInput,
    retrieved_context: RetrievedContext,
    llm_response: TurnLlmResponse,
}

#[derive(Deserialize, Serialize, Clone)]
struct TurnInput {
    text: String,
    matched_clusters: Vec<ClusterMatch>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ClusterMatch {
    id: u32,
    name: String,
    score: f64,
}

#[derive(Deserialize, Serialize, Clone)]
struct RetrievedContext {
    subgraph_nodes: Vec<GraphNode>,
    subgraph_edges: Vec<GraphEdge>,
}

#[derive(Deserialize, Serialize, Clone)]
struct TurnLlmResponse {
    raw: String,
    parsed: ParsedResponse,
}

#[derive(Deserialize, Serialize, Clone)]
struct ParsedResponse {
    positioning: String,
    connections: String,
    exploration: String,
}

// --- Matching ---

fn tokenize(text: &str) -> Vec<String> {
    text.chars()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| w.iter().collect())
        .collect()
}

fn match_clusters(input: &str, kw_index: &[KeywordEntry]) -> Vec<ClusterMatch> {
    let tokens = tokenize(input);
    let mut results: Vec<ClusterMatch> = kw_index
        .iter()
        .map(|entry| {
            let common = entry
                .keywords
                .iter()
                .filter(|kw| tokens.contains(kw))
                .count();
            let score = if entry.keywords.is_empty() {
                0.0
            } else {
                common as f64 / entry.keywords.len() as f64
            };
            ClusterMatch {
                id: entry.id,
                name: entry.name.clone(),
                score,
            }
        })
        .filter(|m| m.score > 0.02)
        .collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(4);
    results
}

fn build_subgraph(
    matched_ids: &[u32],
    graph: &GraphData,
) -> RetrievedContext {
    let ids: std::collections::HashSet<u32> = matched_ids.iter().copied().collect();
    let mut node_ids: std::collections::HashSet<u32> = ids.clone();
    for e in &graph.edges {
        if ids.contains(&e.source) || ids.contains(&e.target) {
            node_ids.insert(e.source);
            node_ids.insert(e.target);
        }
    }
    let subgraph_nodes: Vec<GraphNode> = graph
        .nodes
        .iter()
        .filter(|n| node_ids.contains(&n.id))
        .cloned()
        .collect();
    let subgraph_edges: Vec<GraphEdge> = graph
        .edges
        .iter()
        .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
        .cloned()
        .collect();
    RetrievedContext {
        subgraph_nodes,
        subgraph_edges,
    }
}

fn build_prompt(
    input: &str,
    matched: &[ClusterMatch],
    context: &RetrievedContext,
    descriptions: &[ClusterDescription],
    relation_types: &[RelationType],
) -> String {
    let cluster_lines: Vec<String> = matched
        .iter()
        .map(|m| {
            let desc = descriptions.iter().find(|d| d.id == m.id);
            let evo = desc.map(|d| d.evolution.as_str()).unwrap_or("");
            format!("- {}（簇{}）：{} 演化轨迹：{}", m.name, m.id, evo, m.score)
        })
        .collect();

    let edge_lines: Vec<String> = context
        .subgraph_edges
        .iter()
        .map(|e| {
            let src = context
                .subgraph_nodes
                .iter()
                .find(|n| n.id == e.source)
                .map(|n| n.name.as_str())
                .unwrap_or("?");
            let tgt = context
                .subgraph_nodes
                .iter()
                .find(|n| n.id == e.target)
                .map(|n| n.name.as_str())
                .unwrap_or("?");
            format!("{} → {} : {} [{}]", src, tgt, e.relation_type, e.category)
        })
        .collect();

    let type_lines: Vec<String> = relation_types
        .iter()
        .map(|t| format!("- {}：{}", t.name, t.description))
        .collect();

    format!(
        r#"你是一个基于用户个人意图图谱的思考脚手架。以下是用户的意图图谱信息：

## 匹配到的意图簇

{}

## 相关子图（这些簇之间的已知关系）

{}

## 可用关系类型

{}

---

用户的新想法：{}

任务：基于以上图谱信息，生成一个三层脚手架回复：

1. **定位**：这个想法落在用户的哪个意图簇范围内，为什么
2. **连接**：这个想法与用户图谱中的哪些已知关系相关（引用具体的边）
3. **探索**：基于图谱结构，向用户提出 1-2 个未探索的思考方向

输出 JSON 格式：
{{
  "positioning": "...",
  "connections": "...",
  "exploration": "..."
}}"#,
        cluster_lines.join("\n"),
        edge_lines.join("\n"),
        type_lines.join("\n"),
        input
    )
}

fn parse_scaffold_response(raw: &str) -> ParsedResponse {
    if let Ok(v) = extract_json(raw) {
        ParsedResponse {
            positioning: v["positioning"].as_str().unwrap_or("").to_string(),
            connections: v["connections"].as_str().unwrap_or("").to_string(),
            exploration: v["exploration"].as_str().unwrap_or("").to_string(),
        }
    } else {
        ParsedResponse {
            positioning: raw.lines().nth(0).unwrap_or("").to_string(),
            connections: String::new(),
            exploration: String::new(),
        }
    }
}

fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    format!("{}", now.as_secs())
}

fn session_file_name() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    format!("examples/project-08/data/sessions/session_{}.json", now.as_secs())
}

fn save_turn(session_path: &str, turn: &Turn) {
    let mut session: SessionFile = fs::read_to_string(session_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(SessionFile {
            id: session_path.split('/').last().unwrap_or("unknown").to_string(),
            created: timestamp(),
            turns: Vec::new(),
        });
    session.turns.push(turn.clone());
    fs::write(session_path, serde_json::to_string_pretty(&session).unwrap()).ok();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_path = "data/formal/scaffold-data.json";
    let sessions_dir = "examples/project-08/data/sessions";

    let data: ScaffoldData =
        serde_json::from_str(&fs::read_to_string(data_path)?)?;
    let client = DeepSeekClient::from_env().map_err(|e| e.to_string())?;
    fs::create_dir_all(sessions_dir).ok();
    let session_path = session_file_name();

    println!("=== GraphRAG Scaffold ===");
    println!("Loaded: {} clusters, {} edges", data.cluster_descriptions.len(), data.graph.edges.len());
    println!("Type your thoughts, or 'exit' to quit.\n");

    let stdin = io::stdin();
    let mut turn_count = 0usize;

    for line in stdin.lock().lines() {
        let input = line?;
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }

        turn_count += 1;
        let ts = timestamp();
        let turn_id = format!("{}_{}", ts, turn_count);

        // 1. Match clusters
        let matched = match_clusters(input, &data.keyword_index);
        let matched_ids: Vec<u32> = matched.iter().map(|m| m.id).collect();

        // 2. Retrieve subgraph
        let context = build_subgraph(&matched_ids, &data.graph);

        // 3. Build prompt and call LLM
        let prompt = build_prompt(input, &matched, &context, &data.cluster_descriptions, &data.relation_types);
        println!("  → Calling DeepSeek...");
        let raw_response = client.chat(&prompt).map_err(|e| e.to_string())?;
        let parsed = parse_scaffold_response(&raw_response);

        // 4. Save turn
        let turn = Turn {
            id: turn_id,
            timestamp: ts,
            input: TurnInput {
                text: input.to_string(),
                matched_clusters: matched,
            },
            retrieved_context: context,
            llm_response: TurnLlmResponse {
                raw: raw_response,
                parsed: parsed,
            },
        };
        save_turn(&session_path, &turn);

        // 5. Print response
        println!("\n---");
        if !turn.llm_response.parsed.positioning.is_empty() {
            println!("📍 {}", turn.llm_response.parsed.positioning);
        }
        if !turn.llm_response.parsed.connections.is_empty() {
            println!("🔗 {}", turn.llm_response.parsed.connections);
        }
        if !turn.llm_response.parsed.exploration.is_empty() {
            println!("💡 {}", turn.llm_response.parsed.exploration);
        }
        println!("---\n");
    }

    println!("Session saved to {}", session_path);
    Ok(())
}
