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

| 情境 | 活跃度 | 核心关切 | 演化方向 |
|------|--------|---------|---------|
{{#situations}}
| {{label}} | {{activity}} | {{core_concern}} | {{direction}} |
{{/situations}}

---

## 逐情境分析

{{#situations}}

### {{label}}（{{name}}）

**现象**：{{phenomenon}}

**原因**：{{reason}}

**所以**：{{implication}}

| 关键意向 | 优先级 | 风险 | 来源 |
|---------|--------|------|------|
{{#key_intentions}}
| {{title}} | {{priority}} | {{risk}} | {{evidence}} |
{{/key_intentions}}

---
{{/situations}}

---

## 关键关系

{{#relations}}

### {{source_label}} ↔ {{target_label}}

**关系**：{{type}}（{{strength}}）

**证据链**：{{evidence_chain}}

{{/relations}}

---

## 跨情境心智模型

{{#mental_models}}

### {{name}}

**定义**：{{definition}}

**适用情境**：{{situations}}

**表现模式**：{{behavior_pattern}}

**预测**：{{prediction}}

---
{{/mental_models}}

---

## 与前周对比

{{#comparisons}}

| 情境 | 变化 | 含义 |
|------|------|------|
{{#items}}
| {{label}} | {{change}} | {{implication}} |
{{/items}}

{{/comparisons}}
