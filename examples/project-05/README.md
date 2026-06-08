# 项目五：图模型验证 + 进化测评

用 **全新技术数据**（W24+）验证方案A 和方案B，评估三个维度：
1. **检索能力**：有图比无图多发现了什么？
2. **路径质量**：推理路径是否反映了真实的思维流转？
3. **图进化**：反馈回路能否让图持续变好？

## 数据

**测试集来源**：`../../../assets/raw/2026-W24/`（未参与图构建的全新数据）

**为什么不用 W19-W23**：W19-W23 是图的训练数据，用它们做测试是循环验证。

## 任务

### 5.1 构建 W24 基准

先对 W24 原始日志做人工标注：

1. 阅读 W24 每一段日志
2. 标注涉及的 Intent 簇（用已有 10 个簇的框架）
3. 标注段落间的关系（因果/冲突/支持），直接引用原文证据
4. 输出 W24 mini 基准

**输出**：`data/baseline-w24.json`

```json
[
  {
    "id": "T1",
    "segment": "原文段落...",
    "clusters": [1, 3],
    "relations": [{"from": 1, "to": 3, "type": "支持"}],
    "week": "W24"
  }
]
```

### 5.2 测试执行

将 W24 文本分别输入方案A 和方案B，收集两份输出。

**方案A**
```bash
cat data/segments.json | jq -r '.[].segment' | while read line; do
  echo "$line" | ../project-03/target/debug/project-03 2>/dev/null
done > outputs/raw-a.json
```

**方案B（含反馈循环）**
```bash
for seg in data/segments/*.txt; do
  ../project-04/target/debug/project-04 --keywords ../project-03/data/keywords.json 2>/dev/null < "$seg"
  # 人工审核 candidate_edges → 更新 graph.json
  # 下一轮使用更新后的图
done > outputs/raw-b.json
```

### 5.3 维度一：检索能力对比

与 W24 基准对照，分两个子维度衡量。

**意图匹配准确度**

| 指标 | 定义 | 公式 |
|:----|------|:----:|
| 正检率（A） | 方案A 正确识别的 intent 数 / 基准总 intent 数 | TP_A / (TP_A + FN) |
| 正检率（B） | 方案B 正确识别的 intent 数 / 基准总 intent 数 | TP_B / (TP_B + FN) |
| 增量发现数 | 方案B 正确识别 + 方案A 未识别的 intent 数 | 按段累加 |
| 误报率 | 方案识别到但基准不存在的 intent 数 / 方案识别总数 | FP / (TP + FP) |

**关联发现层次（仅方案B）**

方案B 输出的关联按层次分类，度量推理深度：

| 层次 | 内容 | 示例 |
|:----:|------|------|
| L1 | 直接邻居 | 簇1 → 簇3(支持) |
| L2 | BFS 路径（1-2 跳） | 簇5 → 簇2(冲突) → 簇1(支持) |
| L3 | 冲突检测 | 簇5 ↔ 簇2(冲突) |
| L4 | 候选新边 | 间接相连节点间提议的新关系 |

**关键问题**：额外发现以什么为代价？对照基准检查每个额外发现是否成立。

### 5.4 维度二：推理路径质量评审

对照 W24 原文，对方案B 的每条推理路径（neighbors + bfs_paths）做人工打标：

| 等级 | 标签 | 操作定义 |
|:----:|------|---------|
| 3 | 强洞察 | 路径揭示了原文中隐含的思维跳跃，与作者实际思考一致，且非显而易见的 |
| 2 | 合理 | 路径在逻辑上成立，但属于常识级推理（如"商业→心理冲突"在创业语境中可预期） |
| 1 | 弱关联 | 路径在图上有边，但在原文语境中无实际支撑 |
| 0 | 误推 | 路径在图上有边，但与作者的实际思考方向矛盾 |

**评审表**：每条路径记录一行

```csv
test_id, from, to, relation, path_depth, grade, note
T1, 3, 1, 支持, 1, 2, "认知方法论指导研发——原文提到了方法论"
T1, 8, 1, 支持, 1, 1, "团队建设→研发在图上有边，但原文未涉及团队话题"
```

**关键设计**：评审时必须打开 W24 原文验证上下文，不能仅凭图上边做判断。

### 5.5 维度三：图进化追踪

记录反馈回路效果：

| 指标 | 说明 |
|:----|------|
| 候选边提案数 | 每条推理生成的 candidate_edges 总数 |
| 审核通过率 | 人工保留的边 / 提案总数 |
| 图规模增长 | 节点数和边数随推理轮次的增长曲线 |
| 进化增益 | 图更新后，后续段落的推理质量是否有提升（L2+ 路径比例变化） |
| 重复提案率 | 被 reject_log 拦截的重复提案 / 总提案数 |

### 5.6 结论报告

**输出**：`outputs/evaluation.md`（人读）+ `outputs/evaluation.json`（机读）

**报告结构**：

1. W24 基准摘要——标注了多少段、涉及多少簇
2. 检索能力对比表——每段一行，汇总正检率/增量/误报
3. 推理路径质量评审表——全部路径逐条打标结果
4. 图进化追踪——候选边→审核→通过→增长链条
5. 误差分析——已知问题：簇重叠（如簇5和簇8共享关键词）、自环边、未匹配关系引用
6. 整体结论——图模型是否真实反映了思维模式？反馈回路能否让图持续变好？

**机读格式**（`evaluation.json`）：
```json
{
  "samples": [
    {
      "id": "T1",
      "baseline": {"clusters": [1, 3], "relations": []},
      "approach_a": {"matched": [1]},
      "approach_b": {
        "matched": [1],
        "neighbors": [{"from": 3, "to": 1, "type": "支持"}],
        "bfs_paths": [],
        "conflicts": [],
        "candidate_edges": []
      },
      "metrics": {
        "recall_a": 0.5,
        "recall_b": 0.5,
        "incremental": 0,
        "false_positive_a": 0,
        "false_positive_b": 0
      },
      "path_grades": [
        {"from": 3, "to": 1, "depth": 1, "grade": 2}
      ]
    }
  ],
  "summary": {
    "total_samples": 4,
    "avg_recall_a": 1.0,
    "avg_recall_b": 1.0,
    "total_incremental": 8,
    "avg_path_grade": 2.1
  }
}
```

### 5.7 格式标准化说明

以下规则来自 4 样本预运行的实践经验：

1. **BFS 路径去重**：同一对起止节点的多条路径中，只保留最短路径；不同跳数的路径各自保留一条
2. **自环边过滤**：neighbors 和 bfs_paths 中跳过 `from == to` 的条目（已在 project-04 v2 中修复）
3. **关联层次归类**：neighbors 计为 L1，bfs_paths 计为 L2（含多跳），conflicts 单独计为 L3，candidate_edges 计为 L4
4. **簇重叠处理**：当两个簇因关键词重叠命中同一批证据词时，在报告中标注"簇重叠"风险，但不从匹配结果中剔除
