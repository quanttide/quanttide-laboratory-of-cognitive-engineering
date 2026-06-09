# 情境周报：{{week}}

> 生成时间：{{date}}

---

## 核心判断

{{core_judgment}}

---

## 行动建议

| 优先级 | 行动项 | 负责人 | 时限 | 预期效果 | 风险 |
|--------|--------|-------|------|---------|------|
{{#actions}}
| {{priority}} | {{action}} | {{owner}} | {{deadline}} | {{expected_outcome}} | {{risk}} |
{{/actions}}

---

## 全景概览

{{situation_count}} 个情境，{{intention_count}} 条意向（高优先 {{high_priority_count}}，高风险 {{high_risk_count}}）

| 情境 | 意向数 | 高优先级 | 高风险 |
|------|--------|---------|-------|
{{#situations}}
| {{label}} | {{intention_count}} | {{high_priority}} | {{high_risk}} |
{{/situations}}

---

## 逐情境分析

{{#situations}}

### {{label}}（{{name}}）

**演化**：{{dynamics}}

**现象**：{{ecology}}

**判断**：{{frame}}

| 关键意向 | 优先级 | 风险 | 层级 | 触发 |
|---------|--------|------|------|------|
{{#intentions}}
| {{title}} | {{priority}} | {{risk}} | {{level}} | {{trigger}} |
{{/intentions}}

---

{{/situations}}

---

## 关系分析

{{#situation_relations}}

- **{{source}}** ↔ **{{target}}**：{{type}}（{{strength}}）
  {{logic}}

{{/situation_relations}}

{{#intention_relations}}

- **{{source_title}}** → **{{target_title}}**：{{type}} — {{logic}}

{{/intention_relations}}

---

## 跨情境心智模型

{{#schemas}}

### {{label}}

{{usage}}

{{#causals}}
- IF {{condition}} THEN {{outcome}}
{{/causals}}
{{#biases}}
- 信念：{{belief}}（事实：{{fact}}）
{{/biases}}
{{#boundaries}}
- 边界：{{boundary}}
{{/boundaries}}

{{/schemas}}

---

## 与前周对比

{{#comparisons}}

| 情境 | 变化 | 含义 |
|------|------|------|
{{#items}}
| {{label}} | {{change}} | {{implication}} |
{{/items}}

{{/comparisons}}
