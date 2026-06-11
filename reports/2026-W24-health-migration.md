# 迁移报告：身心健康领域图式（带 products 对照）

## 元信息

| 字段 | 值 |
|------|-----|
| 迁移 ID | migration-003 |
| 日期 | 2026-06-11 |
| 源数据 | health domain, 5 周（W19-W23） |
| 工具 | `lab fill` + `lab assess` |

## 产品对照：cc-thinking-skills

**产品**：[cc-thinking-skills](https://github.com/tjboudreaux/cc-thinking-skills)（396⭐）
**说明**：18 个心智模型与批判性思维框架，编码为 Claude Code 可执行的 Skill。
**对照维度**：

| 维度 | cc-thinking-skills | lab schema | 差异 |
|------|-------------------|------------|------|
| 表达形式 | Markdown Skill 文件，含 prompt 模板 | YAML 结构化数据（entities/causals/biases） | 前者面向 LLM 执行，后者面向分析 |
| 知识颗粒度 | 单个心智模型独立文件 | 跨周合并的领域级 schema | lab 覆盖更广，cc-thinking-skills 更原子化 |
| 因果建模 | 无显式 causal 结构，隐含在 prompt 中 | condition→outcome 显式对 | lab 更可追溯 |
| 偏差处理 | 无显式 bias 字段 | belief→fact 结构化的反例知识 | lab 多一层验证维度 |
| 质量评估 | 无评分机制 | 7 维度自动评分 | lab 可衡量改进 |

**吸收点**：cc-thinking-skills 的"AI 可执行"设计——lab 的 schema 应增加一个 `prompt_template` 字段，使 schema 可直接被 LLM 消费，而不只是供人阅读。

## 质量评分

| 维度 | 分数 | 说明 |
|------|------|------|
| A-1 覆盖度 | 5/5 | 10属性+8动态 覆盖 12实体 |
| A-2 灵活性 | 5/5 | 2条需验证类均有 verify |
| A-3 复杂度 | 2/5 | 12实体偏多，需精选 |
| B-1 内部一致性 | 4/5 | 结构一致 |
| B-2 外部有效性 | 4/5 | 部分可追溯 |
| B-3 任务适用性 | 5/5 | 可执行 |
| B-4 可沟通性 | 4/5 | 良好 |
| **总分** | **4.14** | **优秀** |

## 遗留

- A-3 复杂度 2/5：12 entities 需人工精选为 5-7 个
- `prompt_template` 字段待设计（来自 cc-thinking-skills 的启发）
