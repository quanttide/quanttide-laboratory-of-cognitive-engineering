# 阻碍清单

深度来自于把每个阻碍推到底，而不是来自于造更多工具。

## 已推穿

| 阻碍 | 怎么推穿的 | 验证 |
|------|-----------|------|
| products 零引用 | health 迁移对照 cc-thinking-skills，写对比表进报告 | `reports/2026-W24-health-migration.md` |
| 换人走不通 | transfer-framework 重写为新人可独立走通版本，含精确命令和检查清单 | `cargo build→lab fill→lab assess` 全流程可执行 |
| 第一次只有 think | 补 business + health，3 domain 跨域验证 | 评分 4.57 / 4.29 / 4.14 |

## 待推穿

| 阻碍 | 当前状态 | 怎么推穿 |
|------|---------|---------|
| dynamics 命名冲突 | 绕过（重命名为"方法论成熟度"） | 在 output schema 元信息中补 `maturity` 字段，从根源区分"领域演化"和"图式自身演化" |
| W20 无 think 数据 | "待确认" | 查 journal W20 目录确认是日志遗漏还是 focus 转移 |
| 需验证类比例 2:6 | "待验证" | 统计 gallery qtconsult 和 3 domain 的保留/需验证比例，给出推荐范围 |
| 客户反馈 | 没做过 | 把 journal-schema.yaml 拿给客户看，记录"哪里有用、哪里没用" |
