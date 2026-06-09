# 情境周报：{{week}}

> 生成时间：{{date}}


## 核心发现

{{core_finding}}


{{#domains}}

## {{label}}

### 描述

- **心智模型**：{{mental_model_summary}}
- **情境意识**：{{awareness_summary}}
- **意图识别**：{{intention_summary}}

### 关系

{{#relations}}
- {{source}} → {{target}}：{{type}}（{{strength}}）— {{logic}} [{{confidence}}]
{{/relations}}
{{^relations}}
（暂无推理关系）
{{/relations}}

---

{{/domains}}
