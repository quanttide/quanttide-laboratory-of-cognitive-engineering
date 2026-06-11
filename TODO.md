# TODO

## 目标状态

```
data/
├── journal-schema.yaml      ← 从 journal 迁移生成的 schema
├── baseline-assessment.md   ← 质量基线分
└── transfer-trace.md        ← 五步流水线的每一步追溯记录

docs/
├── alignment-report.md      ← 理论 vs 实现的逐字段差异报告
├── quality-framework.md     ← 理论+工程合并推导的质量评估框架
└── transfer-framework.md    ← 可复用的迁移方法论文档

reports/
└── 2026-Wxx-migration.md    ← 每次迁移的完整记录

schema2.yaml                 ← 最终产出图式

src/                         ← 阶段 2 自动化代码
├── data/                    ← journal 读取管道
├── transfer/                ← 迁移步骤的可组合函数
└── report/                  ← 报告生成器
```

**核心约束**：schema 的每个字段都附带出生证明——

```
journal 原始数据中的第 X 条记录
  → 被 0.3 数据管道读入
    → 经过 1.4 第 3 步映射为 causals[2]
      → 被质量框架打了 4/5 分
        → 在 transfer-trace.md 记录了设计决策
```

**外部视角**：新人能读 `docs/` 理解 schema 与理论的关系，跟 TODO 走一遍产出 schema，每一步有前人决策可参考。

---

## 阶段 0：奠基（一次性）

### 0.1 理论对齐

- [ ] 阅读 `library/theories/schema-theory.md`，提取核心概念（同化/顺应、生命周期、默认值填充、层次组织）
- [ ] 对照 lab 当前 schema 定义（entities/causals/boundaries/properties/dynamics/mappings/biases），逐字段记录：
  - 该字段在理论中的对应概念是什么
  - lab 的实现与理论有哪些偏差
  - 偏差是设计决策还是遗漏
- [ ] 输出 `docs/alignment-report.md`

### 0.2 质量维度（双来源）

**来源 A — 从图式理论推导**（`library/theories/schema-theory.md`）：
| 理论概念 | 工程质量维度 |
|---------|-------------|
| 默认值填充机制 | 覆盖度（Coverage） |
| 同化/顺应 | 灵活性与可修正性（Flexibility） |
| 层次组织 | 复杂度与认知负荷（Complexity） |

**来源 B — 从工程实践引入**（`../../docs/specification/docs/schema.md` 或现有 schema 评审经验）：
- 内部一致性（Internal Consistency）
- 外部有效性（External Validity）
- 任务适用性（Task Fit）
- 可沟通性（Communicability）

- [ ] 从图式理论推导可直接映射的质量维度（覆盖度、复杂度、灵活性）
- [ ] 从工程评估标准引入不可通过理论推导的维度（内部一致性、外部有效性、任务适用性、可沟通性）
- [ ] 合并两个来源，定义统一的评分标准（1-5 分）
- [ ] 输出 `docs/quality-framework.md`

### 0.3 数据接入

- [ ] 探索 `../../data/journal/quanttide-founder/` 的数据结构（situations、intentions、thoughts）
- [ ] 建立从 journal YAML → lab 内部表示（Rust struct 或 JSON）的读取管道
- [ ] 验证管道能正确解析至少一个完整周的数据

### 0.4 基线评估

- [ ] 用 0.2 的质量维度评估从 journal 提取的首批 schema
- [ ] 输出 `data/baseline-assessment.md`

---

## 阶段 1：方法论（手动 + LLM 辅助，每次迁移）

工作模式：开发者做判断和签注，LLM 辅助分类和打分建议，所有决策记录到 trace。

### 1.1 迁移框架

- [ ] 基于阶段 0 的理论对齐，定义迁移的输入/输出/步骤
- [ ] 明确每一步由人判断还是 LLM 辅助
- [ ] 输出 `docs/transfer-framework.md`

### 1.2 映射类型 + 1.3 验证机制

- [ ] 验证 4 种映射类型与 CTA 分解的一致性
- [ ] 为"需验证类"因果链设计准入期验证协议模板

### 1.4 第一次迁移

工作流：
```
数据接入 → 理论对齐审视 → 因果拆解（保留类/需验证类）
  → 结构化填充 → 质量评估 → trace 记录
```

每个决策点格式：
```
开发者：这条因果链应归为"需验证类"，因为……
LLM：  根据 quality-framework.md F-3 标准，建议 4 分
       开发者在 transfer-trace.md 签注确认
```

- [ ] 从 journal 数据出发，按五步流水线执行完整迁移
- [ ] 产出 `data/journal-schema.yaml`
- [ ] 产出 `data/transfer-trace.md`（字段级可追溯）
- [ ] 产出 `reports/2026-Wxx-migration.md`（迁移完整记录）

---

## 阶段 2：自动化

### 2.1 数据层

- [ ] 将 0.3 的数据管道封装为 `src/data/` 模块
- [ ] 集成 `quanttide-think` 数据模型做校验

### 2.2 迁移层

- [ ] 将迁移步骤编码为 `src/transfer/` 中可组合的转换函数
- [ ] 支持手动执行 + LLM 辅助 + 全自动三种模式

### 2.3 报告层 + 2.4 闭环验证

- [ ] `src/report/` 自动记录设计决策和分岔点
- [ ] 用阶段 0 的质量标准自动评估产出，对比基线
- [ ] 输出 `data/baseline-assessment.md` 更新
