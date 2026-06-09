# 情境周报：{{week}}

> 生成时间：{{date}}

---

## 全景速览

| 情境 | 意向数 | 高优先级 | 高风险 | 层级分布 |
|------|--------|---------|--------|---------|
{{#situations}}
| {{label}} | {{intention_count}} | {{high_priority_count}} | {{high_risk_count}} | {{level_distribution}} |
{{/situations}}

**总计**：{{total_situations}} 个情境，{{total_intentions}} 条意向

---

## 逐情境详情

{{#situations}}

### {{label}}（{{name}}）

**演化**：{{dynamics}}

**agenda**：{{agenda}}

**ecology**：{{ecology}}

**frame**：{{frame}}

| 意向 | 优先级 | 风险 | 触发 |
|------|--------|------|------|
{{#intentions}}
| {{title}} | {{priority_label}} | {{risk_label}} | {{trigger_label}} |
{{/intentions}}

---
{{/situations}}

---

## 情境关系

{{#relations}}

- **{{source_label}}** ↔ **{{target_label}}**：{{type}}（{{strength}}）
  - 依据：{{logic}}

{{/relations}}
{{^relations}}
（暂无推理关系）
{{/relations}}

---

## 母题发现

{{#motifs}}

### {{theme}}

贯穿情境：{{situations}}

描述：{{description}}

---
{{/motifs}}
{{^motifs}}
（暂无跨情境母题）
{{/motifs}}

---

## 演化趋势（vs 前周）

{{#trends}}

- **{{label}}**：{{change_summary}}

{{/trends}}
{{^trends}}
（无跨周对比数据）
{{/trends}}
