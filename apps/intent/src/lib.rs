use std::fs;

use intent_llm::{extract_json, DeepSeekClient};
use serde::{Deserialize, Serialize};

// --- Graph data types ---

#[derive(Deserialize)]
pub struct GraphData {
    cluster_descriptions: Vec<ClusterDescription>,
    keyword_index: Vec<KeywordEntry>,
    relation_types: Vec<RelationType>,
}

#[derive(Deserialize)]
pub struct ClusterDescription {
    pub id: u32,
    pub name: String,
    pub evolution: String,
    pub per_week_intents: Vec<PerWeek>,
}

#[derive(Deserialize)]
pub struct PerWeek {
    pub week: String,
    pub intents: Vec<String>,
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

// --- DiscoveryState (multi-turn accumulation) ---

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

// --- Turn/parsed types (with motif) ---

#[derive(Deserialize, Serialize, Clone)]
pub struct ParsedResponse {
    pub positioning: String,
    pub connections: String,
    pub exploration: String,
    pub discovery_update: DiscoveryUpdate,
    pub motif: Option<MotifReport>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MotifReport {
    pub motif_statement: String,
    pub is_new_motif: bool,
    pub variations: Vec<MotifVariation>,
    pub motif_arc: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MotifVariation {
    pub cluster_id: u32,
    pub form: String,
    pub week: String,
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

    /// Process with multi-turn state accumulation and motif discovery
    pub fn process_with_state(&self, input: &str, state: &DiscoveryState) -> Result<(ParsedResponse, String), String> {
        let prompt = self.build_motif_prompt(input, state);
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

    fn build_motif_prompt(&self, input: &str, state: &DiscoveryState) -> String {
        let history = self.build_history_summary(state);
        let all_clusters = self.format_all_clusters();
        let type_lines: Vec<String> = self.data.relation_types.iter().map(|t| format!("- {}：{}", t.name, t.description)).collect();

        format!(
            r#"你是一个基于用户个人意图图谱的母题发现引擎。

## 所有意图簇（共 10 个）

{}

## 可用关系类型

{}

## 探索历史

{}

---

用户当前输入：{}

---

任务：分三步分析，返回 JSON。

**第 1 步：提取潜在关切**——用户没明说但隐含的持续担忧是什么？

**第 2 步：扫描全图**——遍历所有 10 个簇，判断哪些表达了同一个潜在关切。按相关度排序。

**第 3 步：生成母题报告**——如果 2+ 簇表达了同一关切，生成 motif 报告；否则标记 is_new_motif=true。

{{
  "positioning": "当前输入在全图中的定位",
  "connections": "与已有簇和母题的关系",
  "exploration": "探索方向",
  "discovery_update": {{
    "new_clusters": [],
    "new_node_ids": [],
    "new_edge_ids": [],
    "resolved_questions": [],
    "new_open_questions": [],
    "new_insights": []
  }},
  "motif": {{
    "motif_statement": "关切断言",
    "is_new_motif": true/false,
    "variations": [
      {{"cluster_id": 1, "form": "具体表现形态", "week": "W23"}}
    ],
    "motif_arc": "跨簇跨周的演化轨迹描述"
  }}
}}"#,
            all_clusters, type_lines.join("\n"), history, input
        )
    }

    fn format_all_clusters(&self) -> String {
        let mut lines = Vec::new();
        for c in &self.data.cluster_descriptions {
            lines.push(format!("--- 簇{}：{}（{}）---", c.id, c.name, c.evolution));
            for pw in &c.per_week_intents {
                for intent in &pw.intents {
                    lines.push(format!("  {}: {}", pw.week, intent));
                }
            }
        }
        lines.join("\n")
    }

    fn build_history_summary(&self, state: &DiscoveryState) -> String {
        if state.explored_clusters.is_empty() {
            return "尚无探索历史。".to_string();
        }
        let mut lines = Vec::new();
        let names: Vec<String> = state.explored_clusters.iter().map(|id| {
            self.data.cluster_descriptions.iter().find(|d| d.id == *id).map(|d| d.name.clone()).unwrap_or_default()
        }).collect();
        lines.push(format!("已探索簇：{} (ids: {:?})", names.join(", "), state.explored_clusters));
        if !state.insights.is_empty() {
            lines.push("关键洞察：".to_string());
            for ins in &state.insights { lines.push(format!("  - {}", ins)); }
        }
        if !state.open_questions.is_empty() {
            lines.push("遗留问题：".to_string());
            for q in &state.open_questions { lines.push(format!("  - {}", q)); }
        }
        lines.join("\n")
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
        let path = format!("{}/session.json", self.dir);
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(SessionFile {
                id: "current".to_string(), created: ts(), turns: Vec::new(),
            })
    }

    pub fn save(&self, session: &SessionFile) {
        let path = format!("{}/session.json", self.dir);
        fs::write(&path, serde_json::to_string_pretty(session).unwrap()).ok();
    }
}

// --- Helpers ---

fn parse_response(raw: &str) -> ParsedResponse {
    let default = || ParsedResponse {
        positioning: String::new(), connections: String::new(), exploration: String::new(),
        discovery_update: DiscoveryUpdate {
            new_clusters: Vec::new(), new_node_ids: Vec::new(), new_edge_ids: Vec::new(),
            resolved_questions: Vec::new(), new_open_questions: Vec::new(), new_insights: Vec::new(),
        },
        motif: None,
    };
    let v = match extract_json(raw) {
        Ok(v) => v,
        Err(_) => return default(),
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
        motif: extract_motif(&v["motif"]),
    }
}

fn extract_motif(val: &serde_json::Value) -> Option<MotifReport> {
    if val.is_null() || !val.is_object() { return None; }
    let statement = val["motif_statement"].as_str().unwrap_or("").to_string();
    if statement.is_empty() { return None; }
    Some(MotifReport {
        motif_statement: statement,
        is_new_motif: val["is_new_motif"].as_bool().unwrap_or(true),
        variations: val["variations"].as_array().map(|arr| {
            arr.iter().filter_map(|item| {
                Some(MotifVariation {
                    cluster_id: item["cluster_id"].as_u64()? as u32,
                    form: item["form"].as_str()?.to_string(),
                    week: item["week"].as_str()?.to_string(),
                })
            }).collect()
        }).unwrap_or_default(),
        motif_arc: val["motif_arc"].as_str().unwrap_or("").to_string(),
    })
}

fn extract_u32_array(val: &serde_json::Value) -> Vec<u32> {
    val.as_array().map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect()).unwrap_or_default()
}

fn extract_string_array(val: &serde_json::Value) -> Vec<String> {
    val.as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default()
}

fn bigrams(text: &str) -> Vec<String> {
    text.chars().collect::<Vec<_>>().windows(2).map(|w| w.iter().collect()).collect()
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}
