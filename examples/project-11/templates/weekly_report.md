# 情境周报：{{week}}

> 生成时间：{{date}}


## 核心发现

{{core_finding}}


{{#domains}}

## {{label}}

### 心智模型

{{#schemas}}
- **{{label}}**：{{usage}}
  - 因果：{{#causals}}IF {{condition}} → {{outcome}}；{{/causals}}
  - 偏差：{{#biases}}{{belief}}（{{fact}}）{{/biases}}
{{/schemas}}

### 情境意识

{{ecology}}

{{frame}}

### 意图识别

| 意向 | 层级 | 优先级 | 风险 | 触发 |
|------|------|--------|------|------|
{{#intentions}}
| {{title}} | {{level}} | {{priority}} | {{risk}} | {{trigger}} |
{{/intentions}}

{{#relations}}

**关系**：{{source}} → {{target}}：{{type}}（{{strength}}）— {{logic}} [{{confidence}}]
{{/relations}}

---

{{/domains}}
