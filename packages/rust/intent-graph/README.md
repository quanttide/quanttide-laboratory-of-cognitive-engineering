# packages

一级子目录按语言划分，二级子目录为具体包。

```
packages/
  rust/            # Rust 包，用 Cargo 构建
    intent-graph/   # 知识图谱推理库
```

## 使用方式

### 作为依赖引入

在目标项目的 `Cargo.toml` 中添加：

```toml
[dependencies]
intent-graph = { path = "../../packages/rust/intent-graph" }
```

路径相对于目标项目的位置。例如 `examples/project-06/Cargo.toml` 中写 `path = "../../packages/rust/intent-graph"`。

### intent-graph 库

图谱推理的基础设施，提供以下能力：

| 模块 | 功能 |
|:----|:-----|
| `IntentGraph` | 核心图结构，基于 petgraph `DiGraph` |
| `GraphBuilder::load()` | 从 YAML 构建图（解析 intent.yaml + intent-relation.yaml） |
| `graph.match_nodes()` | 关键词匹配节点 |
| `graph.neighbors()` | 发现直接邻居 |
| `graph.bfs()` | BFS 多跳遍历 |
| `graph.detect_conflicts()` | 冲突检测 |
| `graph.candidate_edges()` | 候选新边生成 |
| `graph.infer()` | 一步推理：匹配 + 邻居 + BFS + 冲突 + 候选边 |
| `tokenizer::tokenize()` | 中文 bigram 分词 + 停用词过滤 |

### 示例

```rust
use intent_graph::{IntentGraph, GraphBuilder, models::KeywordTable};

// 从 YAML 构建图
let graph = GraphBuilder::from_yaml("intent.yaml", "intent-relation.yaml")?;

// 加载关键词表
let keywords: KeywordTable = serde_json::from_str(&fs::read_to_string("keywords.json")?)?;

// 推理
let result = graph.infer(&keywords, "输入文本", 0.1);
println!("{}", serde_json::to_string_pretty(&result)?);
```
