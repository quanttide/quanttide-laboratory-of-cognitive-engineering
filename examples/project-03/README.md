# 项目三：图谱驱动的意图推理

以 YAML 文件构建真实的知识图谱，通过图遍历算法进行意图匹配和关联推理。

**概念**：`intent.yaml` 中的 10 个 Intent 簇 = **节点**，`intent-relation.yaml` 中的关系 = **边**，构建一个有向属性图。

## 任务

### 3.1 构建图谱

编写 Python 脚本，从 YAML 加载数据并构建可查询的图结构。

**输入**：`../../../assets/refined/intent.yaml` + `../../../assets/refined/intent-relation.yaml`

**方法**：
1. 从 `intent.yaml` 读取 10 个节点（id/name/type/evolution）
2. 从 `intent-relation.yaml` 读取边（source/target/type/logic），包括 `stable_relations`、`periodic_tensions`、`situational_relations`
3. 将 flywheel 结构也编码为边
4. 输出为 Python pickle 或 JSON 图文件

**输出**：`outputs/graph.json` — 序列化的图结构

```python
# 图的逻辑结构示例
{
  "nodes": [
    {"id": 1, "name": "研发方法论与工具链", "type": "持续关切"}
  ],
  "edges": [
    {"source": 2, "target": 1, "type": "支持", "logic": "心理状态是研发执行力的先决条件", "weight": 5}
  ]
}
```

### 3.2 实现图推理算法

实现基于图的遍历和分析算法。

**输入**：3.1 的 `graph.json`

**算法实现**：

**算法 A：意图匹配**
- 给定输入文本的关键词/主题 → 匹配到最近节点
- 退化为文本关键词匹配 + 节点语义距离

**算法 B：邻居发现**
- 从匹配节点出发，列出所有直接邻居（一层边）
- 按边类型分组（支持/冲突/依赖）

**算法 C：路径推理（BFS）**
- 从匹配节点出发做广度优先遍历
- 输出 2 跳以内的推理路径
- 每条路径附带边的 logic 说明

**算法 D：冲突检测**
- 如果输入匹配到多个节点，检查这些节点之间是否有冲突边
- 如果没有直接冲突，检查是否存在经过中间节点的冲突路径

**输出**：`scripts/graph_engine.py` — 包含上述算法的 Python 模块

### 3.3 执行推理

用 3.2 的引擎处理 3 组测试输入。

**测试输入**：从 `assets/raw/` 外的文本准备 3 段：
- A：表面谈研发疲惫
- B：表面谈招人难
- C：表面谈代码质量

**执行**：对每段输入运行算法 A→B→C→D，记录推理过程

**输出**：`outputs/inference-results.json`
```json
{
  "test_A": {
    "input": "连续加了三天班，感觉脑子不转了，但又不敢停下来",
    "matched_nodes": [{"id": 2, "name": "心理可持续性"}],
    "neighbors": [{"to": 1, "type": "支持", "logic": "..."}],
    "paths_2hop": [{"path": "2→1", "logic": "..."}],
    "conflicts": []
  }
}
```

### 3.4 图谱价值验证

对比图推理结果与纯文本直觉分析的差异。

**对比方法**：
1. 对同一段输入，先凭直觉（不看图谱）列出可能关联
2. 再对比图推理的输出
3. 标出图推理发现的、直觉没发现的有价值关联
4. 评估这些关联是否反直觉但合理

**输出**：`outputs/validation.md`
- 每组输入的直觉分析 vs 图推理对比表
- 被图谱发现但直觉遗漏的关联列表
- 图谱价值结论
