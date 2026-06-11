# TODO

当前优先级：**阶段 2 自动化**。按 ROADMAP 的优先级排序。

## P0：自动化结构化填充

步骤 ④ 的规则最明确，编码收益最大。

- [ ] 复习 `docs/transfer-framework.md` 步骤 ④ 的填充规则（7 要素各从 journal 哪个字段提取）
- [ ] 将填充规则编码为可组合的转换函数（Python：`src/transfer/filler.py`）
  - 输入：journal 内部表示（JSON） + 人工标注（保留类/需验证类）
  - 输出：schema YAML 草稿
- [ ] 用 W23 think 数据测试，对比手动产出 `data/journal-schema.yaml`

## P1：自动化质量评估

`docs/quality-framework.md` 的 7 维度可编码为评分函数。

- [ ] 为每个维度定义可计算的评分逻辑（Python：`src/report/assessor.py`）
- [ ] 输入 schema YAML + journal 数据 → 输出 7 维度分数 + 总分
- [ ] 自动对比基线（`data/baseline-assessment.md`），输出差异

## P2：因果拆解 LLM 辅助

- [ ] 编写 LLM prompt，对 journal 中的 causal 提出分类建议（保留类/需验证类）
- [ ] 输出格式为人工可直接签注的 YAML
- [ ] 测试：用 W19-W23 所有 domain 的 causals 验证分类准确率

## P3：数据层封装

- [ ] 将 `src/data/ingest.py` 封装为可 import 的模块
- [ ] 集成 `quanttide-think` 数据模型做字段校验

## P4：报告层

- [ ] 自动生成 `reports/` 下的迁移记录（含理论引用、设计决策、分岔点）
- [ ] 自动生成 `data/transfer-trace.md`

## 跨阶段

- [ ] 执行第二个 domain（business 或 health）的完整迁移，验证框架多域适用性
- [ ] 每次迁移前先查 `library/products/index.md`（已入 AGENTS.md 工作纪律）
