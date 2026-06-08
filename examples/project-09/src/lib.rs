use std::fs;

use intent_llm::{extract_json, DeepSeekClient};
use serde::{Deserialize, Serialize};

// --- Graph data types ---

#[derive(Deserialize)]
struct GraphData {
    cluster_descriptions: Vec<ClusterDescription>,
    graph: Graph,
    keyword_index: Vec<KeywordEntry>,
    relation_types: Vec<RelationType>,
}

#[derive(Deserialize)]
struct ClusterDescription {
    id: u32,
    name: String,
    evolution: String,
}

#[derive(Deserialize)]
struct Graph {
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

// --- DiscoveryState (core new concept) ---

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscoveryState {
    pub explored_clusters: Vec<u32>,
    pub explored_node_ids: Vec<u32>,
    pub explored_edge_ids: Vec<u32>,
    pub open_questions: Vec<String>,
    pub insights: Vec<String>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        Self {
            explored_clusters: Vec::new(),
            explored_node_ids: Vec::new(),
            explored_edge_ids: Vec::new(),
            open_questions: Vec::new(),
            insights: Vec::new(),
        }
    }

    pub fn merge(&mut self, update: &DiscoveryUpdate) {
        for id in &update.new_clusters {
            if !self.explored_clusters.contains(id) {
                self.explored_clusters.push(*id);
            }
        }
        self.open_questions.retain(|q| !update.resolved_questions.contains(q));
        for q in &update.new_open_questions {
            if !self.open_questions.contains(q) {
                self.open_questions.push(q.clone());
            }
        }
        for ins in &update.new_insights {
            if !self.insights.contains(ins) {
                self.insights.push(ins.clone());
            }
        }
        self.prune();
    }

    fn prune(&mut self) {
        self.explored_clusters.truncate(10);
        self.explored_node_ids.truncate(20);
        self.explored_edge_ids.truncate(20);
        self.insights.truncate(5);
        self.open_questions.truncate(3);
    }
}

// --- Turn/parsed types (extended for multi-turn) ---

#[derive(Clone, Serialize, Deserialize)]
pub struct ParsedResponse {
    pub positioning: String,
    pub connections: String,
    pub exploration: String,
    pub discovery_update: DiscoveryUpdate,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscoveryUpdate {
    pub new_clusters: Vec<u32>,
    pub new_node_ids: Vec<u32>,
    pub new_edge_ids: Vec<u32>,
    pub resolved_questions: Vec<String>,
    pub new_open_questions: Vec<String>,
    pub new_insights: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Turn {
    pub id: String,
    pub timestamp: String,
    pub input: String,
    pub matched_clusters: Vec<ClusterMatch>,
    pub state_before: DiscoveryState,
    pub state_after: DiscoveryState,
    pub llm_raw: String,
    pub parsed: ParsedResponse,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClusterMatch {
    pub id: u32,
    pub name: String,
    pub score: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub id: String,
    pub created: String,
    pub turns: Vec<Turn>,
}

// --- MultiTurnEngine ---

pub struct MultiTurnEngine {
    data: GraphData,
    client: DeepSeekClient,
}

impl MultiTurnEngine {
    pub fn new(graph_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(graph_path).map_err(|e| format!("IO: {}", e))?;
        let data: GraphData = serde_json::from_str(&content).map_err(|e| format!("JSON: {}", e))?;
        let client = DeepSeekClient::from_env()?;
        Ok(Self { data, client })
    }

    pub fn process(&self, input: &str, state: &DiscoveryState) -> Result<(ParsedResponse, String), String> {
        let matched = self.match_with_history(input, state);
        let ids: Vec<u32> = matched.iter().map(|m| m.id).collect();
        let history_summary = self.build_history_summary(state);
        let cluster_lines = self.format_clusters(&matched);
        let edge_lines = self.format_edges_for(&ids);

        let prompt = format!(
            r#"你是一个基于用户个人意图图谱的思考脚手架。

## 探索历史

{}

## 匹配到的意图簇

{}

## 当前相关子图

{}

## 可用关系类型

{}

---

用户本轮输入：{}

任务：生成四层回复 JSON。

注意：
- discovery_update 中的 new_clusters / new_node_ids / new_edge_ids 必须列出**本轮文本中实际引用或暗示**到的簇/节点/边，不要留空。
- 如果本轮探索回答了前一轮的某个遗留问题，将其 id 或文本填入 resolved_questions。
- 示例：
  {{
    "positioning": "...",
    "connections": "...",
    "exploration": "...",
    "discovery_update": {{
      "new_clusters": [1, 5],
      "new_node_ids": [1, 5, 2],
      "new_edge_ids": [14],
      "resolved_questions": ["问题A的文本"],
      "new_open_questions": ["新问题1", "新问题2"],
      "new_insights": ["关键洞察1"]
    }}
  }}"#,
            history_summary, cluster_lines, edge_lines, self.relation_type_lines(), input
        );

        let raw = self.client.chat(&prompt)?;
        let parsed = parse_response(&raw);
        Ok((parsed, raw))
    }

    pub fn match_with_history(&self, text: &str, state: &DiscoveryState) -> Vec<ClusterMatch> {
        let tokens = bigrams(text);
        let mut results: Vec<ClusterMatch> = self.data.keyword_index.iter().map(|e| {
            let common = e.keywords.iter().filter(|kw| tokens.contains(kw)).count();
            let mut score = if e.keywords.is_empty() { 0.0 } else { common as f64 / e.keywords.len() as f64 };
            if state.explored_clusters.contains(&e.id) {
                score *= 1.5;
            }
            ClusterMatch { id: e.id, name: e.name.clone(), score }
        }).filter(|m| m.score > 0.02).collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(4);
        results
    }

    fn build_history_summary(&self, state: &DiscoveryState) -> String {
        let mut lines = Vec::new();
        if state.explored_clusters.is_empty() {
            return "尚无探索历史。".to_string();
        }
        let names: Vec<String> = state.explored_clusters.iter().map(|id| {
            self.data.cluster_descriptions.iter().find(|d| d.id == *id).map(|d| d.name.clone()).unwrap_or_default()
        }).collect();
        lines.push(format!("已探索簇：{} (ids: {:?})", names.join(", "), state.explored_clusters));
        if !state.insights.is_empty() {
            lines.push("关键洞察：".to_string());
            for ins in &state.insights {
                lines.push(format!("  - {}", ins));
            }
        }
        if !state.open_questions.is_empty() {
            lines.push("遗留问题：".to_string());
            for q in &state.open_questions {
                lines.push(format!("  - {}", q));
            }
        }
        lines.join("\n")
    }

    fn format_clusters(&self, matched: &[ClusterMatch]) -> String {
        matched.iter().map(|m| {
            let evo = self.data.cluster_descriptions.iter().find(|d| d.id == m.id).map(|d| d.evolution.as_str()).unwrap_or("");
            format!("- {}（簇{}）：{} 演化轨迹：{}", m.name, m.id, evo, m.score)
        }).collect::<Vec<_>>().join("\n")
    }

    fn format_edges_for(&self, ids: &[u32]) -> String {
        use std::collections::HashSet;
        let set: HashSet<u32> = ids.iter().copied().collect();
        let mut node_ids: HashSet<u32> = set.clone();
        for e in &self.data.graph.edges {
            if set.contains(&e.source) || set.contains(&e.target) {
                node_ids.insert(e.source);
                node_ids.insert(e.target);
            }
        }
        let nodes: Vec<&GraphNode> = self.data.graph.nodes.iter().filter(|n| node_ids.contains(&n.id)).collect();
        let edges: Vec<&GraphEdge> = self.data.graph.edges.iter().filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target)).collect();
        edges.iter().map(|e| {
            let src = nodes.iter().find(|n| n.id == e.source).map(|n| n.name.as_str()).unwrap_or("?");
            let tgt = nodes.iter().find(|n| n.id == e.target).map(|n| n.name.as_str()).unwrap_or("?");
            format!("{} → {} : {} [{}]", src, tgt, e.relation_type, e.category)
        }).collect::<Vec<_>>().join("\n")
    }

    fn relation_type_lines(&self) -> String {
        self.data.relation_types.iter().map(|t| format!("- {}：{}", t.name, t.description)).collect::<Vec<_>>().join("\n")
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

    pub fn load_or_create(&self) -> SessionFile {
        let path = format!("{}/session_multi.json", self.dir);
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(SessionFile {
                id: "multi".to_string(),
                created: ts(),
                turns: Vec::new(),
            })
    }

    pub fn save(&self, session: &SessionFile) {
        let path = format!("{}/session_multi.json", self.dir);
        fs::write(&path, serde_json::to_string_pretty(session).unwrap()).ok();
    }
}

// --- Helpers ---

fn bigrams(text: &str) -> Vec<String> {
    text.chars().collect::<Vec<_>>().windows(2).map(|w| w.iter().collect()).collect()
}

fn parse_response(raw: &str) -> ParsedResponse {
    let default = ParsedResponse {
        positioning: String::new(), connections: String::new(), exploration: String::new(),
        discovery_update: DiscoveryUpdate {
            new_clusters: Vec::new(), new_node_ids: Vec::new(), new_edge_ids: Vec::new(),
            resolved_questions: Vec::new(), new_open_questions: Vec::new(), new_insights: Vec::new(),
        },
    };
    let v = match extract_json(raw) {
        Ok(v) => v,
        Err(_) => return default,
    };
    ParsedResponse {
        positioning: v["positioning"].as_str().unwrap_or("").to_string(),
        connections: v["connections"].as_str().unwrap_or("").to_string(),
        exploration: v["exploration"].as_str().unwrap_or("").to_string(),
        discovery_update: DiscoveryUpdate {
            new_clusters: extract_u32_array(&v["discovery_update"]["new_clusters"]),
            new_node_ids: extract_u32_array(&v["discovery_update"]["new_node_ids"]),
            new_edge_ids: extract_u32_array(&v["discovery_update"]["new_edge_ids"]),
            resolved_questions: extract_string_array(&v["discovery_update"]["resolved_questions"]),
            new_open_questions: extract_string_array(&v["discovery_update"]["new_open_questions"]),
            new_insights: extract_string_array(&v["discovery_update"]["new_insights"]),
        },
    }
}

fn extract_u32_array(val: &serde_json::Value) -> Vec<u32> {
    val.as_array().map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect()).unwrap_or_default()
}

fn extract_string_array(val: &serde_json::Value) -> Vec<String> {
    val.as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default()
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}
