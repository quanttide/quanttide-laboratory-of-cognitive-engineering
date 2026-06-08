# STATUS

最后更新：2026-06-08

## 项目目标

让**意图**成为思考和想法之间的一致性引擎——类似母题在叙事中的作用。

## 验证状态

### ✅ 已验证

| Claim | 证据 |
|-------|------|
| 意图可从原始日志中提取 | 10 个簇覆盖 W19-W23，每簇有周粒度意图列表 |
| 意图关系可建模为有向图 | 19 条边，9 种关系类型，3 种类别（stable/periodic/situational） |
| 关键词检索可匹配输入到簇 | project-06 Top-1 recall 52%，vocab gap 是瓶颈 |
| 图谱可 grounding LLM 推理 | apps/intent 多轮 scaffold 产出定位/连接/探索三层 |
| 多轮累积产生更深探索 | 5 轮实验轨迹：POC→商业 → 压力源 → 模板化 → 元模式认知 → 合并方案 |
| 图结构可自修正 | 初始假设「研发-业务脱节」被图结构自身推翻 → 「POC混乱→心理压力」 |
| 独立簇之间存在结构共振 | "元"出现于簇1/3/4，"低耗能"独立收敛于簇1和簇2 |

### ❌ 未验证

| Claim | 缺口 |
|-------|------|
| 意图驱动叙事生成 | 脚手架产出的定位/连接/探索是思考辅助，不是叙事 |
| 跨周稳定性 | 当前数据限 W19-W23，W24+ 未知 |
| 跨人通用性 | 图谱只对一个人有效，关系类型集是否通用未知 |
| Scaffold vs 直接问答对比 | 没有做对照实验定量比较 |

## 管线

```
data/raw/ → data/cleaned/ → data/refined/ → data/formal/ → ? (叙事?)
            意图 YAML       意图关系 YAML      intent-graph.json
```

当前终点：`data/formal/intent-graph.json`（v0.3.0，19 边，含 session-derived 洞察）

## 工具

```
apps/intent/          → CLI：qtcloud-think-intent（多轮 GraphRAG scaffold）
packages/rust/        → 库：intent-llm（DeepSeek 客户端），intent-graph（图分析）
```

## 核心差距

```
README 承诺：意图 → 叙事（像母题一样驱动一致性）
实际产出：意图 → 脚手架（像图谱一样辅助思考）
```

脚手架是叙事的前提条件，但不是叙事本身。当前在 `formal` 层停滞，缺少 `formal → narrative` 的管线。

## 待解决问题

1. **W24+ 数据**：当前验证窗口 5 周，需要新数据验证簇结构稳定性
2. **叙事引擎**：如何从意图图生成前后一致的叙述？这是 README 核心承诺但未涉足的领域
3. **跨人实验**：换一个主体，同样的方法能否提取出有效的意图图谱？
