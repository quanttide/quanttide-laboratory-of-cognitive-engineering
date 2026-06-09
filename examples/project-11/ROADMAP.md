# Project 11: Intention Engine — ROADMAP

## 现状

情境层（situation）已完整实现：跨周加载、演化追踪、关系推理、报告生成。
意向层（intention）数据已就位（`docs/gallery/intention/`），但工具链尚未覆盖。

## Phase 1: 意向查询

- [ ] `intentions <week> [name]` — 按周/按情境查询意向
- [ ] `intention <id>` — 按 UUID 查询单条意向详情
- [ ] 按 `priority`/`risk`/`level` 过滤与排序
- [ ] 按 `agent` 聚合（创始人意向 vs 人机协作意向）

## Phase 2: 跨周意图追踪

- [ ] `trace <title>` — 追踪同一条意图在跨周中的表述变化
- [ ] `drift <weekA> <weekB>` — 比较同一情境下意向在两周间的 priority/risk 偏移
- [ ] 识别"持续意向"（连续多周出现）vs "单周意向"
- [ ] 输出意图演化表：title → week → priority → risk

## Phase 3: 意图关系

- [ ] `relate-intentions <week>` — LLM 推理同周意向之间的支持/冲突/依赖关系
- [ ] 构建意图依赖图（DAG）：哪些意图是前置条件，哪些是后继
- [ ] 识别"元意图"（为其他意图提供运作前提，如身心健康）

## Phase 4: 情境-意图交叉分析

- [ ] `coverage <week>` — 分析情境对其意向的覆盖程度
- [ ] `tension <week>` — 检测意图层级冲突（顶层 vs 底层意图之间的张力）
- [ ] 输出情境-意图交叉矩阵：情境行 × 意图列 × 优先级

## Phase 5: 心智模型推理

- [ ] 基于跨周意图的 `drift` 输出，识别反复出现的意图模式
- [ ] 跨情境心智模型 → 跨周心智模型（二者交叉验证）
- [ ] 心智模型的意图级证据链：每条模式附具体意向的原文引用
