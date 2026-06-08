# 实验：多轮 GraphRAG 脚手架（exp_multi）

## 目标

将当前单轮对话改造为多轮，每轮累积探索状态（DiscoveryState），形成深度思考轨迹。

## 设计

**核心变化**：新增 `DiscoveryState` 跨轮累积

```
struct DiscoveryState {
    explored_clusters: Vec<u32>,
    explored_node_ids: Vec<u32>,
    explored_edge_ids: Vec<u32>,
    open_questions: Vec<String>,
    insights: Vec<String>,
}
```

### 每轮流程对比

| 环节 | 单轮 | 多轮 |
|------|------|------|
| 匹配 | 仅当前输入匹配 keyword_index | 当前输入匹配 + 历史 explored_clusters 优先级提升（score × 1.5） |
| 检索 | 仅匹配簇的邻居 | 匹配簇邻居 + 历史 explored 节点的全连接子图 |
| Prompt | 图谱 + 用户输入 | 图谱 + 历史状态摘要 + 遗留问题 + 用户输入 |
| LLM 输出 | positioning + connections + exploration | 同上 + discovery_update |
| 状态管理 | 无 | 每轮合并 discovery_update，裁剪历史 |

### 状态裁剪策略

- explored_clusters/edges: 去重 union，最多 10 簇 20 边
- insights: 保留最近 5 条
- open_questions: 保留 3 条

### 提示增强

同一簇连续 3+ 轮出现时，提示 LLM 切换到未探索簇。

### 评估指标

| 指标 | 测量方式 |
|------|---------|
| 新发现边/轮 | 相邻轮 discovery_update.new_edges 数量 |
| 探索深度 | 从入口簇出发的图距离 |
| 有效利用轮次 | 连续 2 轮无新 discovery 即终止 |
| 跨簇跳转 | 相邻轮是否切换簇 |

**假设**：多轮能在第 3-4 轮达成 >2 跳的图探索深度，单轮平均停在 1 跳。

### 中止条件

1. **硬上限**：MAX_TURNS=8，到达后自动结束
2. **收敛中止**：连续 2 轮无新发现（簇/边/洞察均为空），自动结束
3. **用户中止**：输入 `exit`

结束后输出实验总结：总轮次、探索簇数、发现边数、生成洞察数、遗留问题数。

## 输入

- `../../data/formal/intent-graph.json`
- 用户入口想法（控制变量，与单轮实验相同种子）

## 输出

- 运行记录：`data/formal/sessions/session_multi.json`
- 评估报告：输出到 stdout + `data/formal/report/exp_multi_eval.md`

## 实验结果（2026-06-08，5 轮）

| 指标 | 测量结果 |
|------|---------|
| 总轮次 | 5（用户 exit 中止，未触发收敛） |
| 发现簇数 | 14（去重后 4 个唯一簇：1,2,5,10） |
| 发现边数 | 9（edge ids: 14,15,16 等） |
| 生成洞察 | 29 |
| 遗留问题 | 3（已演变为更具体的问题） |
| 探索轨迹 | 簇1(POC→商业) → 簇2(压力源) → 簇1+2(模板化) → 簇10(元模式) → 簇1+2(合并方案) |

### 与单轮对比

| 维度 | 单轮（前次实验） | 多轮 |
|------|-----------------|------|
| 探索深度 | 每轮独立，平均 1 跳邻居 | 跨轮累积，从 1→2→3→10→1+2 跳跃 |
| 问题演化 | 问题每轮重置 | 问题持续跟踪，旧问题被回答后替换 |
| 连贯性 | 无跨轮连接 | 每轮自然承接上一轮结论 |
| discovery_update | 无结构化反馈 | 每轮输出新簇/边/洞察 |

**结论**：多轮 + DiscoveryState 累积使探索深度和连贯性显著优于单轮，第 3 轮起出现跨簇整合 (簇1+2+5)，第 4 轮通过元模式认知完成认知跃迁。

