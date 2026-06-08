# 项目六：图谱推理三阶段独立验证

将图谱推理拆为三个独立阶段逐一验证，解决 project-04 设计中匹配与推理职责混在一起的问题。

## 核心问题

旧设计（project-04）把匹配（输入→簇）和推理（簇→关系）绑在一起，导致匹配失败时推理无法归因——是匹配层没找到正确簇，还是推理层找到了簇但没找到有意义的关系？

新设计将两者拆开，另加反馈回路的跨周验证。

## 技术栈

- Rust + `intent-graph` 库（`packages/rust/intent-graph/`）
- 三个独立的 binary：`exp_match` / `exp_reason` / `exp_feedback`

`Cargo.toml`：
```toml
[package]
name = "project-06"
version = "0.1.0"
edition = "2024"

[dependencies]
intent-graph = { path = "../../packages/rust/intent-graph" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
clap = { version = "4", features = ["derive"] }

[[bin]]
name = "exp_match"
path = "src/bin/exp_match.rs"

[[bin]]
name = "exp_reason"
path = "src/bin/exp_reason.rs"

[[bin]]
name = "exp_feedback"
path = "src/bin/exp_feedback.rs"
```

---

## 任务 6.1：匹配能力验证

**问题**：给定一段文本，能否正确识别它属于哪个 Intent 簇？

### 输入

- 关键词表：从 `../../data/refined/intent.yaml` 的 per_week 表述构建（当前策略）
- 测试集：从 `../../data/raw/` 选取 N 段文本，每段有人工标注的所属簇
- 对比扩展版关键词表：从原始日志文本本身提取同义关键词补充

### 方法

1. **基准匹配**：用当前关键词表 + bigram 分词 + 重叠度阈值，对测试集做匹配，计算正检率
2. **改进匹配**：从原始日志中提取标注文本的关键词追加到关键词表，重新测试
3. **对比**：改进后的正检率提升多少？漏报的文本有什么特征？

### 输出

`outputs/match-result.json`
```json
{
  "baseline": {"recall": 0.3, "false_positive": 0.2, "details": [...]},
  "improved": {"recall": 0.0, "false_positive": 0.0, "details": [...]},
  "miss_patterns": ["口语化表达", "跨簇重叠词", "新话题首次出现"]
}
```

### 关键问题

- 正检率低是因为关键词覆盖不足，还是匹配算法本身的问题？
- 如果改用 Embedding 语义匹配，提升多少？

---

## 任务 6.2：推理能力验证

**问题**：给定一组 Intent 簇，图谱能否找出它们之间有意义的关联？

### 输入

- 图谱：从 `../../data/refined/intent.yaml` + `intent-relation.yaml` 构建
- 测试用例：人工构造的簇对输入（跳过匹配阶段），每对标注了预期关系

### 测试用例示例

| 输入簇 | 预期输出 |
|:------|:---------|
| [4] | 邻居：4→3(支持)、3→4(支持)；路径：4→3→1 |
| [5, 2] | 冲突：5↔2；路径：5→2→1 |
| [3, 4] | 双向支持：3↔4 |
| [2] | 邻居：2→1(支持)、2→5(冲突)、9→2(情感补给) |

### 方法

1. 对每组输入簇，调用 `graph.infer()`（此时 match_nodes 直接用输入簇代替关键词匹配）
2. 将输出的 neighbors / bfs_paths / conflicts 与预期对照
3. 统计关系发现覆盖率（预期中的边多少被找到了）和误报率

### 输出

`outputs/reason-result.json`
```json
{
  "total_cases": 6,
  "coverage": 0.85,
  "false_positive": 0.1,
  "cases": [
    {"input": [4], "expected_relations": 2, "found": 2, "extra": 1}
  ]
}
```

### 关键问题

- 图的覆盖率（预期关系中被找到的比例）是多少？
- 未被覆盖的关系有什么特征？（边方向不对？间接路径找不到？）
- 误报的关系是"弱相关"还是"完全错误"？需要带等级评估。

---

## 任务 6.3：反馈回路验证

**问题**：每周注入新数据后，图的质量是否提升？

### 输入

- 种子图谱：从 W19 的数据构建
- 逐周数据：W20 → W21 → W22 → W23 的原始日志
- 每周的标注结果（人工标注的簇和关系）

### 方法

1. 用 W19 数据构建种子图谱和关键词表
2. 对 W20 数据做推理 → 输出候选边 → 人工审核 → 更新图谱
3. 用更新后的图谱对 W21 数据做推理 → 记录正检率变化
4. 重复 W22、W23
5. 绘制正检率随周数的变化曲线

### 输出

`outputs/feedback-result.json`
```json
{
  "weekly_recall": [
    {"week": "W20", "recall": 0.3, "edges_added": 0},
    {"week": "W21", "recall": 0.4, "edges_added": 2},
    {"week": "W22", "recall": 0.45, "edges_added": 1},
    {"week": "W23", "recall": 0.5, "edges_added": 1}
  ],
  "conclusion": "4 周后正检率从 30% 提升到 50%，增益递减"
}
```

### 关键问题

- 图更新后下一周的正检率是否确实提升？
- 增益是否递减？第 4 周的提升是否显著小于第 2 周？
- 被拒绝的候选边有什么特征（重复提案、不成立的关系）？

---

## 任务 6.4：综合报告

**问题**：如何将三组实验输出汇总为结构化摘要，辅助思考者理解自身意图模式？

### 输入

- `data/output/match-result.json`
- `data/output/reason-result.json`
- `data/output/feedback-result.json`

### 方法

1. 读取三组实验的输出 JSON
2. 统计匹配层的 best/worst 簇、推理层的枢纽/孤立簇、反馈层的 gap 趋势
3. 生成结构化摘要（`experiment-summary.json`）
4. LLM 分析生成意图分析报告（`intent-analysis.md`）

### 输出

`data/report/experiment-summary.json`：结构化汇总
`data/report/intent-analysis.md`：LLM 生成的意图分析

---

## 项目列表

| 任务 | binary | 解决的问题 |
|:----|:-------|:----------|
| 6.1 匹配验证 | `exp_match` | 关键词匹配到底能多准？改进空间在哪？ |
| 6.2 推理验证 | `exp_reason` | 给定正确簇，图能找到多少有意义的关系？ |
| 6.3 反馈验证 | `exp_feedback` | 图每周更新后会不会越来越好？ |
| 6.4 综合报告 | `exp_report` | 三组实验数据汇总为结构化摘要 |
