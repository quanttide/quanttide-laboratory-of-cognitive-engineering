# 项目四：开发待办

## P0 — 基础设施

- [ ] 初始化 Rust 项目（`cargo init`，配置 `Cargo.toml` 依赖：petgraph, serde, serde_json, serde_yaml）
- [ ] 定义数据模型（`src/models.rs`：NodeWeight, EdgeWeight, MatchResult, InferenceOutput, CandidateEdge, RejectLog）

## P0 — 关键词表（共享）

- [ ] 方案B 直接引用方案A 产出的 `data/keywords.json`
- [ ] 若方案A 尚未构建，提取相同逻辑到共享模块（或在 04 内复制一份构建逻辑）
- [ ] 验证：与方案A 加载相同的关键词数据

## P0 — 图谱构建

- [ ] 解析 `intent.yaml` → 节点列表（id, name, type, evolution, per_week_intents）
- [ ] 解析 `intent-relation.yaml` → 边列表（source, target, relation_type, logic, weeks, period_type）
- [ ] 用 `petgraph::graph::DiGraph` 构建有向图：
  - "支持"/"依赖"边按 direction 建单向边
  - "冲突"边建双向
  - "包含"边从父指向子
- [ ] 实现图序列化 → `data/graph-init.json`（用于后续快速加载）
- [ ] 实现反序列化 `load_json()` → IntentGraph
- [ ] 验证：图有 10 个节点，边数与 intent-relation.yaml 一致，方向正确

## P1 — 推理引擎

- [ ] **match_nodes**：用 `keywords.json` 对输入文本分词匹配，返回匹配的 NodeIndex 列表（与方案A 完全相同的逻辑）
- [ ] **neighbors**：给定节点，返回所有出边 + 入边邻接关系
- [ ] **bfs**：从起始节点出发，BFS 遍历 N 跳，返回路径列表（路径 = 节点序列 + 边序列）
  - 注意有向图的遍历方向：跟随出边走（forward BFS），也可选择反向（reverse BFS）
  - 默认 forward，depth=2
- [ ] **detect_conflicts**：
  - 直连冲突：检查匹配节点间是否有 relation_type=冲突 的边
  - 间接冲突：BFS 路径中是否存在冲突边，输出 `{node_a, node_b, via: [中间节点]}`
- [ ] **candidate_edges**：若两个匹配节点经 BFS 间接相连（2 跳内），且起点到终点的路径方向合理，生成一条候选边
- [ ] 整合为 `infer(text) → InferenceOutput`

## P1 — CLI

- [ ] 读取 stdin 文本
- [ ] 调用 inference engine
- [ ] 输出 InferenceOutput 为 stdout JSON
- [ ] 支持 `--graph` 指定图文件路径
- [ ] 支持 `--keywords` 指定关键词表路径
- [ ] 支持 `--depth` 指定 BFS 深度

## P2 — 反馈回路

- [ ] 读取推理输出的 `candidate_edges`
- [ ] 交互式审核（stdin 逐条询问 "保留(y)/拒绝(n)?"）
- [ ] 保留 → 添加边到 graph.json；拒绝 → 写入 reject_log.json
- [ ] 反馈模式 CLI 参数：`--feedback` 开启交互式审核
- [ ] 防止重复提案：加载 reject_log，跳过已拒绝的候选边
- [ ] 验证：连续推理多段文本，确认图在增长且不重复提案

## P2 — 验证

- [ ] 测试图谱构建正确性（节点数、边数、方向）
- [ ] 测试 BFS 路径正确性（已知结构 → 预期路径）
- [ ] 测试冲突检测（已知冲突 → 预期输出）
- [ ] 测试候选边生成（已知间接关系 → 预期候选边）
- [ ] 测试反馈回路（提案 → 审核 → 图更新 → 下次不重复提案）
