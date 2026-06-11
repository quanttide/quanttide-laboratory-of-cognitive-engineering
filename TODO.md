# TODO

当前优先级：**验证链闭合**（换人走通 + 客户反馈）。

## ✅ P0：结构化填充（已完成）

| task | 文件 | 验证 |
|------|------|------|
| 跨周合并 + 注解匹配 | `src/transfer.rs` | `lab fill think --annotations x.yaml` 输出正确 |
| 模糊条件匹配 | `src/transfer.rs` `normalize()` | annotation 跨周同义匹配通过 |

## ✅ P1：质量评估（已完成）

| task | 文件 | 验证 |
|------|------|------|
| 7 维度自动评分 | `src/report.rs` | `lab assess data/journal-schema.yaml` → 4.57/5 |

## ✅ P2：因果拆解 LLM prompt（已完成）

| task | 文件 |
|------|------|
| 分类规则 + 输入输出格式 | 已删除（Rust 化），规则已编码到 `src/transfer.rs` 的 annotation 逻辑 |

## P3：数据层验证

- [ ] 用 `lab fill` 测试所有 domain（think / business / health / innov / meta / infra / org / product / write）
- [ ] 验证 `Repo::periods("quanttide-founder")` 在不同环境下路径正确
- [ ] 确认 `JOURNAL_PATH` 环境变量覆盖机制可用

## P4：报告层

- [ ] `lab report <domain>` 输出完整迁移记录（含 usage、causals 统计、质量评分）
- [ ] 输出到 `reports/` 目录，Markdown 格式

## P5：LLM 分类集成

- [ ] 实现 `lab classify <domain>` 交互流程：读取 causals → 逐条输出 → 等待人工判定
- [ ] 判定结果直接输出为 annotations YAML（`lab fill` 可直接消费）

## 跨阶段

- [x] 执行第二个 domain（business 或 health）的完整迁移，验证框架多域适用性
- [x] 执行第三个 domain（health），验证框架泛化性 → 3 domain 验证完成
- [ ] **换人走通**：让另一个咨询师按 `docs/transfer-framework.md` 跑一次 think 迁移，不用提问能走通
- [ ] **客户反馈**：把 `data/journal-schema.yaml` 拿给客户看，问"有用吗"，记录"哪里有用、哪里没用"
- [ ] 每次迁移前先查 `library/products/index.md`（已入 AGENTS.md 工作纪律）
