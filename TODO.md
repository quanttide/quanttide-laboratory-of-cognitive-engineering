# TODO

当前优先级：**阶段 0：奠基**。按 ROADMAP.md 的三阶段分解为可执行的 task。

## 阶段 0：奠基

### 0.1 理论对齐

- [ ] 阅读 `library/theories/schema-theory.md`，提取核心概念（同化/顺应、生命周期、默认值填充）
- [ ] 对照 lab 当前 schema 定义（entities/causals/boundaries/properties/dynamics/mappings/biases），逐字段记录：
  - 该字段在理论中的对应概念是什么
  - lab 的实现与理论有哪些偏差
  - 偏差是设计决策还是遗漏
- [ ] 输出差异报告到 `docs/alignment-report.md`

### 0.2 质量维度

质量维度来源于**两个渠道**，需合并后统一评分标准：

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
- [ ] 输出质量评估框架到 `docs/quality-framework.md`

### 0.3 数据接入

- [ ] 探索 `../../data/journal/quanttide-founder/` 的数据结构（situations、intentions、thoughts）
- [ ] 建立从 journal YAML → lab 内部表示（Rust struct 或 JSON）的读取管道
- [ ] 验证管道能正确解析至少一个完整周的数据

### 0.4 基线评估

- [ ] 用 0.2 的质量维度评估从 journal 提取的首批 schema
- [ ] 记录基线分数到 `data/baseline-assessment.md`

## 阶段 1：方法论

### 1.1 迁移框架

- [ ] 基于阶段 0 的理论对齐，定义迁移的输入/输出/步骤
- [ ] 输出迁移框架草案到 `docs/transfer-framework.md`

### 1.2 映射类型 + 1.3 验证机制

- [ ] 验证 4 种映射类型与 CTA 分解的一致性
- [ ] 为"需验证类"因果链设计准入期验证协议模板

### 1.4 第一次迁移

- [ ] 从 journal 数据出发，按五步流水线执行完整迁移
- [ ] 产出 `data/schema2.yaml` + 迁移追溯记录

## 阶段 2：自动化

### 2.1 数据层

- [ ] 将 0.3 的数据管道封装为可复用的读取模块
- [ ] 集成 `quanttide-think` 数据模型做校验

### 2.2 迁移层

- [ ] 将迁移步骤编码为可组合的转换函数
- [ ] 支持手动执行 + LLM 辅助 + 全自动三种模式

### 2.3 报告层 + 2.4 闭环验证

- [ ] 迁移过程自动记录设计决策和分岔点
- [ ] 用阶段 0 的质量标准自动评估产出
- [ ] 对比基线，输出质量变化报告
