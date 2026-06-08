# 实验 project-10：Anti-convergence — 发散探索

## 问题

多轮 scaffold（project-09）验证了历史累积能加深探索，但也暴露出**收敛过快**的问题：

1. **历史偏置匹配**（score × 1.5）让已探索簇持续占据 top 4，未探索簇难以进入视野
2. **仅依赖图谱**：LLM 只能看到意图图里的节点和边，看不到原始日志中的鲜活表达和语境
3. **LLM 自身知识未利用**：图结构限制 LLM 只能用已建模的关系，无法引入外部类比、跨领域知识

## 数据隔离规则

- **只读**：从 `../../data/` 读入（intent-graph.json, raw/ 日志）
- **本地**：所有衍生数据和输出写入 `data/`（extracts.json, sessions/）
- **不修改**根目录 `data/` 下的任何文件

## 设计

### 三项核心变更

| 变更 | 当前（project-09） | project-10 |
|------|-------------------|------------|
| 检索范围 | 仅关键词匹配的 top 4 簇 + 1 跳邻居 | 同上 + **1 个冷簇**（随机从未匹配到的簇中选取）+ 对应**原始日志摘要** |
| 提示内容 | 图谱结构 + 探索历史 | 图谱 + 历史 + **冷簇导入** + **原始日志亮点** |
| LLM 输出 | positioning / connections / exploration / discovery_update | 同上 + **external_perspective**（从 LLM 自身知识库引入的类比/跨域概念） |

### A. 冷注入（Cold Injection）

每轮在匹配阶段之后，从**未被匹配也未被探索**的簇中随机选取 1 个，强制加入检索上下文：

```rust
fn cold_inject(&self, matched_ids: &[u32], state: &DiscoveryState) -> Option<u32> {
    let cold: Vec<u32> = self.data.cluster_descriptions.iter()
        .map(|d| d.id)
        .filter(|id| !matched_ids.contains(id) && !state.explored_clusters.contains(id))
        .collect();
    cold.choose(&mut rand::thread_rng()).copied()
}
```

在提示词中标注为「🧊 冷簇（未直接匹配，建议考虑是否存在隐含连接）」。

### B. 日志亮点回溯

从 `../../data/raw/` 扫描每个簇的原始日志（只读），提取 1-2 条代表性语句。存储为项目本地文件 `data/extracts.json`：

```json
[
  {"cluster_id": 1, "quote": "今天花了一天搭建 CLI 工具链，感觉终于有了自己的节奏"},
  {"cluster_id": 2, "quote": "压力大的时候写了几段小说，反而平静下来了"}
]
```

在提示词中，每个匹配到的簇附带对应亮点。

### C. 外部视角（External Perspective）

LLM 输出新增第 5 个字段：

```json
{
  "positioning": "...",
  "connections": "...",
  "exploration": "...",
  "discovery_update": {...},
  "external_perspective": "这个模式让我联想到软件工程中的『Brooks 定律』——在认知工程中同样适用：增加人手并不总能加快进度，降低认知负荷才是关键。类似的还有 Toyota 的『安灯制度』作为低耗能模式的组织级实现。"
}
```

提示词明确要求 LLM 从自身知识中引入 1-2 个图外类比。

### 中止条件

与 project-09 一致：
1. MAX_TURNS = 16
2. 连续 2 轮无新发现（簇/边/洞察）
3. 用户 exit

## 评估指标

| 指标 | 测量方式 | 与 project-09 对比 |
|------|---------|-------------------|
| 唯一簇覆盖率 | 总轮次结束时 visited_clusters / total_clusters | 09 停在 4/10，10 预期 >6/10 |
| 冷簇采纳率 | LLM 在 positioning/exploration 中提及冷簇的比例 | — |
| external_perspective 质量 | 是否引入了有价值的图外类比 | — |
| 新边发现率 | 每轮新 edge_ids 数量 | 09 平均 1.8/轮，10 预期更高 |

## 输入

- `../../data/formal/intent-graph.json`（只读）
- `data/extracts.json`（项目本地，由实验生成）

## 输出

- `data/sessions/session_p10.json`（项目本地）
