# 图式迁移框架

方法：理论引导的图式迁移（Theory-Guided Schema Migration）。
引用来源：`library/theories/schema-theory.md`、`mental-models.md`、`cognitive-task-analysis.md`。

## 输入/输出

| | 内容 | 格式 |
|--|------|------|
| **输入** | journal 原始日志（quanttide-founder 按周组织的 YAML） | 多文件 YAML |
| **中间表示** | lab 内部结构化数据（situations / intentions / thoughts） | JSON |
| **输出** | 结构化 schema（7 要素）+ 追溯记录 + 质量报告 | YAML + MD |

## 五步流水线

```
① 数据接入 ──→ ② 理论审视 ──→ ③ 因果拆解 ──→ ④ 结构化填充 ──→ ⑤ 质量评估
   journal         理论筛子         保留/需验证        7 要素          质量框架
```

### ① 数据接入（Data Ingestion）

**输入**：`../../data/journal/quanttide-founder/{week}/{domain}.yaml`
**输出**：内部表示的 JSON（含 schemas / situations / intentions / thoughts）
**工具**：`src/data/ingest.py`

| 角色 | 工作 |
|------|------|
| 自动 | YAML 解析、字段提取、格式校验 |
| 人工 | 确认数据完整性（无遗漏 domain 或周） |

### ② 理论审视（Theory Alignment Review）

**核心**：用图式理论筛子审视原始数据，标记每段内容的理论角色。

| 理论概念 | 审视问题 |
|---------|---------|
| 同化/顺应 | 这条 causal 是"已有经验的强化"还是"新经验的适应"？ |
| 图式生命周期 | 这个 schema 处于 forming/reinforcing/adjusting/restructuring 哪个阶段？ |
| 默认值填充 | 有哪些隐含的默认假设被当作了显式知识？ |
| 不完备性（心智模型） | schema 是否承认了自己的不完备？边界条件是否显式？ |

**参考**：`docs/alignment-report.md` 中的 7 字段对照表。

| 角色 | 工作 |
|------|------|
| LLM 辅助 | 逐段提出理论对应的分析建议 |
| 人工 | 确认/修正 LLM 的分析，签注设计决策 |

### ③ 因果拆解（Causal Decomposition）

**核心**：将 journal 中的因果链拆为两类。

| 类型 | 同化/顺应 | 判定条件 | 处理 |
|------|----------|---------|------|
| 保留类（retainable） | 同化——新经验融入已有图式 | 跨组织/跨场景通用，不依赖客户特定条件 | 直接写入 schema |
| 需验证类（verifiable） | 顺应——需调整图式以适应新信息 | 依赖客户的组织结构、权力关系、文化等特定条件 | 标注 verify 字段，设计准入期验证协议 |

**参考**：CTA 的任务分解方法——识别哪些认知活动是通用的（skill/rule-based），哪些是情境依赖的（knowledge-based）。

| 角色 | 工作 |
|------|------|
| LLM 辅助 | 对每条 causal 提出分类建议 + 依据 |
| 人工 | 判定分类，对需验证类补充 verify 条件和验证方法 |

### ④ 结构化填充（Schema Filling）

**核心**：将经过审视和拆解的内容填入 schema 七要素。

| 要素 | 填充来源 | 注意 |
|------|---------|------|
| usage | 从 situation.agenda + frame 提炼 | 精确而非宽泛 |
| entities | 从 schema.entities + situation.frame 提取 | 标注关键属性 |
| causals | 从 ③ 的输出，保留类直接填，需验证类带 verify | 每条 condition→outcome 明确 |
| boundaries | 从 schema.boundaries + situation.ecology 提取 | 区分适用和排除 |
| properties | 从 schema.properties + intention.priority/risk 提取 | key-value 形式 |
| dynamics | 从 schema.dynamics + situation.dynamics 提取 | 区分"领域演化"和"图式自身演化" |
| mappings | 从 intention + schema.mappings 提取 | intent→action 对 |
| biases | 从 schema.biases + thoughts 中的反思 | belief→fact 对 |

| 角色 | 工作 |
|------|------|
| LLM 辅助 | 按要素提取建议，填入草稿 |
| 人工 | 逐要素审核，修改，签注 |

### ⑤ 质量评估（Quality Assessment）

**核心**：用 `docs/quality-framework.md` 的 7 维度打分。

| 角色 | 工作 |
|------|------|
| LLM 辅助 | 按框架逐维度建议评分 |
| 人工 | 复核评分，确认扣分项 |
| 自动 | 记录基线对比，输出质量报告 |

## 人机分工总表

| 步骤 | 人工 | LLM 辅助 | 自动 |
|------|------|----------|------|
| ① 数据接入 | 确认完整性 | — | 解析+校验 |
| ② 理论审视 | 签注决策 | 分析建议 | — |
| ③ 因果拆解 | 判定分类，设计验证 | 分类建议 | — |
| ④ 结构化填充 | 逐要素审核 | 草稿建议 | — |
| ⑤ 质量评估 | 复核评分 | 评分建议 | 基线对比 |

## 追溯记录格式

每次迁移的输出附带 `transfer-trace.md`，格式：

```yaml
migration:
  week: 2026-W23
  date: 2026-06-11
  source_domains: [think, business, ...]
  steps:
    - step: 1 数据接入
      status: ok
      detail: 9 domains parsed, 7 thoughts
    - step: 2 理论审视
      decisions:
        - field: causals[0]
          observation: "描述了人类反馈打破AI局部最优"
          theory_alignment: "同化——已有图式（人机协同）的强化"
          decision: "保留"
          signoff: "developer@2026-06-11"
    - step: 3 因果拆解
      classifications:
        - causal: "人类反馈打破AI局部最优 → 辩论收敛"
          type: retainable
          rationale: "跨组织通用的人机协同机制"
    - step: 4 结构化填充
      changes:
        - field: dynamics
          note: "从 '收敛速度' 改为 '领域演化模式'，避免与理论中的'图式自身演化'混淆"
    - step: 5 质量评估
      score: 3.2
      delta: +0.34 from baseline
      weak_spots: [A-2(灵活性), B-2(外部有效性)]
```
