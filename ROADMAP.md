# ROADMAP

## 为什么做这些

这套方法的价值不在方法论本身，而在它改变了交付物：

| 内功（团队内部） | → 外功（客户感知） |
|---|---|
| 保留类/需验证类区分 | 每条建议标注可信级——"已验证"或"需验证" |
| 数据可追溯 | 质疑时溯源到 journal 原始记录，不是"凭经验说" |
| 7 维质量评估 | 交付物附带质检报告——知道哪里好、哪里还有缺口 |

## 当前状态

| 阶段 | 状态 | 说明 |
|------|------|------|
| 0 奠基 | ✅ 完成 | 理论对齐 + 质量框架 + 基线评估 |
| 1 方法论 | ✅ 完成 | 迁移框架 + 映射验证 + 第一次迁移（质量 2.86→4.14） |
| 2 自动化 P0-P2 | ✅ 完成 | filler + assessor + classifier prompt（Rust CLI） |
| 2 自动化 P3-P4 | 🔄 当前 | 数据层验证 + 报告层 |

## 方法概要

**理论引导的图式迁移**——从 journal 原始数据到结构化 schema 的五步流水线：

```
journal → ① Repo::load → ② 理论审视 → ③ 因果拆解（保留类/需验证类）
       → ④ filler（跨周合并 + 注解） → ⑤ assessor（7 维度评分）→ schema + trace
```

工具实现：`lab fill <domain>`（合并填充）+ `lab assess <schema.yaml>`（质量评分）。

## 阶段 2：自动化（剩余）

### 优先级

| 优先级 | 内容 | 产品参考 |
|--------|------|---------|
| P3 | 数据层验证：确认 `qtcloud-think-cli::Repo` 对所有 domain/week 兼容 | — |
| P4 | 报告层：自动生成迁移记录和追溯报告 | `Hindsight`——经验追溯机制 |
| P5 | LLM 分类集成：将 classifier prompt 编码为可交互的 CLI 流程 | `Soar`——规则引擎设计 |

### 验证标准

- `lab fill <domain>` 对所有 domain（think/business/health/etc.）输出合法 YAML
- `lab assess` 评分与手动评估偏差 ≤0.5
- 新增 domain 的迁移能不写 Rust 代码完成

### 遗留问题

| 问题 | 解决方式 |
|------|---------|
| dynamics 命名冲突 | 在 schema 元信息中补充 `maturity` 字段 |
| 需验证类比例 | 执行第二个 domain 迁移，验证 2:6 是否合理 |
| products 零引用 | 每次迁移前先扫 products/index.md（已入 AGENTS.md 工作纪律） |
