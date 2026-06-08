# 重构完成报告：lib.rs → intent-graph

所有阶段已执行完毕。`apps/intent/src/lib.rs` 不再定义任何 intent-graph 已有的类型，scaffold 数据模型全部复用 intent-graph，`Cluster` 全面重构为 `Situation`。

---

## 验收状态

| 标准 | 状态 |
|------|------|
| `cargo build` 0 warning | ✅ |
| 不定义 intent-graph 已有的类型 | ✅ 仅保留 `ScaffoldData`、`RelationType`、`SituationMatch`（session 序列化） |
| `match_with_history()` 复用 `IntentGraph::match_nodes()` | ✅ |
| 关键词索引使用 `tokenizer::tokenize()` | ✅ 50 vs 旧 83 关键词，停用词已被过滤 |
| JSON `graph.nodes` 与 `situations` 共用 `Vec<PerWeek>` | ✅ |
| `Cluster` → `Situation`，输出用 `title` | ✅ |

---

## 执行记录

### Phase 0+1：模型扩展 + Cluster→Situation 重命名

| 文件 | 改动 |
|------|------|
| `intent-graph/src/models.rs` | 新增 `PerWeek`；`Cluster`→`Situation`；`name`→`title`；`per_week_intents: Vec<String>`→`Vec<PerWeek>`；`KeywordEntry.name`→`title`；`MatchedNode` 加 `Deserialize` |
| `intent-graph/src/builder.rs` | `Cluster`→`Situation`；`match_cluster_id`→`match_situation_id`；构造 `Vec<PerWeek>` |
| `intent-graph/src/graph.rs` | `entry.name`→`entry.title`；新增 `from_data()` 方法 |
| `intent-graph/src/analyzer.rs` | `ClusterEntry`→`SituationEntry`；`ClusterKeywordIndex`→`SituationIndex` |
| `intent-graph/src/lib.rs` | 加 `pub use builder::GraphBuilder` |

### Phase 2+3：删除重复类型 + GraphData→ScaffoldData

| 文件 | 改动 |
|------|------|
| `src/lib.rs` | 删 `GraphData`/`ClusterDescription`/`PerWeek`/`KeywordEntry`/`RelationType`；加 `ScaffoldData` 用 `Vec<NodeWeight>`+`KeywordTable`；`ClusterMatch`→`SituationMatch`；`explored_clusters`→`explored_situations`→；`new_clusters`→`new_situations`；`matched_clusters`→`matched_situations`；`cluster_id`→`situation_id`；prompt "簇"→"情境" |

### Phase 4：ScaffoldEngine 持有 IntentGraph

| 改动 | 说明 |
|------|------|
| `ScaffoldEngine` 加 `graph: IntentGraph` | JSON 的 `graph` 子键构造 |
| `IntentGraph::from_data()` | 从 `GraphData` 直接构造，不经过文件 IO |

### Phase 5：统一 tokenization

| 改动 | 说明 |
|------|------|
| `GraphBuilder::build_keyword_table_from_yaml()` 重建索引 | 关键词从 83 降到 50，停用词 + 标点二元组被过滤 |
| `match_with_history()` 改用 `IntentGraph::match_nodes()` | 替代手写 `bigrams()` + 遍历 |
| `fn bigrams()` 删除 | 不再需要 |

### Phase 6：清理

| 项目 | 动作 |
|------|------|
| `fn bigrams()` | 已删除 |
| `fn ts()` | 保留（`pub`） |
| 旧 session.json | 已清理（schema 变更） |
| 临时 example | 已删除 |

### 数据文件

| 文件 | 改动 |
|------|------|
| `data/refined/intent.yaml` | `clusters`→`situations`；`name`→`title` |
| `data/formal/intent-graph.json` | `cluster_descriptions`→`situations`；`keyword_index[].name`→`title`；`graph.nodes[].name`→`title`；`graph.nodes[].per_week_intents`→`[{week, intents}]` |
| `data/cleaned/2026-W23/topic.md`→`situation.md` | 文件名更新 |
