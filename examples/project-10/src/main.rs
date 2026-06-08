use std::fs;
use std::io::{self, BufRead};

use intent_llm::{extract_json, DeepSeekClient};
use serde::{Deserialize, Serialize};

// --- Data types (graph + sessions) ---

#[derive(Deserialize)]
struct GraphFile {
    cluster_descriptions: Vec<ClusterDescription>,
    relation_types: Vec<RelationType>,
}

#[derive(Deserialize)]
struct ClusterDescription {
    id: u32,
    name: String,
    #[allow(dead_code)]
    r#type: String,
    evolution: String,
    per_week_intents: Vec<PerWeek>,
}

#[derive(Deserialize)]
struct PerWeek {
    week: String,
    intents: Vec<String>,
}

#[derive(Deserialize)]
struct RelationType {
    name: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct MotifReport {
    motif_statement: String,
    is_new_motif: bool,
    variations: Vec<MotifVariation>,
    motif_arc: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct MotifVariation {
    cluster_id: u32,
    form: String,
    week: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ParsedResponse {
    positioning: String,
    connections: String,
    exploration: String,
    motif: MotifReport,
}

#[derive(Clone, Serialize, Deserialize)]
struct Turn {
    id: String,
    timestamp: String,
    input: String,
    response: ParsedResponse,
}

#[derive(Clone, Serialize, Deserialize)]
struct SessionFile {
    id: String,
    created: String,
    turns: Vec<Turn>,
}

// --- MotifEngine ---

struct MotifEngine {
    clusters: Vec<ClusterDescription>,
    relation_types: Vec<RelationType>,
    client: DeepSeekClient,
}

impl MotifEngine {
    fn new(graph_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(graph_path).map_err(|e| format!("IO: {}", e))?;
        let file: GraphFile = serde_json::from_str(&content).map_err(|e| format!("JSON: {}", e))?;
        let client = DeepSeekClient::from_env()?;
        Ok(Self { clusters: file.cluster_descriptions, relation_types: file.relation_types, client })
    }

    fn process(&self, input: &str, history: &[Turn]) -> Result<ParsedResponse, String> {
        let cluster_block = self.format_clusters();
        let history_block = if history.is_empty() {
            "尚无探索历史。".to_string()
        } else {
            let mut lines = Vec::new();
            for (i, t) in history.iter().enumerate() {
                lines.push(format!("第{}轮：{}", i + 1, t.input));
                lines.push(format!("  → 母题发现：{}", t.response.motif.motif_statement));
                lines.push(format!("  → 涉及簇：{:?}", t.response.motif.variations.iter().map(|v| v.cluster_id).collect::<Vec<_>>()));
            }
            lines.join("\n")
        };
        let type_lines: Vec<String> = self.relation_types.iter()
            .map(|t| format!("- {}：{}", t.name, t.description)).collect();

        let prompt = format!(
            r#"你是一个基于用户个人意图图谱的母题发现引擎。

## 所有意图簇（共 10 个）

每个簇是一个持续关切的模式。请阅读每个簇的演化轨迹和周粒度意图，理解每个簇的核心关切。

{}

## 可用关系类型

{}

## 探索历史

{}

---

用户当前输入：{}

---

任务：根据用户输入和探索历史，按以下步骤分析：

**第 1 步：提取潜在关切**
用户没有明说但隐含的持续担忧或追求是什么？用一个简洁的关切断言表达。

**第 2 步：扫描全图**
遍历所有 10 个簇，逐个判断：这个簇的核心关切是否与潜在关切本质相同？
- 如果相同，给出证据（该簇的哪些意图表达了这一点）
- 按相关度排序

**第 3 步：生成母题报告**
如果 2+ 个簇表达了同一个关切，则生成母题报告。
如果只有 0-1 个，标记为新母题（is_new_motif=true），并建议关注后续是否重复。

返回 JSON 格式：

{{
  "positioning": "当前输入在全图中的定位",
  "connections": "与已有母题和簇的关系分析",
  "exploration": "建议的探索方向",
  "motif": {{
    "motif_statement": "关切断言",
    "is_new_motif": true/false,
    "variations": [
      {{"cluster_id": 1, "form": "在簇1中的具体表现形态", "week": "W23"}}
    ],
    "motif_arc": "这个关切在跨簇跨周的演化轨迹描述"
  }}
}}"#,
            cluster_block, type_lines.join("\n"), history_block, input
        );

        let raw = self.client.chat(&prompt)?;
        let parsed = parse_response(&raw);
        Ok(parsed)
    }

    fn format_clusters(&self) -> String {
        let mut lines = Vec::new();
        for c in &self.clusters {
            lines.push(format!("--- 簇{}：{}（{}）---", c.id, c.name, c.evolution));
            for pw in &c.per_week_intents {
                for intent in &pw.intents {
                    lines.push(format!("  {}: {}", pw.week, intent));
                }
            }
        }
        lines.join("\n")
    }
}

// --- SessionManager ---

struct SessionManager {
    dir: String,
}

impl SessionManager {
    fn new(dir: &str) -> Self {
        fs::create_dir_all(dir).ok();
        Self { dir: dir.to_string() }
    }
    fn load(&self) -> SessionFile {
        let path = format!("{}/session_p10.json", self.dir);
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(SessionFile { id: "p10".into(), created: ts(), turns: Vec::new() })
    }
    fn save(&self, session: &SessionFile) {
        let path = format!("{}/session_p10.json", self.dir);
        fs::write(&path, serde_json::to_string_pretty(session).unwrap()).ok();
    }
}

// --- Helpers ---

fn parse_response(raw: &str) -> ParsedResponse {
    let default = || ParsedResponse {
        positioning: String::new(), connections: String::new(), exploration: String::new(),
        motif: MotifReport {
            motif_statement: raw.to_string(), is_new_motif: true,
            variations: Vec::new(), motif_arc: String::new(),
        },
    };
    let v = match extract_json(raw) {
        Ok(v) => v,
        Err(_) => return default(),
    };
    ParsedResponse {
        positioning: v["positioning"].as_str().unwrap_or("").to_string(),
        connections: v["connections"].as_str().unwrap_or("").to_string(),
        exploration: v["exploration"].as_str().unwrap_or("").to_string(),
        motif: MotifReport {
            motif_statement: v["motif"]["motif_statement"].as_str().unwrap_or("").to_string(),
            is_new_motif: v["motif"]["is_new_motif"].as_bool().unwrap_or(true),
            variations: v["motif"]["variations"].as_array().map(|arr| {
                arr.iter().filter_map(|item| {
                    Some(MotifVariation {
                        cluster_id: item["cluster_id"].as_u64()? as u32,
                        form: item["form"].as_str()?.to_string(),
                        week: item["week"].as_str()?.to_string(),
                    })
                }).collect()
            }).unwrap_or_default(),
            motif_arc: v["motif"]["motif_arc"].as_str().unwrap_or("").to_string(),
        },
    }
}

fn ts() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}

fn main() -> Result<(), String> {
    let engine = MotifEngine::new("data/formal/intent-graph.json")?;
    let sessions = SessionManager::new("examples/project-10/data");

    println!("=== project-10: 意图即母题 — 跨簇共鸣 ===");
    println!("输入你的想法，脚手架会识别它在变奏哪个已有的关切。\n");

    let stdin = io::stdin();
    let mut session = sessions.load();
    let mut history: Vec<Turn> = session.turns.clone();
    let mut n = history.len();

    for line in stdin.lock().lines() {
        let input = line.map_err(|e| e.to_string())?;
        let input = input.trim();
        if input.is_empty() { continue; }
        if input == "exit" { break; }

        n += 1;
        println!("  → 正在扫描全图，追踪潜在关切...");

        let parsed = engine.process(input, &history)?;
        let turn = Turn {
            id: format!("{}_{}", ts(), n),
            timestamp: ts(),
            input: input.to_string(),
            response: parsed.clone(),
        };
        history.push(turn.clone());
        session.turns.push(turn);
        sessions.save(&session);

        println!("\n---");
        if !parsed.positioning.is_empty() { println!("📍 {}", parsed.positioning); }
        if !parsed.connections.is_empty() { println!("🔗 {}", parsed.connections); }
        if !parsed.exploration.is_empty() { println!("💡 {}", parsed.exploration); }

        let m = &parsed.motif;
        if !m.motif_statement.is_empty() {
            if m.is_new_motif {
                println!("🆕 新母题：{}", m.motif_statement);
            } else {
                println!("🎵 发现母题：{}", m.motif_statement);
                println!("   变奏：");
                for v in &m.variations {
                    println!("   簇{} ({}): {}", v.cluster_id, v.week, v.form);
                }
            }
        }
        if !m.motif_arc.is_empty() {
            println!("   演化弧：{}", m.motif_arc);
        }
        println!("---\n");
    }

    Ok(())
}
