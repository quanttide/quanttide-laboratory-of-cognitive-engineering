# 阻碍

## 已发现

| 阻碍 | 状态 |
|------|------|
| schema 评分 4.14 但推导不出 PoC | ✅ 已定位根因：质量框架缺 B-5 推导深度维度 |
| B-5 加入后 schema2.yaml 评分从 4.57→4.375 | ✅ 已评估，B-5=3/5 |
| 每条 causal 缺 timeline/trigger/gate/effort | ✅ 已明确修复方向 |

## 待推穿

| 阻碍 | 怎么推穿 |
|------|---------|
| schema 的 causal 补全 4 个字段（timeline/trigger/gate/effort） | 选一条 causal 做范例，定义字段格式，然后批量补全 |
| 用补全后的 causal 重新推导 PoC，验证 B-5 从 3→5 | 重新执行 PoC 追溯审计 |
