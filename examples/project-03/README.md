# 项目三：图谱推理 vs 文本匹配对比实验

## 核心问题

对于一段新输入文本，**使用知识图谱的推理**比**纯文本关键词匹配**能否发现更多有价值的隐含关联？

## 技术栈

- Rust + [petgraph](https://crates.io/crates/petgraph)（图数据结构 + BFS 等图算法）
- serde + serde_json（YAML/JSON 序列化）
- 两个 binary：`approach_a`（文本匹配）+ `approach_b`（图谱推理）

## 验证方法

同一段输入 → 走两条技术路线 → 对比输出质量

```
输入文本
    ├─→ 方案A：纯文本分词 + 关键词匹配（不用图）
    └─→ 方案B：加载知识图谱 + BFS 遍历（用图）
                ↓
         对比：B 比 A 多发现了什么？
```

## 任务

### 3.1 构建知识图谱

从 YAML 加载并构建有向图。

**输入**：`../../../assets/refined/intent.yaml` + `../../../assets/refined/intent-relation.yaml`

**方法**：
1. 从 `intent.yaml` 提取 10 个节点（id, name, type, per_week intents）
2. 从 `intent-relation.yaml` 提取边（source, target, type, logic, weeks），包含 stable/periodic/situational 三类
3. 序列化为 `data/graph.json`

**输出**：`data/graph.json` + `src/graph.rs`（基于 petgraph 的图数据结构）

```rust
// src/graph.rs
use petgraph::graph::{Graph, NodeIndex};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct NodeWeight {
    id: u32,
    name: String,
    r#type: String,     // "持续关切" | "主干议题" | "单周验证"
    evolution: String,
}

#[derive(Serialize, Deserialize)]
struct EdgeWeight {
    relation_type: String,  // "支持" | "冲突" | "依赖" | "包含"
    logic: String,
}

// 实际用于图计算的类型
type IntentGraph = Graph<NodeWeight, EdgeWeight>;

impl IntentGraph {
    // 从 YAML 加载构建
    fn from_yaml(intent_path: &str, relation_path: &str) -> Self

    // 匹配：给定文本关键词，返回匹配的节点 ID 列表（方案 A + B 共用）
    fn match_nodes(&self, keywords: &[String]) -> Vec<NodeIndex>

    // 邻居发现：返回节点的直接邻居及连接边
    fn neighbors(&self, node: NodeIndex) -> Vec<(NodeIndex, NodeIndex, &EdgeWeight)>

    // BFS 遍历：从节点出发走 n 跳
    fn bfs(&self, start: NodeIndex, max_depth: usize) -> Vec<Vec<(NodeIndex, NodeIndex, &EdgeWeight)>>

    // 冲突检测：检查匹配的节点之间是否有冲突路径
    fn detect_conflicts(&self, nodes: &[NodeIndex]) -> Vec<(NodeIndex, NodeIndex)>

    // 导出推理结果 JSON
    fn infer(&self, text: &str) -> String
}
```

### 3.2 方案A：文本匹配（不用图）

基于分词和关键词表的纯文本分析。

**输入**：`data/graph.json`（只读取节点名和 per_week 中的 intent 表述作为关键词库）+ 测试文本

**算法**：
1. 对测试文本做分词
2. 用每个 Intent 簇的 per_week 表述作为关键词表，计算词重叠率作为匹配度
3. 匹配度 > 阈值则判定为该簇
4. 输出：匹配到的簇列表 + 关键词证据

**输出**：`src/approach_a.rs`

### 3.3 方案B：图谱推理（用图）

基于知识图谱的图遍历推理。

**输入**：`data/graph.json`（完整的图和关系）+ 测试文本

**算法**：
1. **节点匹配**：调用 `graph.match_nodes()` 匹配到起始节点
2. **邻居发现**：调用 `graph.neighbors()` 列出所有直接邻居（支持/冲突/依赖）
3. **多跳路径**：调用 `graph.bfs()` 遍历 2 跳，输出推理路径
4. **冲突检测**：调用 `graph.detect_conflicts()` 检查匹配节点间的冲突

**输出**：`src/approach_b.rs`

### 3.4 对比执行

以 `assets/analysis/intent-analysis-report.md` 为基准输出，对比两个方案的处理结果。

**基准**：`../../../assets/analysis/intent-analysis-report.md`（人工审核通过的跨周意图分析）

**测试输入**：从 `assets/raw/2026-W19/` 中选取一段原始日志文本

**方法**：
1. 将原始日志文本分别输入方案A和方案B
2. 方案A输出匹配的 Intent 簇列表
3. 方案B输出匹配的 Intent 簇 + 推理路径 + 冲突检测
4. 将两个方案的输出与基准报告对照：
   - 基准报告中该周正确识别了哪些 Intent 簇？
   - 方案A/B 分别匹配到了哪些？
   - 方案B 额外推理出了哪些基准报告中提到的关联？

**执行**：
```bash
cargo run --bin approach_a < input.txt  # 输出 results-a.json
cargo run --bin approach_b < input.txt  # 输出 results-b.json
```

**输出**：`outputs/results-a.json` + `outputs/results-b.json`

### 3.5 对比分析

以 `intent-analysis-report.md` 为 truth，逐项对比两个方案的输出。

| 对比维度 | 基准（人工分析） | 方案A（文本匹配） | 方案B（图谱推理） |
|---------|----------------|-----------------|-----------------|
| 匹配到的 Intent 簇 | 报告中的准确列表 | 关键词匹配结果 | 关键词匹配结果 |
| 推理出的关联 | 报告中标注的关系 | 无（只能匹配） | 邻居 + 2跳路径 |
| 检测到的冲突 | 报告中标注的冲突 | 无 | 冲突边 + 间接冲突 |
| 可追溯性 | 引用原文段落 | 关键词证据 | 引用图谱边+logic |

**评估指标**：
1. **召回率**：方案匹配到的 Intent 数 ÷ 基准中正确 Intent 数
2. **额外发现价值**：方案B 推理出的、基准中也认可的关联数
3. **误报率**：方案匹配到但基准中不存在的 Intent 数

**输出**：`outputs/comparison.md`
- 三列对比表（基准 / 方案A / 方案B）
- 召回率和误报率计算
- 图谱推理的增量价值结论
