# TODO

## 阶段一：自然语言 REPL

- [x] 意图分类器
- [x] 合并 REPL 帮助
- [x] 默认行为 = 关键词搜索

## 阶段二：关系替代图

- [x] `KeywordIndex::search()` 加载 `SituationRelation`
- [x] `RelationGraph` 模块：`neighbors()`、`bfs()`
- [x] 搜索结果显示匹配情境 + 邻居 + 关系类型

## 阶段三：LLM 增强

- [x] 含分析关键词时自动调用 `quanttide-agent`
- [x] prompt 模板：匹配情境 + 关系 → LLM 综合分析

## 阶段四：完全废弃 situation app

- [x] 添加 `AGENTS.md` 记录废弃说明
- [ ] 删除/归档 `assets/situation-graph.json`
- [ ] 移除 `crates/llm/` 和 `crates/core/` 目录（代码保留在 git 历史中）
