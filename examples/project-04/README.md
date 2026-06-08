# 项目四：方案B — 图谱推理意图识别

基于 petgraph 知识图谱的图遍历意图推理，利用关系边发现隐含关联。

## 核心问题

通过知识图谱的图遍历，能否发现纯文本匹配无法发现的隐含关切关联？

## 技术栈

- Rust + [petgraph](https://crates.io/crates/petgraph)（图数据结构 + BFS 等图算法）
- serde + serde_json（YAML/JSON 序列化）
- serde_yaml（YAML 反序列化）
- 单 binary：`approach_b`

## 任务

### 4.1 构建知识图谱

从 YAML 加载并构建 petgraph 有向图。

**输入**：`../../../assets/refined/intent.yaml` + `../../../assets/refined/intent-relation.yaml`

**方法**：
1. 从 `intent.yaml` 提取 10 个节点（id, name, type, per_week intents）
2. 从 `intent-relation.yaml` 提取边（source, target, type, logic, weeks），包含 stable/periodic/situational 三类
3. 用 petgraph `Graph::new_undirected()` 构建无向图（边带 relation_type 方向属性）

**输出**：`data/graph.json`（序列化后的图）+ `src/graph.rs`

```rust
// src/graph.rs
use petgraph::graph::{Graph, NodeIndex};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NodeWeight {
    id: u32,
    name: String,
    r#type: String,       // "持续关切" | "主干议题" | "单周验证"
    evolution: String,
    per_week_intents: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EdgeWeight {
    relation_type: String,  // "支持" | "冲突" | "依赖" | "包含"
    logic: String,
    weeks: Vec<String>,
}

type IntentGraph = Graph<NodeWeight, EdgeWeight>;

impl IntentGraph {
    /// 从 YAML 文件加载构建图
    fn from_yaml(intent_path: &str, relation_path: &str) -> Self;

    /// 保存图为 JSON
    fn save_json(&self, path: &str);

    /// 从 JSON 加载图
    fn load_json(path: &str) -> Self;

    /// 给定文本关键词，返回匹配的节点 ID 列表（token 级匹配，与方案 A 同一逻辑）
    fn match_nodes(&self, keywords: &[String]) -> Vec<NodeIndex>;

    /// 返回节点的直接邻居及连接边
    fn neighbors(&self, node: NodeIndex) -> Vec<(NodeIndex, NodeIndex, EdgeWeight)>;

    /// BFS 遍历：从节点出发走 n 跳，返回所有路径
    fn bfs(&self, start: NodeIndex, max_depth: usize) -> Vec<Vec<(NodeIndex, NodeIndex, EdgeWeight)>>;

    /// 冲突检测：检查匹配的节点之间是否有冲突路径
    fn detect_conflicts(&self, nodes: &[NodeIndex]) -> Vec<(NodeIndex, NodeIndex, EdgeWeight)>;

    /// 完整推理：输入文本，输出推理结果 JSON
    fn infer(&self, text: &str) -> Value;
}
```

### 4.2 推理执行

**输入**：`data/graph.json` + stdin 文本

**算法**：
1. **节点匹配**：对文本分词，调用 `graph.match_nodes()` 匹配起始节点
2. **邻居发现**：对每个匹配节点，调用 `graph.neighbors()` 列出直接邻居
3. **多跳路径**：对每个匹配节点，调用 `graph.bfs()` 遍历 2 跳，输出推理路径
4. **冲突检测**：调用 `graph.detect_conflicts()` 检查匹配节点间是否存在冲突路径

**输出**：stdout JSON

```json
{
  "text": "输入文本片段",
  "match_nodes": [
    {"id": 1, "name": "研发方法论", "evidence": ["研发", "效率"]},
    {"id": 3, "name": "认知工程", "evidence": ["认知"]}
  ],
  "neighbors": [
    {"from": 1, "to": 2, "relation": "支持", "logic": "方法论驱动反...", "depth": 1},
    {"from": 1, "to": 6, "relation": "依赖", "logic": "方法论需要平...", "depth": 1}
  ],
  "bfs_paths": [
    [{"from": 1, "to": 2, "relation": "支持"}, {"from": 2, "to": 5, "relation": "冲突"}]
  ],
  "conflicts": [
    {"node_a": 1, "node_b": 8, "relation_type": "冲突", "via": [2, 5]}
  ]
}
```

### 4.3 执行

```bash
echo "测试文本..." | cargo run --bin approach_b
```

**源码**：`src/graph.rs` + `src/main.rs`
