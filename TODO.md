# 阶段1：手动图式迁移

实验目标：验证认知工程建模能否加快"从自己的混乱中学习经验 → 迁移到客户"的过程。
测试用例：单租户→多租户技术升级方案（方案只是验证材料，非最终产出）。
真实产出：图式迁移方法论 + 过程记录 + 可复用性评估。

## Step 1：提取源图式

- [x] 从 9 个 domain 中提取 entities / causals / boundaries / mappings
- [x] 标注了每个 schema 在创始人语境中的角色
- [x] 发现遗漏：biases（反例知识）被忽略，已在验证中补充

**产出**: `data/intermediate/source-schemas.md`

## Step 2：复盘映射过程

- [x] 识别 4 个关键分岔点（目标错误、结果非过程、框架非方法、验证非验证）
- [x] 记录每个分岔点的推理路径和纠正方向
- [x] 输出反事实路径——正确路径应该是什么样

**产出**: `data/intermediate/decision-log.md`

## Step 3：提炼方法论

- [x] 定义 0-5 步的图式转移方法
- [x] 定义 4 种映射类型（结构同构/隐喻迁移/反例学习/约束标记）
- [x] 每个类型有判定条件和转移规则
- [x] 区分"可复用的方法" vs "仅本次有效的直觉"

**产出**: `data/intermediate/transfer-method.md`

## Step 4：评估可自动化程度

- [x] 全自动层：YAML 解析 + 结构提取 + 按类型分类
- [x] LLM 辅助层：角色标注、映射类型识别、方法执行
- [x] 人工层：目标设定、推理记录、验证判断

**产出**: `data/intermediate/automation-boundary.md`

## Step 5：验证方法

- [x] 按 transfer-method.md 重新执行转移
- [x] 发现方法遗漏（缺少 biases 提取、缺少 dynamics 映射规则）
- [x] 验证结果：方法可用，可复现性 9/10，完整性 6/10

**产出**: `data/output/validation-record.md`

## 阶段 1 真实产出

| 产出 | 类型 | 位置 |
|------|------|------|
| 源图式清单 | 原始材料 | `data/intermediate/source-schemas.md` |
| 映射决策日志 | 过程记录 | `data/intermediate/decision-log.md` |
| 图式转移方法草案 | 方法论 | `data/intermediate/transfer-method.md` |
| 可自动化边界报告 | 评估 | `data/intermediate/automation-boundary.md` |
| 方法验证记录 | 验证 | `data/output/validation-record.md` |

## 阶段 1 结论

- 方法可用但完整性不足（6/10）：缺少 biases 和 dynamics 的映射规则
- 方法的价值不在于"让结果更正确"，而在于"让过程和限制可见"
- 阶段 2 自动化方向已明确：代码做结构化提取，LLM 做映射执行，人工做判断
