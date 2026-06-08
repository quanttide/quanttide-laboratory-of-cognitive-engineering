# 合并路线图：lib.rs → intent-graph

目标：消除 `apps/intent/src/lib.rs` 与 `crates/intent-graph` 之间的所有类型/逻辑重复，使 lib.rs 只保留 scaffold 层编排，数据模型全部复用 intent-graph。

---

## 重复全景

| lib.rs | intent-graph | 差异 | 合并策略 |
|--------|-------------|------|----------|
| `GraphData` | `GraphData` | schema 完全不同 | 重命名 + 改用 `NodeWeight` |
| `ClusterDescription` / `PerWeek` | `NodeWeight` | `PerWeek` 结构化 vs flat `Vec<String>` | 将 `PerWeek` 并入 intent-graph |
| `KeywordEntry`（私有） | `KeywordEntry`（pub） | 字段一致 | 删 lib 版，直接用 |
| `RelationType`（私有） | 无对应 | scaffold 元数据 | 保持或移入 scaffold 模块 |
| `bigrams()` | `tokenizer::tokenize()` | 停用词过滤差异 | 对齐后替换 |
| `match_with_history()` | `IntentGraph::match_nodes()` | 后者无 explored_clusters boost | 前者以后者为基座 |

---

## Phase 0：intent-graph 模型扩展

在 `intent-graph/src/models.rs` 中：

```
[PerWeek]     ← 新增 pub struct PerWeek { pub week: String, pub intents: Vec<String> }

[NodeWeight]  ← per_week_intents: Vec<String> → Vec<PerWeek>
```

**影响范围：**

| 文件 | 改动 |
|------|------|
| `intent-graph/src/models.rs` | 新增 `PerWeek`；改 `NodeWeight.per_week_intents` 类型 |
| `intent-graph/src/builder.rs` | `from_yaml()`：构造 `Vec<PerWeek>` 替代 flat flatten |
| `intent-graph/src/graph.rs` | `to_data()` / `load_json()`：序列化自动适配新类型 |
| `build.rs` / JSON 生成脚本 | `graph.nodes.per_week_intents` 改为 `[{week, intents}]` |

> 改写后 `cluster_descriptions[N].per_week_intents` 与 `graph.nodes[N].per_week_intents` schema 完全一致，可共用同一份数据。

---

## Phase 1：删除 lib.rs 重复类型

### 1. `KeywordEntry`
```
src/lib.rs:29-34  →  删除，靠 use intent_graph::KeywordEntry
```

### 2. `RelationType`
简单元组类型，仅 `build_motif_prompt` 中使用。移入 scaffold 模块或内联为 `Vec<(String, String)>`。

---

## Phase 2：重命名 GraphData → ScaffoldData

lib.rs 的 `GraphData` 不是图数据，是 LLM prompt 模板的上下文数据。命名为 `ScaffoldData` 以消除与 intent-graph 的命名冲突。

```
struct GraphData { ... }  →  struct ScaffoldData { ... }
```

内部字段：
- `nodes: Vec<NodeWeight>`（来自 intent-graph，Phase 0 后含 PerWeek）
- `keyword_index: KeywordTable`（来自 intent-graph）
- `relation_types: Vec<RelationType>`（scaffold 特有）

删除 `ClusterDescription` 和 `PerWeek`——它们被 `NodeWeight` 替代。

---

## Phase 3：ScaffoldEngine 持有 IntentGraph

```
struct ScaffoldEngine {
    data: ScaffoldData,
    client: DeepSeekClient,
}
```

改为：

```
struct ScaffoldEngine {
    graph: IntentGraph,
    data: ScaffoldData,
    client: DeepSeekClient,
}
```

将 `match_with_history()` 以 `IntentGraph::match_nodes()` 为基座实现，上层叠加 `explored_clusters` boost：

```
fn match_with_history(&self, text: &str, state: &DiscoveryState) -> Vec<ClusterMatch> {
    let matched = self.graph.match_nodes(&self.data.keyword_index, text, 0.02);
    // 对 explored_clusters 内的节点 score *= 1.5
    // 截断 top 4
}
```

---

## Phase 4：统一 tokenization

JSON 的关键词索引当前基于 `bigrams()`（全字符二元组）生成。用 `GraphBuilder::build_keyword_table()` 重建索引：

```
GraphBuilder::build_keyword_table_from_yaml("data/refined/intent.yaml")
```

重建后索引基于 `tokenizer::tokenize()`，此时：

1. 删 lib.rs 中的 `bigrams()`
2. `match_with_history()` 不再手动 tokenize——完全委托 `IntentGraph::match_nodes()`

---

## Phase 5：清理

| 项目 | 动作 |
|------|------|
| `fn bigrams()` | 删除 |
| `fn ts()` | 保留（lib.rs 中，已 `pub`） |
| `SessionSummary` | 已抽为 `summary.rs`，保持 |
| 未使用的 import | 清理 |

---

## 验收标准

- `cargo build` 0 warning
- `apps/intent` 不定义任何 intent-graph 已有的类型
- `ScaffoldEngine.match_with_history()` 复用 `IntentGraph::match_nodes()`
- 关键词索引使用 `tokenizer::tokenize()` 而非 `bigrams()`
- JSON 数据文件中 `graph.nodes` 与 `cluster_descriptions` 共用同一 `Vec<PerWeek>` schema
