# 实验三：图谱驱动的意图推理引擎

以 `assets/refined/intent.yaml` 和 `intent-relation.yaml` 为知识图谱，构建一个能对新输入进行意图匹配、关系推理和冲突检测的推理引擎。

## 任务

### 3.1 引擎设计

定义推理引擎的输入输出协议和推理流程。

**输入**：`assets/refined/intent.yaml` + `assets/refined/intent-relation.yaml` + 新文本

**方法**：
1. **意图匹配** — 将输入文本与 10 个 Intent 簇进行语义匹配，输出匹配度排序
2. **关系推演** — 根据匹配到的 Intent 簇，从 `intent-relation.yaml` 提取关联意图及关系类型
3. **冲突检测** — 如果输入同时匹配多个存在冲突关系的 Intent 簇，标注冲突
4. **路径生成** — 沿图谱边输出从输入 Intent 到关联 Intent 的推理路径

**输出协议**：
```json
{
  "input": "原始输入文本",
  "matched_intents": [
    {"cluster_id": 2, "name": "心理可持续性", "confidence": 0.85, "evidence": "匹配的关键词句"}
  ],
  "inferred_relations": [
    {"from": 2, "to": 1, "type": "支持", "logic": "心理状态是研发执行力的先决条件"}
  ],
  "conflicts": [
    {"between": [5, 2], "description": "商业高压与心理可持续存在内在矛盾"}
  ],
  "reasoning_path": "心理可持续性 → 支持 → 研发方法论"
}
```

### 3.2 测试用例

准备 3 组测试输入，覆盖不同场景。

**测试 A：单意图匹配**
- 输入：仅涉及单一 Intent 簇的文本（如仅谈心理状态）
- 预期：匹配到 1 个 Intent，推理路径短

**测试 B：多意图冲突**
- 输入：同时涉及商业压力和研发探索的文本
- 预期：匹配到 2+ 个 Intent，检测到冲突

**测试 C：新话题/模糊匹配**
- 输入：YAML 中未覆盖的话题
- 预期：低置信度匹配，标注"未覆盖"

**方法**：以 `intent.yaml` 和 `intent-relation.yaml` 为上下文，将测试输入逐条提交给 AI 进行推理

**输出**：`outputs/test-results.md`，每条测试的推理过程和输出 JSON

### 3.3 效果评估

**输入**：3.2 的测试结果

**方法**：从以下维度评估
1. **匹配准确率** — 匹配到的 Intent 簇是否合理
2. **推理相关性** — 推理出的关系是否与图谱一致
3. **冲突敏感度** — 是否能正确检测冲突
4. **边界处理** — 对新话题/模糊输入的处理是否合理

**输出**：`outputs/evaluation.md`，每组的推理结果记录和改进建议
