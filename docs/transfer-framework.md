# 图式迁移框架

方法：理论引导的图式迁移。

## 快速开始

```bash
# 1. 先查产品和理论
open library/products/index.md    # 看有没有相关实践
open library/theories/index.md    # 看有没有相关理论

# 2. 执行迁移 + 评估
cd examples/default
cargo build
./target/debug/lab fill <domain> --weeks W19,W21,W22,W23 --annotations annotations.yaml
./target/debug/lab assess data/auto-schema.yaml
```

## 输入/输出

| | 路径 |
|--|------|
| 源数据 | `../../data/journal/quanttide-founder/{week}/{domain}.yaml` |
| 输出 | `data/auto-schema.yaml` + `reports/` 迁移报告 |

## 五步流水线

### ① 数据接入

**自动运行**：`./target/debug/lab fill <domain> --weeks W19,W21,...`

不需要手动解析。工具自动从 journal 读取指定 domain 的跨周数据。

**人工检查**：确认输出的 domain 列表和周数正确。

### ② 理论审视

读 `docs/alignment-report.md`，熟悉 7 字段的图式理论对应关系。

对每条 causal 问：
- 这是"已有经验的强化"（同化）还是"新经验的适应"（顺应）？
- 这个 schema 处于 forming/reinforcing/adjusting/restructuring 哪个阶段？

### ③ 因果拆解

**人工判定**：对每条 causal，判定是保留类还是需验证类。

| 类型 | 判定条件 | 处理 |
|------|---------|------|
| 保留类 | 跨组织通用，不依赖客户特定条件 | 不处理，工具默认标记 |
| 需验证类 | 依赖客户的组织结构、权力关系、文化等 | 创建 annotations YAML |

**需验证类需要创建 annotations.yaml**，格式：

```yaml
causals:
  - condition: "xxx"    # 与 journal 中一致的 condition 文本
    type: 需验证
    verify: "验证通过的条件"
```

保存后传给工具：`lab fill <domain> --annotations annotations.yaml`

**示例**：`/tmp/think-annotations.yaml`、`/tmp/health-annotations.yaml`。

### ④ 结构化填充

**自动运行**（包含在 `lab fill` 中）：跨周合并 entities/causals/boundaries/properties/dynamics/mappings/biases。

7 要素自动合并规则：

| 要素 | 合并策略 |
|------|---------|
| usage | 从各周的 situation.agenda 汇总 |
| entities | 按 name 去重合并 attributes |
| causals | 按 condition 去重，应用 annotations 中的 type/verify |
| boundaries | 跨周并集 |
| properties | 按 key 合并，后周覆盖前周 |
| dynamics | 同上 |
| mappings | 按 intent 去重 |
| biases | 按 UUID 去重 |

**人工精选**：自动输出可能包含重复或冗余内容，需要：
- entities > 7 个时精选（保留核心，合并相似）
- usage 太长时精简为 1-2 句话
- boundaries 去重同义项

### ⑤ 质量评估

**自动运行**：`./target/debug/lab assess data/auto-schema.yaml`

7 维度评分，目标 ≥4.0 分。重点检查：
- A-2 灵活性：是否有需验证类标记
- A-3 复杂度：实体是否过多

## 首次迁移检查清单

- [ ] 查了 `products/index.md`，找到可参考产品并写进迁移报告
- [ ] `lab fill` 输出合法 YAML
- [ ] 需验证类都有 verify 条件
- [ ] `lab assess` 评分 ≥4.0
- [ ] 迁移报告写到 `reports/` 目录
