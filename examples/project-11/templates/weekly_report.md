# 情境周报：{{week}}

> 生成时间：{{date}}


## 核心发现

{{core_finding}}


## 行动建议

| 优先级 | 行动 | 负责人 | 时限 | 预期效果 |
|--------|------|-------|------|---------|
{{#actions}}
| {{priority}} | {{action}} | {{owner}} | {{deadline}} | {{expected}} |
{{/actions}}


## 全景

{{situation_count}} 个情境 · {{intention_count}} 条意向 · 高优先 {{high_p}} · 高风险 {{high_r}}

| 情境 | 意向 | 优先级 | 风险 |
|------|------|--------|------|
{{#situations}}
| {{label}} | {{count}} | {{priority}} | {{risk}} |
{{/situations}}


## 重点分析

{{#spotlights}}

### {{label}}

**演化**：{{dynamics}}

**所以**：{{implication}}

| 意向 | 优先级 | 风险 |
|------|--------|------|
{{#intentions}}
| {{title}} | {{priority}} | {{risk}} |
{{/intentions}}

{{/spotlights}}


## 关键关系

{{#relations}}

- **{{source}}** → **{{target}}**：{{type}}（{{strength}}）
  {{logic}} [{{confidence}}]

{{/relations}}


## 心智模型

{{#schemas}}

### {{label}}

{{#causals}}
- IF {{condition}} → {{outcome}}
{{/causals}}
{{#biases}}
- 偏差：{{belief}}（事实：{{fact}}）
{{/biases}}

{{/schemas}}


## 变化

{{#changes}}

| 情境 | 变化 | 含义 |
|------|------|------|
{{#items}}
| {{label}} | {{change}} | {{meaning}} |
{{/items}}

{{/changes}}
