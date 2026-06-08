use std::fs;

use intent_llm::{extract_json, DeepSeekClient};
use serde::{Deserialize, Serialize};

// --- Data types ---

#[derive(Deserialize)]
pub struct GraphData {
    cluster_descriptions: Vec<ClusterDescription>,
    graph: Graph,
    keyword_index: Vec<KeywordEntry>,
    relation_types: Vec<RelationType>,
}

#[derive(Deserialize)]
pub struct ClusterDescription {
    pub id: u32,
    pub name: String,
    pub evolution: String,
}

#[derive(Deserialize)]
struct Graph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct GraphNode {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct GraphEdge {
    pub source: u32,
    pub target: u32,
    pub relation_type: String,
    pub category: String,
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
pub struct SessionFile {
    pub id: String,
    pub created: String,
    pub turns: Vec<Turn>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Turn {
    pub id: String,
    pub timestamp: String,
    pub input: TurnInput,
    pub retrieved_context: RetrievedContext,
    pub llm_response: TurnLlmResponse,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TurnInput {
    pub text: String,
    pub matched_clusters: Vec<ClusterMatch>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ClusterMatch {
    pub id: u32,
    pub name: String,
    pub score: f64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RetrievedContext {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TurnLlmResponse {
    pub raw: String,
    pub parsed: ParsedResponse,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ParsedResponse {
    pub positioning: String,
    pub connections: String,
    pub exploration: String,
}

// --- ScaffoldEngine ---

pub struct ScaffoldEngine {
    data: GraphData,
    client: DeepSeekClient,
}

impl ScaffoldEngine {
    pub fn new(graph_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(graph_path).map_err(|e| format!("IO: {}", e))?;
        let data: GraphData = serde_json::from_str(&content).map_err(|e| format!("JSON: {}", e))?;
        let client = DeepSeekClient::from_env()?;
        Ok(Self { data, client })
    }

    pub fn process(&self, input: &str) -> Result<(ParsedResponse, String), String> {
        let matched = self.match_clusters(input);
        let ids: Vec<u32> = matched.iter().map(|m| m.id).collect();
        let ctx = self.retrieve(&ids);
        let prompt = self.build_prompt(input, &matched, &ctx);
        let raw = self.client.chat(&prompt)?;
        let parsed = parse_response(&raw);
        Ok((parsed, raw))
    }

    pub fn build_turn(
        &self, input: &str, parsed: &ParsedResponse, raw: &str,
        turn_id: &str, ts: &str,
    ) -> Turn {
        let matched = self.match_clusters(input);
        let ids: Vec<u32> = matched.iter().map(|m| m.id).collect();
        let ctx = self.retrieve(&ids);
        Turn {
            id: turn_id.to_string(),
            timestamp: ts.to_string(),
            input: TurnInput {
                text: input.to_string(),
                matched_clusters: matched,
            },
            retrieved_context: ctx,
            llm_response: TurnLlmResponse {
                raw: raw.to_string(),
                parsed: parsed.clone(),
            },
        }
    }

    fn match_clusters(&self, text: &str) -> Vec<ClusterMatch> {
        let tokens = bigrams(text);
        let mut results: Vec<ClusterMatch> = self.data.keyword_index.iter().map(|e| {
            let common = e.keywords.iter().filter(|kw| tokens.contains(kw)).count();
            let score = if e.keywords.is_empty() { 0.0 } else { common as f64 / e.keywords.len() as f64 };
            ClusterMatch { id: e.id, name: e.name.clone(), score }
        }).filter(|m| m.score > 0.02).collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(4);
        results
    }

    fn retrieve(&self, ids: &[u32]) -> RetrievedContext {
        use std::collections::HashSet;
        let set: HashSet<u32> = ids.iter().copied().collect();
        let mut node_ids: HashSet<u32> = set.clone();
        for e in &self.data.graph.edges {
            if set.contains(&e.source) || set.contains(&e.target) {
                node_ids.insert(e.source);
                node_ids.insert(e.target);
            }
        }
        RetrievedContext {
            nodes: self.data.graph.nodes.iter().filter(|n| node_ids.contains(&n.id)).cloned().collect(),
            edges: self.data.graph.edges.iter().filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target)).cloned().collect(),
        }
    }

    fn build_prompt(&self, input: &str, matched: &[ClusterMatch], ctx: &RetrievedContext) -> String {
        let cluster_lines: Vec<String> = matched.iter().map(|m| {
            let evo = self.data.cluster_descriptions.iter().find(|d| d.id == m.id).map(|d| d.evolution.as_str()).unwrap_or("");
            format!("- {}（簇{}）：{} 演化轨迹：{}", m.name, m.id, evo, m.score)
        }).collect();
        let edge_lines: Vec<String> = ctx.edges.iter().map(|e| {
            let src = ctx.nodes.iter().find(|n| n.id == e.source).map(|n| n.name.as_str()).unwrap_or("?");
            let tgt = ctx.nodes.iter().find(|n| n.id == e.target).map(|n| n.name.as_str()).unwrap_or("?");
            format!("{} → {} : {} [{}]", src, tgt, e.relation_type, e.category)
        }).collect();
        let type_lines: Vec<String> = self.data.relation_types.iter().map(|t| format!("- {}：{}", t.name, t.description)).collect();

        format!(
            r#"你是一个基于用户个人意图图谱的思考脚手架。

## 匹配到的意图簇

{}

## 相关子图

{}

## 可用关系类型

{}

---

用户的新想法：{}

任务：生成三层脚手架回复（JSON）：{{"positioning":"...","connections":"...","exploration":"..."}}"#,
            cluster_lines.join("\n"), edge_lines.join("\n"), type_lines.join("\n"), input
        )
    }
}

// --- SessionManager ---

pub struct SessionManager {
    dir: String,
}

impl SessionManager {
    pub fn new(dir: &str) -> Self {
        fs::create_dir_all(dir).ok();
        Self { dir: dir.to_string() }
    }

    pub fn save_turn(&self, turn: &Turn) {
        let path = format!("{}/session.json", self.dir);
        let mut session: SessionFile = fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(SessionFile {
                id: "current".to_string(), created: ts(), turns: Vec::new(),
            });
        session.turns.push(turn.clone());
        fs::write(&path, serde_json::to_string_pretty(&session).unwrap()).ok();
    }
}

// --- Helpers ---

fn bigrams(text: &str) -> Vec<String> {
    text.chars().collect::<Vec<_>>().windows(2).map(|w| w.iter().collect()).collect()
}

fn parse_response(raw: &str) -> ParsedResponse {
    if let Ok(v) = extract_json(raw) {
        ParsedResponse {
            positioning: v["positioning"].as_str().unwrap_or("").to_string(),
            connections: v["connections"].as_str().unwrap_or("").to_string(),
            exploration: v["exploration"].as_str().unwrap_or("").to_string(),
        }
    } else {
        ParsedResponse { positioning: raw.to_string(), connections: String::new(), exploration: String::new() }
    }
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}
