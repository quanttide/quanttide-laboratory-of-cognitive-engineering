# 示例项目

历史实验目录，已全部完成并提取到正式模块。

## 清理历史

| 项目 | 去向 |
|:----|------|
| project-01 ~ project-07 | 删除，可复用模块提取到 `packages/rust/` |
| project-08（单轮 GraphRAG scaffold） | 迁移为正式 CLI → `apps/intent/` |
| project-09（多轮实验） | 合并入 `apps/intent/` |

## 当前状态

`examples/` 为空。核心工具和数据管线：

```
apps/intent/          ← CLI 工具（qtcloud-think-intent，多轮 scaffold）
packages/rust/        ← 基础设施库（intent-graph, intent-llm）
data/                 ← 数据管线（raw → cleaned → refined → formal）
```
