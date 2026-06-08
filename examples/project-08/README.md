# 项目八：GraphRAG 思考脚手架

## 问题

你有一个意图关系图（10 个簇，45 对关系），记录了你的思维结构和连接模式。当你产生一个新想法时，如果直接跟大模型对话，大模型不知道这个想法在你的整体思想图谱中处于什么位置。

## 方案：GraphRAG 脚手架

### 架构

```
用户输入新想法
      │
      ▼
意图关系图 ──→ 检索相关子图（簇、关系、路径）
(预加载)         │
                 ▼
           LLM 生成脚手架
                 │
          ┌──────┴──────┐
          ▼              ▼
      结构化回复      可选：确认后更新图谱
```

每轮交互完整记录到 `data/sessions/`，方便溯源。

### 与直接 LLM 对话的区别

| 维度 | 直接 LLM | GraphRAG 脚手架 |
|:----|:--------|:---------------|
| 上下文 | 通用知识 | 你的意图关系图 + 你的历史思考模式 |
| 召回 | 无 | 检索到和输入最相关的簇和关系子图 |
| 响应 | 通用建议 | 基于你的思维结构的针对性脚手架 |
| 溯源 | 无 | 每轮交互记录保存，可回溯 |

### 检索方式

用户输入一个想法后，引擎：
1. 用 **ClusterKeywordIndex** 匹配输入文本到相关簇
2. 提取这些簇的邻居和关系子图
3. 将子图结构化为自然语言上下文，送入 LLM

### 脚手架回复结构

LLM 的回复包含三块：

**定位**：这个想法在你的意图图谱中的位置
**连接**：与哪些已知关系相关
**探索**：引导你向未探索的方向思考

---

## 输入数据溯源结构

为了记录每一轮交互并方便追溯，采用如下数据结构存放在 `data/sessions/` 目录下。

### 文件组织

```
data/sessions/
├── session_2026-06-08_001.json      # 每次对话一个文件
├── session_2026-06-08_002.json
└── ...
```

### 单条交互记录（Turn）

每条交互记录（`turn`）的结构：

```
turn
├── id: "2026-06-08_001_3"          # 会话ID_第几轮
├── timestamp: "2026-06-08T14:30:00"
├── input
│   ├── text: "用户输入的原始文本"
│   └── matched_clusters             # 检索到的相关簇
│       ├── [{id: 1, name: "研发方法论", score: 0.65}, ...]
│       └── [{id: 5, name: "商业模式", score: 0.32}, ...]
├── retrieved_context               # 检索到的图上下文
│   ├── subgraph_nodes
│   │   └── [{id, name, type}, ...]
│   ├── subgraph_edges
│   │   └── [{from, to, relation_type, category}, ...]
│   └── paths_between               # 输入涉及的簇之间的路径
│       └── [{path: [1,2,5], hops: 2, type: "支持链"}, ...]
├── llm_response
│   ├── raw: "LLM 返回的完整文本"
│   ├── parsed
│   │   ├── positioning: "定位部分"
│   │   ├── connections: "连接部分"
│   │   └── exploration: "探索部分"
│   └── token_count: 1234
├── feedback                        # 用户确认的新关系（可选）
│   └── [{from: 1, to: 5, confirmed_type: "支持"}, ...]
└── metadata
    ├── model: "deepseek-chat"
    ├── temperature: 0.3
    └── scaffold_version: "0.1.0"
```

### 溯源方式

给定一条 LLM 回复，可以追溯：

**输入链**：`turn.llm_response` → `turn.input.text`（用户当时说了什么）

**上下文链**：`turn.llm_response` → `turn.retrieved_context.subgraph_edges`（LLM 当时看到了哪些图信息）

**历史链**：`session_xxx.json` 中按 `id` 顺序排列的 turns → 可以看到该会话中想法如何演化

**反馈链**：`turn.feedback` → 后续会话中该反馈是否被纳入图谱

### 示例

```json
{
  "id": "session_2026-06-08_001",
  "created": "2026-06-08T14:00:00",
  "turns": [
    {
      "id": "session_2026-06-08_001_0",
      "timestamp": "2026-06-08T14:00:00",
      "input": {
        "text": "今天我把 POC 流程标准化了，感觉效率提升了很多",
        "matched_clusters": [
          {"id": 1, "name": "研发方法论与工具链", "score": 0.72}
        ]
      },
      "retrieved_context": {
        "subgraph_nodes": [
          {"id": 1, "name": "研发方法论与工具链"},
          {"id": 5, "name": "商业模式与商业增长"}
        ],
        "subgraph_edges": [
          {"from": 3, "to": 1, "relation_type": "支持"},
          {"from": 5, "to": 2, "relation_type": "冲突"}
        ]
      },
      "llm_response": {
        "raw": "你的输入『POC 流程标准化』...",
        "parsed": {
          "positioning": "属于研发方法论（簇1）",
          "connections": "研发方法论→商业模式（支持路径）",
          "exploration": "标准化和研发之间有一条周期张力"
        }
      }
    }
  ]
}
```

<div style="page-break-before: always"></div>

## 实现

### binary

`exp_scaffold`：交互式（REPL），逐行输入，每轮输出脚手架。每轮交互自动保存到 `data/sessions/`。

### 依赖

- `intent-graph`：加载图谱、簇关键词索引、关系查询
- `intent-llm`：DeepSeek 客户端

### 工作流

1. 启动 `exp_scaffold`
2. 输入你的想法或问题
3. 引擎检索意图关系图 → LLM 生成脚手架
4. 继续对话深入，或输入 `--save` 确认新关系
5. 输入 `exit` 退出

### 文件结构

```
data/
├── scaffold-data.json     ← 初始数据（已准备）
└── sessions/
    └── session_*.json     ← 交互记录（引擎运行时产生）
```

## 项目列表

| 任务 | binary | 解决的问题 |
|:----|:-------|:----------|
| 8.1 GraphRAG 脚手架 | `exp_scaffold` | 如何让新想法在已有意图图谱中获得定位和连接建议？ |
