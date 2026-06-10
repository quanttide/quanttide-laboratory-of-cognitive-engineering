# 实验 TODO: 认知约束 vs 自由提取

验证 `data/insight/index.md` 的核心假设：**用预设认知类别约束 AI 做分类提取，比自由形式的提取质量更高。**

## 实验设计

| 组别 | Prompt 策略 | 预期特征 |
|------|-------------|---------|
| 对照组 | "提取关键信息" | 平庸摘要、正确废话、结构松散 |
| 实验组 | "按 Mental Model → Schematic → Situation → Intent 分类提取" | 结构化、噪声低、认知增量高 |

## 实验步骤

### Step 1: 准备输入数据
- [ ] 从 `data/journal/quanttide-founder/2026-W23/thoughts/` 选择一段原始日志
- [ ] 提取到 lab 内（遵守 AGENTS.md：复制而非直接编辑 journal）

### Step 2: LLM 调用
- [ ] 使用 `quanttide-agent::LLM::default()` 发起调用
- [ ] 实现两个 prompt 模板
- [ ] 确保使用相同的模型参数，仅 prompt 不同

### Step 3: 对比评估
- [ ] 打印两边输出
- [ ] 对比标准：结构化程度、噪声密度、认知增量

### Step 4: 结论
- [ ] 核心假设是否成立？
- [ ] 如果成立：认知工程的元假设得到验证
- [ ] 如果不成立：哪里出了问题？

## 实现文件

| 文件 | 用途 |
|------|------|
| `src/main.rs` | 入口：读取输入、调用 LLM、输出对比 |
| `src/prompt.rs` | 两个 prompt 模板定义 |
| `src/eval.rs` | 输出格式化与对比展示 |

## 依赖

- `quanttide-agent` — LLM 调用（已配置）
- `quanttide-think` — 数据模型（已配置）

## 验证标准

- 实验组输出明显比对照组更结构化
- 实验组噪声更少（没有"正确的废话"）
- 实验组有可识别的认知增量（新洞察，而非已有信息的重述）
