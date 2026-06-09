# 认知周报：{{week}}

> 生成时间：{{date}}

---

## 认知快照

本周核心认知活动：{{cognitive_summary}}

主导心智模型：{{dominant_schemas}}

关键认知转折：{{cognitive_turns}}

---

## 心智模型活跃度

{{#schemas}}

### {{label}}

{{usage}}

| 维度 | 内容 |
|------|------|
| 因果信念 | {{#causals}}IF {{condition}} → {{outcome}}；{{/causals}} |
| 常见偏差 | {{#biases}}{{belief}}（事实：{{fact}}）{{/biases}} |
| 适用边界 | {{#boundaries}}{{.}}；{{/boundaries}} |

{{/schemas}}

---

## 跨域认知模式

{{#patterns}}

### {{pattern}}

**涉及情境**：{{situations}}

**表现**：{{manifestation}}

**心智模型支撑**：{{schema_reference}}

---
{{/patterns}}

---

## 领域脉搏

{{#domains}}

| 领域 | 活跃度 | 核心意图 | 优先级 | 风险 |
|------|--------|---------|--------|------|
{{#items}}
| {{label}} | {{activity}} | {{core_intention}} | {{priority}} | {{risk}} |
{{/items}}

{{/domains}}

---

## 意图偏移

{{#drifts}}

| 意图 | 上周 | 本周 | 偏移 |
|------|------|------|------|
{{#items}}
| {{title}} | {{prev_priority}} / {{prev_risk}} | {{curr_priority}} / {{curr_risk}} | {{shift}} |
{{/items}}

{{/drifts}}

---

## 认知偏差识别

{{#biases_identified}}

- **{{bias}}**（情境：{{situation}}）
  - 信念：{{belief}}
  - 事实：{{fact}}
  - 状态：{{status}}（{{status_detail}}）

{{/biases_identified}}
