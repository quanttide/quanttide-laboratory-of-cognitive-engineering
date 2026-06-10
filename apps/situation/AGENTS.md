# AGENTS.md — 已废弃

本应用（qtcloud-think-situation）的母题发现能力已迁移到 `apps/cli/`。

- 图模型（situation-graph.json + petgraph）→ gallery 的 `situation-relation/` 数据 + `RelationGraph`
- LLM 客户端（crates/llm）→ `quanttide-agent` crate
- CJK 分词器 → `apps/cli/src/tokenizer.rs`
- 母题发现 prompt → `apps/cli/` 的 LLM 分析入口

保留代码供参考，不再开发。
